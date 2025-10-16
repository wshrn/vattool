use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(target_os = "windows")]
use std::process::Command;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use serde::Serialize;
use url::Url;

use crate::FileWriteLock;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEnvironmentStatus {
    pub python_exists: bool,
    pub python_env_var_set: bool,
    pub python_path_set: bool,
    pub scripts_path_set: bool,
    pub mirror_configured: bool,
    pub mirror_name: String,
    pub python_path: Option<String>,
    pub scripts_path: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonInitResult {
    pub success: bool,
    pub message: String,
    pub status: PythonEnvironmentStatus,
}

#[tauri::command]
pub async fn python_environment_status() -> Result<PythonEnvironmentStatus, String> {
    tokio::task::spawn_blocking(|| detect_python_status())
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn initialize_python_environment(
    mirror: String,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<PythonInitResult, String> {
    let shared = lock.0.clone();
    tokio::task::spawn_blocking(move || initialize_environment_blocking(mirror, &shared))
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())
}

fn detect_python_status() -> Result<PythonEnvironmentStatus> {
    let exe_dir = current_app_dir()?;
    let python_path = python_executable_path(&exe_dir);
    let python_exists = python_path.exists();
    let python_dir = if python_exists {
        python_path.parent().map(|p| p.to_path_buf())
    } else {
        None
    };
    let scripts_path = python_dir.as_ref().map(|dir| dir.join("Scripts"));
    let python_exe_display = if python_exists {
        Some(python_path.to_string_lossy().to_string())
    } else {
        None
    };

    let env_python3 = std::env::var("PYTHON3").ok();
    let python_env_var_set = match (&env_python3, &python_dir) {
        (Some(value), Some(dir)) => normalize_path(value) == normalize_path(dir.to_string_lossy()),
        _ => false,
    };

    let path_entries = split_paths(std::env::var_os("PATH"));
    let python_path_set = python_dir
        .as_ref()
        .map(|dir| contains_path(&path_entries, dir))
        .unwrap_or(false);
    let scripts_path_set = scripts_path
        .as_ref()
        .map(|dir| contains_path(&path_entries, dir))
        .unwrap_or(false);

    let (mirror_configured, mirror_name, pip_details) = detect_pip_mirror();
    let mut details = Vec::new();
    if let Some(detail) = pip_details {
        details.push(detail);
    }
    if let Some(ref exe_display) = python_exe_display {
        details.push(format!("python.exe 路径：{exe_display}"));
    }
    let detail_text = if details.is_empty() {
        None
    } else {
        Some(details.join("\n"))
    };

    Ok(PythonEnvironmentStatus {
        python_exists,
        python_env_var_set,
        python_path_set,
        scripts_path_set,
        mirror_configured,
        mirror_name,
        python_path: python_exe_display,
        scripts_path: scripts_path.map(|p| p.to_string_lossy().to_string()),
        details: detail_text,
    })
}

fn initialize_environment_blocking(mirror: String, lock: &Arc<Mutex<()>>) -> Result<PythonInitResult> {
    let status_before = detect_python_status()?;
    if !status_before.python_exists {
        return Ok(PythonInitResult {
            success: false,
            message: "未检测到 python.exe，请确认工具与 Python 同目录运行".into(),
            status: status_before,
        });
    }

    let python_exe = PathBuf::from(
        status_before
            .python_path
            .clone()
            .ok_or_else(|| anyhow!("无法解析 Python 路径"))?,
    );
    let python_dir_path = python_exe
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("无法解析 Python 目录"))?;
    let scripts_dir_path = python_dir_path.join("Scripts");
    let python_dir = python_dir_path.to_string_lossy().to_string();
    let scripts_dir = scripts_dir_path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        set_user_env_variable("PYTHON3", &python_dir)?;
        ensure_user_path_contains(&python_dir)?;
        ensure_user_path_contains(&scripts_dir)?;
        update_process_environment(&python_dir, &scripts_dir);
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err(anyhow!("该初始化流程仅支持 Windows 系统"));
    }

    let host = Url::parse(&mirror)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_string()))
        .ok_or_else(|| anyhow!("镜像地址无效"))?;

    {
        let _guard = lock.lock();
        write_pip_config(&mirror, &host)?;
    }

    let status_after = detect_python_status()?;
    Ok(PythonInitResult {
        success: true,
        message: format!("Python 环境初始化完成，已配置镜像源 {host}"),
        status: status_after,
    })
}

fn current_app_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("无法定位当前可执行文件")?;
    exe.parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("无法确定可执行文件所在目录"))
}

fn python_executable_path(base: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        base.join("python.exe")
    }
    #[cfg(not(target_os = "windows"))]
    {
        base.join("python3")
    }
}

fn split_paths(variable: Option<std::ffi::OsString>) -> HashSet<String> {
    variable
        .and_then(|value| value.into_string().ok())
        .map(|value| {
            value
                .split(if cfg!(target_os = "windows") { ';' } else { ':' })
                .filter_map(|segment| {
                    let trimmed = segment.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(normalize_path(trimmed))
                    }
                })
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default()
}

fn contains_path(entries: &HashSet<String>, target: &Path) -> bool {
    let normalized = normalize_path(target.to_string_lossy());
    entries.contains(&normalized)
}

fn normalize_path<T: AsRef<str>>(value: T) -> String {
    let mut path = value.as_ref().replace('\', "/");
    if cfg!(target_os = "windows") {
        path = path.to_ascii_lowercase();
    }
    path.trim_end_matches('/').to_string()
}

fn detect_pip_mirror() -> (bool, String, Option<String>) {
    let path = pip_config_path();
    if let Some(path) = path {
        if let Ok(content) = fs::read_to_string(&path) {
            let mut index_url: Option<String> = None;
            let mut trusted_host: Option<String> = None;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("index-url") {
                    if let Some((_, value)) = trimmed.split_once('=') {
                        index_url = Some(value.trim().to_string());
                    }
                }
                if trimmed.starts_with("trusted-host") {
                    if let Some((_, value)) = trimmed.split_once('=') {
                        trusted_host = Some(value.trim().to_string());
                    }
                }
            }
            if let Some(url) = index_url {
                let host = trusted_host.unwrap_or_else(|| url.clone());
                let info = format!("pip.ini 路径：{}", path.display());
                return (true, host, Some(info));
            }
        }
        let info = format!("pip.ini 路径：{}", path.display());
        return (false, String::from("未配置"), Some(info));
    }
    (false, String::from("未配置"), None)
}

fn pip_config_path() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        dirs_next::data_dir().map(|mut dir| {
            dir.push("pip");
            dir.push("pip.ini");
            dir
        })
    } else {
        dirs_next::config_dir().map(|mut dir| {
            dir.push("pip");
            dir.push("pip.conf");
            dir
        })
    }
}

fn write_pip_config(mirror: &str, host: &str) -> Result<()> {
    let path = pip_config_path().ok_or_else(|| anyhow!("无法确定 pip 配置路径"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("创建 pip 配置目录失败")?;
    }
    let content = format!(
        "[global]\nindex-url = {mirror}\ntrusted-host = {host}\n" \
        "timeout = 60\n[install]\ntrusted-host = {host}\n"
    );
    fs::write(&path, content).context("写入 pip 配置失败")?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_user_env_variable(key: &str, value: &str) -> Result<()> {
    let escaped_value = value.replace('\'', "''");
    let script = format!(
        "[Environment]::SetEnvironmentVariable('{key}', '{escaped_value}', 'User')"
    );
    run_powershell(&script).context(format!("设置环境变量 {key} 失败"))
}

#[cfg(not(target_os = "windows"))]
fn set_user_env_variable(_key: &str, _value: &str) -> Result<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn ensure_user_path_contains(target: &str) -> Result<()> {
    let escaped = target.replace('\'', "''");
    let script = format!(
        "$current = [Environment]::GetEnvironmentVariable('Path','User');\n\
         if ([string]::IsNullOrEmpty($current)) {{ $segments = @() }} else {{ $segments = $current.Split(';') }};\n\
         if ($segments -notcontains '{escaped}') {{\n\
             $segments += '{escaped}';\n\
             [Environment]::SetEnvironmentVariable('Path',[string]::Join(';',$segments),'User')\n\
         }}"
    );
    run_powershell(&script).context("更新 PATH 失败")
}

#[cfg(not(target_os = "windows"))]
fn ensure_user_path_contains(_target: &str) -> Result<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn update_process_environment(python_dir: &str, scripts_dir: &str) {
    std::env::set_var("PYTHON3", python_dir);
    let mut segments: Vec<String> = std::env::var("PATH")
        .unwrap_or_default()
        .split(';')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect();

    if !segments
        .iter()
        .any(|entry| normalize_path(entry) == normalize_path(python_dir))
    {
        segments.push(python_dir.to_string());
    }

    if !segments
        .iter()
        .any(|entry| normalize_path(entry) == normalize_path(scripts_dir))
    {
        segments.push(scripts_dir.to_string());
    }

    std::env::set_var("PATH", segments.join(";"));
}

#[cfg(not(target_os = "windows"))]
fn update_process_environment(_python_dir: &str, _scripts_dir: &str) {}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<()> {
    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .status()
        .context("执行 PowerShell 命令失败")?;
    if !status.success() {
        return Err(anyhow!("PowerShell 命令执行失败"));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn run_powershell(_script: &str) -> Result<()> {
    Ok(())
}
