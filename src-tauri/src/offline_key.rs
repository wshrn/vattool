use std::path::PathBuf;

use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use rand::RngCore;
use rsa::{pkcs8::DecodePrivateKey, Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::theme::FileWriteLock;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQCjygMzy0uwuFF5
2hTpTvrMnTqNiqymyLGNvihK0bhNQpUuDNJ7vLCBtFnUkHSm4X0vJydzDkNP1TQ6
ogvQ84VxD6CtIU7r/NZ2a8zCegswiUbpZckgLSbAIaV2CZIcvRGHS/x3b5ZI9Xzb
diFGd8Ky+atqJE5O92zjli5eIFO13aHNAeV6KFdaFVrhErv9hX75GI4zCDJs/cez
GvTStO6/a2jg7lT21WcECNKkdCWYo17IZIJccUNHu9kEdzu7aG3xpVM/BRWj3DsH
FPG7dgqtKHt+0MvUeS0tYkW879nmcHwvJaeYjYz145lZELJjOxz9+vwMRjEm/Cye
OW7ihd25AgMBAAECggEAA81xVJcAK1qkHZmYCbCZNv3heZ7CEl8vbBSBl/cVecm1
b+vBSTkgueoneb1ez+8FOn6EtnGa03QmMcRIVPIKY40S1GIlFGp6yiYHi25M6zQM
rud7i2Ew+J3vWvWu7DbhfMwhUN6I1SnJ28h6zkmCfig4uA6euMuavg/jBLFocBzr
HbjXItoVB+45SzA4Zhy75NIpy3vsQOGrJVKxbCWbgp8e/5QedRlMzdFnA/jduwwm
E4KP8eRQvfOkF+Ztb8/qQGG08Ai0Vu8s0TaqTZIz57R2acykn32kqbIVmPyeeEhs
FH7CwZD0tPTFN2/S6pY3HEhU6BSC/yWr4Q++fCYDWQKBgQDQ3ePNwXMdv6xqHicU
guiXSAQH9OC/Ue+H/jRiYp+3KAmvHsO68cobMuUqP2H5hlXeGh2k1Fx5wP2YiAVL
JNxHHZXYZ7Ap4J+NDjgGvTSijgcxvt2wLN5z06iAGVxJf6iby1sByvFCbxx7XvBT
1lnPkQClxP3YsPGGMCvq2lAd9QKBgQDIwANytxKgUhn9u1jAvtFmS1xEAniXu+uq
+HtvuFfnYRjvZfFjjXSj4OuoKd15ZjK6s7jIFjZ/9DZ2Z6l9zD7pcxGHgP407iO8
SuPl0lcIEEMA2e+A6/SL3Vf3bjQApOKRaHIE5XeQyam6QEgvtNGJk8U4gxxrBsv4
f8hwcszCNQKBgE5ziaU3DC4YWIJjYPprUUHBYwI6EFDMTdQevz5VHPiGqVyFia8m
MmuU6k68D+jRdF9AH/JRcYqp2pb4QETBS5vKmQX3rEuOe19X/+NIHgUQo9MjhdEu
iT+oOJok7G0O7h3WfBBoUcZKRcBxIIPc907nf/7DxzUlARMN9PD5ny4hAoGBAL3d
3WUCAXvgKvTv3GFwGFzHqdwo+iWIdrydhyGDGKCeFRdXM5cUktzsPfYuomnPXut2
T63uVF1wfJJAOO3h3x6s1kdymPs2wFW1/xW+etIFj1mexgcJI6GlFy3N7SEu0Zr1
TeGTpiIqmXyuj5ePTR7xw/ZPCvGu2/uL3+d6rG6BAoGBAJpPwv4pgIaX+9L+bY24
uNkhYUGqfmvFiZw8XNmOhZ4IbqPNiKUHua//9NHA1CnhzrWGoJMlWBxerIRy33xJ
2+RV99he2gNdAdE8d6SO1cvAoCo6PCqNgndgKkphncNjLAJ3uLQtoNe6pfnXJYDi
SwqRtFDFd6Q2Y7k0FehtpEgi
-----END PRIVATE KEY-----";

const OFFLINE_AES_KEY_B64: &str = "2aOM3YfXlSg8BBOFXvft2InEjBUST2Yqybk1HJ3ZUwE=";
const OFFLINE_KEY_SEPARATOR: &str = "|||";
pub const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY)
        .expect("无法解析离线密钥私钥")
});

static AES_KEY: Lazy<[u8; 32]> = Lazy::new(|| {
    let bytes = general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .expect("无效的 AES 密钥");
    bytes
        .try_into()
        .expect("AES 密钥长度错误，应为 32 字节")
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

impl OfflineLicensePayload {
    fn from_value(value: &Value) -> Result<Self, OfflineKeyError> {
        serde_json::from_value(value.clone())
            .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineKeyValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub expires_at: Option<String>,
    pub payload: Option<OfflineLicensePayload>,
}

#[derive(Debug, thiserror::Error)]
pub enum OfflineKeyError {
    #[error("无效数据: {0}")]
    InvalidData(String),
    #[error("加解密失败: {0}")]
    Crypto(String),
    #[error("IO 错误: {0}")]
    Io(String),
    #[error("其他错误: {0}")]
    Other(String),
}

fn build_invalid_result(reason: String, expires_at: Option<String>, payload: Option<OfflineLicensePayload>) -> OfflineKeyValidationResult {
    OfflineKeyValidationResult {
        is_valid: false,
        reason: Some(reason),
        expires_at,
        payload,
    }
}

fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    let mut parts = raw.splitn(2, OFFLINE_KEY_SEPARATOR);
    let rsa_part = parts
        .next()
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥格式错误".into()))?;
    let aes_part = parts
        .next()
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥缺少时间部分".into()))?;
    Ok((rsa_part.to_string(), aes_part.to_string()))
}

fn decrypt_license_payload(encoded: &str) -> Result<OfflineLicensePayload, OfflineKeyError> {
    let encrypted = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("RSA 密文格式无效".into()))?;

    let padding = Oaep::new::<Sha256>();
    let decrypted = PRIVATE_KEY
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
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), now.to_rfc3339().as_bytes())
        .map_err(|err| OfflineKeyError::Crypto(format!("AES 加密失败: {err}")))?;

    let mut combined = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(combined))
}

fn combine_offline_key(rsa: &str, aes: &str) -> String {
    format!("{rsa}{OFFLINE_KEY_SEPARATOR}{aes}")
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
    hash_identifier(uuid.as_bytes())
        .ok_or_else(|| OfflineKeyError::Other("无法生成设备ID".into()))
}

#[cfg(windows)]
fn get_machine_uid() -> Result<String, OfflineKeyError> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey("SOFTWARE\\Microsoft\\Cryptography")
        .map_err(|err| OfflineKeyError::Other(format!("无法读取机器标识: {err}")))?;
    key.get_value::<String, _>("MachineGuid")
        .map_err(|err| OfflineKeyError::Other(format!("无法获取机器标识: {err}")))
}

#[cfg(not(windows))]
fn get_machine_uid() -> Result<String, OfflineKeyError> {
    Err(OfflineKeyError::Other("非 Windows 平台上无法获取机器标识".into()))
}

async fn read_file_with_lock(path: &PathBuf, lock: &Mutex<()>) -> Result<String, OfflineKeyError> {
    let _guard = lock.lock().await;
    std::fs::read_to_string(path)
        .map_err(|err| OfflineKeyError::Io(err.to_string()))
}

async fn write_file_with_lock(path: &PathBuf, data: &[u8], lock: &Mutex<()>) -> Result<(), OfflineKeyError> {
    let _guard = lock.lock().await;
    std::fs::write(path, data).map_err(|err| OfflineKeyError::Io(err.to_string()))
}

fn validate_payload_device(payload: &OfflineLicensePayload) -> Result<(), OfflineKeyError> {
    let device_id = generate_device_id()?;
    if payload.device_id != device_id {
        return Err(OfflineKeyError::InvalidData(
            "当前设备与离线密钥绑定设备不一致".into(),
        ));
    }
    Ok(())
}

pub async fn try_validate_from_env(lock: &FileWriteLock) -> Result<OfflineKeyValidationResult, OfflineKeyError> {
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

    let raw = read_file_with_lock(&env_path, &lock.0).await?;
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

    validate_payload_device(&payload)?;

    if now < server_time {
        return Ok(build_invalid_result(
            "检测到离线密钥时间被回滚，请重新获取".into(),
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    let updated_aes_part = encrypt_current_time(now)?;
    let updated_raw = combine_offline_key(&rsa_part, &updated_aes_part);
    write_file_with_lock(&env_path, updated_raw.as_bytes(), &lock.0).await?;

    Ok(OfflineKeyValidationResult {
        is_valid: true,
        reason: None,
        expires_at: Some(payload.expires_at.clone()),
        payload: Some(payload),
    })
}
