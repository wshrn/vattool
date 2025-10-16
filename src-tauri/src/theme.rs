use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt, sync::Mutex};

pub const TOOLBOX_THEME_ENV_NAME: &str = "ZHIGONG_TOOLBOX_THEME";

#[derive(Debug, Default, Serialize, Deserialize)]
struct ConfigStore {
    entries: HashMap<String, String>,
}

pub async fn read_theme(app: &tauri::AppHandle, lock: &Arc<Mutex<()>>) -> Result<String> {
    let _guard = lock.lock().await;
    if let Ok(value) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
        if matches!(value.as_str(), "light" | "dark" | "auto") {
            return Ok(value);
        }
    }

    let path = config_file_path(app.config())?;
    if !path.exists() {
        return Ok("auto".into());
    }

    let bytes = fs::read(&path).await?;
    if bytes.is_empty() {
        return Ok("auto".into());
    }

    let store: ConfigStore = serde_json::from_slice(&bytes).unwrap_or_default();
    Ok(store.entries.get("theme").cloned().unwrap_or_else(|| "auto".into()))
}

pub async fn save_config_internal(
    app: &tauri::AppHandle,
    key: &str,
    value: &str,
    lock: &Arc<Mutex<()>>,
) -> Result<()> {
    let _guard = lock.lock().await;
    let path = config_file_path(app.config())?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let mut store = if path.exists() {
        let bytes = fs::read(&path).await?;
        serde_json::from_slice::<ConfigStore>(&bytes).unwrap_or_default()
    } else {
        ConfigStore::default()
    };

    store.entries.insert(key.to_string(), value.to_string());
    let json = serde_json::to_vec_pretty(&store)?;
    let mut file = fs::File::create(&path).await?;
    file.write_all(&json).await?;
    file.flush().await?;
    Ok(())
}

pub fn emit_theme_update(app: &tauri::AppHandle) {
    let _ = app.emit_all("theme-changed", ());
}

fn config_file_path(config: &tauri::Config) -> Result<PathBuf> {
    let base_dir = tauri::api::path::app_config_dir(config).context("无法确定配置目录")?;
    Ok(base_dir.join("vattool-config.json"))
}
