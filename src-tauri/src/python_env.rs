use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Serialize;
use url::Url;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusLine {
    pub label: String,
    pub message: String,
    pub level: StatusLevel,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatusLevel {
    Success,
    Warning,
    Error,
    Info,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonMirror {
    pub id: String,
    pub label: String,
    pub index_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEnvironmentStatus {
    pub status_lines: Vec<StatusLine>,
    pub available_mirrors: Vec<PythonMirror>,
    pub selected_mirror: String,
    pub can_initialize: bool,
    pub running_on_windows: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializationResult {
    pub success: bool,
    pub message: String,
    pub status: Option<PythonEnvironmentStatus>,
}

const PYTHON_ENV_NAME: &str = "python3";
const DEFAULT_MIRROR_ID: &str = "tsinghua";

static PYTHON_MIRRORS: Lazy<Vec<PythonMirror>> = Lazy::new(|| {
    vec![
        PythonMirror {
            id: "tsinghua".into(),
            label: "清华大学开源镜像站".into(),
            index_url: "https://pypi.tuna.tsinghua.edu.cn/simple".into(),
        },
        PythonMirror {
            id: "aliyun".into(),
            label: "阿里云开源镜像站".into(),
            index_url: "https://mirrors.aliyun.com/pypi/simple".into(),
        },
        PythonMirror {
            id: "douban".into(),
            label: "豆瓣开源镜像站".into(),
            index_url: "https://pypi.douban.com/simple".into(),
        },
        PythonMirror {
            id: "tencent".into(),
            label: "腾讯云开源镜像站".into(),
            index_url: "https://mirrors.cloud.tencent.com/pypi/simple".into(),
        },
    ]
});

pub fn get_python_environment_status() -> PythonEnvironmentStatus {
    #[cfg(windows)]
    {
        collect_windows_status()
    }

    #[cfg(not(windows))]
    {
        PythonEnvironmentStatus {
            status_lines: vec![StatusLine {
                label: "系统兼容性".into(),
                message: "当前仅支持在 Windows 平台执行环境变量初始化".into(),
                level: StatusLevel::Warning,
            }],
            available_mirrors: PYTHON_MIRRORS.clone(),
            selected_mirror: DEFAULT_MIRROR_ID.into(),
            can_initialize: false,
            running_on_windows: false,
        }
    }
}

pub fn initialize_python_environment(
    mirror_id: &str,
    lock: &Arc<Mutex<()>>,
) -> InitializationResult {
    #[cfg(windows)]
    {
        match initialize_windows_environment(mirror_id, lock) {
            Ok(_) => InitializationResult {
                success: true,
                message: "Python 环境初始化完成".into(),
                status: Some(collect_windows_status()),
            },
            Err(err) => InitializationResult {
                success: false,
                message: format!("初始化失败: {err}"),
                status: Some(collect_windows_status()),
            },
        }
    }

    #[cfg(not(windows))]
    {
        InitializationResult {
            success: false,
            message: "当前平台不支持自动初始化".into(),
            status: Some(get_python_environment_status()),
        }
    }
}

fn collect_windows_status() -> PythonEnvironmentStatus {
    #[cfg(windows)]
    {
        let exe_dir = current_directory();
        let python_exists = exe_dir
            .as_ref()
            .map(|dir| dir.join("python.exe").exists())
            .unwrap_or(false);

        let mut lines = Vec::new();

        lines.push(if python_exists {
            StatusLine {
                label: "Python 可执行文件".into(),
                message: "已在当前目录检测到 python.exe".into(),
                level: StatusLevel::Success,
            }
        } else {
            StatusLine {
                label: "Python 可执行文件".into(),
                message: "未在当前目录检测到 python.exe".into(),
                level: StatusLevel::Error,
            }
        });

        let python_env_value = read_user_env(PYTHON_ENV_NAME).ok().flatten();
        let python_env_status = match (&exe_dir, &python_env_value) {
            (Some(dir), Some(value)) => {
                let normalized_dir = normalize_path(dir);
                if normalize_str(&value) == normalize_str(&normalized_dir) {
                    StatusLine {
                        label: "python3 目录变量".into(),
                        message: "python3 环境变量已设置".into(),
                        level: StatusLevel::Success,
                    }
                } else {
                    StatusLine {
                        label: "python3 目录变量".into(),
                        message: "python3 变量未指向当前目录".into(),
                        level: StatusLevel::Warning,
                    }
                }
            }
            (Some(_), None) => StatusLine {
                label: "python3 目录变量".into(),
                message: "未设置 python3 环境变量".into(),
                level: StatusLevel::Warning,
            },
            _ => StatusLine {
                label: "python3 目录变量".into(),
                message: "未检测到工具所在目录".into(),
                level: StatusLevel::Error,
            },
        };
        lines.push(python_env_status);

        let path_status = read_user_env("Path").ok().flatten().unwrap_or_default();
        let has_python_path = path_contains(&path_status, "%python3%")
            || exe_dir
                .as_ref()
                .map(|dir| path_contains(&path_status, &dir.to_string_lossy()))
                .unwrap_or(false);
        lines.push(if has_python_path {
            StatusLine {
                label: "PATH 中的 Python".into(),
                message: "PATH 中已包含 python3 目录".into(),
                level: StatusLevel::Success,
            }
        } else {
            StatusLine {
                label: "PATH 中的 Python".into(),
                message: "PATH 中未包含 python3 目录".into(),
                level: StatusLevel::Warning,
            }
        });

        let has_scripts_path = path_contains(&path_status, "%python3%\\Scripts")
            || exe_dir
                .as_ref()
                .map(|dir| {
                    let scripts = dir.join("Scripts");
                    path_contains(&path_status, &scripts.to_string_lossy())
                })
                .unwrap_or(false);
        lines.push(if has_scripts_path {
            StatusLine {
                label: "PATH 中的 Scripts".into(),
                message: "PATH 中已包含 Scripts 目录".into(),
                level: StatusLevel::Success,
            }
        } else {
            StatusLine {
                label: "PATH 中的 Scripts".into(),
                message: "PATH 中未包含 Scripts 目录".into(),
                level: StatusLevel::Warning,
            }
        });

        let (selected_mirror, mirror_status) = detect_mirror_status();
        lines.push(mirror_status);

        PythonEnvironmentStatus {
            status_lines: lines,
            available_mirrors: PYTHON_MIRRORS.clone(),
            selected_mirror,
            can_initialize: python_exists,
            running_on_windows: true,
        }
    }

    #[cfg(not(windows))]
    {
        unreachable!()
    }
}

#[cfg(windows)]
fn current_directory() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
}

#[cfg(windows)]
fn initialize_windows_environment(mirror_id: &str, lock: &Arc<Mutex<()>>) -> Result<(), String> {
    let exe_dir = current_directory().ok_or_else(|| "无法确定可执行文件目录".to_string())?;
    let python_path = exe_dir.join("python.exe");
    if !python_path.exists() {
        return Err("当前目录缺少 python.exe".into());
    }

    set_user_env(PYTHON_ENV_NAME, exe_dir.to_string_lossy().as_ref())
        .map_err(|err| format!("设置 python3 变量失败: {err}"))?;

    let mut path_value = read_user_env("Path")
        .map_err(|err| format!("读取 PATH 失败: {err}"))?
        .unwrap_or_else(|| String::new());
    let python_marker = "%python3%";
    let scripts_marker = "%python3%\\Scripts";

    if !path_contains(&path_value, python_marker) {
        append_path(&mut path_value, python_marker);
    }

    if !path_contains(&path_value, scripts_marker) {
        append_path(&mut path_value, scripts_marker);
    }

    set_user_env("Path", &path_value).map_err(|err| format!("写入 PATH 失败: {err}"))?;

    configure_pip_mirror(mirror_id, lock)?;
    broadcast_environment_change();
    Ok(())
}

#[cfg(windows)]
fn append_path(path_value: &mut String, segment: &str) {
    if !path_value.is_empty() && !path_value.ends_with(';') {
        path_value.push(';');
    }
    path_value.push_str(segment);
}

#[cfg(windows)]
fn configure_pip_mirror(mirror_id: &str, lock: &Arc<Mutex<()>>) -> Result<(), String> {
    let mirror = PYTHON_MIRRORS
        .iter()
        .find(|m| m.id == mirror_id)
        .unwrap_or_else(|| {
            PYTHON_MIRRORS
                .iter()
                .find(|m| m.id == DEFAULT_MIRROR_ID)
                .unwrap()
        });

    let appdata = std::env::var("APPDATA").map_err(|_| "无法获取 APPDATA 目录".to_string())?;
    let pip_dir = Path::new(&appdata).join("pip");
    let pip_file = pip_dir.join("pip.ini");

    {
        let _guard = lock.lock();
        fs::create_dir_all(&pip_dir).map_err(|err| format!("创建 pip 配置目录失败: {err}"))?;
        let host = Url::parse(&mirror.index_url)
            .ok()
            .and_then(|url| url.host_str().map(|h| h.to_string()))
            .unwrap_or_default();
        let content = format!(
            "[global]\nindex-url = {url}\ntrusted-host = {host}\n",
            url = mirror.index_url,
            host = host
        );
        fs::write(&pip_file, content).map_err(|err| format!("写入 pip 配置失败: {err}"))?;
    }

    Ok(())
}

#[cfg(windows)]
fn detect_mirror_status() -> (String, StatusLine) {
    let mirrors = PYTHON_MIRRORS.clone();
    let current = read_pip_mirror();
    if let Some(current_url) = current {
        if let Some(mirror) = mirrors
            .iter()
            .find(|m| normalize_str(&m.index_url) == normalize_str(&current_url))
        {
            return (
                mirror.id.clone(),
                StatusLine {
                    label: "PIP 镜像源".into(),
                    message: format!("已配置为 {}", mirror.label),
                    level: StatusLevel::Success,
                },
            );
        }
        (
            DEFAULT_MIRROR_ID.into(),
            StatusLine {
                label: "PIP 镜像源".into(),
                message: "已设置镜像源，但不在预设列表中".into(),
                level: StatusLevel::Info,
            },
        )
    } else {
        (
            DEFAULT_MIRROR_ID.into(),
            StatusLine {
                label: "PIP 镜像源".into(),
                message: "尚未配置 pip 镜像源".into(),
                level: StatusLevel::Warning,
            },
        )
    }
}

#[cfg(windows)]
fn read_pip_mirror() -> Option<String> {
    let appdata = std::env::var("APPDATA").ok()?;
    let pip_file = Path::new(&appdata).join("pip").join("pip.ini");
    let content = fs::read_to_string(pip_file).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.to_ascii_lowercase().starts_with("index-url") {
            let parts: Vec<&str> = trimmed.split(|c| c == '=' || c == ':').collect();
            if let Some(value) = parts.last() {
                let url = value.trim();
                if !url.is_empty() {
                    return Some(url.to_string());
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

#[cfg(windows)]
fn normalize_str(value: &str) -> String {
    value.replace('\\', "/").to_lowercase()
}

#[cfg(windows)]
fn path_contains(path_value: &str, segment: &str) -> bool {
    let normalized_segment = normalize_str(segment);
    path_value
        .split(';')
        .map(|entry| normalize_str(entry))
        .any(|entry| entry.contains(&normalized_segment))
}

#[cfg(windows)]
fn read_user_env(name: &str) -> Result<Option<String>, String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey("Environment")
        .map_err(|err| format!("打开环境变量注册表失败: {err}"))?;
    match env.get_value::<String, _>(name) {
        Ok(value) => Ok(Some(value)),
        Err(winreg::Error::NotFound) => Ok(None),
        Err(err) => Err(format!("读取环境变量失败: {err}")),
    }
}

#[cfg(windows)]
fn set_user_env(name: &str, value: &str) -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::enums::KEY_ALL_ACCESS;
    use winreg::types::ExpandString;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_ALL_ACCESS)
        .or_else(|_| hkcu.create_subkey("Environment").map(|tuple| tuple.0))
        .map_err(|err| format!("打开环境变量注册表失败: {err}"))?;
    if name.eq_ignore_ascii_case("path") && value.contains('%') {
        env.set_value(name, &ExpandString(value.to_string()))
            .map_err(|err| format!("写入环境变量失败: {err}"))
    } else {
        env.set_value(name, &value)
            .map_err(|err| format!("写入环境变量失败: {err}"))
    }
}

#[cfg(windows)]
fn broadcast_environment_change() {
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::HWND_BROADCAST;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    let parameter: Vec<u16> = "Environment"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            parameter.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            null_mut(),
        );
    }
}
