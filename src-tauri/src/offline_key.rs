use crate::FileWriteLock;

// 标准库
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// 第三方库 - 加密
use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use anyhow::{anyhow, Result as AnyResult};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use get_if_addrs::{get_if_addrs, IfAddr};
use hostname;
use machine_uid::get as get_machine_uid;
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rand::RngCore;
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::{Oaep, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tauri::async_runtime;
use tauri::State;
use thiserror::Error;
use uuid::Uuid;

// RSA 私钥（2048位）- 用于解密许可证信息
const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCOgtPRErWlPg+c
h+BoCKPzTa9tDj6HUnit9/HrVC6nTdZYICxkkWmmKzyi7NAE+z9YPe81DEFLl16h
EI7OrL6TO5lx622UgmaJF1HLb/qdPsNtW91EeTIbFcwWw3Xeo02mgF+08IFkp5+Q
mZvfwDs27XxfzLS6K7elLHl0CsqbAmrTKM8jasM0t07aa7QltLSuBLNGr7Idxmqs
R3J3eNychoTRIW75jNdzBS3NJ3QikpFe/vpg9aKh9tTt8PS1s3yYmQL8wRvpD1XY
XmrhSa4iCX3vwy9A3dloATyP32GPRenWJ00Xvp906NCgdSdhjrGDyw8C7bf53ciN
WV3fb3ndAgMBAAECggEACh+aC2aQV8IuWri+RLWka+KvXQhfgb5mizkrSsPKoqDh
YpY6gRRhVGgK4SoAZvIwIEUaCoMp2kRQ+RETi/Pyf8QClPib6qpOvVtWOKTmQShV
up2FfNk9KXZlbpKI+31PRU4mPlS9ZFiR+bQnwf/GVRMOvEFu7zCu1YFwyUEA/nzU
0Je5MfbrVMQFlDJDBBAtfkpP/ljZH9QxHhBqyPikWKLOfL62uTcxvSVfKAZLrTWe
g73atRHgD3QkJSBQr6P+b3puNM78O83AAvD2ZQaoMhHzJ1bQ2SJUPzIWsSvVkihy
zpAWKxfv0fkk3IModjRinkTGxRkoiN2M2qGp9+KmUQKBgQDGiHEhNSbw/7ENvdfn
EYObSRfDE5sfaOXZ4ZWXUzg6YOMBeG1LoAo1N4ibtJSoxM+6PYS3BSZIpw2WDCjE
f1dkgr6Ah6fK3zwQ+lePDuY04LpDf0AOFbTfYhkklFEk8UJWwyiR2ycYimHwttKS
T5B7hd0ltyVr6vkb56U2W/XymQKBgQC3wxXxFtAUwsQVflsmqkXatXearux12vCD
6Qvvk3ifTaOdxAfVKNmLwDIgd63WeURFgDPvczQATYxPK+Gojr6FDAuhwqZjR1vJ
5uitzAHNLtNFefHIuNQpwH4N2bVnJsUU7fBBs6L/NXp72iCQGfKCD3L0VAHV4te2
ZdVVpXqP5QKBgQCDpHVfT59Crkj4k6lVzoc1sIHRGG8DpMrEbpCChuwByby8fN38
B2K9ZtVuaY8pWVdkZuZZdVCtXBfmJqnhoY6RVyB6mXjpTJzHpTfp5DdBWMIpf3c5
36rPdHy3hKFJnbAfV8jO7kI+Q4Qt2QxHd/qV7W12VLq7lFMZ7b/fC5tKAQKBgBQ6
RzoIltMSNV9gT0xMRfAzNX0zcBfxB7SdfJDcnNR8SVxebbZLDtdRSrNEOUB4jMlR
uWXMOpcl8iHs5KQXQmWG0+j0cjhPbI0m+8nVwQUC+IKXT8QWhFQsOjPwGQXHwL3z
CxVQXsrU6iyj8B2snYMvjCnU7XbLx47uWrNYycr5AoGAJJwddEa1ezmC7TNe8FKt
fE9mvzQGKK2sgB7MS2gUs1W9rVVHtWXM4MhGPawTkxvB0TboJAYO/AVe85NSKGkp
AEFRaeN96CUIvG5Ig2olnVJIkYu6AhGTPblClq5AzGRdIcQa0gxQgT/ebMXfjSak
0ReKx1fBgykoiX51bhsuFAA=
-----END PRIVATE KEY-----";

// RSA 公钥（2048位）- 用于生成密钥时加密
const OFFLINE_RSA_PUBLIC_KEY: &str = r"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAjoLT0RK1pT4PnIfgaAij
802vbQ4+h1J4rffx61Qup03WWCAsZJFppis8ouzQBPs/WD3vNQxBS5deoRCOzqy+
kzuZcettlIJmiRdRy2/6nT7DbVvdRHkyGxXMFsN13qNNpoBftPCBZKefkJmb38A7
Nu18X8y0uiu3pSx5dArKmwJq0yjPI2rDNLdO2mu0JbS0rgSzRq+yHcZqrEdyd3jc
nIaE0SFu+YzXcwUtzSd0IpKRXv76YPWiofbU7fD0tbN8mJkC/MEb6Q9V2F5q4Umu
Igl978MvQN3ZaAE8j99hj0Xp1idNF76fdOjQoHUnYY6xg8sPAu23+d3IjVld3295
3QIDAQAB
-----END PUBLIC KEY-----";

// AES-256 密钥（Base64 编码）- 用于时间戳加密
const OFFLINE_AES_KEY_B64: &str = "94/AR7dd8gIstLEXp3LCs865DptiMKlh8nLjjDEcO40=";

// 密钥分隔符
pub const OFFLINE_KEY_SEPARATOR: &str = "|||";

// 环境变量名称
pub const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

// 懒加载初始化加密密钥
static PRIVATE_KEY: Lazy<Result<RsaPrivateKey, String>> = Lazy::new(|| {
    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY)
        .map_err(|err| format!("加载 RSA 私钥失败: {err}"))
});

#[allow(dead_code)]
static PUBLIC_KEY: Lazy<Result<RsaPublicKey, String>> = Lazy::new(|| {
    RsaPublicKey::from_public_key_pem(OFFLINE_RSA_PUBLIC_KEY)
        .map_err(|err| format!("加载 RSA 公钥失败: {err}"))
});

static AES_KEY: Lazy<Result<Vec<u8>, String>> = Lazy::new(|| {
    general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .map_err(|err| format!("加载 AES 密钥失败: {err}"))
});

fn get_private_key() -> Result<&'static RsaPrivateKey, OfflineKeyError> {
    PRIVATE_KEY
        .as_ref()
        .map_err(|err| OfflineKeyError::Crypto(err.clone()))
}

#[allow(dead_code)]
fn get_public_key() -> Result<&'static RsaPublicKey, OfflineKeyError> {
    PUBLIC_KEY
        .as_ref()
        .map_err(|err| OfflineKeyError::Crypto(err.clone()))
}

fn get_aes_key() -> Result<&'static [u8], OfflineKeyError> {
    AES_KEY
        .as_ref()
        .map(|key| key.as_slice())
        .map_err(|err| OfflineKeyError::Crypto(err.clone()))
}

#[derive(Error, Debug)]
pub enum OfflineKeyError {
    #[error("密钥格式无效: {0}")]
    InvalidData(String),

    #[error("加密操作失败: {0}")]
    Crypto(String),

    #[error("文件操作失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 序列化失败: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineLicensePayload {
    pub user_id: u32,
    pub username: String,
    pub email: String,
    pub device_id: String,
    pub expires_at: String,
    pub issued_at: String,
}

impl OfflineLicensePayload {
    fn from_value(value: &Value) -> Result<Self, OfflineKeyError> {
        serde_json::from_value(value.clone())
            .map_err(|_| OfflineKeyError::InvalidData("许可证数据格式无效".into()))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineKeyValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub expires_at: Option<String>,
    pub payload: Option<OfflineLicensePayload>,
}

fn build_invalid_result(
    reason: impl Into<String>,
    expires_at: Option<String>,
    payload: Option<OfflineLicensePayload>,
) -> OfflineKeyValidationResult {
    OfflineKeyValidationResult {
        is_valid: false,
        reason: Some(reason.into()),
        expires_at,
        payload,
    }
}

/// 设备ID生成逻辑（优先级从高到低）
pub fn generate_device_id() -> AnyResult<String> {
    if let Ok(machine_id) = get_machine_uid() {
        if let Some(id) = hash_identifier(machine_id.as_bytes()) {
            return Ok(id);
        }
    }

    let mut hasher = Sha256::new();

    if let Ok(hostname) = hostname::get() {
        hasher.update(hostname.to_string_lossy().as_bytes());
    }

    if let Ok(interfaces) = get_if_addrs() {
        for interface in interfaces {
            match interface.addr {
                IfAddr::V4(ifv4) => {
                    hasher.update(ifv4.ip.octets());
                    hasher.update(ifv4.netmask.octets());
                    if let Some(broadcast) = ifv4.broadcast {
                        hasher.update(broadcast.octets());
                    }
                }
                IfAddr::V6(ifv6) => {
                    hasher.update(ifv6.ip.octets());
                    hasher.update(ifv6.netmask.octets());
                    if let Some(broadcast) = ifv6.broadcast {
                        hasher.update(broadcast.octets());
                    }
                }
            }
        }
    }

    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update(std::env::consts::ARCH.as_bytes());

    let fingerprint = hasher.finalize();
    let fingerprint_hex = format!("{:x}", fingerprint);

    if let Some(id) = shorten_hex(fingerprint_hex) {
        return Ok(id);
    }

    let uuid = Uuid::new_v4();
    hash_identifier(uuid.as_bytes()).ok_or_else(|| anyhow!("无法生成设备ID"))
}

fn hash_identifier(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hex = format!("{:x}", hasher.finalize());
    shorten_hex(hex)
}

fn shorten_hex(hex: String) -> Option<String> {
    if hex.is_empty() {
        return None;
    }
    let id: String = hex.chars().take(32).collect();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

fn decrypt_license_payload(encoded: &str) -> Result<OfflineLicensePayload, OfflineKeyError> {
    let private_key = get_private_key()?;

    let encrypted = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("RSA 密文格式无效".into()))?;

    let padding = Oaep::new::<Sha256>();
    let decrypted = private_key
        .decrypt(padding, &encrypted)
        .map_err(|err| OfflineKeyError::Crypto(format!("RSA 解密失败: {err}")))?;

    let value: Value = serde_json::from_slice(&decrypted)
        .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))?;

    OfflineLicensePayload::from_value(&value)
}

fn decrypt_server_time(encoded: &str) -> Result<DateTime<Utc>, OfflineKeyError> {
    let combined = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("时间密文格式无效".into()))?;

    if combined.len() <= 12 {
        return Err(OfflineKeyError::InvalidData("离线时间数据损坏".into()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);

    let nonce_array: [u8; 12] = nonce_bytes
        .try_into()
        .map_err(|_| OfflineKeyError::InvalidData("离线时间数据损坏".into()))?;
    let nonce = Nonce::from(nonce_array);

    let cipher = Aes256Gcm::new_from_slice(get_aes_key()?)
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;

    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 解密失败: {err}")))?;

    let time_str = String::from_utf8(plaintext)
        .map_err(|_| OfflineKeyError::InvalidData("服务器时间格式无效".into()))?;

    DateTime::parse_from_rfc3339(&time_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| OfflineKeyError::InvalidData("服务器时间格式无效".into()))
}

fn encrypt_current_time(now: DateTime<Utc>) -> Result<String, OfflineKeyError> {
    let cipher = Aes256Gcm::new_from_slice(get_aes_key()?)
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, now.to_rfc3339().as_bytes())
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 加密失败: {err}")))?;

    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

fn combine_offline_key(rsa_part: &str, aes_part: &str) -> String {
    format!("{rsa_part}{OFFLINE_KEY_SEPARATOR}{aes_part}")
}

fn split_offline_key(key: &str) -> Result<(String, String), OfflineKeyError> {
    let parts: Vec<&str> = key.split(OFFLINE_KEY_SEPARATOR).collect();
    if parts.len() != 2 {
        return Err(OfflineKeyError::InvalidData(
            "密钥格式错误，应包含RSA和AES两部分".into(),
        ));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

async fn read_file_with_lock(
    path: &Path,
    lock: &Arc<Mutex<()>>,
) -> Result<String, OfflineKeyError> {
    let path = path.to_path_buf();
    let lock = Arc::clone(lock);

    async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| OfflineKeyError::InvalidData("获取文件锁失败".into()))?;

        std::fs::read_to_string(path).map_err(OfflineKeyError::Io)
    })
    .await
    .map_err(|err| OfflineKeyError::InvalidData(format!("读取离线密钥失败: {err}")))?
}

async fn write_file_with_lock(
    path: &Path,
    content: &[u8],
    lock: &Arc<Mutex<()>>,
) -> Result<(), OfflineKeyError> {
    let path = path.to_path_buf();
    let data = content.to_vec();
    let lock = Arc::clone(lock);

    async_runtime::spawn_blocking(move || {
        let _guard = lock
            .lock()
            .map_err(|_| OfflineKeyError::InvalidData("获取文件锁失败".into()))?;

        std::fs::write(path, data).map_err(OfflineKeyError::Io)
    })
    .await
    .map_err(|err| OfflineKeyError::InvalidData(format!("写入离线密钥失败: {err}")))?
}

async fn try_validate_from_env(
    lock: &Arc<Mutex<()>>,
) -> Result<OfflineKeyValidationResult, OfflineKeyError> {
    let env_path = match std::env::var(OFFLINE_KEY_ENV_NAME) {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            return Ok(build_invalid_result(
                format!("环境变量 {OFFLINE_KEY_ENV_NAME} 未设置"),
                None,
                None,
            ));
        }
    };

    if !env_path.exists() {
        return Ok(build_invalid_result("离线密钥文件不存在", None, None));
    }

    let raw = read_file_with_lock(&env_path, lock).await?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(build_invalid_result("离线密钥文件为空", None, None));
    }

    let (rsa_part, aes_part) = split_offline_key(trimmed)?;
    let payload = decrypt_license_payload(&rsa_part)?;
    let server_time = decrypt_server_time(&aes_part)?;

    let expires_at = DateTime::parse_from_rfc3339(&payload.expires_at)
        .map_err(|_| OfflineKeyError::InvalidData("离线密钥到期时间无效".into()))?
        .with_timezone(&Utc);

    let now = Utc::now();
    if now > expires_at {
        return Ok(build_invalid_result(
            "离线密钥已过期",
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    let device_id = generate_device_id()
        .map_err(|err| OfflineKeyError::InvalidData(format!("无法获取设备标识: {err}")))?;
    if payload.device_id != device_id {
        return Ok(build_invalid_result(
            "当前设备与离线密钥绑定设备不一致",
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    if now < server_time {
        return Ok(build_invalid_result(
            "检测到离线密钥时间被回滚，请重新获取",
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    let updated_aes_part = encrypt_current_time(now)?;
    let updated_raw = combine_offline_key(&rsa_part, &updated_aes_part);
    write_file_with_lock(&env_path, updated_raw.as_bytes(), lock).await?;

    Ok(OfflineKeyValidationResult {
        is_valid: true,
        reason: None,
        expires_at: Some(payload.expires_at.clone()),
        payload: Some(payload),
    })
}

#[tauri::command]
pub async fn validate_offline_key(
    lock: State<'_, FileWriteLock>,
) -> Result<OfflineKeyValidationResult, String> {
    try_validate_from_env(&lock.0)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_device_id() -> Result<String, String> {
    generate_device_id().map_err(|err| err.to_string())
}
