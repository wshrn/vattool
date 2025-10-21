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
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use thiserror::Error;
use uuid::Uuid;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDVKT08cxtJyhsx
l9A0+6o7PR5EN12E/gaVXSInY/g9GgTeCcaXBm7FifosFCaVGcvmjJrhxTaNsp/7
VM+S7jJVfmIYuVnwhhGdis5iJlG8fL2ekG1c1HtShxK7vcrhv8lDrc3zEIbX1v6r
9sy4T8gZH8peg6dnzmMKHRPhXGscr2SIHn3xsdkP/kCY+4mOsRLdV0IESho0BsNC
W4smTp4lx9zZKM9Q6DNF62B/2Gd4v+vTixogouQVbSJspTg/AbRx+snZbvX1Fe2d
I1pFvosWO/rh/K2Wt4PW9KUoQOQXt1WUWIQv5+4FQI82zcRR0BuUf4bfPnUy64ms
b8cfUA+1AgMBAAECggEAAP1hTitPG/u3iLR3KIeQfq0dmpZ8ED8KS0T2xtgktxUT
NCGfY2/tkB08yTkZwd2vmW6szCbQ8M8u75K5pjkwPYNeOphG6/AJ3utPIqjE2DM
e44Imv3ybL5PV5++bucJkZ3JXzjIhBO5k78Ha2ZVvoJqxwuzhfE5SKWziUvfNPZ
YZpQi6J4UhAY6MV6LR18QHs/8R0ElS5x89y8KeHp4Y0rrxsQTkF7TZcf/IO36yC
f4Wdbe3Es5zjfyEeKmR9Q0aiVXexMQm35sGRLDYSVX+cQ3c51lHPJWLhBzv3Qgf
Udcm51Nm2uoDxZ2hxIa9WtSgFRGo4eFse70hSE1EQKBgQD0r9XsWcAweJWx4f7d
lWRRp2guI0NeedzuqQ6+ew5gSqNtMnQFD31VP0M8Gx+1+nD9G17pNPhn7wiShkJz
tI8NwScUsfvv1AOJNkyeoLmq3pymC6CW8sbh08uGeRd+7pScGtMEtLOFZOhKs3B
c7R1yjL+k8IiQOCMG4gTk4sbwKBgQDa1TG7L4bn/BQigL+BKbP7+b6dMSbtn4SS
12a7xk9QxcYwbgj+oRL1ZkEnEwPaYZXpBOPn0IZLmw+fXNcJakl1QT+PqcRrJv0
54S1LcwkMZrL0lC7eWZQtt5nqRxXOARFrZRcw4q7th8x8qS2Fum1KwFvLkp+2EG
/INg/Q0NXwKBgQDsatE4IjO7EwzO592mqWED+FJ/5AUmuSWGubjaGAKf7akIGiY
Nv7kOXYNgU8Hjh1sMKT5AlSZh4if4+zpmP7r7BcMu915f9e2s9Gok2xgfpv5xKr
SIachFPnXQXxLGxjyBGPwmRBJcd6pWMK71cp6IbGfv3uQ4ds8I8ylTDewKBgCYD
3axQYFbbcoRKC5UIQ0gFK6810OcQi+7DKuGB8DIn33ASyNDUdK1+wNGOdhpIaFS
aC8EXcw5idVO/Fq/vh1p4/k9dCHAxY6Mmkt6hU4+lsFrlq9SRs8K3cAOWqzQtEa
/OW80z98aZsKmv7pavGswwZlCbSRpV8DpRxpKH9AoGBAJeGbspe/QzuwmMPVRLD
c+oL2ZqfFKw0vWD1+W5xvrkuGQ6Ob9KEUXO1qPaZkp3pcme0ayifPqEAQ+cvbbm
ikZgvYcVsFkSV/pfUll+3Z0MMfZ0zBXIR+vZ7Si1P5W4iGuN3m8dMXwNVdmlWMS
BSXJQZfwmJpL77n0ASk8Joo
-----END PRIVATE KEY-----";
const OFFLINE_AES_KEY_B64: &str = "94/AR7dd8gIstLEXp3LCs865DptiMKlh8nLjjDEcO40=";
pub const OFFLINE_KEY_SEPARATOR: &str = "|||";
pub const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY).expect("invalid offline private key")
});

static AES_KEY: Lazy<[u8; 32]> = Lazy::new(|| {
    let decoded = general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .expect("invalid AES key");
    decoded
        .try_into()
        .expect("AES key must be 32 bytes for AES-256")
});

#[derive(Debug, Error)]
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
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥缺少 RSA 部分".into()))?;
    let aes_part = parts
        .next()
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥缺少 AES 部分".into()))?;
    Ok((rsa_part.to_string(), aes_part.to_string()))
}

fn combine_offline_key(rsa_part: &str, aes_part: &str) -> String {
    format!("{rsa_part}{OFFLINE_KEY_SEPARATOR}{aes_part}")
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
    serde_json::from_slice::<OfflineLicensePayload>(&decrypted)
        .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
}

fn decrypt_server_time(encoded: &str) -> Result<DateTime<Utc>, OfflineKeyError> {
    let combined = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("时间密文格式无效".into()))?;
    if combined.len() <= 12 {
        return Err(OfflineKeyError::InvalidData("离线时间数据损坏".into()));
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(&AES_KEY[..])
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 密钥初始化失败: {err}")))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
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
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), now.to_rfc3339().as_bytes())
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 加密失败: {err}")))?;
    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
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

    if let Some(id) = shorten_hex(fingerprint_hex) {
        return Ok(id);
    }

    let uuid = Uuid::new_v4();
    hash_identifier(uuid.as_bytes()).ok_or_else(|| OfflineKeyError::Other("无法生成设备ID".into()))
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
