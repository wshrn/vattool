use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde_json::{Map, Value};
use tauri::AppHandle;
use tokio::io::AsyncWriteExt;

pub const TOOLBOX_THEME_ENV_NAME: &str = "ZHIGONG_TOOLBOX_THEME";
const CONFIG_FILE_NAME: &str = "settings.json";

#[derive(Default)]
pub struct FileWriteLock(pub tokio::sync::Mutex<()>);

pub async fn save_config_internal(app: &AppHandle, key: &str, value: &str, lock: &tokio::sync::Mutex<()>) -> Result<()> {
    let _guard = lock.lock().await;
    let path = config_path(app)?;
    let mut map = read_config_map(&path).await.unwrap_or_default();
    map.insert(key.to_string(), Value::String(value.to_string()));

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let serialized = serde_json::to_string_pretty(&Value::Object(map))?;
    let mut file = tokio::fs::File::create(&path).await?;
    file.write_all(serialized.as_bytes()).await?;
    Ok(())
}

pub async fn read_config_value(app: &AppHandle, key: &str, lock: &tokio::sync::Mutex<()>) -> Result<Option<String>> {
    let _guard = lock.lock().await;
    let path = config_path(app)?;
    if !path.exists() {
        return Ok(None);
    }

    let map = read_config_map(&path).await.unwrap_or_default();
    Ok(map
        .get(key)
        .and_then(|value| value.as_str().map(|s| s.to_string())))
}

pub async fn read_theme(app: &AppHandle, lock: &tokio::sync::Mutex<()>) -> Result<String> {
    if let Ok(env_value) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
        if let Some(normalized) = normalize_theme(&env_value) {
            return Ok(normalized.to_string());
        }
    }

    if let Some(config_value) = read_config_value(app, "theme", lock).await? {
        if let Some(normalized) = normalize_theme(&config_value) {
            return Ok(normalized.to_string());
        }
    }

    Ok("auto".to_string())
}

fn normalize_theme(value: &str) -> Option<&'static str> {
    match value {
        "light" | "dark" | "auto" => Some(value),
        _ => None,
    }
}

async fn read_config_map(path: &PathBuf) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }

    let content = tokio::fs::read_to_string(path).await?;
    if content.trim().is_empty() {
        return Ok(Map::new());
    }

    let value: Value = serde_json::from_str(&content).unwrap_or(Value::Object(Map::new()));
    match value {
        Value::Object(map) => Ok(map),
        _ => Ok(Map::new()),
    }
}

fn config_path(app: &AppHandle) -> Result<PathBuf> {
    app.path_resolver()
        .app_config_dir()
        .map(|dir| dir.join(CONFIG_FILE_NAME))
        .ok_or_else(|| anyhow!("无法获取配置目录"))
        .context("无法定位配置目录")
}
