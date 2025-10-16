use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
#[cfg(target_os = "windows")]
use anyhow::Context;
#[cfg(target_os = "windows")]
use async_process::Command;
use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusInfo {
  pub ok: bool,
  pub message: String,
  pub level: StatusLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StatusLevel {
  Success,
  Warning,
  Error,
  Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonStatusReport {
  pub python_executable: StatusInfo,
  pub python_root_variable: StatusInfo,
  pub python_path: StatusInfo,
  pub python_scripts: StatusInfo,
  pub python_mirror: StatusInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorOption {
  pub id: String,
  pub label: String,
  pub index_url: String,
  pub trusted_host: String,
}

struct DetectionContext {
  python_executable: Option<PathBuf>,
  python_dir: Option<PathBuf>,
  scripts_dir: Option<PathBuf>,
}

impl DetectionContext {
  fn new() -> Result<Self> {
    let current_dir = std::env::current_dir()?;
    let executables = ["python.exe", "python3.exe", "python"];
    for name in executables {
      let candidate = current_dir.join(name);
      if candidate.exists() {
        let python_dir = candidate.parent().map(|p| p.to_path_buf());
        let scripts_dir = python_dir.clone().map(|mut p| {
          p.push("Scripts");
          p
        });
        return Ok(Self {
          python_executable: Some(candidate),
          python_dir,
          scripts_dir,
        });
      }
    }

    Ok(Self {
      python_executable: None,
      python_dir: None,
      scripts_dir: None,
    })
  }
}

fn create_status(ok: bool, message: impl Into<String>, level: StatusLevel) -> StatusInfo {
  StatusInfo {
    ok,
    message: message.into(),
    level,
  }
}

fn normalize_path(path: &Path) -> String {
  if cfg!(target_os = "windows") {
    path.display().to_string().to_lowercase()
  } else {
    path.display().to_string()
  }
}

fn path_contains(path: &str, target: &Path) -> bool {
  let normalized_target = normalize_path(target);
  let separator = if cfg!(target_os = "windows") { ';' } else { ':' };
  path.split(separator)
    .map(|segment| {
      let trimmed = segment.trim();
      if cfg!(target_os = "windows") {
        trimmed.to_lowercase()
      } else {
        trimmed.to_string()
      }
    })
    .any(|segment| {
      if cfg!(target_os = "windows") {
        segment == normalized_target
      } else {
        segment == normalized_target
      }
    })
}

fn pip_config_paths() -> Vec<PathBuf> {
  let mut paths = Vec::new();

  if cfg!(target_os = "windows") {
    if let Ok(appdata) = std::env::var("APPDATA") {
      let mut path = PathBuf::from(appdata);
      path.push("pip");
      path.push("pip.ini");
      paths.push(path);
    }
  }

  if let Some(mut home) = home_dir() {
    home.push(".pip");
    home.push(if cfg!(target_os = "windows") { "pip.ini" } else { "pip.conf" });
    paths.push(home);
  }

  paths
}

async fn read_mirror_config() -> Result<Option<String>> {
  for path in pip_config_paths() {
    if path.exists() {
      let content = fs::read_to_string(&path).await?;
      for line in content.lines() {
        let line = line.trim();
        if line.to_lowercase().starts_with("index-url") {
          if let Some((_, value)) = line.split_once('=') {
            return Ok(Some(value.trim().to_string()));
          }
        }
      }
    }
  }
  Ok(None)
}

async fn write_mirror_config(index_url: &str, trusted_host: &str) -> Result<()> {
  let path = pip_config_paths()
    .into_iter()
    .next()
    .ok_or_else(|| anyhow!("无法定位 pip 配置目录"))?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).await?;
  }
  let content = format!(
    "[global]\nindex-url = {index_url}\ntrusted-host = {trusted_host}\n"
  );
  fs::write(path, content).await?;
  Ok(())
}

pub async fn collect_status(preferred_mirror: Option<String>) -> Result<PythonStatusReport> {
  let context = DetectionContext::new()?;
  let python_status = if let Some(exe) = &context.python_executable {
    create_status(true, format!("已检测到 {}", exe.display()), StatusLevel::Success)
  } else {
    create_status(false, "当前目录未找到 python 可执行文件", StatusLevel::Error)
  };

  let python_dir_status = if let Some(dir) = &context.python_dir {
    match std::env::var("PYTHON3") {
      Ok(value) => {
        if normalize_path(Path::new(&value)) == normalize_path(dir.as_path()) {
          create_status(true, format!("PYTHON3 已指向 {}", dir.display()), StatusLevel::Success)
        } else {
          create_status(false, format!("PYTHON3 指向 {value}, 建议修改为 {}", dir.display()), StatusLevel::Warning)
        }
      }
      Err(_) => create_status(false, "未设置 PYTHON3 环境变量", StatusLevel::Error),
    }
  } else {
    create_status(false, "未找到 Python 目录，无法检测 PYTHON3 变量", StatusLevel::Error)
  };

  let path_env = std::env::var("PATH").unwrap_or_default();
  let python_path_status = if let Some(dir) = &context.python_dir {
    if path_contains(&path_env, dir.as_path()) {
      create_status(true, format!("PATH 已包含 {}", dir.display()), StatusLevel::Success)
    } else {
      create_status(false, format!("PATH 缺少 {}", dir.display()), StatusLevel::Warning)
    }
  } else {
    create_status(false, "无法检测 Python 路径", StatusLevel::Error)
  };

  let scripts_path_status = if let Some(dir) = &context.scripts_dir {
    if path_contains(&path_env, dir.as_path()) {
      create_status(true, format!("PATH 已包含 {}", dir.display()), StatusLevel::Success)
    } else {
      create_status(false, format!("PATH 缺少 {}", dir.display()), StatusLevel::Warning)
    }
  } else {
    create_status(false, "未找到 Scripts 目录", StatusLevel::Warning)
  };

  let preferred = preferred_mirror.unwrap_or_default();
  let mirror_status = match read_mirror_config().await? {
    Some(url) => {
      if preferred.is_empty() || url == preferred {
        create_status(true, format!("pip 源已配置为 {url}"), StatusLevel::Success)
      } else {
        create_status(false, format!("pip 源当前为 {url}"), StatusLevel::Warning)
      }
    }
    None => create_status(false, "未检测到 pip 国内源配置", StatusLevel::Warning),
  };

  Ok(PythonStatusReport {
    python_executable: python_status,
    python_root_variable: python_dir_status,
    python_path: python_path_status,
    python_scripts: scripts_path_status,
    python_mirror: mirror_status,
  })
}

async fn ensure_windows_environment_variable(name: &str, value: &str) -> Result<()> {
  #[cfg(target_os = "windows")]
  {
    let sanitized = value.replace("'", "''");
    let script = format!(
      "$current = [Environment]::GetEnvironmentVariable('{name}', 'User'); \n[Environment]::SetEnvironmentVariable('{name}', '{sanitized}', 'User');"
    );
    let status = Command::new("powershell")
      .args(["-NoProfile", "-Command", &script])
      .status()
      .await
      .context(format!("设置环境变量 {name} 失败"))?;
    if status.success() {
      Ok(())
    } else {
      Err(anyhow!(format!("设置环境变量 {name} 失败")))
    }
  }
  #[cfg(not(target_os = "windows"))]
  {
    let mut env_file = home_dir().ok_or_else(|| anyhow!("无法定位用户目录"))?;
    env_file.push(".python-env-switcher");
    let line = format!("export {name}='{value}'\n");
    fs::write(env_file, line).await?;
    Ok(())
  }
}

async fn ensure_path_contains(target: &Path) -> Result<()> {
  #[cfg(target_os = "windows")]
  {
    let target_str = target.display().to_string();
    let sanitized = target_str.replace("'", "''");
    let lower = target_str.to_lowercase();
    let lower_sanitized = lower.replace("'", "''");
    let script = format!(
      "$current = [Environment]::GetEnvironmentVariable('Path', 'User');\n$segments = @();\nif ($current) { $segments = $current.Split(';') }\n$found = $false;\nforeach ($segment in $segments) { if ($segment.ToLower() -eq '{lower_sanitized}') { $found = $true } }\nif (-not $found) {\n  if ([string]::IsNullOrEmpty($current)) { $next = '{sanitized}' } else { $next = $current + ';{sanitized}' }\n  [Environment]::SetEnvironmentVariable('Path', $next, 'User');\n}\"
    );
    let status = Command::new("powershell")
      .args(["-NoProfile", "-Command", &script])
      .status()
      .await
      .context("更新 PATH 失败")?;
    if status.success() {
      Ok(())
    } else {
      Err(anyhow!("更新 PATH 失败"))
    }
  }
  #[cfg(not(target_os = "windows"))]
  {
    let mut env_file = home_dir().ok_or_else(|| anyhow!("无法定位用户目录"))?;
    env_file.push(".python-env-switcher-path");
    let line = format!("export PATH='{}:$PATH'\n", target.display());
    fs::write(env_file, line).await?;
    Ok(())
  }
}

pub async fn initialize_environment(mirror: MirrorOption) -> Result<()> {
  let context = DetectionContext::new()?;
  let python_dir = context
    .python_dir
    .ok_or_else(|| anyhow!("当前目录未找到 Python 可执行文件"))?;
  let scripts_dir = context
    .scripts_dir
    .ok_or_else(|| anyhow!("未找到 Scripts 目录"))?;

  ensure_windows_environment_variable("PYTHON3", &python_dir.display().to_string()).await?;
  ensure_path_contains(&python_dir).await?;
  ensure_path_contains(&scripts_dir).await?;
  write_mirror_config(&mirror.index_url, &mirror.trusted_host).await?;
  Ok(())
}
