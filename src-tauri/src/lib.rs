mod config;
mod offline_key;
mod python_env;

use std::sync::{Arc, Mutex};

pub struct FileWriteLock(pub Arc<Mutex<()>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(FileWriteLock(Arc::new(Mutex::new(()))))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            offline_key::validate_offline_key,
            python_env::get_python_environment_status,
            python_env::initialize_python_environment,
            config::tool_read_theme,
            config::save_config
        ])
        .setup(|app| {
            config::ensure_config_dir(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
