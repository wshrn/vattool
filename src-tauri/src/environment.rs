use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use url::Url;

use crate::state::FileWriteLock;

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub python_found: bool,
    pub python_path: Option<String>,
    pub python3_env_exists: bool,
    pub python3_env_value: Option<String>,
    pub path_has_python: bool,
    pub path_has_scripts: bool,
    pub pip_configured: bool,
    pub pip_index_url: Option<String>,
    pub messages: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InitializationResult {
    pub success: bool,
    pub message: String,
    pub status: Option<EnvironmentStatus>,
}

#[tauri::command]
pub async fn get_environment_status() -> Result<EnvironmentStatus, String> {
    determine_environment_status()
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn initialize_environment(
    mirror: String,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<InitializationResult, String> {
    let guard = lock.clone_arc();
    match perform_initialization(&mirror, &guard).await {
        Ok(message) => {
            let status = determine_environment_status()
                .await
                .map_err(|err| err.to_string())?;
            Ok(InitializationResult {
                success: true,
                message,
                status: Some(status),
            })
        }
        Err(error) => {
            let status = determine_environment_status().await.ok();
            Ok(InitializationResult {
                success: false,
                message: error.to_string(),
                status,
            })
        }
    }
}

async fn determine_environment_status() -> Result<EnvironmentStatus> {
    let current_dir = std::env::current_dir().context("无法获取当前目录")?;
    let python_candidates =
        ["python.exe", "python3.exe", "python"].map(|name| current_dir.join(name));
    let mut status = EnvironmentStatus::default();

    if let Some(path) = python_candidates
        .iter()
        .find(|candidate| candidate.exists())
    {
        status.python_found = true;
        status.python_path = Some(path.to_string_lossy().to_string());
    }

    if let Ok(value) = std::env::var("python3") {
        status.python3_env_exists = true;
        status.python3_env_value = Some(value.clone());
    }

    let path_entries = split_path_entries(std::env::var_os("PATH"));
    let python_dir = status
        .python_path
        .as_ref()
        .and_then(|p| Path::new(p).parent().map(|dir| dir.to_path_buf()));

    status.path_has_python = python_dir
        .as_ref()
        .map(|dir| path_contains(&path_entries, dir, "%python3%"))
        .unwrap_or(false);

    let scripts_dir = python_dir.map(|dir| dir.join("Scripts"));
    status.path_has_scripts = scripts_dir
        .as_ref()
        .map(|dir| path_contains(&path_entries, dir, "%python3%\\Scripts"))
        .unwrap_or(false);

    let (pip_configured, pip_index) = read_pip_mirror().await?;
    status.pip_configured = pip_configured;
    status.pip_index_url = pip_index;

    if !status.python_found {
        status
            .messages
            .push("未在工具所在目录检测到 python 可执行文件".into());
    }
    if !status.python3_env_exists {
        status
            .messages
            .push("建议创建名为 python3 的用户环境变量指向 Python 目录".into());
    }
    if !status.path_has_python {
        status
            .messages
            .push("PATH 中未检测到 python3 目录，请将 %python3% 添加至 PATH".into());
    }
    if !status.path_has_scripts {
        status
            .messages
            .push("PATH 中未检测到 python3\\Scripts 目录，请补充配置".into());
    }
    if !status.pip_configured {
        status
            .messages
            .push("未检测到 pip 国内镜像，建议使用工具进行初始化".into());
    }

    Ok(status)
}

async fn perform_initialization(mirror: &str, lock: &Arc<Mutex<()>>) -> Result<String> {
    let status = determine_environment_status().await?;
    let python_path = status
        .python_path
        .clone()
        .ok_or_else(|| anyhow!("未在当前目录找到 python 可执行文件"))?;

    let python_dir = Path::new(&python_path)
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("无法确定 Python 安装目录"))?;

    let python_dir_str = python_dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        set_user_env_var("python3", &python_dir_str)?;
        ensure_path_contains("%python3%")?;
        ensure_path_contains("%python3%\\Scripts")?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err(anyhow!("初始化流程仅在 Windows 上受支持"));
    }

    let mirror_url = mirror_url(mirror).ok_or_else(|| anyhow!("不支持的镜像源"))?;
    write_pip_config(mirror_url, lock).await?;

    Ok(format!(
        "已完成 Python 环境初始化，镜像源设置为 {mirror_url}"
    ))
}

fn mirror_url(key: &str) -> Option<&'static str> {
    let mut mirrors: HashMap<&'static str, &'static str> = HashMap::new();
    mirrors.insert("tsinghua", "https://pypi.tuna.tsinghua.edu.cn/simple");
    mirrors.insert("ustc", "https://pypi.mirrors.ustc.edu.cn/simple");
    mirrors.insert("aliyun", "https://mirrors.aliyun.com/pypi/simple/");
    mirrors.insert(
        "huawei",
        "https://repo.huaweicloud.com/repository/pypi/simple",
    );
    mirrors.insert("tencent", "https://mirrors.cloud.tencent.com/pypi/simple");
    mirrors.get(key).copied()
}

async fn read_pip_mirror() -> Result<(bool, Option<String>)> {
    if let Some(path) = pip_config_path() {
        if path.exists() {
            let content = tokio::fs::read_to_string(path).await.unwrap_or_default();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("index-url") {
                    let value = trimmed.split('=').nth(1).map(|v| v.trim().to_string());
                    return Ok((true, value));
                }
            }
            return Ok((true, None));
        }
    }
    Ok((false, None))
}

async fn write_pip_config(url: &str, lock: &Arc<Mutex<()>>) -> Result<()> {
    let Some(path) = pip_config_path() else {
        return Err(anyhow!("无法确定 pip 配置文件路径"));
    };

    let host = Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|h| h.to_string()))
        .unwrap_or_default();

    let content = format!("[global]\nindex-url = {url}\ntrusted-host = {host}\n");

    let _guard = lock.lock().await;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::File::create(path).await?;
    file.write_all(content.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}

fn split_path_entries(path: Option<std::ffi::OsString>) -> Vec<String> {
    let Some(value) = path else {
        return Vec::new();
    };
    let separator = if cfg!(target_os = "windows") {
        ';'
    } else {
        ':'
    };
    value
        .to_string_lossy()
        .split(separator)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn path_contains(entries: &[String], expected: &Path, placeholder: &str) -> bool {
    let expected_str = expected.to_string_lossy().to_lowercase();
    entries.iter().any(|entry| {
        let entry_lower = entry.to_lowercase();
        entry_lower == expected_str
            || entry_lower.contains(&expected_str)
            || entry_lower == placeholder.to_lowercase()
    })
}

#[cfg(target_os = "windows")]
fn set_user_env_var(name: &str, value: &str) -> Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("无法打开用户环境变量注册表")?;
    env.set_value(name, &value).context("写入环境变量失败")?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn ensure_path_contains(entry: &str) -> Result<bool> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("无法打开用户环境变量注册表")?;
    let mut path: String = env.get_value("Path").unwrap_or_default();
    let separator = ';';
    let entries: Vec<String> = path
        .split(separator)
        .map(|s| s.trim().to_lowercase())
        .collect();
    if entries
        .iter()
        .any(|existing| existing == &entry.to_lowercase())
    {
        return Ok(false);
    }
    if !path.is_empty() && !path.ends_with(separator) {
        path.push(separator);
    }
    path.push_str(entry);
    env.set_value("Path", &path)
        .context("写入 Path 环境变量失败")?;
    Ok(true)
}

#[cfg(target_os = "windows")]
fn pip_config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|dir| dir.join("pip").join("pip.ini"))
}

#[cfg(not(target_os = "windows"))]
fn pip_config_path() -> Option<PathBuf> {
    tauri::api::path::home_dir().map(|dir| dir.join(".pip").join("pip.conf"))
}
