#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod device;
mod offline_key;
mod python_env;
mod settings;

use std::sync::Arc;

use parking_lot::Mutex;
use settings::{read_theme, save_config_internal};
use tauri::{AppHandle, Manager, State};

pub struct FileWriteLock(pub Arc<Mutex<()>>);

#[tauri::command]
async fn validate_offline_key(lock: State<'_, FileWriteLock>) -> Result<offline_key::OfflineKeyValidationResult, String> {
  offline_key::try_validate_from_env(&lock.0)
    .await
    .map_err(|err| err.to_string())
}

#[tauri::command]
async fn tool_read_theme() -> Result<String, String> {
  read_theme().await.map_err(|err| err.to_string())
}

#[tauri::command]
async fn save_config(
  app: AppHandle,
  key: String,
  value: String,
  lock: State<'_, FileWriteLock>,
) -> Result<(), String> {
  let should_emit_theme = key == "theme";
  save_config_internal(&key, &value, &lock.0)
    .await
    .map_err(|err| err.to_string())?;

  if should_emit_theme {
    settings::emit_theme_update(&app);
  }
  Ok(())
}

#[tauri::command]
async fn fetch_python_status(
  preferred_mirror: Option<String>,
) -> Result<python_env::PythonStatusReport, String> {
  python_env::collect_status(preferred_mirror)
    .await
    .map_err(|err| err.to_string())
}


#[tauri::command]
async fn initialize_python_environment(
  preferred_mirror: python_env::MirrorOption,
) -> Result<(), String> {
  python_env::initialize_environment(preferred_mirror)
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
      fetch_python_status,
      initialize_python_environment
    ])
    .setup(|app| {
      app.listen_global("theme-updated", |_| {});
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
