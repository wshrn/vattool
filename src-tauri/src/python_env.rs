use anyhow::{anyhow, Context, Result};
use directories::BaseDirs;
use regex::Regex;
use serde::Serialize;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use url::Url;

const PYTHON_EXECUTABLES: &[&str] = &["python.exe", "python3.exe"];
const DEFAULT_MIRRORS: &[(&str, &str, &[&str])] = &[
    (
        "清华大学 TUNA",
        "https://pypi.tuna.tsinghua.edu.cn/simple",
        &["pypi.tuna.tsinghua.edu.cn"],
    ),
    (
        "阿里云",
        "https://mirrors.aliyun.com/pypi/simple",
        &["mirrors.aliyun.com"],
    ),
    (
        "中国科学技术大学",
        "https://pypi.mirrors.ustc.edu.cn/simple",
        &["pypi.mirrors.ustc.edu.cn"],
    ),
    (
        "华为云",
        "https://mirrors.huaweicloud.com/repository/pypi/simple",
        &["mirrors.huaweicloud.com"],
    ),
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusLine {
    pub ok: bool,
    pub message: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipMirrorStatus {
    pub ok: bool,
    pub message: String,
    pub detail: Option<String>,
    pub current_mirror: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorOption {
    pub label: String,
    pub value: String,
    pub trusted_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEnvStatus {
    pub python_present: StatusLine,
    pub python_env_var: StatusLine,
    pub path_configured: StatusLine,
    pub scripts_configured: StatusLine,
    pub pip_mirror_configured: PipMirrorStatus,
    pub python_path: Option<String>,
    pub scripts_path: Option<String>,
    pub mirror_candidates: Vec<MirrorOption>,
}

struct DetectionContext {
    python_dir: Option<PathBuf>,
    scripts_dir: Option<PathBuf>,
    python_env_var: Option<String>,
    path_entries: Vec<String>,
    pip_mirror: Option<String>,
}

#[tauri::command]
pub async fn get_python_environment_status() -> Result<PythonEnvStatus, String> {
    collect_status().map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn initialize_python_environment(mirror: String) -> Result<PythonEnvStatus, String> {
    initialize_impl(mirror).map_err(|err| err.to_string())
}

fn collect_status() -> Result<PythonEnvStatus> {
    let context = detect_context();
    let mirror_candidates = mirror_candidates();

    let python_path = context
        .python_dir
        .as_ref()
        .map(|path| path.display().to_string());
    let scripts_path = context
        .scripts_dir
        .as_ref()
        .map(|path| path.display().to_string());

    let python_present = if let Some(dir) = &context.python_dir {
        status_ok(
            format!("找到 Python 可执行文件，目录：{}", dir.display()),
            None,
        )
    } else {
        status_err(
            "当前目录未检测到 python.exe 或 python3.exe",
            Some("请确认工具与 Python 可执行文件位于同一目录".into()),
        )
    };

    let python_env_var = match (&context.python_dir, context.python_env_var.as_ref()) {
        (Some(dir), Some(value)) if path_equals(value, dir) => {
            status_ok("环境变量 python3 已正确指向当前目录", None)
        }
        (Some(_), Some(value)) => status_err(
            "环境变量 python3 未指向当前目录",
            Some(format!("当前值：{value}")),
        ),
        (Some(_), None) => status_err(
            "未检测到名为 python3 的环境变量",
            Some("初始化时会自动创建".into()),
        ),
        (None, _) => status_err("未检测到 Python 目录，无法校验环境变量", None),
    };

    let path_configured = match context.python_dir.as_ref() {
        Some(dir) if contains_path(&context.path_entries, dir) => {
            status_ok("PATH 中包含当前 Python 目录", None)
        }
        Some(dir) => status_err(
            "PATH 未包含当前 Python 目录",
            Some(format!("期望目录：{}", dir.display())),
        ),
        None => status_err("未检测到 Python 目录，无法校验 PATH", None),
    };

    let scripts_configured = match context.scripts_dir.as_ref() {
        Some(dir) if contains_path(&context.path_entries, dir) => {
            status_ok("PATH 中包含 Scripts 目录", None)
        }
        Some(dir) => status_err(
            "PATH 未包含 Scripts 目录",
            Some(format!("期望目录：{}", dir.display())),
        ),
        None => status_err("未检测到 Scripts 目录，无法校验 PATH", None),
    };

    let pip_mirror_configured = build_pip_status(&context, &mirror_candidates);

    Ok(PythonEnvStatus {
        python_present,
        python_env_var,
        path_configured,
        scripts_configured,
        pip_mirror_configured,
        python_path,
        scripts_path,
        mirror_candidates,
    })
}

fn initialize_impl(mirror: String) -> Result<PythonEnvStatus> {
    let context = detect_context();
    let python_dir = context
        .python_dir
        .clone()
        .ok_or_else(|| anyhow!("未在当前目录找到 python.exe 或 python3.exe"))?;
    let scripts_dir = python_dir.join("Scripts");

    if !platform::get_env("python3")?
        .map(|value| path_equals(&value, &python_dir))
        .unwrap_or(false)
    {
        platform::set_env("python3", python_dir.to_string_lossy().as_ref())?;
    }

    let mut segments: Vec<String> = split_path_entries(&platform::get_user_path()?);
    let mut changed = false;
    changed |= ensure_entry(&mut segments, &python_dir);
    if scripts_dir.exists() {
        changed |= ensure_entry(&mut segments, &scripts_dir);
    }

    if changed {
        let combined = join_path_entries(&segments);
        platform::set_user_path(&combined)?;
    }

    let mirror_option = resolve_mirror_option(mirror);
    configure_pip_mirror(&mirror_option)?;

    platform::broadcast_environment_change();

    collect_status()
}

fn detect_context() -> DetectionContext {
    let python_dir = detect_python_directory();
    let scripts_dir = python_dir
        .as_ref()
        .map(|dir| dir.join("Scripts"))
        .filter(|path| path.exists());
    let python_env_var = platform::get_env("python3").ok().flatten();
    let path_entries = platform::get_user_path()
        .map(|value| split_path_entries(&value))
        .unwrap_or_else(|_| split_path_entries(&std::env::var("PATH").unwrap_or_default()));
    let pip_mirror = detect_pip_mirror();

    DetectionContext {
        python_dir,
        scripts_dir,
        python_env_var,
        path_entries,
        pip_mirror,
    }
}

fn detect_python_directory() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    for exe in PYTHON_EXECUTABLES {
        let candidate = current_dir.join(exe);
        if candidate.exists() {
            return candidate.parent().map(|p| p.to_path_buf());
        }
    }
    None
}

fn split_path_entries(raw: &str) -> Vec<String> {
    raw.split(';')
        .filter_map(|entry| normalize_path_entry(entry))
        .collect()
}

fn join_path_entries(entries: &[String]) -> String {
    entries.join(";")
}

fn ensure_entry(entries: &mut Vec<String>, path: &Path) -> bool {
    let value = normalize_path_from_path(path);
    if entries
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(&value))
    {
        false
    } else {
        entries.push(value);
        true
    }
}

fn contains_path(entries: &[String], target: &Path) -> bool {
    let value = normalize_path_from_path(target);
    entries
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(&value))
}

fn path_equals(entry: &str, target: &Path) -> bool {
    normalize_path_entry(entry)
        .map(|normalized| normalized.eq_ignore_ascii_case(&normalize_path_from_path(target)))
        .unwrap_or(false)
}

fn normalize_path_entry(entry: &str) -> Option<String> {
    let trimmed = entry.trim().trim_matches('"');
    if trimmed.is_empty() {
        None
    } else {
        Some(normalize_path_text(trimmed))
    }
}

fn normalize_path_from_path(path: &Path) -> String {
    normalize_path_text(&path.to_string_lossy())
}

fn normalize_path_text(value: &str) -> String {
    let mut normalized = value.replace('/', "\\");
    while normalized.ends_with('\\') {
        let len = normalized.len();
        if len == 0 {
            break;
        }
        if len == 3 && normalized.as_bytes()[1] == b':' {
            break;
        }
        normalized.pop();
    }
    normalized
}

fn mirror_candidates() -> Vec<MirrorOption> {
    DEFAULT_MIRRORS
        .iter()
        .map(|(label, value, hosts)| MirrorOption {
            label: (*label).into(),
            value: (*value).into(),
            trusted_hosts: hosts.iter().map(|host| (*host).into()).collect(),
        })
        .collect()
}

fn build_pip_status(context: &DetectionContext, candidates: &[MirrorOption]) -> PipMirrorStatus {
    match context.pip_mirror.as_ref() {
        Some(current) => {
            let matched = candidates
                .iter()
                .find(|option| option.value.eq_ignore_ascii_case(current));
            if let Some(option) = matched {
                PipMirrorStatus {
                    ok: true,
                    message: format!("Pip 已配置为 {}", option.label),
                    detail: None,
                    current_mirror: Some(option.value.clone()),
                }
            } else {
                PipMirrorStatus {
                    ok: true,
                    message: "检测到自定义 pip 源".into(),
                    detail: Some(current.clone()),
                    current_mirror: Some(current.clone()),
                }
            }
        }
        None => PipMirrorStatus {
            ok: false,
            message: "未检测到 pip 全局镜像配置".into(),
            detail: Some("初始化时将写入 pip.ini".into()),
            current_mirror: None,
        },
    }
}

fn detect_pip_mirror() -> Option<String> {
    let path = pip_config_path()?;
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    let re = Regex::new(r"(?i)^\s*index-url\s*=\s*(?P<url>.+)$").ok()?;
    for line in content.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(url) = captures.name("url") {
                return Some(url.as_str().trim().to_string());
            }
        }
    }
    None
}

fn pip_config_path() -> Option<PathBuf> {
    BaseDirs::new().map(|dirs| {
        #[cfg(target_os = "windows")]
        {
            dirs.data_dir().join("pip").join("pip.ini")
        }
        #[cfg(not(target_os = "windows"))]
        {
            dirs.home_dir().join(".config").join("pip").join("pip.conf")
        }
    })
}

fn configure_pip_mirror(option: &MirrorOption) -> Result<()> {
    let path = pip_config_path().ok_or_else(|| anyhow!("无法定位 pip 配置目录"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建 pip 配置目录: {}", parent.display()))?;
    }
    let mut lines = vec![format!("index-url = {}", option.value)];
    if !option.trusted_hosts.is_empty() {
        let mut unique = HashSet::new();
        for host in &option.trusted_hosts {
            if unique.insert(host.to_lowercase()) {
                lines.push(format!("trusted-host = {}", host));
            }
        }
    }
    let mut content = String::from("[global]\n");
    for line in lines {
        content.push_str(&line);
        content.push('\n');
    }
    fs::write(&path, content).with_context(|| format!("写入 pip 配置失败: {}", path.display()))?;
    Ok(())
}

fn resolve_mirror_option(value: String) -> MirrorOption {
    let candidates = mirror_candidates();
    if let Some(option) = candidates
        .iter()
        .find(|candidate| candidate.value.eq_ignore_ascii_case(&value))
        .cloned()
    {
        return option;
    }

    let host = Url::parse(&value)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.to_string()))
        .into_iter()
        .collect();
    MirrorOption {
        label: "自定义镜像".into(),
        value,
        trusted_hosts: host,
    }
}

fn status_ok(message: impl Into<String>, detail: Option<String>) -> StatusLine {
    StatusLine {
        ok: true,
        message: message.into(),
        detail,
    }
}

fn status_err(message: impl Into<String>, detail: Option<String>) -> StatusLine {
    StatusLine {
        ok: false,
        message: message.into(),
        detail,
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use anyhow::Result;
    use std::io;
    use windows::core::w;
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;

    pub fn get_env(name: &str) -> Result<Option<String>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ)?;
        match env.get_value::<String, _>(name) {
            Ok(value) => Ok(Some(value)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn set_env(name: &str, value: &str) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (env, _) = hkcu.create_subkey("Environment")?;
        env.set_value(name, &value)?;
        Ok(())
    }

    pub fn get_user_path() -> Result<String> {
        Ok(get_env("Path")?.unwrap_or_default())
    }

    pub fn set_user_path(value: &str) -> Result<()> {
        set_env("Path", value)
    }

    pub fn broadcast_environment_change() {
        unsafe {
            let param = w!("Environment");
            let _ = SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM::default(),
                LPARAM(param.as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                5000,
                None,
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use anyhow::{anyhow, Result};

    pub fn get_env(name: &str) -> Result<Option<String>> {
        Ok(std::env::var(name).ok())
    }

    pub fn set_env(_: &str, _: &str) -> Result<()> {
        Err(anyhow!("仅支持在 Windows 上修改系统环境变量"))
    }

    pub fn get_user_path() -> Result<String> {
        Ok(std::env::var("PATH").unwrap_or_default())
    }

    pub fn set_user_path(_: &str) -> Result<()> {
        Err(anyhow!("仅支持在 Windows 上修改系统环境变量"))
    }

    pub fn broadcast_environment_change() {}
}
