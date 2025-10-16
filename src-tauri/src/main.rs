#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod environment;
mod license;
mod theme;

use std::sync::Arc;

use environment::{EnvironmentStatus, MirrorOption};
use license::OfflineKeyValidationResult;
use tauri::Manager;
use tokio::sync::Mutex;

type SharedLock = Arc<Mutex<()>>;

pub struct FileWriteLock(pub SharedLock);

#[tauri::command]
async fn validate_offline_key(state: tauri::State<'_, FileWriteLock>) -> Result<OfflineKeyValidationResult, String> {
    license::try_validate_from_env(&state.0)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn tool_read_theme(app: tauri::AppHandle, state: tauri::State<'_, FileWriteLock>) -> Result<String, String> {
    theme::read_theme(&app, &state.0)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn save_config(
    app: tauri::AppHandle,
    key: String,
    value: String,
    state: tauri::State<'_, FileWriteLock>,
) -> Result<(), String> {
    let should_emit_theme = key == "theme";

    theme::save_config_internal(&app, &key, &value, &state.0)
        .await
        .map_err(|err| err.to_string())?;

    if should_emit_theme {
        theme::emit_theme_update(&app);
    }

    Ok(())
}

#[tauri::command]
async fn get_environment_status() -> Result<EnvironmentStatus, String> {
    environment::get_environment_status()
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn initialize_python_environment(mirror: MirrorOption) -> Result<EnvironmentStatus, String> {
    environment::initialize_python_environment(mirror)
        .await
        .map_err(|err| err.to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(FileWriteLock(Arc::new(Mutex::new(()))))
        .invoke_handler(tauri::generate_handler![
            validate_offline_key,
            tool_read_theme,
            save_config,
            get_environment_status,
            initialize_python_environment
        ])
        .setup(|app| {
            let app_handle = app.handle();
            let lock = app.state::<FileWriteLock>().0.clone();
            tauri::async_runtime::spawn(async move {
                match license::try_validate_from_env(&lock).await {
                    Ok(result) if !result.is_valid => {
                        let _ = app_handle.emit_all("license-invalid", result);
                    }
                    Err(err) => {
                        let _ = app_handle.emit_all("license-error", err.to_string());
                    }
                    _ => {}
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
