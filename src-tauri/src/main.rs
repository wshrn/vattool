#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod device;
mod offline_key;
mod python_env;

use std::sync::Arc;

use config::{emit_theme_update, read_config_value, save_config_internal, TOOLBOX_THEME_ENV_NAME};
use offline_key::OfflineKeyValidationResult;
use parking_lot::Mutex;
use python_env::{InitializationResult, PythonEnvironmentStatus};

pub struct FileWriteLock(pub Arc<Mutex<()>>);

#[tauri::command]
pub async fn save_config(
    app: tauri::AppHandle,
    key: String,
    value: String,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<(), String> {
    let should_emit_theme = key == "theme";
    save_config_internal(&app, &key, &value, &lock.0)
        .await
        .map_err(|err| err.to_string())?;

    if should_emit_theme {
        emit_theme_update(&app);
    }

    Ok(())
}

#[tauri::command]
pub async fn tool_read_theme(
    app: tauri::AppHandle,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<String, String> {
    if let Ok(value) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
        return Ok(value);
    }

    let value = read_config_value(&app, "theme", &lock.0)
        .await
        .map_err(|err| err.to_string())?
        .unwrap_or_else(|| "auto".into());
    Ok(value)
}

#[tauri::command]
pub async fn validate_offline_key(
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<OfflineKeyValidationResult, String> {
    offline_key::try_validate_from_env(&lock.0)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_python_environment_status() -> PythonEnvironmentStatus {
    python_env::get_python_environment_status()
}

#[tauri::command]
pub async fn initialize_python_environment(
    mirror_id: String,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<InitializationResult, String> {
    Ok(python_env::initialize_python_environment(
        &mirror_id, &lock.0,
    ))
}

fn main() {
    tauri::Builder::default()
        .manage(FileWriteLock(Arc::new(Mutex::new(()))))
        .invoke_handler(tauri::generate_handler![
            save_config,
            tool_read_theme,
            validate_offline_key,
            get_python_environment_status,
            initialize_python_environment
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
