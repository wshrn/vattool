use std::path::PathBuf;

use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use rand::{rngs::OsRng, RngCore};
use rsa::{pkcs8::DecodePrivateKey, Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::config::FileWriteLock;
use crate::device;

const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCbTE3swUJBRzZO
dpqG6mtH82VZRlPso5iXCiije7Ox88nmbam3s8haURbmfdsQGLfXKonFR3KrqvAz
Goi9OxwNq2zLNnLIbXearu6uqXeO/omWI3chTpqBmZ2jAysyzvvCWzj08+Mn7fXA
cAOa0yXfkIrBh/OFOTv5rPwqoIf4Lh1+P2rFiBQT1cLQqD7u6/LYoT/pEIYHBxwx
qTfPBFkk0bos2GRJgDc90VqENW/9AYsEj2VDkg9UL9ppLRVzUjdRN/xAYH1/LAdu
byh8MOVXFQ5KrdxC7LAQ/zkn1uLFjiQq5gLDcpMQOtKiP3xqDaJh1+wvdShG4k2Q
q/pvr3X9AgMBAAECggEAEn9oEr4W7wvqlIr9BKVHH7Fb+OJvHFMvhDdyDWhZHZD9
E1osP9TawE3uAoiq1K5/tfx7n4jPSFJaklkOlizvE6kVEtl1RO6rszwzSDuoTDCB
C+Xv39hAgBUKQ+tHNHdCC+2YtFiZIzLtYD1suH7Tq97yysgvl0abdbWx7ZPkeN45
o5jnH0M8MrdXd610QFL73BXJyyIhHiDJegLrsIyemhCaiE+41q1eVyXYkisTDJ7D
l71resofIIZT/5fcyh8a62hEOsqZ8pci57GjpAvUlwfQvuyk9zQheCrx5g45M2ea
f9NSMDStuPBfjWFHEISWQ27UPpTWwsBvbSrS34Y18wKBgQDZWWaMVPRRqyX1CaVy
0mSmZ2A0+nyfTeh1SDhkjbe4yE51e7r4AvSO9fMipZPosoijQINwWTG6bJty3gnL
GWSxwl5Cix8/VtPFvmmgeO7OTtzaQW3d47qOCPpkxl+Lb8LMGx+7KegsgiVux8Xr
Y2SFN2NHwDDRaAN50lmYXB9Q4wKBgQC26ha78gLbmiwmPuCxeoP69kT/mTivkhHp
0GDLAhrd2cPTpW/rFEJZlYkw7BU54kM/FM2OzBkdkH1cK/0qRIyYkUmfHBZ7VGEx
eeogO9A5Awkff7Umgw8brQAT4y5gL3ryHG2ee5C14DQaO3FvbeBoRS+HAoilC2CO
gVfPLZUznwKBgGDfgd9mNgb4e1B93ioRMB8i0DSMuGLgfI1ZhRj8OTi4vo+KeEj2
OD5HzPhSSFxcxp3MsUiv2IQ4yAgogsDtLn40HYMXMvQuJPFr7vRBMl7ts23r36YY
TBvmUIP+DrO9Oltyc3AeO4cq8rgvfj1t7W2axDtEo/2RmIsBgEFyPdr1AoGABAy8
wh6Cl7AbKCGeQe+vel/3eR1AVwrJ4L5fVj9OrvjCUC5Kgw7oxSj9Z5rigWyUBpKE
VpQIPlJTpDXJHiV0Y3BcC+zIRqTAniLP3zTT0F9T1WGzdg3SyM8UwFv4S+LhOvkM
KEUeGBjFrosAYPIy1LwraDbTJaxRz/XnJswgkIsCgYBTSsXbJtmmizi0ccg+qmCh
6XL2PkOQwx7qlfV+mdh6gQUvieCh9kpNv9N8Bw4HUIGhFICTNy3QK772tJdsjYnF
V9GdizVXme4z2T9QJbHdNSLBSicYuk516pjtJNhMThlZxKFrcddqBlHuZeEkMGKr
KSHRkfjQuTi/SLpmZyTsmQ==
-----END PRIVATE KEY-----";
const OFFLINE_AES_KEY_B64: &str = "LA/f4NvczHfiqUQjz8N1oNgGoH6y9l7UwTWopTTXe7Q=";
const OFFLINE_KEY_SEPARATOR: &str = "|||";
const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY)
        .expect("无法解析离线 RSA 私钥")
});

static AES_KEY: Lazy<[u8; 32]> = Lazy::new(|| {
    let decoded = general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .expect("无法解析离线 AES 密钥");
    decoded
        .try_into()
        .expect("AES 密钥长度必须为 32 字节")
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflineKeyValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
    pub expires_at: Option<String>,
    pub payload: Option<OfflineLicensePayload>,
}

#[derive(thiserror::Error, Debug)]
pub enum OfflineKeyError {
    #[error("数据无效: {0}")]
    InvalidData(String),
    #[error("加解密失败: {0}")]
    Crypto(String),
    #[error("IO 错误: {0}")]
    Io(String),
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

    let expires_at = chrono::DateTime::parse_from_rfc3339(&payload.expires_at)
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

    let device_id = device::get_device_id()
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

fn decrypt_license_payload(encoded: &str) -> Result<OfflineLicensePayload, OfflineKeyError> {
    let encrypted = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| OfflineKeyError::InvalidData("RSA 密文格式无效".into()))?;

    let padding = Oaep::new::<Sha256>();
    let decrypted = PRIVATE_KEY
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

fn combine_offline_key(rsa_part: &str, aes_part: &str) -> String {
    format!("{rsa_part}{OFFLINE_KEY_SEPARATOR}{aes_part}")
}

fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    raw.split_once(OFFLINE_KEY_SEPARATOR)
        .map(|(rsa, aes)| (rsa.to_string(), aes.to_string()))
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥格式无效".into()))
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

async fn read_file_with_lock(path: &PathBuf, lock: &FileWriteLock) -> Result<String, OfflineKeyError> {
    let _guard = lock.0.lock().await;
    tokio::fs::read_to_string(path)
        .await
        .map_err(|err| OfflineKeyError::Io(format!("读取离线密钥失败: {err}")))
}

async fn write_file_with_lock(path: &PathBuf, data: &[u8], lock: &FileWriteLock) -> Result<(), OfflineKeyError> {
    let _guard = lock.0.lock().await;
    tokio::fs::write(path, data)
        .await
        .map_err(|err| OfflineKeyError::Io(format!("写入离线密钥失败: {err}")))
}

pub fn build_invalid_result_public(reason: &str) -> OfflineKeyValidationResult {
    build_invalid_result(reason.to_string(), None, None)
}
