use std::path::PathBuf;
use std::sync::Arc;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::RngCore;
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use tauri::State;

use crate::device;
use crate::FileWriteLock;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDfJcEg5r7eE3f6
405D8m+ttCsWfgCr+8LAVib13+/z0hqsTjcm9qRWEcBZLwoG8h/KJ0vGBgVtGNm3
zt71MQerWRYm3eFjdhjaeKeaJDVxptGG3VRX9BbV+z8Zn09CZnpYH5sr+FkwX5qz
pGRjE2aP8ZSu6AfZyuzfqQytGD2lbDstuOcy/60XNxXJd5XC3vh6sdDw4/jx0vf6
UM2B4KqvXFaEPOtLWvD2tH0rkxuWJdreZoJciwulcc+tLlXqeS2eyfVEUDNxTVGK
SYU4m1iHBznDGtgroeann1n6f5/frf62WY1kAHWjTuvbhaYJqJCFdYEzfm7qqR1c
aMQL4iKjAgMBAAECggEAIkcck3nD5b2MMibFnda5b1pG6NBtOP4A92ZPMM4iqsu3
sN0znVFzgrcnsBYeOQfreImwcEL6MBKKVdu6Jbmk3x4zCBW7ACTNgjiDgmrqfMKu
hUQtfHSbJ2lN/ZkaZe2eVeLGWtWY7xSDNpzbli6OPtv8xiC6yAAN1/jnYMA3BAMX
5qwg/rg+VUrZzAOUjRz2FPZogkNizNXznTu+9O4hODzHIOSXnE68yemKeJldfPBR
py2Jms6vjfsCGB/T68Twigw9NeHXKPi4K/d5RV5GYxU0T417rboRZEPGIg/+XevW
WDeHgKz3iKHxg6biR7cW+Ecj/8GIk1ZBxMFmki9NsQKBgQDvX6rvkXj3n7HOHljJ
7SJu7cqJ9EfOqWaG3nRckXXDrD/8rH1UFOMXKZG/lk5T5qvv2M03IXBPhF4+RE/h
ufvulP6SWRdIwvK0HT1lZP8zJycMiVjQ9pOBbSufB8S0G5fxWVzAacBs+idu/XkZ
JlDNs0QZo6Vq69hwgskNszs4EwKBgQDupZD3P8ddkNFokZwFSqAu2tgcCFB6T7pk
X2ESIEkt8RpKVLKYpMIrXsGQ2pJXUrLPL3nRa6QyKKpbZh25jKCEZY+25mGJ7pZC
fZtGtScvtfsayphdcNjDIUGJHpv4n2HjVfsQ4HP2ohb56tNXMi5OhQ4QSYsrG9jJ
5f2qNczdMQKBgQDUpB2LF6VoA7rFqXuiVT9jX2WuywwG78EeSfsASE9e9WMaKHhk
+vEIoGrS0MjgC2ftdqBGOzQzzxiRI8n6cWb1d/H1O6NNhbBohBkIO4HghQVRQ6lc
Z859COfZK+N3Q5PkWEfmvYqsxEPHeu/agYkCUh9Kz26g3sW0nNnRi8gtdQKBgQCM
rLxPgyEaXouSgILf+6WA+S/7FkuaTfUpxa+K5807b6x3tIWyxSxQNIMVyEILh4wv
5WSBtloL1zzUs9VB00urv9J/lj5Y+HNKrpAhlheLYFKs8E/whNzB6ZCgK5L1c75Y
LA7bW/P/pny0xV/fPo1da8FmiHOo7blVCmYFN8GTgQKBgGbx3aDJSuGKNlsbWGyL
JT41DsrCzGw85pNbDxw7jf50JT419/bmbLcmcO3qAcCd1A74vDv34jxOV0ng5M05
7v7jai9DeUfkiwDvmbtBFptw23tf3C4GPrM41pdhSj6DMh1BvLeSKTC4gvTb5HQW
DeoPfre/09ClsHtFZbWjW4NN
-----END PRIVATE KEY-----";
const OFFLINE_AES_KEY_B64: &str = "fCpTzB16wHtAGRpkf+sZZEnXgixMdXrRccpZg3rjXp0=";
const OFFLINE_KEY_SEPARATOR: &str = "|||";
const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY).expect("无效的离线密钥私钥")
});
static AES_KEY: Lazy<[u8; 32]> = Lazy::new(|| {
    let bytes = general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .expect("无效的 AES 密钥 Base64 编码");
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    array
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

#[derive(thiserror::Error, Debug)]
pub enum OfflineKeyError {
    #[error("数据格式错误: {0}")]
    InvalidData(String),
    #[error("加解密失败: {0}")]
    Crypto(String),
    #[error("IO 错误: {0}")]
    Io(String),
}

impl OfflineLicensePayload {
    fn from_value(value: &Value) -> Result<Self, OfflineKeyError> {
        serde_json::from_value(value.clone())
            .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
    }
}

pub async fn read_file_with_lock(path: &PathBuf, lock: &Arc<Mutex<()>>) -> Result<String, OfflineKeyError> {
    let _guard = lock.lock();
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(|err| OfflineKeyError::Io(format!("读取离线密钥失败: {err}")))?;
    Ok(content)
}

pub async fn write_file_with_lock(
    path: &PathBuf,
    data: &[u8],
    lock: &Arc<Mutex<()>>,
) -> Result<(), OfflineKeyError> {
    let _guard = lock.lock();
    tokio::fs::write(path, data)
        .await
        .map_err(|err| OfflineKeyError::Io(format!("写入离线密钥失败: {err}")))
}

fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    let parts: Vec<&str> = raw.splitn(2, OFFLINE_KEY_SEPARATOR).collect();
    if parts.len() != 2 {
        return Err(OfflineKeyError::InvalidData("离线密钥格式无效".into()));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
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

async fn try_validate_from_env(lock: &Arc<Mutex<()>>) -> Result<OfflineKeyValidationResult, OfflineKeyError> {
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
        return Ok(build_invalid_result("离线密钥文件不存在".into(), None, None));
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

    let device_id = device::get_device_id()
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

#[tauri::command]
pub async fn validate_offline_key(
    lock: State<'_, FileWriteLock>,
) -> Result<OfflineKeyValidationResult, String> {
    let inner = lock.0.clone();
    try_validate_from_env(&inner)
        .await
        .map_err(|err| err.to_string())
}
