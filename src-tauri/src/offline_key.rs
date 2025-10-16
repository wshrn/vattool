use std::path::PathBuf;
use std::sync::Arc;

use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose;
use base64::Engine;
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rand::RngCore;
use rsa::pkcs8::DecodePrivateKey;
use rsa::{Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::device;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC9DQjmnBG05KZ3
y8CQBJCE4I2pyZ1O+zw/B0fLFcEFbsNJscBEP0mTd6Gj9DhtsOFgXNUQqTnngErU
v355iB2HBnc7h1cJDc2zL1toheE+Mc49xL3p354sGVXvePkby3QG3MMmCzTVqKYY
M9O+oht3AFkzOwGSyHH8Hyxr2eH2/K5ohCKJVvz/oiZJm1dwIgaDG4lxpbCtkZWv
/XfNqnC3HoWE/nDch9adgYPtvQpGi+arZ54Q2jXn8qaSj0Jg20daAPCCekWGOS+q
oIsUR9izv0WCii0+177fhIhoU6IEaL4Wgd7c6xcrWAMvGyb6aIg5mg798aP96q8S
TMp+BSC3AgMBAAECggEAXAzpPRRYAMrmjWUN5XH7hl+qSrZLHMZyzsO2PtngSstk
VlEw62ofYqj48RJ8mVhrXNbGvd6QPbr3dMCrDfTrX7ATxU4AYH2yn2FbMjkh8/0f
TW8rA6Ho69MIR1qRfOmjHKZIdRE20CPWf+lYXDWlhtuKE0pTWUu1SIxLPlZwUqhe
1BUBi3cM/FJoxV8VitbPqyM2vYxyYXWPntXGSOW+nGoKV6vaTe7+AoOonZvNFGBm
k3dOfWgia/9hDHhiOYNtwPhOHnreCSF7uAm2BMRAs5OWwfrLfEfK+8xuRYGQFwcT
sbPMe9oqp+yR2cnF+H8efSaXL5/LHTANU5fi1AnA2QKBgQDiwJspgdXKZLb9J71e
dGYbzNYiDVbNIaxH+5DLeqpnhXEtyTJ0Y6iJAAlKgS5kt7n7rP8oHsn1oMpxGvk9
LfAbhpApLURBmTl3eYssZJ+b5bkvzeJ2cDMjY1D8Zb5xDK3huiXP2pyWnnArT0Wq
Hl270d+lbk9Aqs9EzmPvVa0RyQKBgQDVb4WLsRYSCPqw/XJFpXhOxKwdukqIgjN5
OSwxHDf7/ONlVXwv7gaLntgYbPWBQDyCKWGU81iYRFnOpQ7HztqBpppIMeqYpU9s
cp2Fr/rbWoS1UMeE8ngOgIrMYvtR9ZEItej9xo3oTMOyKoB0sc2mzD+pju/BJKfl
WS3sAFXefwKBgQCVluEF8fx7aBP8UJIyVPHj1y9RCaHA9Dz2w/RS+Jqgbr0qqnfZ
psFRmC+I8k4L7dUJPqiZ7C3qGGhVU6knfj3Puucx8wX7jL0Hn6x73eoZ6+ROsDF9
Vym1Z9jOmcSYSfgQWUCGrj7tcpqnR8W6pAvMU3mKZopsLANO7iWph8kEWQKBgQDO
QgCXd2GxuhWxlquonHi//hWqgM3oT5K4skR/jRqZHNEPq2Xg17BhmvaP3DQAXPhc
fOr9S5ExTNRQ+3dmJgNJSZxM6lggfZhcYALUH7VX54jZw5+cXckhkp0PWRQwJxSq
i0kGfavDcvOnKKWMoAryBu36yNPvHaJW5DCZodVZtwKBgDQ0c7XCiIf5dvVATh0D
NQ7ldlLGlfMQpzPIwDQOO+zm3m8olr8cAoBcW9Ki2Mq0YShuAkNlgvi7XWXu1EF5
7zgQOZofcf+XI/AtBEXCS+nzh7svzuti+NNaoMtDltJIlBKdJB+AD9nlwTgePQha
3pn1355rqX8K/IX9Urmu4Rzk
-----END PRIVATE KEY-----";
const OFFLINE_AES_KEY_B64: &str = "94/AR7dd8gIstLEXp3LCs865DptiMKlh8nLjjDEcO40=";
const OFFLINE_KEY_SEPARATOR: &str = "|||";
pub const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

static PRIVATE_KEY: OnceCell<RsaPrivateKey> = OnceCell::new();
static AES_KEY: OnceCell<[u8; 32]> = OnceCell::new();

#[derive(Debug, thiserror::Error)]
pub enum OfflineKeyError {
    #[error("离线密钥数据无效: {0}")]
    InvalidData(String),
    #[error("离线密钥加密错误: {0}")]
    Crypto(String),
    #[error("离线密钥文件错误: {0}")]
    Io(String),
}

impl From<std::io::Error> for OfflineKeyError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
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

fn private_key() -> Result<&'static RsaPrivateKey, OfflineKeyError> {
    PRIVATE_KEY.get_or_try_init(|| {
        RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY)
            .map_err(|err| OfflineKeyError::Crypto(format!("无法解析私钥: {err}")))
    })
}

fn aes_key() -> Result<&'static [u8; 32], OfflineKeyError> {
    AES_KEY
        .get_or_try_init(|| {
            let decoded = general_purpose::STANDARD
                .decode(OFFLINE_AES_KEY_B64)
                .map_err(|_| OfflineKeyError::InvalidData("AES 密钥格式无效".into()))?;
            if decoded.len() != 32 {
                return Err(OfflineKeyError::InvalidData("AES 密钥长度错误".into()));
            }
            let mut array = [0u8; 32];
            array.copy_from_slice(&decoded);
            Ok(array)
        })
        .map(|key| &key[..])
}

fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    let parts: Vec<&str> = raw.splitn(2, OFFLINE_KEY_SEPARATOR).collect();
    if parts.len() != 2 {
        return Err(OfflineKeyError::InvalidData("离线密钥格式错误".into()));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
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
    serde_json::from_slice(&decrypted)
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
    let cipher = Aes256Gcm::new_from_slice(aes_key()?)
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
    let cipher = Aes256Gcm::new_from_slice(aes_key()?)
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

fn read_file_with_lock(path: &PathBuf, lock: &Arc<Mutex<()>>) -> Result<String, OfflineKeyError> {
    let _guard = lock.lock();
    std::fs::read_to_string(path).map_err(OfflineKeyError::from)
}

fn write_file_with_lock(
    path: &PathBuf,
    data: &[u8],
    lock: &Arc<Mutex<()>>,
) -> Result<(), OfflineKeyError> {
    let _guard = lock.lock();
    std::fs::write(path, data).map_err(OfflineKeyError::from)
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

pub async fn try_validate_from_env(
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
    write_file_with_lock(&env_path, updated_raw.as_bytes(), lock)?;

    Ok(OfflineKeyValidationResult {
        is_valid: true,
        reason: None,
        expires_at: Some(payload.expires_at.clone()),
        payload: Some(payload),
    })
}
