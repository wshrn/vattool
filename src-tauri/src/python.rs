use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEnvironmentStatus {
    pub python_executable_found: bool,
    pub python_executable_path: Option<String>,
    pub python_env_var_set: bool,
    pub python_env_var_value: Option<String>,
    pub python_path_configured: bool,
    pub scripts_path_configured: bool,
    pub pip_mirror_configured: bool,
    pub pip_mirror_url: Option<String>,
}

impl Default for PythonEnvironmentStatus {
    fn default() -> Self {
        Self {
            python_executable_found: false,
            python_executable_path: None,
            python_env_var_set: false,
            python_env_var_value: None,
            python_path_configured: false,
            scripts_path_configured: false,
            pip_mirror_configured: false,
            pip_mirror_url: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MirrorOptionKey {
    #[serde(rename = "tsinghua")]
    Tsinghua,
    #[serde(rename = "aliyun")]
    Aliyun,
    #[serde(rename = "douban")]
    Douban,
    #[serde(rename = "huawei")]
    Huawei,
    #[serde(rename = "pypi")]
    Pypi,
}

impl MirrorOptionKey {
    pub fn url(self) -> &'static str {
        match self {
            MirrorOptionKey::Tsinghua => "https://pypi.tuna.tsinghua.edu.cn/simple",
            MirrorOptionKey::Aliyun => "https://mirrors.aliyun.com/pypi/simple/",
            MirrorOptionKey::Douban => "https://pypi.doubanio.com/simple/",
            MirrorOptionKey::Huawei => "https://mirrors.huaweicloud.com/repository/pypi/simple/",
            MirrorOptionKey::Pypi => "https://pypi.org/simple",
        }
    }
}

pub async fn get_python_env_status_command() -> Result<PythonEnvironmentStatus, String> {
    #[cfg(target_os = "windows")]
    {
        tokio::task::spawn_blocking(|| get_python_env_status_inner())
            .await
            .map_err(|err| err.to_string())?
            .map_err(|err| err.to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Python 环境检测仅支持 Windows 平台".into())
    }
}

pub async fn initialize_python_environment_command(
    mirror: MirrorOptionKey,
) -> Result<PythonEnvironmentStatus, String> {
    #[cfg(target_os = "windows")]
    {
        tokio::task::spawn_blocking(move || initialize_python_environment_inner(mirror))
            .await
            .map_err(|err| err.to_string())?
            .map_err(|err| err.to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Python 环境初始化仅支持 Windows 平台".into())
    }
}

#[cfg(target_os = "windows")]
fn get_python_env_status_inner() -> anyhow::Result<PythonEnvironmentStatus> {
    use anyhow::Context;

    let python_executable = find_python_executable();
    let python_dir = python_executable
        .as_ref()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()));
    let python_dir_str = python_dir
        .as_ref()
        .map(|path| path.to_string_lossy().to_string());

    let mut status = PythonEnvironmentStatus {
        python_executable_found: python_executable.is_some(),
        python_executable_path: python_executable
            .as_ref()
            .map(|path| path.to_string_lossy().to_string()),
        ..PythonEnvironmentStatus::default()
    };

    let env = WindowsEnvironment::open()?;
    let python_var = env.read_variable("python3")?;
    status.python_env_var_value = python_var.clone();

    if let (Some(dir), Some(var_value)) = (python_dir_str.as_ref(), python_var.as_ref()) {
        if normalize_path(var_value) == normalize_path(dir) {
            status.python_env_var_set = true;
        }
    }

    let path_value = env.read_variable("Path")?.unwrap_or_default();
    if let Some(dir) = python_dir_str.as_ref() {
        status.python_path_configured = path_contains(&path_value, "%python3%")
            || path_contains(&path_value, &normalize_path(dir));

        let scripts_path_env = "%python3%\\Scripts";
        let scripts_real = format!("{}\\Scripts", dir);
        status.scripts_path_configured = path_contains(&path_value, scripts_path_env)
            || path_contains(&path_value, &normalize_path(&scripts_real));
    }

    let pip_info = read_pip_mirror()?;
    status.pip_mirror_configured = pip_info.is_some();
    status.pip_mirror_url = pip_info;

    Ok(status)
}

#[cfg(target_os = "windows")]
fn initialize_python_environment_inner(
    mirror: MirrorOptionKey,
) -> anyhow::Result<PythonEnvironmentStatus> {
    use anyhow::{anyhow, Context};

    let python_executable = find_python_executable()
        .ok_or_else(|| anyhow!("未在当前目录找到 python.exe 或 python3.exe"))?;
    let python_dir = python_executable
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow!("无法确定 Python 所在目录"))?;
    let python_dir_str = python_dir.to_string_lossy().to_string();

    let env = WindowsEnvironment::open()?;
    env.set_variable("python3", &python_dir_str)?;

    let mut path_value = env.read_variable("Path")?.unwrap_or_default();
    ensure_path_entry(&mut path_value, "%python3%")?;
    ensure_path_entry(&mut path_value, "%python3%\\Scripts")?;
    env.set_variable("Path", &path_value)?;

    write_pip_mirror(mirror.url())?;
    broadcast_environment_change();

    get_python_env_status_inner()
}

#[cfg(target_os = "windows")]
fn ensure_path_entry(path_value: &mut String, entry: &str) -> anyhow::Result<()> {
    if path_contains(path_value, entry) {
        return Ok(());
    }
    if !path_value.is_empty() && !path_value.ends_with(';') {
        path_value.push(';');
    }
    path_value.push_str(entry);
    Ok(())
}

#[cfg(target_os = "windows")]
fn path_contains(path_value: &str, entry: &str) -> bool {
    let target = normalize_path(entry);
    path_value
        .split(';')
        .map(|part| normalize_path(part.trim()))
        .any(|part| part == target)
}

#[cfg(target_os = "windows")]
fn normalize_path(path: &str) -> String {
    path.replace('/', "\\").to_uppercase()
}

#[cfg(target_os = "windows")]
struct WindowsEnvironment {
    key: winreg::RegKey,
}

#[cfg(target_os = "windows")]
impl WindowsEnvironment {
    fn open() -> anyhow::Result<Self> {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
        let hkey = winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags("Environment", KEY_READ | KEY_SET_VALUE)
            .context("无法访问用户环境变量")?;
        Ok(Self { key: hkey })
    }

    fn read_variable(&self, name: &str) -> anyhow::Result<Option<String>> {
        match self.key.get_value(name) {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    fn set_variable(&self, name: &str, value: &str) -> anyhow::Result<()> {
        self.key
            .set_value(name, &value)
            .with_context(|| format!("写入环境变量 {name} 失败"))
    }
}

#[cfg(target_os = "windows")]
fn find_python_executable() -> Option<std::path::PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    for candidate in ["python.exe", "python3.exe"] {
        let path = current_dir.join(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn read_pip_mirror() -> anyhow::Result<Option<String>> {
    use anyhow::Context;
    use regex::Regex;
    use std::fs;

    if let Some(path) = existing_pip_config_path() {
        let content = fs::read_to_string(&path).with_context(|| format!("读取 {:?} 失败", path))?;
        let re = Regex::new(r"(?im)^\s*index-url\s*=\s*(?P<url>\S+)")?;
        if let Some(captures) = re.captures(&content) {
            if let Some(url) = captures.name("url") {
                return Ok(Some(url.as_str().to_string()));
            }
        }
    }

    Ok(None)
}

#[cfg(target_os = "windows")]
fn write_pip_mirror(url: &str) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::fs;

    if let Some(path) = pip_config_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("创建目录 {:?} 失败", parent))?;
        }
        let content = format!("[global]\nindex-url = {url}\n");
        fs::write(&path, content).with_context(|| format!("写入 {:?} 失败", path))?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn pip_config_path() -> Option<std::path::PathBuf> {
    pip_config_candidates().into_iter().next()
}

#[cfg(target_os = "windows")]
fn existing_pip_config_path() -> Option<std::path::PathBuf> {
    pip_config_candidates()
        .into_iter()
        .find(|path| path.exists())
}

#[cfg(target_os = "windows")]
fn pip_config_candidates() -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        candidates.push(
            std::path::PathBuf::from(appdata)
                .join("pip")
                .join("pip.ini"),
        );
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        candidates.push(
            std::path::PathBuf::from(userprofile)
                .join("pip")
                .join("pip.ini"),
        );
    }
    candidates
}

#[cfg(target_os = "windows")]
fn broadcast_environment_change() {
    use std::ptr;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    let mut message: Vec<u16> = "Environment\0".encode_utf16().collect();
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            message.as_mut_ptr() as isize,
            SMTO_ABORTIFHUNG,
            2000,
            ptr::null_mut(),
        );
    }
}
