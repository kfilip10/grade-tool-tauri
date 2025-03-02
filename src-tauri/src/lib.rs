mod r_shiny;
mod update; // Import the update module // Import the R process module

use std::env;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use update::UpdateProgress;

fn set_global_env_vars() {
    let mut is_dev: bool = cfg!(debug_assertions);
    //is_dev = false; // Set to false for production
    // Get the current executable directory for absolute paths
    let app_dir = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::new())
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::new());

    println!("App directory: {:?}", app_dir);

    //if in dev mode then use local path else use appdata path
    let base_path = if is_dev {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::new())
            .join("assets")
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|exe_path| exe_path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| ".".into())
            .join("assets")
    };
    println!("Base path: {:?}", base_path);
    std::fs::create_dir_all(&base_path).unwrap_or_default();

    // With these lines:
    let r_home_path = base_path.join("r-win");
    let r_lib_path = r_home_path.join("library");
    let rscript_path = r_home_path.join("bin").join("Rscript.exe");
    let shiny_app_path = base_path.join("shiny");
    let start_shiny_path = base_path.join("start-shiny.R");
    let shiny_url = if is_dev {
        // Use localhost in development mode
        "http://127.0.0.1".to_string()
    } else {
        // In production, could use a different host if needed
        "http://127.0.0.1".to_string()
    };
    // For environment variables, convert paths to strings:
    let r_home = r_home_path.to_string_lossy().to_string();
    let r_lib = r_lib_path.to_string_lossy().to_string();
    let rscript = rscript_path.to_string_lossy().to_string();
    let shiny_app = shiny_app_path.to_string_lossy().to_string();
    let start_shiny = start_shiny_path.to_string_lossy().to_string();

    // Set environment variables:
    env::set_var("RHOME", &r_home);
    env::set_var("R_HOME_DIR", &r_home);
    env::set_var("R_LIBS", &r_lib);
    env::set_var("R_LIB_PATH", &r_lib);
    env::set_var("RSCRIPT_PATH", &rscript);
    env::set_var("SHINY_APP_PATH", &shiny_app);
    env::set_var("START_SHINY_PATH", &start_shiny);
    env::set_var("SHINY_URL", &shiny_url);

    println!("Environment Variables Set:");
    println!("  R_HOME_DIR = {}", r_home);
    println!("  R_LIBS = {}", r_lib);
    println!("  RSCRIPT_PATH = {}", rscript);
    println!("  SHINY_APP_PATH = {}", shiny_app);
    println!("  START_SHINY_PATH = {}", start_shiny);
    println!("  SHINY_URL = {}", shiny_url);
    println!("Checking if files exist:");
    println!(
        "  Rscript exists: {}",
        std::path::Path::new(&rscript_path).exists()
    );
    println!(
        "  start_shiny.R exists: {}",
        std::path::Path::new(&start_shiny_path).exists()
    );
    println!(
        "  shiny_app_path exists: {}",
        std::path::Path::new(&shiny_app_path).exists()
    );
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(update::UpdateState {
                progress: Arc::new(Mutex::new(UpdateProgress::default())),
            });

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            app.handle().plugin(tauri_plugin_dialog::init())?;

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            app.handle().plugin(tauri_plugin_http::init())?;
            set_global_env_vars(); // Set all paths once at startup
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            update::check_for_updates, // Move updates to the new module
            update::download_and_install_update,
            update::get_update_progress,
            r_shiny::start_r_shiny, // Register R Shiny commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
