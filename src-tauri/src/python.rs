use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct StatusItem {
    pub status: bool,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PythonStatus {
    pub python_exists: StatusItem,
    pub python_env_var: StatusItem,
    pub python_in_path: StatusItem,
    pub scripts_in_path: StatusItem,
    pub pip_mirror: StatusItem,
}

#[derive(Serialize, Clone, Debug)]
pub struct InitializationStep {
    pub name: String,
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct InitializationResult {
    pub steps: Vec<InitializationStep>,
}

#[derive(Clone, Debug)]
struct MirrorInfo {
    key: &'static str,
    label: &'static str,
    url: &'static str,
    host: &'static str,
}

static MIRRORS: &[MirrorInfo] = &[
    MirrorInfo {
        key: "tsinghua",
        label: "清华大学 TUNA 镜像",
        url: "https://pypi.tuna.tsinghua.edu.cn/simple",
        host: "pypi.tuna.tsinghua.edu.cn",
    },
    MirrorInfo {
        key: "aliyun",
        label: "阿里云镜像",
        url: "https://mirrors.aliyun.com/pypi/simple/",
        host: "mirrors.aliyun.com",
    },
    MirrorInfo {
        key: "huawei",
        label: "华为云镜像",
        url: "https://repo.huaweicloud.com/repository/pypi/simple",
        host: "repo.huaweicloud.com",
    },
    MirrorInfo {
        key: "douban",
        label: "豆瓣镜像",
        url: "https://pypi.doubanio.com/simple",
        host: "pypi.doubanio.com",
    },
];

static MIRROR_MAP: Lazy<HashMap<&'static str, &'static MirrorInfo>> = Lazy::new(|| {
    MIRRORS.iter().map(|mirror| (mirror.key, mirror)).collect()
});

pub fn mirror_for(key: &str) -> Option<&'static MirrorInfo> {
    MIRROR_MAP.get(key).copied()
}

pub fn collect_status() -> Result<PythonStatus> {
    #[cfg(target_os = "windows")]
    {
        collect_status_windows()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(PythonStatus {
            python_exists: StatusItem {
                status: false,
                message: "该功能仅支持 Windows 环境".into(),
            },
            python_env_var: StatusItem {
                status: false,
                message: "该功能仅支持 Windows 环境".into(),
            },
            python_in_path: StatusItem {
                status: false,
                message: "该功能仅支持 Windows 环境".into(),
            },
            scripts_in_path: StatusItem {
                status: false,
                message: "该功能仅支持 Windows 环境".into(),
            },
            pip_mirror: StatusItem {
                status: false,
                message: "该功能仅支持 Windows 环境".into(),
            },
        })
    }
}

pub fn initialize_environment(mirror_key: &str) -> Result<InitializationResult> {
    #[cfg(target_os = "windows")]
    {
        initialize_environment_windows(mirror_key)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err(anyhow!("初始化仅支持在 Windows 上执行"))
    }
}

#[cfg(target_os = "windows")]
fn collect_status_windows() -> Result<PythonStatus> {
    let python_path = find_python_executable();
    let python_dir = python_path
        .as_ref()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()));

    let python_exists = match &python_path {
        Some(path) => StatusItem {
            status: true,
            message: format!("已检测到: {}", path.display()),
        },
        None => StatusItem {
            status: false,
            message: "未在当前目录找到 python.exe".into(),
        },
    };

    let env_status = python_env_var_status(python_dir.as_ref());
    let path_status = python_in_path_status(python_dir.as_ref());
    let scripts_status = python_scripts_path_status(python_dir.as_ref());
    let pip_status = pip_mirror_status();

    Ok(PythonStatus {
        python_exists,
        python_env_var: env_status,
        python_in_path: path_status,
        scripts_in_path: scripts_status,
        pip_mirror: pip_status,
    })
}

#[cfg(target_os = "windows")]
fn initialize_environment_windows(mirror_key: &str) -> Result<InitializationResult> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let mut steps = Vec::new();

    let python_path = match find_python_executable() {
        Some(path) => path,
        None => {
            steps.push(InitializationStep {
                name: "检测 Python 可执行文件".into(),
                success: false,
                message: "未在当前目录找到 python.exe".into(),
            });
            return Ok(InitializationResult { steps });
        }
    };

    let python_dir = python_path
        .parent()
        .map(|parent| parent.to_path_buf())
        .ok_or_else(|| anyhow!("无法确定 Python 安装目录"))?;

    let mirror = match mirror_for(mirror_key) {
        Some(info) => info,
        None => {
            steps.push(InitializationStep {
                name: "选择国内镜像".into(),
                success: false,
                message: "未找到对应的镜像配置".into(),
            });
            return Ok(InitializationResult { steps });
        }
    };

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|err| anyhow!("无法打开 Environment 注册表项: {err}"))?;

    match set_expand_environment_value(&env, "python3", &format_windows_path(&python_dir)) {
        Ok(changed) => {
            steps.push(InitializationStep {
                name: "设置 python3 变量".into(),
                success: true,
                message: if changed {
                    "已更新 python3 环境变量".into()
                } else {
                    "python3 环境变量已存在".into()
                },
            });
        }
        Err(err) => {
            steps.push(InitializationStep {
                name: "设置 python3 变量".into(),
                success: false,
                message: format!("设置失败: {err}"),
            });
        }
    }

    match ensure_path_entry(&env, "%python3%") {
        Ok(changed) => {
            steps.push(InitializationStep {
                name: "写入 PATH (python3)".into(),
                success: true,
                message: if changed {
                    "已将 %python3% 添加到 PATH".into()
                } else {
                    "PATH 已包含 %python3%".into()
                },
            });
        }
        Err(err) => {
            steps.push(InitializationStep {
                name: "写入 PATH (python3)".into(),
                success: false,
                message: format!("添加失败: {err}"),
            });
        }
    }

    match ensure_path_entry(&env, "%python3%\\Scripts") {
        Ok(changed) => {
            steps.push(InitializationStep {
                name: "写入 PATH (Scripts)".into(),
                success: true,
                message: if changed {
                    "已将 %python3%\\Scripts 添加到 PATH".into()
                } else {
                    "PATH 已包含 %python3%\\Scripts".into()
                },
            });
        }
        Err(err) => {
            steps.push(InitializationStep {
                name: "写入 PATH (Scripts)".into(),
                success: false,
                message: format!("添加失败: {err}"),
            });
        }
    }

    match configure_pip_mirror(mirror) {
        Ok(_) => {
            steps.push(InitializationStep {
                name: "配置 pip 国内源".into(),
                success: true,
                message: format!("已写入 {}", mirror.label),
            });
        }
        Err(err) => {
            steps.push(InitializationStep {
                name: "配置 pip 国内源".into(),
                success: false,
                message: format!("配置失败: {err}"),
            });
        }
    }

    Ok(InitializationResult { steps })
}

#[cfg(target_os = "windows")]
fn python_env_var_status(python_dir: Option<&PathBuf>) -> StatusItem {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ)
        .and_then(|key| key.get_value::<String, _>("python3"));

    match env {
        Ok(value) => StatusItem {
            status: true,
            message: format!("python3 = {}", value),
        },
        Err(_) => {
            if let Some(dir) = python_dir {
                StatusItem {
                    status: false,
                    message: format!("建议设置为 {}", format_windows_path(dir)),
                }
            } else {
                StatusItem {
                    status: false,
                    message: "尚未设置 python3 环境变量".into(),
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn python_in_path_status(python_dir: Option<&PathBuf>) -> StatusItem {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ);

    let path_value: Option<String> = env
        .ok()
        .and_then(|key| key.get_value("Path").ok());

    let contains = path_value.as_deref().map_or(false, |value| {
        path_contains(value, "%python3%")
            || python_dir
                .map(|dir| path_contains(value, &format_windows_path(dir)))
                .unwrap_or(false)
    });

    if contains {
        StatusItem {
            status: true,
            message: "PATH 已包含 python3".into(),
        }
    } else {
        StatusItem {
            status: false,
            message: "PATH 未包含 python3".into(),
        }
    }
}

#[cfg(target_os = "windows")]
fn python_scripts_path_status(python_dir: Option<&PathBuf>) -> StatusItem {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu.open_subkey_with_flags("Environment", KEY_READ);

    let path_value: Option<String> = env
        .ok()
        .and_then(|key| key.get_value("Path").ok());

    let scripts_entry = "%python3%\\Scripts";
    let contains = path_value.as_deref().map_or(false, |value| {
        path_contains(value, scripts_entry)
            || python_dir
                .map(|dir| {
                    let scripts_path = dir.join("Scripts");
                    path_contains(value, &format_windows_path(&scripts_path))
                })
                .unwrap_or(false)
    });

    if contains {
        StatusItem {
            status: true,
            message: "PATH 已包含 Scripts".into(),
        }
    } else {
        StatusItem {
            status: false,
            message: "PATH 未包含 Scripts".into(),
        }
    }
}

#[cfg(target_os = "windows")]
fn pip_mirror_status() -> StatusItem {
    match current_pip_mirror() {
        Some((label, url)) => StatusItem {
            status: true,
            message: format!("当前镜像：{} ({})", label, url),
        },
        None => StatusItem {
            status: false,
            message: "尚未检测到 pip 国内镜像配置".into(),
        },
    }
}

#[cfg(target_os = "windows")]
fn current_pip_mirror() -> Option<(String, String)> {
    let path = pip_config_path()?;
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("index-url") {
            let url = trimmed.split('=').nth(1)?.trim();
            for mirror in MIRRORS {
                if url.contains(mirror.host) {
                    return Some((mirror.label.into(), mirror.url.into()));
                }
            }
            return Some(("自定义镜像".into(), url.into()));
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn configure_pip_mirror(mirror: &MirrorInfo) -> Result<()> {
    let path = pip_config_path().ok_or_else(|| anyhow!("无法定位 pip 配置目录"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = format!(
        "[global]\nindex-url = {url}\ntrusted-host = {host}\n[install]\ntrusted-host = {host}\n",
        url = mirror.url,
        host = mirror.host
    );
    fs::write(path, content)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn pip_config_path() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok()?;
    let mut path = PathBuf::from(appdata);
    path.push("pip");
    path.push("pip.ini");
    Some(path)
}

#[cfg(target_os = "windows")]
fn ensure_path_entry(env: &winreg::RegKey, entry: &str) -> Result<bool> {
    let current: String = env.get_value("Path").unwrap_or_default();
    if path_contains(&current, entry) {
        return Ok(false);
    }

    let mut new_value = current.clone();
    if !new_value.is_empty() && !new_value.ends_with(';') {
        new_value.push(';');
    }
    new_value.push_str(entry);

    set_expand_environment_value(env, "Path", &new_value)?;
    Ok(true)
}

#[cfg(target_os = "windows")]
fn set_expand_environment_value(env: &winreg::RegKey, name: &str, value: &str) -> Result<bool> {
    use std::slice;
    use winreg::enums::RegType;
    use winreg::RegValue;

    let current: String = env.get_value(name).unwrap_or_default();
    if normalize_entry(&current) == normalize_entry(value) {
        return Ok(false);
    }

    let mut wide: Vec<u16> = value.encode_utf16().collect();
    wide.push(0);
    let bytes = unsafe {
        slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2)
    };
    let reg_value = RegValue {
        vtype: RegType::REG_EXPAND_SZ,
        bytes: bytes.to_vec(),
    };
    env.set_raw_value(name, &reg_value)?;
    Ok(true)
}

#[cfg(target_os = "windows")]
fn path_contains(path_value: &str, needle: &str) -> bool {
    let needle_norm = normalize_entry(needle);
    path_value
        .split(';')
        .map(|part| normalize_entry(part))
        .any(|part| part == needle_norm)
}

#[cfg(target_os = "windows")]
fn normalize_entry(entry: &str) -> String {
    entry
        .trim()
        .trim_end_matches(';')
        .replace('/', "\\")
        .to_ascii_lowercase()
}

#[cfg(target_os = "windows")]
fn format_windows_path(path: &Path) -> String {
    path.display().to_string().replace('/', "\\")
}

fn find_python_executable() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    for candidate in ["python.exe", "python3.exe"] {
        let path = current_dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}
