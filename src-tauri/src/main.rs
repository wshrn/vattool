#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod environment;
mod offline_key;
mod state;
mod theme;

use environment::{get_environment_status, initialize_environment};
use offline_key::validate_offline_key;
use state::FileWriteLock;
use theme::{save_config, tool_read_theme};

fn main() {
    tauri::Builder::default()
        .manage(FileWriteLock::default())
        .invoke_handler(tauri::generate_handler![
            validate_offline_key,
            save_config,
            tool_read_theme,
            initialize_environment,
            get_environment_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
