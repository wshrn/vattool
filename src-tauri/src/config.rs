use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use parking_lot::Mutex;
use tokio::fs;

const CONFIG_FILE_NAME: &str = "settings.json";

fn config_file_path() -> Result<PathBuf, anyhow::Error> {
    let mut dir = dirs_next::config_dir().ok_or_else(|| anyhow::anyhow!("无法定位配置目录"))?;
    dir.push("python-env-tool");
    std::fs::create_dir_all(&dir).context("创建配置目录失败")?;
    dir.push(CONFIG_FILE_NAME);
    Ok(dir)
}

async fn read_config_map() -> Result<HashMap<String, String>, anyhow::Error> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read(&path).await.context("读取配置文件失败")?;
    if content.is_empty() {
        return Ok(HashMap::new());
    }
    let map: HashMap<String, String> = serde_json::from_slice(&content).unwrap_or_default();
    Ok(map)
}

pub async fn read_config_value(key: &str) -> Result<String, String> {
    let map = read_config_map().await.map_err(|err| err.to_string())?;
    Ok(map.get(key).cloned().unwrap_or_else(|| "auto".to_string()))
}

pub async fn save_config_value(
    key: &str,
    value: &str,
    lock: &Arc<Mutex<()>>,
) -> Result<(), anyhow::Error> {
    let _guard = lock.lock();
    let mut map = read_config_map().await.unwrap_or_default();
    map.insert(key.to_string(), value.to_string());
    let path = config_file_path()?;
    let serialized = serde_json::to_vec_pretty(&map).context("序列化配置失败")?;
    fs::write(path, serialized).await.context("写入配置文件失败")
}
