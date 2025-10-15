use std::path::PathBuf;
use std::sync::Arc;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use if_addrs::{get_if_addrs, IfAddr};
use machine_uid::get as get_machine_uid;
use once_cell::sync::Lazy;
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::state::FileWriteLock;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDahqDENvOx4kym
xtiR4Mqq275YWHQMdNjeREJ/0e2qOzuJZWnaIu5XByUdkvTKh7Khp3LFZ4sldKKe
AaPVTyyO1+fK326yl89ih7XWv2ELamn3aZCSM/tuNpa3fQZWCvK48nsK6IUwWWsh
K1Q4k6fq5mr8ko1OGHQlXhDZR5nmHaq1+Wjnx09/POEXIe2cs7ebBqExn5v9Hfnj
loRO6LX9nWk79SJMGrl/SeahrJOGVklzylfy5kSMXJZfu9o9r8T9uH0jUFKSZ2Jq
IYJU8UWZbriJPjUMTOQMFJnB9knzargamPkrNAvXXIiuJfgTw/ZYpJ8k4cS3Xk7t
CtQRq9ABAgMBAAECggEAAN9ZyufB4sJlskKj6qbvQzXu81YI0lQI+b/ztKRARNJB
SFDePpqeKUz6GI1prplyqLlhFIV/l1DpLWyh8HoNOlWlh1xrkhEn6OekHc8oPGAW
m8gtNX65crrrQJC6UMeQ2RT/gZNcpQUN39k/GA3bnT0R1Tfh8ltn0w5+Xmo2LK5s
sB+OB5BR/kRMC8lgcKQUbZT9vDNgof3naX32lJqMxQbklXPAn8/tWQClg/hqBApE
ieb3+qGMKrCdWcQpWv8YDXKeGtLpRZTT56XmFetS1+dGsbsP85oid0Byqbvf5hGk
2vcFiteojGCGkkIViafDrhvD5X2xA5pQGps+DcaPYQKBgQD3MCrO852Z9EeNt/R/
lBiTCUWR2/DtgtFK+DaipNwSpf5Cb7W2zti7/8a76RrP4snjgaLUIZ756VygGLYA
YhZtkjjn/YZaD01r4cjG3/4SvzxbyQeUH8PCJ4BSC+NmrjrpFkB8zyRmcfrahrjg
S+P5MNUHYaEtHHNIbyGF57dE4QKBgQDiUOPDJNTe0yqWCLJcIVoax7SUuhIHLgqd
OhKr0Xw0tblGr4uwhjoZNMtGPvOtXUFp6D20s0JKrfnWiQ1A1LVE6YNC5f61ttPm
WhDGpEa/DwjmRJ/+Slhz2hW5XLLI6L+8UjeE9jRhk+eV95PwUcammfAO3+/XEKVr
SHbz85rPIQKBgQCpuWn7VWzGU+N5nHCF1OMWuowJPbJs8qDQUCP7gyUUrOMrLNbh
YZ+RXmtPhRwC15511wI9k6Q7xo7x4mW9V5w8ueTW7c39Mnqfc5VPcGdc6fAAucS5
YSIhwsuqCj2muET83GmFCRneOa7bsxtn7tPmuy/adJ+dgOZrOtsSvWqqIQKBgDaS
YMl+iSIN81foUvHqTOrmOwId+BjpHkAZb8Ukp3miVEHaNKnlQ/8t/tI6fdHRCeP6
z15SV2rglr0HoT1/zRIH6NHp2ciBhIkoMKFlnXWWr4OvCpFr5fZRoSloU+gkT4+e
l6qlH65j1tUPtQme/nLP7dX4Y8INToYXHC/pDuxhAoGAKqKvHvLL07qjb1XFATm9
usRnBlnYyVRxeFFrMpEeJvAtAAhtkJ0S2JUP4aIsWKJthX+Axh1qYbnobye8TnjT
LTOz2l2/2YDpgAi9ZYd9z3uzgwI2Fbv+1/ItRga/rbPl3j6SxAkslqUqpPX5hxUI
n7pByfFx9/dhFI5Xf+HbYN4=
-----END PRIVATE KEY-----";
const OFFLINE_AES_KEY_B64: &str = "Q5lhjTmPAHaHHNcbheW3x3tUL05rn+NuC/AgRk+ZVH4=";
const OFFLINE_KEY_SEPARATOR: &str = "|||";
const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY)
        .expect("invalid RSA private key configuration")
});

static AES_KEY: Lazy<[u8; 32]> = Lazy::new(|| {
    let decoded = general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .expect("invalid AES key");
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&decoded);
    bytes
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
    #[error("离线密钥文件读取失败: {0}")]
    Io(String),
    #[error("离线密钥数据无效: {0}")]
    InvalidData(String),
    #[error("加解密失败: {0}")]
    Crypto(String),
    #[error("内部错误: {0}")]
    Internal(String),
}

#[tauri::command]
pub async fn validate_offline_key(
    lock: tauri::State<'_, FileWriteLock>,
) -> Result<OfflineKeyValidationResult, String> {
    let guard = lock.clone_arc();
    try_validate_from_env(&guard)
        .await
        .map_err(|err| err.to_string())
}

async fn read_file_with_lock(
    path: &PathBuf,
    lock: &Arc<Mutex<()>>,
) -> Result<String, OfflineKeyError> {
    let _guard = lock.lock().await;
    tokio::fs::read_to_string(path)
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))
}

async fn write_file_with_lock(
    path: &PathBuf,
    data: &[u8],
    lock: &Arc<Mutex<()>>,
) -> Result<(), OfflineKeyError> {
    let _guard = lock.lock().await;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    }
    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    file.write_all(data)
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    file.flush()
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))
}

fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    let parts: Vec<&str> = raw.split(OFFLINE_KEY_SEPARATOR).collect();
    if parts.len() != 2 {
        return Err(OfflineKeyError::InvalidData("离线密钥格式不正确".into()));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn combine_offline_key(rsa_part: &str, aes_part: &str) -> String {
    format!("{}{}{}", rsa_part, OFFLINE_KEY_SEPARATOR, aes_part)
}

fn decrypt_license_payload(encoded: &str) -> Result<OfflineLicensePayload, OfflineKeyError> {
    let encrypted = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("RSA 密文格式无效".into()))?;

    let padding = Oaep::new::<Sha256>();
    let decrypted = PRIVATE_KEY
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

    let (nonce, ciphertext) = combined.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(&AES_KEY[..])
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
    if let Some(id) = shorten_hex(format!("{:x}", fingerprint)) {
        return Ok(id);
    }

    let uuid = Uuid::new_v4();
    hash_identifier(uuid.as_bytes())
        .ok_or_else(|| OfflineKeyError::Internal("无法生成设备ID".into()))
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

pub async fn try_validate_from_env(
    lock: &Arc<Mutex<()>>,
) -> Result<OfflineKeyValidationResult, OfflineKeyError> {
    let env_path = match std::env::var(OFFLINE_KEY_ENV_NAME) {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            return Ok(build_invalid_result(
                format!("环境变量 {} 未设置", OFFLINE_KEY_ENV_NAME),
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
    write_file_with_lock(&env_path, updated_raw.as_bytes(), lock).await?;

    Ok(OfflineKeyValidationResult {
        is_valid: true,
        reason: None,
        expires_at: Some(payload.expires_at.clone()),
        payload: Some(payload),
    })
}
