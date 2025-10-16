use tauri::{App, AppHandle};

pub fn emit_theme_update(app: &AppHandle) {
    if let Err(err) = app.emit_all("theme-updated", ()) {
        eprintln!("主题更新事件广播失败: {err}");
    }
}

pub fn setup_theme_listener(app: &mut App) {
    if let Some(window) = app.get_window("main") {
        if let Err(err) = window.emit("theme-ready", ()) {
            eprintln!("初始化主题事件失败: {err}");
        }
    }
}
