use std::{collections::HashSet, env, fs, path::{Path, PathBuf}};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use tokio::sync::Mutex;
use url::Url;

use crate::theme::FileWriteLock;

#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusLevel {
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEnvStatusItem {
    pub key: String,
    pub label: String,
    pub ok: bool,
    pub detail: Option<String>,
    pub level: StatusLevel,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorSource {
    pub key: String,
    pub label: String,
    pub url: String,
    pub description: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEnvStatusResponse {
    pub items: Vec<PythonEnvStatusItem>,
    pub mirror_sources: Vec<MirrorSource>,
    pub selected_mirror: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializationResult {
    pub success: bool,
    pub message: String,
    pub status: Option<PythonEnvStatusResponse>,
}

fn mirror_sources() -> Vec<MirrorSource> {
    vec![
        MirrorSource {
            key: "tsinghua".into(),
            label: "清华大学 TUNA 镜像".into(),
            url: "https://pypi.tuna.tsinghua.edu.cn/simple".into(),
            description: "由清华大学 TUNA 协会维护，速度快，默认推荐".into(),
        },
        MirrorSource {
            key: "aliyun".into(),
            label: "阿里云镜像".into(),
            url: "https://mirrors.aliyun.com/pypi/simple".into(),
            description: "阿里云开发者服务提供，覆盖面广".into(),
        },
        MirrorSource {
            key: "tencent".into(),
            label: "腾讯云镜像".into(),
            url: "https://mirrors.cloud.tencent.com/pypi/simple".into(),
            description: "腾讯云提供的 PyPI 镜像".into(),
        },
        MirrorSource {
            key: "huawei".into(),
            label: "华为云镜像".into(),
            url: "https://repo.huaweicloud.com/repository/pypi/simple".into(),
            description: "华为云开发者服务镜像".into(),
        },
        MirrorSource {
            key: "ustc".into(),
            label: "中国科学技术大学镜像".into(),
            url: "https://pypi.mirrors.ustc.edu.cn/simple".into(),
            description: "由中国科学技术大学维护的镜像".into(),
        },
    ]
}

fn find_mirror<'a>(key: &str, sources: &'a [MirrorSource]) -> &'a MirrorSource {
    sources
        .iter()
        .find(|mirror| mirror.key == key)
        .unwrap_or_else(|| &sources[0])
}

fn find_python_in_dir(dir: &Path) -> Option<PathBuf> {
    let candidates = ["python.exe", "python3.exe", "python"];
    for candidate in candidates {
        let path = dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn detect_python_executable() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(path) = find_python_in_dir(parent) {
                return Some(path);
            }
        }
    }

    if let Ok(current) = std::env::current_dir() {
        if let Some(path) = find_python_in_dir(&current) {
            return Some(path);
        }
    }

    None
}

fn path_delimiter() -> char {
    if cfg!(windows) { ';' } else { ':' }
}

fn split_path_var(path: &str) -> Vec<String> {
    path
        .split(path_delimiter())
        .filter_map(|segment| {
            let trimmed = segment.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.trim_matches('"').to_string())
            }
        })
        .collect()
}

fn contains_case_insensitive(entries: &[String], target: &str) -> bool {
    let target_lower = target.to_ascii_lowercase();
    entries
        .iter()
        .any(|entry| entry.trim().eq_ignore_ascii_case(target) || entry.trim().to_ascii_lowercase() == target_lower)
}

fn directory_in_path(entries: &[String], dir: &Path) -> bool {
    if dir.as_os_str().is_empty() {
        return false;
    }
    let dir_str = dir.to_string_lossy().to_ascii_lowercase();
    entries
        .iter()
        .map(|entry| entry.trim().trim_matches('"').to_ascii_lowercase())
        .any(|entry| entry == dir_str)
}

fn pip_candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(appdata) = env::var("APPDATA") {
        paths.push(PathBuf::from(appdata).join("pip").join("pip.ini"));
    }
    if let Ok(userprofile) = env::var("USERPROFILE") {
        paths.push(PathBuf::from(userprofile).join("pip").join("pip.ini"));
    }
    if let Some(home) = tauri::api::path::home_dir() {
        paths.push(home.join(".pip").join("pip.ini"));
        paths.push(home.join(".pip").join("pip.conf"));
        paths.push(home.join(".config").join("pip").join("pip.conf"));
    }
    paths
}

fn parse_ini_value(content: &str, key: &str) -> Option<String> {
    let target = key.to_ascii_lowercase();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') || line.starts_with('[') {
            continue;
        }
        if let Some((raw_key, raw_value)) = line.split_once('=') {
            if raw_key.trim().eq_ignore_ascii_case(&target) {
                return Some(raw_value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

fn detect_pip_mirror(sources: &[MirrorSource]) -> (bool, Option<String>, Option<PathBuf>, Option<String>) {
    for path in pip_candidate_paths() {
        if !path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(value) = parse_ini_value(&content, "index-url") {
                for mirror in sources {
                    if value.contains(&mirror.url) {
                        return (true, Some(mirror.key.clone()), Some(path.clone()), Some(value));
                    }
                }
                return (false, None, Some(path.clone()), Some(value));
            }
            return (false, None, Some(path.clone()), None);
        }
    }
    (false, None, None, None)
}

fn gather_status_internal() -> Result<PythonEnvStatusResponse> {
    let sources = mirror_sources();
    let python_path = detect_python_executable();
    let python_dir = python_path.as_ref().and_then(|path| path.parent()).map(Path::to_path_buf);

    let mut items = Vec::new();

    let python_detail = python_path
        .as_ref()
        .map(|path| format!("Python 路径：{}", path.display()))
        .unwrap_or_else(|| "未在当前目录检测到 python 可执行文件".into());
    items.push(PythonEnvStatusItem {
        key: "python-executable".into(),
        label: "Python 可执行文件检测".into(),
        ok: python_path.is_some(),
        detail: Some(python_detail),
        level: if python_path.is_some() {
            StatusLevel::Success
        } else {
            StatusLevel::Error
        },
    });

    let python3_env = env::var_os("python3").map(PathBuf::from);
    let python3_ok = python3_env
        .as_ref()
        .map(|path| path.exists())
        .unwrap_or(false);
    let python3_detail = if let Some(path) = python3_env {
        if python3_ok {
            format!("python3 变量指向：{}", path.display())
        } else {
            format!("python3 变量值无效：{}", path.display())
        }
    } else {
        "未设置 python3 环境变量".into()
    };
    items.push(PythonEnvStatusItem {
        key: "python3-variable".into(),
        label: "python3 变量".into(),
        ok: python3_ok,
        detail: Some(python3_detail),
        level: if python3_ok {
            StatusLevel::Success
        } else {
            StatusLevel::Warning
        },
    });

    let (path_ok, scripts_ok, path_detail, scripts_detail) = {
        if let Ok(path_var) = env::var("PATH") {
            let entries = split_path_var(&path_var);
            let python_dir_ok = if let Some(dir) = &python_dir {
                directory_in_path(&entries, dir) || contains_case_insensitive(&entries, "%python3%")
            } else {
                false
            };
            let scripts_dir_ok = if let Some(dir) = &python_dir {
                let scripts_dir = dir.join("Scripts");
                directory_in_path(&entries, &scripts_dir)
                    || contains_case_insensitive(&entries, "%python3%\\Scripts")
            } else {
                false
            };

            let path_detail = if python_dir_ok {
                "Path 已包含 python 目录".into()
            } else if let Some(dir) = &python_dir {
                format!("Path 中缺少：{}", dir.display())
            } else {
                "缺少 python 目录信息".into()
            };

            let scripts_detail = if scripts_dir_ok {
                "Path 已包含 Scripts 目录".into()
            } else if let Some(dir) = &python_dir {
                format!("Path 中缺少：{}", dir.join("Scripts").display())
            } else {
                "缺少 Scripts 目录信息".into()
            };

            (python_dir_ok, scripts_dir_ok, path_detail, scripts_detail)
        } else {
            (false, false, "无法读取 PATH 环境变量".into(), "无法读取 PATH 环境变量".into())
        }
    };

    items.push(PythonEnvStatusItem {
        key: "python-path".into(),
        label: "Path 中的 python 目录".into(),
        ok: path_ok,
        detail: Some(path_detail),
        level: if path_ok {
            StatusLevel::Success
        } else {
            StatusLevel::Warning
        },
    });

    items.push(PythonEnvStatusItem {
        key: "python-scripts".into(),
        label: "Path 中的 Scripts 目录".into(),
        ok: scripts_ok,
        detail: Some(scripts_detail),
        level: if scripts_ok {
            StatusLevel::Success
        } else {
            StatusLevel::Warning
        },
    });

    let (pip_ok, detected_key, pip_path, detected_value) = detect_pip_mirror(&sources);
    let mut selected_mirror = detected_key.unwrap_or_else(|| "tsinghua".into());
    if !sources.iter().any(|mirror| mirror.key == selected_mirror) {
        selected_mirror = "tsinghua".into();
    }

    let pip_detail = if let Some(path) = pip_path {
        if let Some(value) = detected_value {
            format!("pip 配置文件：{}\nindex-url = {value}", path.display())
        } else {
            format!("pip 配置文件：{}\n缺少 index-url 配置", path.display())
        }
    } else {
        "未找到 pip 配置，将自动创建".into()
    };

    items.push(PythonEnvStatusItem {
        key: "pip-mirror".into(),
        label: "pip 国内镜像".into(),
        ok: pip_ok,
        detail: Some(pip_detail),
        level: if pip_ok {
            StatusLevel::Success
        } else {
            StatusLevel::Warning
        },
    });

    Ok(PythonEnvStatusResponse {
        items,
        mirror_sources: sources,
        selected_mirror,
    })
}

#[tauri::command]
pub async fn python_env_status() -> Result<PythonEnvStatusResponse, String> {
    gather_status_internal().map_err(|err| err.to_string())
}

fn pip_config_target() -> Result<PathBuf> {
    for path in pip_candidate_paths() {
        if path.parent().map(|dir| dir.exists()).unwrap_or(false) {
            return Ok(path);
        }
    }

    if let Some(home) = tauri::api::path::home_dir() {
        return Ok(home.join(".pip").join("pip.ini"));
    }

    Ok(std::env::current_dir()?.join("pip.ini"))
}

async fn write_pip_config(mirror: &MirrorSource, lock: &Mutex<()>) -> Result<PathBuf> {
    let path = pip_config_target()?;
    let _guard = lock.lock().await;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("无法创建 pip 配置目录: {:?}", parent))?;
    }

    let mut lines = vec!["[global]".to_string(), format!("index-url = {}", mirror.url)];
    if let Ok(parsed) = Url::parse(&mirror.url) {
        if let Some(host) = parsed.host_str() {
            lines.push(format!("trusted-host = {}", host));
        }
    }
    lines.push(String::new());

    fs::write(&path, lines.join("\n")).with_context(|| format!("无法写入 pip 配置文件: {:?}", path))?;
    Ok(path)
}

#[cfg(windows)]
fn update_windows_environment(python_dir: &Path) -> Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    use windows::core::w;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env_key, _) = hkcu.create_subkey("Environment")?;

    let python_dir_str = python_dir.to_string_lossy().to_string();
    env_key.set_value("python3", &python_dir_str)?;

    let mut path_value: String = env_key.get_value("Path").unwrap_or_default();
    let mut segments: Vec<String> = if path_value.trim().is_empty() {
        Vec::new()
    } else {
        path_value.split(';').map(|s| s.to_string()).collect()
    };

    ensure_path_entry(&mut segments, "%python3%");
    ensure_path_entry(&mut segments, "%python3%\\Scripts");

    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for segment in segments {
        let key = segment.trim().to_ascii_lowercase();
        if key.is_empty() {
            continue;
        }
        if seen.insert(key.clone()) {
            unique.push(segment);
        }
    }

    let new_path = unique.join(";");
    env_key.set_value("Path", &new_path)?;

    unsafe {
        let _ = SendMessageTimeoutW(
            HWND(HWND_BROADCAST.0),
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(w!("Environment").as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            5000,
            None,
        );
    }

    Ok(())
}

#[cfg(windows)]
fn ensure_path_entry(segments: &mut Vec<String>, entry: &str) {
    if !segments
        .iter()
        .any(|existing| existing.trim().eq_ignore_ascii_case(entry))
    {
        segments.push(entry.to_string());
    }
}

#[cfg(not(windows))]
fn update_windows_environment(_python_dir: &Path) -> Result<()> {
    Ok(())
}

#[cfg(not(windows))]
fn ensure_path_entry(_segments: &mut Vec<String>, _entry: &str) {}

pub async fn initialize_environment_internal(
    mirror_key: &str,
    lock: &Mutex<()>,
) -> Result<InitializationResult> {
    let sources = mirror_sources();
    let python_path = detect_python_executable();
    if python_path.is_none() {
        let status = gather_status_internal()?;
        return Ok(InitializationResult {
            success: false,
            message: "未检测到 python 可执行文件，请确认工具与 python 同目录".into(),
            status: Some(status),
        });
    }
    let python_path = python_path.unwrap();
    let python_dir = python_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("无法确定 python 目录"))?;

    update_windows_environment(&python_dir)?;

    let mirror = find_mirror(mirror_key, &sources).clone();
    write_pip_config(&mirror, lock).await?;

    let status = gather_status_internal()?;
    Ok(InitializationResult {
        success: true,
        message: format!("已完成环境变量和 pip 镜像配置（{}）", mirror.label),
        status: Some(status),
    })
}

#[tauri::command]
pub async fn initialize_python_environment(
    mirror_key: String,
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<InitializationResult, String> {
    initialize_environment_internal(&mirror_key, &lock.0)
        .await
        .map_err(|err| err.to_string())
}
