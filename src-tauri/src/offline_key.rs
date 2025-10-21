use crate::FileWriteLock;
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Utc};
use if_addrs::{get_if_addrs, IfAddr};
use machine_uid::get as get_machine_uid;
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rand::RngCore;
use rsa::{pkcs8::DecodePrivateKey, Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    convert::TryInto,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use thiserror::Error;
use uuid::Uuid;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCydjvCe9dSjiNe
HYoZsty/TffOevm3B803yOdu3YkP3IFtjTJbQvzXmZjH96E315WccXW0lG3Npjzq
8n8VJ5t29MNizW9ck3jJoKDzSkXfFtO8e4ergz4PovRLFkh+gKxPC6RoWAm9c2w9
0/jfsEEiUaVAWw6SXGHDnYVnyrjZp3tz/dqo9enOAseFR2+RkwnIPIpAVZ+UpZ99
9S2T4vqEkFCftcs4GMjXClQBe0SgQhHK80GDPhlNbwyF5+l0DAViqd0kexmC8b7i
72aoHh3YeuHbgZ98bAEdR92ynoKQlZd6n3zljZIE8UqY6flnL6yMraV13mVYmKGg
nEQfy4kxAgMBAAECggEALZBTVKXixWCl9gcLteD9TFifPtgV/p2ezzHbqOol7udJ
IkvNAhD51iAQqml2f3fq6lzrhPXqjPl3DzNnr+KDZl06r3StDJFYYv5Aaa1aZomA
+Nv/ORKSm1JrFeq0CpxWof3idYOYxQZ9qdF/drkdACKhUuuMrmCo01VZ9LSE+ojD
So4rSgQQ9R8FleDyUWxHClyggOjYMV15VVUySaenbtzGQCy0QZeG79IqgqZz1nre
1f9o2r1r4lbnynuHfYgo6S0gGym3fZJ0dhFhSQgTNLfdvgGWPyc7dkna81ipuWau
i3Lfr3cWHgpTfkdjUYP1GaqSI/0fQDzVm1BNYfg+AwKBgQDonyUCosm6UjaugPcR
eIsKCriHnIdzeQQRfHjJeK+eRCbl0nXkH712P7s6V76fgrFAH41hdxQIGVCmKa7Q
7U+3RoWVcF3LoaEOlc4+AZgOTnSnF1a3mbCSMoGGrMLrvD9+pS7rmf/fWcmrjLkx
DzAvUkQY8Ztf3P2+NGuYDzzDRwKBgQDEZaxgxsX+EChQDQjBQ5AFNS+ySytt4p1c
DkDAYVWw9Riz3ZkP3yHLuko0+7RzxoIK3nTdBghaSxQ3SvfGVBWEjF0NbkZP9s/Z
u28mjFNHHjUKCA3sr6HGLYeR3pzWQLhKW0A/3ZtOPYLj7Lao81SNkPOhZFm808xO
8CSDfG7bxwKBgQCO4Hi99riKnUaCxil6bJyRrWYLvUOg1BqAlwAlVuAfCGMP08Wf
OTOIdrqLqismFALEjNysmZQPKWVUudNq9ed5fXI9CEhD82FV8QM9KIN5fgy+OGKF
4HsIQMc3rdMHMZeaNODtyqfTSnXIzWVN0bNZzWCQJY22QqkDc3UGb411rwKBgCgk
K9Zf6kniXYr3DwoJWB9oXoZPjOHZxpXxJ9TqUAxqHBFvQoCW955fRhmMNLbRJPU5
wKMIP57M56XhgcEcoIVF9yLunhpr9NGo2LAFUGQhzW9udAIjZ6pM1f+/g0jbU4+H
FRu4nKyiL+WMFU105pxEuzcKfrj0hTbBKIVjYnkzAoGBAKl2p8d84so6JcD5RYaC
clpoYEj6FOPgmhQ9E5BJoBH4epM+c/Vu9JK2fi6XivT5lxH/dWF7B5uf9APUqtf/
CDlno4vHTRWwc7jcWCjYTr8HkXXk2yZx0O7hmi1Nritj8fDPnZZsjusVcTWjkD2P
FiGkZG9JMgvYIbk35WnPeiSK
-----END PRIVATE KEY-----";
const OFFLINE_AES_KEY_B64: &str = "deG64Nsdhem0HpZE2Gm/vj6VE5hGNRDhTsaw/PNxMS8=";
pub const OFFLINE_KEY_SEPARATOR: &str = "|||";
pub const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";
const OFFLINE_RSA_PRIVATE_KEY_ENV: &str = "ZHIGONG_OFFLINE_RSA_PRIVATE_KEY";
const OFFLINE_RSA_PRIVATE_KEY_PATH_ENV: &str = "ZHIGONG_OFFLINE_RSA_PRIVATE_KEY_PATH";
const OFFLINE_AES_KEY_ENV: &str = "ZHIGONG_OFFLINE_AES_KEY_B64";

static PRIVATE_KEY: Lazy<Result<RsaPrivateKey, OfflineKeyError>> = Lazy::new(load_rsa_private_key);

static AES_KEY: Lazy<Result<[u8; 32], OfflineKeyError>> = Lazy::new(load_aes_key);

#[derive(Debug, Error, Clone)]
pub enum OfflineKeyError {
    #[error("输入输出错误: {0}")]
    Io(String),
    #[error("数据无效: {0}")]
    InvalidData(String),
    #[error("加解密失败: {0}")]
    Crypto(String),
    #[error("互斥锁损坏")]
    Poison,
    #[error("其他错误: {0}")]
    Other(String),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineKeyValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub expires_at: Option<String>,
    pub payload: Option<OfflineLicensePayload>,
}

fn read_file_with_lock(path: &Path, lock: &Arc<Mutex<()>>) -> Result<String, OfflineKeyError> {
    let guard = lock.lock().map_err(|_| OfflineKeyError::Poison)?;
    let content = fs::read_to_string(path).map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    drop(guard);
    Ok(content)
}

fn write_file_with_lock(
    path: &Path,
    data: &[u8],
    lock: &Arc<Mutex<()>>,
) -> Result<(), OfflineKeyError> {
    let guard = lock.lock().map_err(|_| OfflineKeyError::Poison)?;
    fs::write(path, data).map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    drop(guard);
    Ok(())
}

fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    let mut parts = raw.splitn(2, OFFLINE_KEY_SEPARATOR);
    let rsa_part = parts
        .next()
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥缺少 RSA 部分".into()))?
        .trim();
    let aes_part = parts
        .next()
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥缺少 AES 部分".into()))?
        .trim();
    Ok((rsa_part.to_string(), aes_part.to_string()))
}

fn combine_offline_key(rsa_part: &str, aes_part: &str) -> String {
    format!("{rsa_part}{OFFLINE_KEY_SEPARATOR}{aes_part}")
}

fn decrypt_license_payload(encoded: &str) -> Result<OfflineLicensePayload, OfflineKeyError> {
    let private_key = private_key()?;
    let encrypted = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("RSA 密文格式无效".into()))?;
    let padding = Oaep::new::<Sha256>();
    let decrypted = private_key
        .decrypt(padding, &encrypted)
        .map_err(|err| OfflineKeyError::Crypto(format!("RSA 解密失败: {err}")))?;
    serde_json::from_slice::<OfflineLicensePayload>(&decrypted)
        .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
}

fn decrypt_server_time(encoded: &str) -> Result<DateTime<Utc>, OfflineKeyError> {
    let aes_key = aes_key()?;
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
    let cipher = Aes256Gcm::new_from_slice(&aes_key[..])
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;
    let nonce = Nonce::from(nonce_array);
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
    let aes_key = aes_key()?;
    let cipher = Aes256Gcm::new_from_slice(&aes_key[..])
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = cipher
        .encrypt(&nonce, now.to_rfc3339().as_bytes())
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 加密失败: {err}")))?;
    let nonce_bytes: &[u8] = nonce.as_ref();
    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(nonce_bytes);
    combined.extend_from_slice(&ciphertext);
    Ok(general_purpose::STANDARD.encode(combined))
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

fn hash_identifier(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hex = format!("{:x}", hasher.finalize());
    shorten_hex(hex)
}

fn generate_device_id() -> Result<String, OfflineKeyError> {
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
    if let Some(shortened) = shorten_hex(fingerprint_hex) {
        return Ok(shortened);
    }

    let uuid = Uuid::new_v4();
    hash_identifier(uuid.as_bytes()).ok_or_else(|| OfflineKeyError::Other("无法生成设备ID".into()))
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

fn load_rsa_private_key() -> Result<RsaPrivateKey, OfflineKeyError> {
    if let Ok(path) = std::env::var(OFFLINE_RSA_PRIVATE_KEY_PATH_ENV) {
        let path_buf = PathBuf::from(path);
        let pem = fs::read_to_string(&path_buf).map_err(|err| {
            OfflineKeyError::Io(format!("读取离线私钥失败 ({}): {err}", path_buf.display()))
        })?;
        return RsaPrivateKey::from_pkcs8_pem(pem.trim())
            .map_err(|err| OfflineKeyError::Crypto(format!("解析离线私钥失败: {err}")));
    }

    if let Ok(inline) = std::env::var(OFFLINE_RSA_PRIVATE_KEY_ENV) {
        return RsaPrivateKey::from_pkcs8_pem(inline.trim())
            .map_err(|err| OfflineKeyError::Crypto(format!("解析离线私钥失败: {err}")));
    }

    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY)
        .map_err(|err| OfflineKeyError::Crypto(format!("解析离线私钥失败: {err}")))
}

fn load_aes_key() -> Result<[u8; 32], OfflineKeyError> {
    if let Ok(inline) = std::env::var(OFFLINE_AES_KEY_ENV) {
        return decode_aes_key(inline.trim());
    }

    decode_aes_key(OFFLINE_AES_KEY_B64)
}

fn decode_aes_key(value: &str) -> Result<[u8; 32], OfflineKeyError> {
    let decoded = general_purpose::STANDARD
        .decode(value)
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥格式无效: {err}")))?;
    decoded
        .try_into()
        .map_err(|_| OfflineKeyError::Crypto("AES 密钥长度错误".into()))
}

fn private_key() -> Result<&'static RsaPrivateKey, OfflineKeyError> {
    PRIVATE_KEY.as_ref().map_err(|err| err.clone())
}

fn aes_key() -> Result<&'static [u8; 32], OfflineKeyError> {
    AES_KEY.as_ref().map_err(|err| err.clone())
}

fn try_validate_from_env(
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
        return Ok(build_invalid_result(
            "离线密钥文件不存在".into(),
            None,
            None,
        ));
    }

    let raw = read_file_with_lock(&env_path, lock)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(build_invalid_result("离线密钥文件为空".into(), None, None));
    }

    let (rsa_part, aes_part) = split_offline_key(trimmed)?;
    let payload = decrypt_license_payload(&rsa_part)?;
    let server_time = decrypt_server_time(&aes_part)?;

    let expires_at = chrono::DateTime::parse_from_rfc3339(&payload.expires_at)
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

    let device_id = generate_device_id()?;
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
    write_file_with_lock(&env_path, updated_raw.as_bytes(), lock)?;

    Ok(OfflineKeyValidationResult {
        is_valid: true,
        reason: None,
        expires_at: Some(payload.expires_at.clone()),
        payload: Some(payload),
    })
}

#[tauri::command]
pub async fn validate_offline_key(
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<OfflineKeyValidationResult, String> {
    let inner = lock.inner().0.clone();
    tokio::task::spawn_blocking(move || try_validate_from_env(&inner))
        .await
        .map_err(|err| err.to_string())?
        .map_err(|err| err.to_string())
}
