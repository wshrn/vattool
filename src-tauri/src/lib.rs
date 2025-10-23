mod config;
mod device;
mod offline_key;
mod python_env;
mod storage;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct FileWriteLock(pub Arc<Mutex<()>>);

impl Default for FileWriteLock {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(())))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(FileWriteLock::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            offline_key::validate_offline_key,
            offline_key::get_device_id,
            python_env::get_python_environment_status,
            python_env::initialize_python_environment,
            config::tool_read_theme,
            config::save_config,
            config::sync_theme_env
        ])
        .setup(|app| {
            let handle = app.handle();
            config::ensure_config_dir(&handle)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
