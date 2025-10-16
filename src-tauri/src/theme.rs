use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize)]
struct ThemeChangedPayload<'a> {
    theme: &'a str,
}

pub fn emit_theme_update(app: &AppHandle, theme: &str) {
    let payload = ThemeChangedPayload { theme };
    if let Err(err) = app.emit_all("theme-changed", payload) {
        eprintln!("Failed to emit theme change event: {err}");
    }
}
