use std::{collections::HashSet, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use tokio::task;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorOption {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub python_executable_found: bool,
    pub python_env_variable_ready: bool,
    pub python_in_path: bool,
    pub scripts_in_path: bool,
    pub pip_mirror: Option<String>,
    pub available_mirrors: Vec<MirrorOption>,
}

fn default_mirrors() -> Vec<MirrorOption> {
    vec![
        MirrorOption {
            label: "清华大学".into(),
            value: "https://pypi.tuna.tsinghua.edu.cn/simple".into(),
            description: Some("推荐，速度快".into()),
        },
        MirrorOption {
            label: "阿里云".into(),
            value: "https://mirrors.aliyun.com/pypi/simple/".into(),
            description: Some("阿里云开源镜像".into()),
        },
        MirrorOption {
            label: "腾讯云".into(),
            value: "https://mirrors.cloud.tencent.com/pypi/simple".into(),
            description: Some("腾讯开源镜像".into()),
        },
        MirrorOption {
            label: "华为云".into(),
            value: "https://repo.huaweicloud.com/repository/pypi/simple".into(),
            description: Some("华为云镜像".into()),
        },
    ]
}

pub async fn get_environment_status() -> Result<EnvironmentStatus> {
    task::spawn_blocking(|| detect_environment()).await?
}

fn detect_environment() -> Result<EnvironmentStatus> {
    let mirrors = default_mirrors();
    let (python_found, python_dir) = detect_python_executable();
    let python_env_ready = detect_python_env_variable(&python_dir);
    let python_in_path = detect_python_in_path(&python_dir)?;
    let scripts_in_path = detect_scripts_in_path(&python_dir)?;
    let pip_mirror = detect_pip_mirror()?;

    Ok(EnvironmentStatus {
        python_executable_found: python_found,
        python_env_variable_ready: python_env_ready,
        python_in_path,
        scripts_in_path,
        pip_mirror,
        available_mirrors: mirrors,
    })
}

fn detect_python_executable() -> (bool, Option<PathBuf>) {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return (false, None),
    };

    let candidates = [
        current_dir.join("python.exe"),
        current_dir.join("python3.exe"),
        current_dir.join("python"),
        current_dir.join("python3"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return (true, candidate.parent().map(|p| p.to_path_buf()));
        }
    }

    (false, None)
}

fn detect_python_env_variable(python_dir: &Option<PathBuf>) -> bool {
    if let Ok(value) = std::env::var("python3") {
        if !value.is_empty() {
            return true;
        }
    }

    if let Some(dir) = python_dir {
        if let Some(current) = dir.to_str() {
            if let Ok(var) = std::env::var("PYTHON_HOME") {
                return var == current;
            }
        }
    }

    false
}

fn detect_python_in_path(python_dir: &Option<PathBuf>) -> Result<bool> {
    let Some(dir) = python_dir else {
        return Ok(false);
    };

    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let path_entries: HashSet<PathBuf> = std::env::split_paths(&path_var).collect();

    Ok(path_entries.contains(dir))
}

fn detect_scripts_in_path(python_dir: &Option<PathBuf>) -> Result<bool> {
    let Some(dir) = python_dir else {
        return Ok(false);
    };

    let scripts_dir = dir.join("Scripts");
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let path_entries: HashSet<PathBuf> = std::env::split_paths(&path_var).collect();

    Ok(path_entries.contains(&scripts_dir))
}

fn detect_pip_mirror() -> Result<Option<String>> {
    if let Ok(url) = std::env::var("PIP_INDEX_URL") {
        if !url.trim().is_empty() {
            return Ok(Some(url));
        }
    }

    if let Some(home) = home_dir() {
        let windows_config = home.join("AppData/Local/pip/pip.ini");
        let user_config = home.join("pip/pip.ini");
        let unix_config = home.join(".pip/pip.conf");

        for path in [windows_config, user_config, unix_config] {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(url) = parse_index_url(&content) {
                    return Ok(Some(url));
                }
            }
        }
    }

    Ok(None)
}

fn parse_index_url(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("index-url") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let url = parts[1].trim();
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
    }
    None
}

pub async fn initialize_python_environment(mirror: MirrorOption) -> Result<EnvironmentStatus> {
    task::spawn_blocking(move || initialize_environment_blocking(mirror)).await?
}

fn initialize_environment_blocking(mirror: MirrorOption) -> Result<EnvironmentStatus> {
    let (python_found, python_dir) = detect_python_executable();
    if !python_found {
        anyhow::bail!("未找到 Python 可执行文件，请确认工具与 python.exe 同目录");
    }
    let python_dir = python_dir.context("无法确定 Python 安装目录")?;

    persist_python_env(&python_dir)?;
    persist_path_entries(&python_dir)?;
    persist_pip_mirror(&mirror.value)?;

    detect_environment()
}

fn persist_python_env(python_dir: &PathBuf) -> Result<()> {
    let path_str = python_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Python 安装路径包含非法字符"))?;

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (env_key, _) = hkcu.create_subkey("Environment")?;
        env_key.set_value("python3", &path_str)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::env::set_var("python3", path_str);
    }

    Ok(())
}

fn persist_path_entries(python_dir: &PathBuf) -> Result<()> {
    let python_path = python_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Python 安装路径包含非法字符"))?;
    let scripts_path = python_dir.join("Scripts");
    let scripts_path_str = scripts_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Scripts 目录路径包含非法字符"))?;

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{RegType, HKEY_CURRENT_USER};
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (env_key, _) = hkcu.create_subkey("Environment")?;
        let existing: String = env_key.get_value("Path").unwrap_or_default();
        let mut entries: Vec<String> = existing
            .split(';')
            .filter(|entry| !entry.is_empty())
            .map(|s| s.to_string())
            .collect();

        if !entries.iter().any(|entry| entry.eq_ignore_ascii_case(python_path)) {
            entries.push(python_path.to_string());
        }

        if !entries
            .iter()
            .any(|entry| entry.eq_ignore_ascii_case(scripts_path_str))
        {
            entries.push(scripts_path_str.to_string());
        }

        let new_path = entries.join(";");
        env_key.set_value_with_type("Path", &new_path, &RegType::REG_EXPAND_SZ)?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut paths: Vec<PathBuf> = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).collect();
        if !paths.contains(python_dir) {
            paths.push(python_dir.clone());
        }
        if !paths.contains(&scripts_path) {
            paths.push(scripts_path.clone());
        }
        let new_path = std::env::join_paths(paths)?;
        std::env::set_var("PATH", &new_path);
    }

    Ok(())
}

fn persist_pip_mirror(mirror: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let home = home_dir().context("无法找到用户目录")?;
        let pip_dir = home.join("AppData/Local/pip");
        std::fs::create_dir_all(&pip_dir)?;
        let pip_ini = pip_dir.join("pip.ini");
        let content = format!("[global]\nindex-url = {mirror}\ntrusted-host = {}\n", extract_host(mirror));
        std::fs::write(pip_ini, content)?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = home_dir().context("无法找到用户目录")?;
        let pip_dir = home.join(".pip");
        std::fs::create_dir_all(&pip_dir)?;
        let pip_conf = pip_dir.join("pip.conf");
        let content = format!("[global]\nindex-url = {mirror}\ntrusted-host = {}\n", extract_host(mirror));
        std::fs::write(pip_conf, content)?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

fn extract_host(url: &str) -> String {
    url.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or_default()
        .to_string()
}
