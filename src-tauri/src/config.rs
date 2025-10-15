use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Manager;
use tokio::sync::Mutex;

pub const TOOLBOX_THEME_ENV_NAME: &str = "ZHIGONG_TOOLBOX_THEME";
const CONFIG_FILE_NAME: &str = "settings.json";

pub struct FileWriteLock(pub Arc<Mutex<()>>);

impl Default for FileWriteLock {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(())))
    }
}

impl FileWriteLock {
    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.0.lock().await
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct AppConfig {
    #[serde(flatten)]
    entries: BTreeMap<String, Value>,
}

pub async fn save_config_command(
    app: tauri::AppHandle,
    key: String,
    value: String,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<(), String> {
    let path = config_file_path(&app).map_err(|e| e.to_string())?;
    let should_emit_theme = key == "theme";

    if let Err(err) = save_config_internal(&path, &key, &value, &lock).await {
        return Err(err.to_string());
    }

    if should_emit_theme {
        if let Err(err) = emit_theme_update(&app, &value) {
            eprintln!("failed to emit theme update: {err}");
        }
    }

    Ok(())
}

pub async fn tool_read_theme_command(
    app: tauri::AppHandle,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<String, String> {
    if let Ok(value) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
        if matches!(value.as_str(), "light" | "dark" | "auto") {
            return Ok(value);
        }
    }

    let path = config_file_path(&app).map_err(|e| e.to_string())?;
    match read_config(&path, &lock).await {
        Ok(config) => Ok(config
            .entries
            .get("theme")
            .and_then(|value| value.as_str())
            .filter(|v| matches!(*v, "light" | "dark" | "auto"))
            .unwrap_or("auto")
            .to_string()),
        Err(err) => Err(err.to_string()),
    }
}

pub async fn save_config_internal(
    path: &PathBuf,
    key: &str,
    value: &str,
    lock: &tauri::State<'_, FileWriteLock>,
) -> Result<()> {
    let mut config = read_config(path, lock).await?;
    config
        .entries
        .insert(key.to_string(), Value::String(value.to_string()));
    write_config(path, &config, lock).await
}

async fn read_config(path: &PathBuf, lock: &tauri::State<'_, FileWriteLock>) -> Result<AppConfig> {
    let guard = lock.lock().await;
    if !path.exists() {
        drop(guard);
        return Ok(AppConfig::default());
    }
    let content = std::fs::read(path)?;
    drop(guard);
    if content.is_empty() {
        return Ok(AppConfig::default());
    }
    let config: AppConfig = serde_json::from_slice(&content)?;
    Ok(config)
}

async fn write_config(
    path: &PathBuf,
    config: &AppConfig,
    lock: &tauri::State<'_, FileWriteLock>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let guard = lock.lock().await;
    let data = serde_json::to_vec_pretty(config)?;
    std::fs::write(path, data)?;
    drop(guard);
    Ok(())
}

fn config_file_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    app.path_resolver()
        .app_config_dir()
        .map(|dir| dir.join(CONFIG_FILE_NAME))
        .ok_or_else(|| anyhow!("无法确定配置目录"))
}

fn emit_theme_update(app: &tauri::AppHandle, theme: &str) -> Result<()> {
    app.emit_all("theme://updated", theme.to_string())?;
    Ok(())
}
