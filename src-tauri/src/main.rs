#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod device;
mod offline_key;
mod python;
mod theme;

use config::{read_theme, save_config_internal, FileWriteLock};
use offline_key::{try_validate_from_env, OfflineKeyValidationResult};
use python::{InitializationResult, PythonStatus};
use tauri::{async_runtime, State};

#[tauri::command]
async fn validate_offline_key(lock: State<'_, FileWriteLock>) -> Result<OfflineKeyValidationResult, String> {
    try_validate_from_env(&*lock)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn fetch_python_status() -> Result<PythonStatus, String> {
    async_runtime::spawn_blocking(python::collect_status)
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn initialize_environment(mirror: String) -> Result<InitializationResult, String> {
    async_runtime::spawn_blocking(move || python::initialize_environment(&mirror))
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())
}

#[tauri::command]
async fn save_config(
    app: tauri::AppHandle,
    key: String,
    value: String,
    lock: State<'_, FileWriteLock>,
) -> Result<(), String> {
    save_config_internal(&app, &key, &value, &lock.0)
        .await
        .map_err(|err| err.to_string())?;

    if key == "theme" {
        theme::emit_theme_update(&app, &value);
    }

    Ok(())
}

#[tauri::command]
async fn tool_read_theme(app: tauri::AppHandle, lock: State<'_, FileWriteLock>) -> Result<String, String> {
    read_theme(&app, &lock.0)
        .await
        .map_err(|err| err.to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(FileWriteLock::default())
        .invoke_handler(tauri::generate_handler![
            validate_offline_key,
            fetch_python_status,
            initialize_environment,
            save_config,
            tool_read_theme
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
