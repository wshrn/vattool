#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod device;
mod offline_key;
mod python;
mod theme;

use std::sync::Arc;

use parking_lot::Mutex;
use tauri::Manager;

pub struct FileWriteLock(pub Arc<Mutex<()>>);

#[tauri::command]
async fn tool_read_theme() -> Result<String, String> {
  config::read_config_value("theme").await
}

#[tauri::command]
async fn save_config(
  app: tauri::AppHandle,
  key: String,
  value: String,
  lock: tauri::State<'_, FileWriteLock>,
) -> Result<(), String> {
  let should_emit_theme = key == "theme";
  config::save_config_value(&key, &value, &lock.0)
    .await
    .map_err(|err| err.to_string())?;
  if should_emit_theme {
    theme::emit_theme_update(&app);
  }
  Ok(())
}

fn main() {
  let lock = FileWriteLock(Arc::new(Mutex::new(())));
  tauri::Builder::default()
    .manage(lock)
    .invoke_handler(tauri::generate_handler![
      offline_key::validate_offline_key,
      tool_read_theme,
      save_config,
      python::python_environment_status,
      python::initialize_python_environment
    ])
    .setup(|app| {
      theme::setup_theme_listener(app);
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running python environment tool")
}
