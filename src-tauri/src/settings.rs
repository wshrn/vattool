use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use tokio::task;

pub const TOOLBOX_THEME_ENV_NAME: &str = "ZHIGONG_TOOLBOX_THEME";

#[derive(Debug, Default, Serialize, Deserialize)]
struct AppConfig {
  #[serde(default)]
  theme: Option<String>,
  #[serde(flatten)]
  other: HashMap<String, String>,
}

fn config_path() -> Result<PathBuf> {
  if let Ok(path) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
    return Ok(PathBuf::from(path));
  }
  if let Some(mut dir) = tauri::api::path::config_dir() {
    dir.push("python-env-switcher");
    fs::create_dir_all(&dir).ok();
    dir.push("config.json");
    Ok(dir)
  } else {
    Err(anyhow!("无法确定配置目录"))
  }
}

async fn read_config() -> Result<AppConfig> {
  let path = config_path()?;
  if !path.exists() {
    return Ok(AppConfig::default());
  }
  let text = task::spawn_blocking(move || fs::read_to_string(path))
    .await
    .context("join error")??;
  let config: AppConfig = serde_json::from_str(&text).unwrap_or_default();
  Ok(config)
}

async fn write_config(config: &AppConfig, lock: &Arc<Mutex<()>>) -> Result<()> {
  let path = config_path()?;
  let json = serde_json::to_string_pretty(config)?;
  let lock = lock.clone();
  task::spawn_blocking(move || -> Result<(), anyhow::Error> {
    let _guard = lock.lock();
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(path, &json)?;
    Ok(())
  })
  .await
  .context("join error")??;
  Ok(())
}

pub async fn read_theme() -> Result<String> {
  if let Ok(value) = std::env::var(TOOLBOX_THEME_ENV_NAME) {
    if matches!(value.as_str(), "light" | "dark" | "auto") {
      return Ok(value);
    }
  }

  let config = read_config().await?;
  Ok(config.theme.unwrap_or_else(|| "auto".into()))
}

pub async fn save_config_internal(key: &str, value: &str, lock: &Arc<Mutex<()>>) -> Result<()> {
  let mut config = read_config().await?;
  match key {
    "theme" => config.theme = Some(value.to_string()),
    _ => {
      config.other.insert(key.to_string(), value.to_string());
    }
  }
  write_config(&config, lock).await
}

pub fn emit_theme_update(app: &AppHandle) {
  let _ = app.emit_all("theme-updated", ());
}
