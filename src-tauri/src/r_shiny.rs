use lazy_static::lazy_static;
use reqwest::blocking::Client;
use std::net::{TcpListener, TcpStream};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::{env, sync::Mutex, thread, time};
use tauri::Emitter;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

lazy_static! {
    static ref R_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
}

const PORT_RANGE: (u16, u16) = (3000, 8000); // Define a sensible port range

fn find_available_port(start: u16, end: u16) -> Option<u16> {
    let shiny_url = env::var("SHINY_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string());
    let host = if shiny_url.starts_with("http://") {
        &shiny_url["http://".len()..]
    } else {
        &shiny_url
    };

    for port in start..end {
        let addr = format!("{}:{}", host, port);
        // If we can bind to the address, it's free.
        if TcpListener::bind(&addr).is_ok() {
            return Some(port);
        }
    }
    None
}

/// Starts the R Shiny app using the installed `r-win`.
#[tauri::command]
pub fn start_r_shiny(app_handle: tauri::AppHandle) -> Result<String, String> {
    let rscript_path = env::var("RSCRIPT_PATH").expect("RSCRIPT_PATH not set");
    let r_home = env::var("R_HOME_DIR").expect("R_HOME_DIR not set");
    let start_shiny_path = env::var("START_SHINY_PATH").expect("START_SHINY_PATH not set");
    let r_lib_path = env::var("R_LIB_PATH").expect("R_LIB_PATH not set");
    let shiny_app_path = env::var("SHINY_APP_PATH").expect("SHINY_APP_PATH not set");
    let shiny_url = env::var("SHINY_URL").expect("SHINY_URL not set");

    let mut retries = 0;
    let max_retries = 4;
    let mut delay = 1000; // Start with 1s delay, increase with retries

    while retries < max_retries {
        // Inform frontend we're attempting to start
        let mut value = app_handle
            .emit(
                "shiny-status",
                format!("Attempting to start (try {}/{})", retries + 1, max_retries),
            )
            .unwrap_or_else(|e| eprintln!("Failed to emit status event: {}", e));

        if let Some(port) = find_available_port(PORT_RANGE.0, PORT_RANGE.1) {
            println!(
                "Trying to launch Shiny app on port {} (Attempt {}/{})",
                port,
                retries + 1,
                max_retries
            );

            // Create command but don't spawn it yet
            let mut command = Command::new(&rscript_path);

            // Configure the command with all your arguments
            command
                .arg("--vanilla")
                .arg(&start_shiny_path)
                .arg("--verbose")
                .env("RHOME", &r_home)
                .env("R_HOME_DIR", &r_home)
                .env("RE_SHINY_PORT", port.to_string())
                .env("RE_SHINY_PATH", &shiny_app_path)
                .env("RE_SHINY_HOST", "0.0.0.0") // Make Shiny bind to all interfaces
                .env("R_LIBS", &r_lib_path)
                .env("R_LIBS_USER", &r_lib_path)
                .env("R_LIBS_SITE", &r_lib_path)
                .env("R_LIB_PATHS", &r_lib_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            // On Windows, add the CREATE_NO_WINDOW flag to hide the console window
            #[cfg(target_os = "windows")]
            {
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                command.creation_flags(CREATE_NO_WINDOW);
            }

            // Now spawn the process
            let process_result = command.spawn();

            let (tx_ready, rx_ready) = std::sync::mpsc::channel::<bool>();
            let (tx_stdout, rx_stdout) = std::sync::mpsc::channel::<String>();
            let (tx_stderr, rx_stderr) = std::sync::mpsc::channel::<String>();

            // When launching the process...
            match process_result {
                Ok(mut process) => {
                    let pid = process.id();

                    // For stdout
                    if let Some(stdout) = process.stdout.take() {
                        let tx_stdout_clone = tx_stdout.clone();
                        let tx_ready_clone = tx_ready.clone();
                        let app_handle_clone = app_handle.clone(); // Clone before moving
                        std::thread::spawn(move || {
                            use std::io::{BufRead, BufReader};
                            let reader = BufReader::new(stdout);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    println!("SHINY OUT: {}", line);
                                    let _ = tx_stdout_clone.send(line.clone());

                                    // Check for signs of Shiny ready
                                    if line.contains("Listening on") {
                                        let _ = tx_ready_clone.send(true);
                                    }
                                }
                            }
                        });
                    }

                    // For stderr (similar pattern with additional package detection)
                    if let Some(stderr) = process.stderr.take() {
                        let tx_stderr_clone = tx_stderr.clone();
                        let tx_ready_clone = tx_ready.clone();
                        let app_handle_clone = app_handle.clone(); // Clone before moving
                        std::thread::spawn(move || {
                            use std::io::{BufRead, BufReader};
                            let reader = BufReader::new(stderr);
                            let mut package_loading_count = 0;
                            let mut attaching_count = 0;

                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    println!("SHINY ERR: {}", line);
                                    let _ = tx_stderr_clone.send(line.clone());

                                    // Track package loading status
                                    if line.contains("Loading required package:")
                                        || line.contains("Attaching package:")
                                    {
                                        package_loading_count += 1;
                                        attaching_count += 1;
                                        app_handle_clone
                                            .emit(
                                                "shiny-status",
                                                format!(
                                                    "Loading packages ({} loaded)",
                                                    package_loading_count
                                                ),
                                            )
                                            .unwrap_or_default();
                                    }

                                    // Check for signs of Shiny ready
                                    if line.contains("Listening on") {
                                        let _ = tx_ready_clone.send(true);
                                    }
                                }
                            }
                        });
                    }

                    *R_PROCESS.lock().unwrap() = Some(process);

                    // Set up the URL
                    let full_url = format!("http://127.0.0.1:{}", port);
                    println!("Waiting for Shiny server on {}", full_url);

                    // Monitor for ready signal or timeout
                    let client = Client::new();
                    let timeout = Duration::from_secs(40); // Generous timeout for package loading
                    let start_time = std::time::Instant::now();

                    // Main monitoring loop
                    app_handle
                        .emit("shiny-status", "Waiting for packages to load...")
                        .unwrap_or_default();

                    // Wait for either "Listening on" message or timeout
                    let mut is_ready = false;
                    while start_time.elapsed() < timeout {
                        // Check for ready message
                        match rx_ready.try_recv() {
                            Ok(true) => {
                                is_ready = true;
                                break;
                            }
                            _ => {
                                // If not ready yet, wait a bit then try connecting
                                thread::sleep(Duration::from_millis(500));

                                // Also check for TCP connectivity
                                if let Ok(_) = TcpStream::connect(format!("127.0.0.1:{}", port)) {
                                    // Try an HTTP request
                                    match client
                                        .head(&full_url)
                                        .timeout(Duration::from_secs(1))
                                        .send()
                                    {
                                        Ok(response) if response.status().is_success() => {
                                            is_ready = true;
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }

                    if is_ready {
                        app_handle
                            .emit("shiny-started", &full_url)
                            .unwrap_or_else(|e| eprintln!("Failed to emit started event: {}", e));
                        return Ok(full_url);
                    } else {
                        return Err(
                            "Timed out waiting for Shiny to complete package loading".to_string()
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to start Shiny app: {}. Retrying...", e);
                    retries += 1;
                    thread::sleep(time::Duration::from_millis(delay));
                    delay *= 2; // Exponential backoff
                }
            }
        } else {
            eprintln!(
                "No available ports in range {}-{}. Retrying...",
                PORT_RANGE.0, PORT_RANGE.1
            );
            retries += 1;
            thread::sleep(time::Duration::from_millis(delay));
            delay *= 2;
        }
    }

    // Emit failure event
    app_handle
        .emit("shiny-error", "Failed to launch Shiny app")
        .unwrap_or_else(|e| eprintln!("Failed to emit error event: {}", e));

    Err("Failed to launch Shiny app.".to_string())
}
