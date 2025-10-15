#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod license;
mod python;

use config::{save_config_command, tool_read_theme_command, FileWriteLock};
use license::validate_offline_key_command;
use python::{get_python_env_status_command, initialize_python_environment_command};
use tauri::State;

#[tauri::command]
async fn save_config(
    app: tauri::AppHandle,
    key: String,
    value: String,
    lock: State<'_, FileWriteLock>,
) -> Result<(), String> {
    save_config_command(app, key, value, lock).await
}

#[tauri::command]
async fn tool_read_theme(
    app: tauri::AppHandle,
    lock: State<'_, FileWriteLock>,
) -> Result<String, String> {
    tool_read_theme_command(app, lock).await
}

#[tauri::command]
async fn validate_offline_key(
    lock: State<'_, FileWriteLock>,
) -> Result<license::OfflineKeyValidationResult, String> {
    validate_offline_key_command(lock).await
}

#[tauri::command]
async fn get_python_env_status() -> Result<python::PythonEnvironmentStatus, String> {
    get_python_env_status_command().await
}

#[tauri::command]
async fn initialize_python_environment(
    mirror: python::MirrorOptionKey,
) -> Result<python::PythonEnvironmentStatus, String> {
    initialize_python_environment_command(mirror).await
}

fn main() {
    tauri::Builder::default()
        .manage(FileWriteLock::default())
        .invoke_handler(tauri::generate_handler![
            save_config,
            tool_read_theme,
            validate_offline_key,
            get_python_env_status,
            initialize_python_environment
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
