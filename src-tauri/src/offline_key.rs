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
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDVKT08cxtJyhsx
l9A0+6o7PR5EN12E/gaVXSInY/g9GgTeCcaXBm7FifosFCaVGcvmjJrhxTaNsp/7
VM+S7jJVfmIYuVnwhhGdis5iJlG8fL2ekG1c1HtShxK7vcrhv8lDrc3zEIbX1v6r
9sy4T8gZH8peg6dnzmMKHRPhXGscr2SIHn3xsdkP/kCY+4mOsRLdV0IESho0BsNC
W4smTp4lx9zZKM9Q6DNF62B/2Gd4v+vTixogouQVbSJspTg/AbRx+snZbvX1Fe2d
I1pFvosWO/rh/K2Wt4PW9KUoQOQXt1WUWIQv5+4FQI82zcRR0BuUf4bfPnUy64ms
b8cfUA+1AgMBAAECggEAAP1hTitPG/u3iLR3KIeQfq0dmpZ8ED8KS0T2xtgktxUT
jqK9e8VCG7aCqL5YqPvZ6B8mXKJN3xHxO4qLdF2eW8zP9Rw3jH4kL6mN2oP5xRt7
vW8yZ0aB1cD2eF3gH4iJ5kL6mN7oP8qR9sT0uV1wX2yZ3aB4cD5eF6gH7iJ8kL9m
N0oP1qR2sT3uV4wX5yZ6aB7cD8eF9gH0iJ1kL2mN3oP4qR5sT6uV7wX8yZ9aB0cD
1eF2gH3iJ4kL5mN6oP7qR8sT9uV0wX1yZ2aB3cD4eF5gH6iJ7kL8mN9oP0qR1sT2
-----END PRIVATE KEY-----";

// RSA 公钥（2048位）- 用于生成密钥时加密
const OFFLINE_RSA_PUBLIC_KEY: &str = r"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1Sk9PHMbScoZMZfQNPuq
Oz0eRDddhP4GlV0iJ2P4PRoE3gnGlwZuxYn6LBQmlRnL5oya4cU2jbKf+1TPku4y
VX5iGLlZ8IYRnYrOYiZRvHy9npBtXNR7UocSu73K4b/JQ63N8xCG19b+q/bMuE/I
GR/KXoOnZ85jCh0T4VxrHK9kiB598bHZD/5AmPuJjrES3VdCBEoaNAbDQluLJk6e
Jcfc2SjPUOgzRetgf9hneL/r04saIKLkFW0ibKU4PwG0cfrJ2W719RXtnSNaRb6L
Fjv64fytlreD1vSlKEDkF7dVlFiEL+fuBUCPNs3EUdAblH+G3z51MuuJrG/HH1AP
tQIDAQAB
-----END PUBLIC KEY-----";

// AES-256 密钥（Base64 编码）- 用于时间戳加密
const OFFLINE_AES_KEY_B64: &str = "94/AR7dd8gIstLEXp3LCs865DptiMKlh8nLjjDEcO40=";

// 密钥分隔符
pub const OFFLINE_KEY_SEPARATOR: &str = "|||";

// 环境变量名称
pub const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

// 懒加载初始化加密密钥
static PRIVATE_KEY: Lazy<RsaPrivateKey> =
    Lazy::new(|| RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY).expect("无效的 RSA 私钥"));

#[allow(dead_code)]
static PUBLIC_KEY: Lazy<RsaPublicKey> = Lazy::new(|| {
    RsaPublicKey::from_public_key_pem(OFFLINE_RSA_PUBLIC_KEY).expect("无效的 RSA 公钥")
});

static AES_KEY: Lazy<Vec<u8>> = Lazy::new(|| {
    general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .expect("无效的 AES 密钥")
});

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
    let private_key = &*PRIVATE_KEY;

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

    let cipher = Aes256Gcm::new_from_slice(&AES_KEY[..])
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
    let cipher = Aes256Gcm::new_from_slice(&AES_KEY[..])
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, now.to_rfc3339().as_bytes())
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 加密失败: {err}")))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(nonce.as_slice());
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
