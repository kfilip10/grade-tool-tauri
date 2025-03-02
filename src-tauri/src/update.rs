use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager}; // Add Manager trait here
use tauri_plugin_updater::UpdaterExt;

pub struct UpdateState {
    pub progress: Arc<Mutex<UpdateProgress>>,
}

pub struct UpdateProgress {
    pub downloading: bool,
    pub percent: f64,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub complete: bool,
    pub error: Option<String>,
}

impl Default for UpdateProgress {
    fn default() -> Self {
        Self {
            downloading: false,
            percent: 0.0,
            downloaded: 0,
            total: None,
            complete: false,
            error: None,
        }
    }
}

/// Checks for updates but does not install them.
#[tauri::command]
pub async fn check_for_updates(app_handle: AppHandle) -> Result<String, String> {
    let updater = app_handle.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(update) => {
            if let Some(update) = update {
                let update_info = serde_json::json!({
                    "available": true,
                    "version": update.version,
                    "body": update.body,
                    "downloadUrl": update.download_url
                });
                Ok(serde_json::to_string(&update_info).unwrap())
            } else {
                Ok(r#"{"available": false}"#.to_string())
            }
        }
        Err(e) => Err(format!("Failed to check for updates: {}", e)),
    }
}

/// Downloads and installs the update.
#[tauri::command]
pub async fn download_and_install_update(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<UpdateState>();
    let progress_clone = state.progress.clone();

    {
        let mut progress = progress_clone.lock().unwrap();
        progress.downloading = true;
        progress.percent = 0.0;
        progress.downloaded = 0;
        progress.complete = false;
        progress.error = None;
    }

    let updater = app_handle.updater().map_err(|e| e.to_string())?;
    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            let mut progress = progress_clone.lock().unwrap();
            progress.error = Some("No update available".to_string());
            return Err("No update available".to_string());
        }
        Err(e) => {
            let mut progress = progress_clone.lock().unwrap();
            progress.error = Some(e.to_string());
            return Err(e.to_string());
        }
    };

    let progress_for_progress = progress_clone.clone();
    let progress_for_complete = progress_clone.clone();

    match update
        .download_and_install(
            move |chunk_length, content_length| {
                let mut progress = progress_for_progress.lock().unwrap();
                progress.downloaded += chunk_length as u64;
                progress.total = content_length;

                if let Some(total) = content_length {
                    if total > 0 {
                        progress.percent = (progress.downloaded as f64 / total as f64) * 100.0;
                    }
                }

                println!(
                    "Downloaded {} of {} bytes ({:.1}%)",
                    progress.downloaded,
                    content_length.unwrap_or(0),
                    progress.percent
                );
            },
            move || {
                let mut progress = progress_for_complete.lock().unwrap();
                progress.complete = true;
                progress.percent = 100.0;
                println!("Download finished");
            },
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            let mut progress = progress_clone.lock().unwrap();
            progress.error = Some(e.to_string());
            Err(e.to_string())
        }
    }
}

/// Retrieves the update progress as JSON.
#[tauri::command]
pub fn get_update_progress(app_handle: AppHandle) -> Result<String, String> {
    let state = app_handle.state::<UpdateState>();
    let progress = state.progress.lock().unwrap();

    let progress_json = serde_json::json!({
        "downloading": progress.downloading,
        "percent": progress.percent,
        "downloaded": progress.downloaded,
        "total": progress.total,
        "complete": progress.complete,
        "error": progress.error
    });

    Ok(serde_json::to_string(&progress_json).unwrap())
}
