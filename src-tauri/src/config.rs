use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

pub const TOOLBOX_THEME_ENV_NAME: &str = "ZHIGONG_TOOLBOX_THEME";

#[derive(Debug, Default, Serialize, Deserialize)]
struct StoredConfig {
    #[serde(default)]
    values: HashMap<String, String>,
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    app.path_resolver()
        .app_config_dir()
        .map(|dir| dir.join("settings.json"))
        .ok_or_else(|| anyhow!("无法确定配置目录"))
}

fn read_config_from_disk(path: &PathBuf) -> StoredConfig {
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(parsed) = serde_json::from_str::<StoredConfig>(&content) {
            return parsed;
        }
    }
    StoredConfig::default()
}

fn write_config_to_disk(path: &PathBuf, data: &StoredConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| "创建配置目录失败")?;
    }
    let serialized = serde_json::to_string_pretty(data).context("序列化配置失败")?;
    std::fs::write(path, serialized).context("写入配置失败")?;
    Ok(())
}

pub async fn save_config_internal(
    app: &tauri::AppHandle,
    key: &str,
    value: &str,
    lock: &Arc<Mutex<()>>,
) -> Result<()> {
    let path = config_path(app)?;
    let key = key.to_string();
    let value = value.to_string();
    let lock = Arc::clone(lock);

    tokio::task::spawn_blocking(move || {
        let _guard = lock.lock();
        let mut config = read_config_from_disk(&path);
        config.values.insert(key, value);
        write_config_to_disk(&path, &config)
    })
    .await
    .map_err(|err| anyhow!("无法保存配置: {err}"))??;

    Ok(())
}

pub async fn read_config_value(
    app: &tauri::AppHandle,
    key: &str,
    lock: &Arc<Mutex<()>>,
) -> Result<Option<String>> {
    let path = match config_path(app) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };
    let key = key.to_string();
    let lock = Arc::clone(lock);

    let value = tokio::task::spawn_blocking(move || {
        let _guard = lock.lock();
        let config = read_config_from_disk(&path);
        Ok(config.values.get(&key).cloned())
    })
    .await
    .map_err(|err| anyhow!("无法读取配置: {err}"))??;

    Ok(value)
}

pub fn emit_theme_update(app: &tauri::AppHandle) {
    let _ = app.emit_all("theme-updated", ());
}
