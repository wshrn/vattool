#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod offline_key;
mod python_env;
mod theme;

use offline_key::OfflineKeyValidationResult;
use theme::FileWriteLock;

#[tauri::command]
async fn validate_offline_key(
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<OfflineKeyValidationResult, String> {
    offline_key::try_validate_from_env(lock.inner())
        .await
        .map_err(|err| err.to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(FileWriteLock::default())
        .invoke_handler(tauri::generate_handler![
            validate_offline_key,
            theme::save_config,
            theme::tool_read_theme,
            python_env::python_env_status,
            python_env::initialize_python_environment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
