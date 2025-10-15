use std::path::PathBuf;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use rand::RngCore;
use rsa::{pkcs8::DecodePrivateKey, Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::config::FileWriteLock;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDVKT08cxtJyhsx
l9A0+6o7PR5EN12E/gaVXSInY/g9GgTeCcaXBm7FifosFCaVGcvmjJrhxTaNsp/7
VM+S7jJVfmIYuVnwhhGdis5iJlG8fL2ekG1c1HtShxK7vcrhv8lDrc3zEIbX1v6r
9sy4T8gZH8peg6dnzmMKHRPhXGscr2SIHn3xsdkP/kCY+4mOsRLdV0IESho0BsNC
W4smTp4lx9zZKM9Q6DNF62B/2Gd4v+vTixogouQVbSJspTg/AbRx+snZbvX1Fe2d
I1pFvosWO/rh/K2Wt4PW9KUoQOQXt1WUWIQv5+4FQI82zcRR0BuUf4bfPnUy64ms
b8cfUA+1AgMBAAECggEAAP1hTitPG/u3iLR3KIeQfq0dmpZ8ED8KS0T2xtgktxUT
...
-----END PRIVATE KEY-----";
const OFFLINE_AES_KEY_B64: &str = "94/AR7dd8gIstLEXp3LCs865DptiMKlh8nLjjDEcO40=";
const OFFLINE_KEY_SEPARATOR: &str = "|||";
const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

static PRIVATE_KEY: Lazy<Result<RsaPrivateKey, rsa::pkcs8::Error>> =
    Lazy::new(|| RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY));

static AES_KEY: Lazy<Result<[u8; 32], String>> = Lazy::new(|| {
    let decoded = general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .map_err(|err| format!("无法解析 AES 密钥: {err}"))?;
    if decoded.len() != 32 {
        return Err("AES 密钥长度无效".to_string());
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&decoded);
    Ok(bytes)
});

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineKeyValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub expires_at: Option<String>,
    pub payload: Option<OfflineLicensePayload>,
}

#[derive(Debug, Error)]
pub enum OfflineKeyError {
    #[error("{0}")]
    InvalidData(String),
    #[error("{0}")]
    Crypto(String),
    #[error("IO 错误: {0}")]
    Io(String),
}

pub async fn validate_offline_key_command(
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<OfflineKeyValidationResult, String> {
    match try_validate_from_env(&lock).await {
        Ok(result) => Ok(result),
        Err(err) => Err(err.to_string()),
    }
}

async fn try_validate_from_env(
    lock: &tauri::State<'_, FileWriteLock>,
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
        return Ok(build_invalid_result(
            "离线密钥文件不存在".into(),
            None,
            None,
        ));
    }

    let raw = read_file_with_lock(&env_path, lock).await?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(build_invalid_result("离线密钥文件为空".into(), None, None));
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
            "离线密钥已过期".into(),
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    let device_id = generate_device_id()
        .map_err(|err| OfflineKeyError::InvalidData(format!("无法获取设备标识: {err}")))?;
    if payload.device_id != device_id {
        return Ok(build_invalid_result(
            "当前设备与离线密钥绑定设备不一致".into(),
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    if now < server_time {
        return Ok(build_invalid_result(
            "检测到离线密钥时间被回滚，请重新获取".into(),
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

fn build_invalid_result(
    reason: String,
    expires_at: Option<String>,
    payload: Option<OfflineLicensePayload>,
) -> OfflineKeyValidationResult {
    OfflineKeyValidationResult {
        is_valid: false,
        reason: Some(reason),
        expires_at,
        payload,
    }
}

fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    let parts: Vec<&str> = raw.split(OFFLINE_KEY_SEPARATOR).collect();
    if parts.len() != 2 {
        return Err(OfflineKeyError::InvalidData("离线密钥格式无效".into()));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn combine_offline_key(rsa_part: &str, aes_part: &str) -> String {
    format!("{rsa_part}{OFFLINE_KEY_SEPARATOR}{aes_part}")
}

fn decrypt_license_payload(encoded: &str) -> Result<OfflineLicensePayload, OfflineKeyError> {
    let private_key = match &*PRIVATE_KEY {
        Ok(key) => key,
        Err(err) => {
            return Err(OfflineKeyError::Crypto(format!("无法加载 RSA 私钥: {err}")));
        }
    };

    let encrypted = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("RSA 密文格式无效".into()))?;

    let padding = Oaep::new::<Sha256>();
    let decrypted = private_key
        .decrypt(padding, &encrypted)
        .map_err(|err| OfflineKeyError::Crypto(format!("RSA 解密失败: {err}")))?;

    let value: Value = serde_json::from_slice(&decrypted)
        .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))?;

    serde_json::from_value(value)
        .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
}

fn decrypt_server_time(encoded: &str) -> Result<DateTime<Utc>, OfflineKeyError> {
    let combined = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("时间密文格式无效".into()))?;

    if combined.len() <= 12 {
        return Err(OfflineKeyError::InvalidData("离线时间数据损坏".into()));
    }

    let (nonce, ciphertext) = combined.split_at(12);

    let key = match &*AES_KEY {
        Ok(bytes) => bytes,
        Err(err) => return Err(OfflineKeyError::Crypto(err.clone())),
    };

    let cipher = Aes256Gcm::new_from_slice(&key[..])
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 解密失败: {err}")))?;

    let time_str = String::from_utf8(plaintext)
        .map_err(|_| OfflineKeyError::InvalidData("服务器时间格式无效".into()))?;

    DateTime::parse_from_rfc3339(&time_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| OfflineKeyError::InvalidData("服务器时间格式无效".into()))
}

fn encrypt_current_time(now: DateTime<Utc>) -> Result<String, OfflineKeyError> {
    let key = match &*AES_KEY {
        Ok(bytes) => bytes,
        Err(err) => return Err(OfflineKeyError::Crypto(err.clone())),
    };

    let cipher = Aes256Gcm::new_from_slice(&key[..])
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), now.to_rfc3339().as_bytes())
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 加密失败: {err}")))?;

    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

async fn read_file_with_lock(
    path: &PathBuf,
    lock: &tauri::State<'_, FileWriteLock>,
) -> Result<String, OfflineKeyError> {
    let guard = lock.lock().await;
    let result = std::fs::read_to_string(path).map_err(|err| OfflineKeyError::Io(err.to_string()));
    drop(guard);
    result
}

async fn write_file_with_lock(
    path: &PathBuf,
    data: &[u8],
    lock: &tauri::State<'_, FileWriteLock>,
) -> Result<(), OfflineKeyError> {
    let guard = lock.lock().await;
    let result = std::fs::write(path, data).map_err(|err| OfflineKeyError::Io(err.to_string()));
    drop(guard);
    result
}

fn generate_device_id() -> Result<String> {
    if let Ok(machine_id) = machine_uid::get() {
        if let Some(id) = hash_identifier(machine_id.as_bytes()) {
            return Ok(id);
        }
    }

    let mut hasher = Sha256::new();

    if let Ok(hostname) = hostname::get() {
        hasher.update(hostname.to_string_lossy().as_bytes());
    }

    if let Ok(interfaces) = get_if_addrs::get_if_addrs() {
        for interface in interfaces {
            match interface.addr {
                get_if_addrs::IfAddr::V4(ifv4) => {
                    hasher.update(ifv4.ip.octets());
                    hasher.update(ifv4.netmask.octets());
                    if let Some(broadcast) = ifv4.broadcast {
                        hasher.update(broadcast.octets());
                    }
                }
                get_if_addrs::IfAddr::V6(ifv6) => {
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

mod machine_uid {
    use std::io;

    #[cfg(target_os = "windows")]
    pub fn get() -> io::Result<String> {
        use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path = "SOFTWARE\\Microsoft\\Cryptography";
        let key = hklm.open_subkey_with_flags(path, KEY_READ)?;
        key.get_value("MachineGuid")
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get() -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "machine uid not supported",
        ))
    }
}
