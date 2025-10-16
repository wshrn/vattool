use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use tokio::sync::Mutex;
use serde_json::Value;
use tauri::{AppHandle, Manager};

pub const TOOLBOX_THEME_ENV_NAME: &str = "ZHIGONG_TOOLBOX_THEME";

#[derive(Default)]
pub struct FileWriteLock(pub Mutex<()>);

impl FileWriteLock {
    pub fn new() -> Self {
        Self(Mutex::new(()))
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf> {
    let resolver = app.path_resolver();
    resolver
        .app_config_dir()
        .ok_or_else(|| anyhow!("无法确定配置目录"))
        .map(|dir| dir.join("config.json"))
}

fn read_config(path: &PathBuf) -> HashMap<String, Value> {
    if !path.exists() {
        return HashMap::new();
    }
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn write_config(path: &PathBuf, data: &HashMap<String, Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("无法创建配置目录: {:?}", parent))?;
    }
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json).with_context(|| format!("无法写入配置文件: {:?}", path))?;
    Ok(())
}

pub fn emit_theme_update(app: &AppHandle) {
    let _ = app.emit_all("theme-changed", ());
}

async fn save_config_internal(
    app: &AppHandle,
    key: &str,
    value: &str,
    lock: &Mutex<()>,
) -> Result<()> {
    let path = config_path(app)?;
    let _guard = lock.lock().await;
    let mut data = read_config(&path);
    data.insert(key.to_string(), Value::String(value.to_string()));
    write_config(&path, &data)
}

fn read_theme_from_map(map: &HashMap<String, Value>) -> Option<String> {
    map.get("theme").and_then(|value| value.as_str().map(|s| s.to_string()))
}

async fn read_theme_internal(app: &AppHandle, lock: &Mutex<()>) -> Result<Option<String>> {
    let path = config_path(app)?;
    let _guard = lock.lock().await;
    let data = read_config(&path);
    Ok(read_theme_from_map(&data))
}

#[tauri::command]
pub async fn save_config(
    app: AppHandle,
    key: String,
    value: String,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<(), String> {
    let should_emit_theme = key == "theme";
    save_config_internal(&app, &key, &value, &lock.0)
        .await
        .map_err(|err| err.to_string())?;

    if should_emit_theme {
        emit_theme_update(&app);
    }

    Ok(())
}

#[tauri::command]
pub async fn tool_read_theme(
    app: AppHandle,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<String, String> {
    if let Ok(theme) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
        let trimmed = theme.trim();
        if matches!(trimmed, "light" | "dark" | "auto") {
            return Ok(trimmed.to_string());
        }
    }

    read_theme_internal(&app, &lock.0)
        .await
        .map(|value| value.unwrap_or_else(|| "auto".to_string()))
        .map_err(|err| err.to_string())
}
