use crate::FileWriteLock;
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};
use std::{collections::HashMap, fs, path::PathBuf};

pub const TOOLBOX_THEME_ENV_NAME: &str = "ZHIGONG_TOOLBOX_THEME";
const CONFIG_FILE_NAME: &str = "settings.json";

pub fn ensure_config_dir(app: &tauri::AppHandle) -> Result<()> {
    let path = app
        .path_resolver()
        .app_config_dir()
        .ok_or_else(|| anyhow!("无法定位配置目录"))?;
    fs::create_dir_all(&path).with_context(|| format!("无法创建配置目录: {}", path.display()))?;
    Ok(())
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    ensure_config_dir(app)?;
    let dir = app
        .path_resolver()
        .app_config_dir()
        .ok_or_else(|| anyhow!("无法定位配置目录"))?;
    Ok(dir.join(CONFIG_FILE_NAME))
}

fn load_config(path: &PathBuf) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(Map::new());
    }

    let map: Map<String, Value> = serde_json::from_str(&content)
        .with_context(|| format!("解析配置文件失败: {}", path.display()))?;
    Ok(map)
}

fn save_config_map(path: &PathBuf, map: &Map<String, Value>) -> Result<()> {
    let content = serde_json::to_string_pretty(map)?;
    fs::write(path, content).with_context(|| format!("写入配置文件失败: {}", path.display()))?;
    Ok(())
}

fn with_lock<T>(lock: &FileWriteLock, task: impl FnOnce() -> Result<T>) -> Result<T> {
    let guard = lock.0.lock().map_err(|_| anyhow!("配置锁已损坏"))?;
    let result = task();
    drop(guard);
    result
}

fn sanitize_theme(theme: &str) -> &str {
    match theme {
        "light" | "dark" | "auto" => theme,
        _ => "auto",
    }
}

#[tauri::command]
pub fn tool_read_theme(
    app: tauri::AppHandle,
    lock: tauri::State<FileWriteLock>,
) -> Result<String, String> {
    if let Ok(value) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
        return Ok(sanitize_theme(value.trim()).to_string());
    }

    let path = config_path(&app).map_err(|err| err.to_string())?;
    let lock_ref = lock.inner();
    let result = with_lock(lock_ref, || {
        let mut map = load_config(&path)?;
        if let Some(value) = map.remove("theme") {
            Ok(sanitize_theme(value.as_str().unwrap_or("auto")).to_string())
        } else {
            Ok(String::from("auto"))
        }
    });

    result.map_err(|err| err.to_string())
}

#[tauri::command]
pub fn save_config(
    app: tauri::AppHandle,
    key: String,
    value: String,
    lock: tauri::State<FileWriteLock>,
) -> Result<(), String> {
    let should_emit_theme = key == "theme";
    let path = config_path(&app).map_err(|err| err.to_string())?;

    with_lock(lock.inner(), || {
        let mut map = load_config(&path)?;
        map.insert(key.clone(), json!(value));
        save_config_map(&path, &map)?;
        Ok(())
    })
    .map_err(|err| err.to_string())?;

    if should_emit_theme {
        emit_theme_update(&app);
    }

    Ok(())
}

pub fn emit_theme_update(app: &tauri::AppHandle) {
    let _ = app.emit("theme-changed", HashMap::<String, String>::new());
}
