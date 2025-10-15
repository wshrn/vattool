use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::state::FileWriteLock;

pub const TOOLBOX_THEME_ENV_NAME: &str = "ZHIGONG_TOOLBOX_THEME";

#[derive(Debug, Serialize, Deserialize, Default)]
struct ThemeConfig {
    #[serde(default)]
    theme: Option<String>,
}

#[tauri::command]
pub async fn tool_read_theme(
    app: tauri::AppHandle,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<String, String> {
    match read_theme_internal(&app, &lock.clone_arc()).await {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Ok("auto".into()),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn save_config(
    app: tauri::AppHandle,
    key: String,
    value: String,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<(), String> {
    let should_emit_theme = key == "theme";
    if let Err(err) = save_config_internal(&app, &key, &value, &lock.clone_arc()).await {
        return Err(err.to_string());
    }

    if should_emit_theme {
        emit_theme_update(&app);
    }

    Ok(())
}

pub fn emit_theme_update(app: &tauri::AppHandle) {
    let _ = app.emit_all("theme-changed", HashMap::from([("key", "theme")]));
}

async fn config_file_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    let dir = tauri::api::path::app_config_dir(app.config())
        .ok_or_else(|| anyhow!("无法确定配置目录"))?;
    Ok(dir.join("theme.json"))
}

async fn read_theme_internal(
    app: &tauri::AppHandle,
    lock: &Arc<Mutex<()>>,
) -> Result<Option<String>> {
    if let Ok(value) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
        if matches!(value.as_str(), "light" | "dark" | "auto") {
            return Ok(Some(value));
        }
    }

    let path = config_file_path(app).await?;
    if !path.exists() {
        return Ok(None);
    }

    let _guard = lock.lock().await;
    let data = tokio::fs::read_to_string(&path)
        .await
        .context("读取主题配置失败")?;
    let config: ThemeConfig = serde_json::from_str(&data).unwrap_or_default();
    Ok(config.theme)
}

async fn save_config_internal(
    app: &tauri::AppHandle,
    key: &str,
    value: &str,
    lock: &Arc<Mutex<()>>,
) -> Result<()> {
    let mut config = ThemeConfig::default();
    let path = config_file_path(app).await?;

    if path.exists() {
        let existing = {
            let _guard = lock.lock().await;
            tokio::fs::read_to_string(&path).await.unwrap_or_default()
        };
        config = serde_json::from_str(&existing).unwrap_or_default();
    }

    if key == "theme" {
        config.theme = Some(value.to_string());
        std::env::set_var(TOOLBOX_THEME_ENV_NAME, value);
    }

    let serialized = serde_json::to_string_pretty(&config)?;
    {
        let _guard = lock.lock().await;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut file = tokio::fs::File::create(&path).await?;
        file.write_all(serialized.as_bytes()).await?;
        file.flush().await?;
    }
    Ok(())
}
