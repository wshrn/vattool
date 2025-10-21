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
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC4QLFhLhn4V/pk
dFIMj9jFnjFbEdnWVCXQ9LXhDtfRWXxawtx0r804MobEd25z8iI/nxJdKpbJs7Tm
8UHb+uqJ/zfJ2l6jDjkDbDSjV6MWmgnmosFdOTPYPTn2TjVjzWpXB3r9Y5sMSvF0
/3/1WMWAdhf2Nabeh5eflmSeQ1Uv9C0Q6VXvuFCSn7UoMCDLOHXJNQORpodgiC1p
7Vl5ZytKtcdcVi5/AcjL/XNIOLJk0n7+unzP1Nla9oKBJgzV2FB9ecuxpArwQ6dQ
rW9ARXtHQQv4l0q0/j1eI/6HnZvk6f1xuJmQjOE4OWza4Twz6NWzi+2RSrKaCsqX
/1pQoNMJAgMBAAECggEAKUMPliBBX5ywLdPg1gBWvra0/dyLCJTynQ9YNczhpvff
weGWhikij529EX1fhmaopc/FSIjzmLr+XaOUqKNR59J4V2NoQyK5wNr4FMZY9wRL
CFPVcr+PLTNU6iRMj4ueb1v0/o7SV5fm59kZ+kNFg4Wuywvr0TTTT0FaShjxGFo4
YzErmhh/RZBdcA0JTNUyjbmgwOHoOq9tlxSpHqv0ORMxe8jqDSPexb/Z2c4/Zm2A
uPugoqwR5nBmBHd426FobuZT0/MKbo3HwP+NZp6RAPVGWRcygWN/E9Y0p6nAWdL+
tpWUktyRVVIAz1pXBSWyniQxwylHhEiADmgvrpa5qQKBgQDkGIVPQJP9u2rVW1H/
2Q186jtT9v2skMn0Pj/gqYk1ZWf6ch6q/F9yGDNDenrqiX3W/JmQ5O4HWFwh0ATu
HJxrqmYwmBrVby0Z0JJG6JS933HtfDMu8QkFOYnEu0X+3eDoPpXSc6xdknT8zVf4
4vYmKtlPd5MIfae3Ms0OAO9upQKBgQDOyxlvczKjRMEEzoZ0bgMbxoao0ZWWT9K7
NzP2JofSIyqbxSKYB+YdSX2sAvC4DOwi11OS0q/4VPr8VjnkVPNLq8F/lmlgvBW4
BW8Tr+z1i34FnV/rpdn7noOp4SlzXSmI2mqYkUv+YteLSJ8IqjMConAcE0NKmfmp
pk5wyckplQKBgA9HbTaf1sn6Ue+0zEtdGMAzWIIJW3jBwiVwPgsokB5ZipuGJXPC
sAoOgPCWNcGcMCfEh+ziyOcJDjLdolbo57l2kp3Ssol1hwnhpMrHLZ+CZjlIRo1w
a/BDqGzbNpcZ+cTU3Ghag0NJWjjM8IWlfmOUHzZphhndgOyOpJm5ilBZAoGAO43u
Q1SPzslsNTAtNLbCGmuwOEozpFhUvioFwuwRzYjnKnk5n0MXGHQjxzgJj1fZYadV
oEEhAImoxqcmgQWeE7rhPRdaPcutDZQzCx5tRcHoh0FtcHYRMw/Rp0j7IQhBf/I3
JL0jf52Dqc8+TcaGbknNs6gwhvmVFzCYAo96aYkCgYEA4wZbjcs008F41+83BatW
O04wWEGuJEenV+gmmUxP7Zm4bF+PUEFma1A8lODA61//rJZsLqyzcQV9sLxlLTrS
Q0Ng98hW9rZfgc6Smzz5O5Z9tUEg9FCYSGWwBLq7JIGNtR2qWv0favoMF1Lu1OLW
tf48SkBaDknGHpihfb3iEnE=
-----END PRIVATE KEY-----";

// RSA 公钥（2048位）- 用于生成密钥时加密
const OFFLINE_RSA_PUBLIC_KEY: &str = r"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAuECxYS4Z+Ff6ZHRSDI/Y
xZ4xWxHZ1lQl0PS14Q7X0Vl8WsLcdK/NODKGxHduc/IiP58SXSqWybO05vFB2/rq
if83ydpeow45A2w0o1ejFpoJ5qLBXTkz2D059k41Y81qVwd6/WObDErxdP9/9VjF
gHYX9jWm3oeXn5ZknkNVL/QtEOlV77hQkp+1KDAgyzh1yTUDkaaHYIgtae1ZeWcr
SrXHXFYufwHIy/1zSDiyZNJ+/rp8z9TZWvaCgSYM1dhQfXnLsaQK8EOnUK1vQEV7
R0EL+JdKtP49XiP+h52b5On9cbiZkIzhODls2uE8M+jVs4vtkUqymgrKl/9aUKDT
CQIDAQAB
-----END PUBLIC KEY-----";

// AES-256 密钥（Base64 编码）- 用于时间戳加密
const OFFLINE_AES_KEY_B64: &str = "94/AR7dd8gIstLEXp3LCs865DptiMKlh8nLjjDEcO40=";

// 密钥分隔符
pub const OFFLINE_KEY_SEPARATOR: &str = "|||";

// 环境变量名称
pub const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

const OFFLINE_RSA_PRIVATE_KEY_ENV: &str = "ZHIGONG_OFFLINE_RSA_PRIVATE_KEY";
const OFFLINE_RSA_PRIVATE_KEY_PATH_ENV: &str = "ZHIGONG_OFFLINE_RSA_PRIVATE_KEY_PATH";

// 懒加载初始化加密密钥
static PRIVATE_KEY: Lazy<Result<RsaPrivateKey, String>> = Lazy::new(|| {
    let pem = match resolve_private_key_pem() {
        Ok(value) => value,
        Err(err) => return Err(err),
    };

    parse_private_key(&pem)
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

fn resolve_private_key_pem() -> Result<String, String> {
    if let Some(path) = read_env_var(OFFLINE_RSA_PRIVATE_KEY_PATH_ENV) {
        let path_buf = PathBuf::from(&path);
        return std::fs::read_to_string(&path_buf)
            .map_err(|err| format!("无法从 {} 读取 RSA 私钥: {err}", path_buf.display()));
    }

    if let Some(pem) = read_env_var(OFFLINE_RSA_PRIVATE_KEY_ENV) {
        return Ok(pem);
    }

    Ok(OFFLINE_RSA_PRIVATE_KEY.to_string())
}

fn parse_private_key(pem: &str) -> Result<RsaPrivateKey, String> {
    let normalized = normalize_private_key_pem(pem);
    RsaPrivateKey::from_pkcs8_pem(&normalized).map_err(|err| format!("加载 RSA 私钥失败: {err}"))
}

fn normalize_private_key_pem(pem: &str) -> String {
    let trimmed = pem.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.contains("-----BEGIN") {
        trimmed.to_string()
    } else {
        let mut wrapped = String::from("-----BEGIN PRIVATE KEY-----\n");
        wrapped.push_str(trimmed);
        if !trimmed.ends_with('\n') {
            wrapped.push('\n');
        }
        wrapped.push_str("-----END PRIVATE KEY-----");
        wrapped
    }
}

fn read_env_var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
