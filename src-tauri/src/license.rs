use std::{path::PathBuf, sync::Arc};

use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use once_cell::sync::Lazy;
use rsa::{pkcs8::DecodePrivateKey, Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::{fs, io::AsyncReadExt, io::AsyncWriteExt, sync::Mutex};

pub const OFFLINE_KEY_SEPARATOR: &str = "|||";
pub const OFFLINE_KEY_ENV_NAME: &str = "keyzhigongfile";

pub const OFFLINE_RSA_PRIVATE_KEY: &str = r"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDVKT08cxtJyhsx
l9A0+6o7PR5EN12E/gaVXSInY/g9GgTeCcaXBm7FifosFCaVGcvmjJrhxTaNsp/7
VM+S7jJVfmIYuVnwhhGdis5iJlG8fL2ekG1c1HtShxK7vcrhv8lDrc3zEIbX1v6r
9sy4T8gZH8peg6dnzmMKHRPhXGscr2SIHn3xsdkP/kCY+4mOsRLdV0IESho0BsNC
W4smTp4lx9zZKM9Q6DNF62B/2Gd4v+vTixogouQVbSJspTg/AbRx+snZbvX1Fe2d
I1pFvosWO/rh/K2Wt4PW9KUoQOQXt1WUWIQv5+4FQI82zcRR0BuUf4bfPnUy64ms
b8cfUA+1AgMBAAECggEAAP1hTitPG/u3iLR3KIeQfq0dmpZ8ED8KS0T2xtgktxUT
-----END PRIVATE KEY-----";

pub const OFFLINE_AES_KEY_B64: &str = "94/AR7dd8gIstLEXp3LCs865DptiMKlh8nLjjDEcO40=";

static PRIVATE_KEY: Lazy<RsaPrivateKey> = Lazy::new(|| {
    RsaPrivateKey::from_pkcs8_pem(OFFLINE_RSA_PRIVATE_KEY)
        .expect("无法解析离线 RSA 私钥，请检查配置")
});

static AES_KEY: Lazy<[u8; 32]> = Lazy::new(|| {
    let bytes = general_purpose::STANDARD
        .decode(OFFLINE_AES_KEY_B64)
        .expect("无法解析 AES 密钥 Base64 表示");
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    key
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
    pub fn from_value(value: &Value) -> Result<Self, OfflineKeyError> {
        serde_json::from_value(value.clone())
            .map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
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

#[derive(Debug, thiserror::Error)]
pub enum OfflineKeyError {
    #[error("无效数据: {0}")]
    InvalidData(String),
    #[error("加解密失败: {0}")]
    Crypto(String),
    #[error("IO 错误: {0}")]
    Io(String),
}

pub fn build_invalid_result(
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

pub async fn read_file_with_lock(path: &PathBuf, lock: &Arc<Mutex<()>>) -> Result<String, OfflineKeyError> {
    let _guard = lock.lock().await;
    let mut file = fs::File::open(path)
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    Ok(content)
}

pub async fn write_file_with_lock(
    path: &PathBuf,
    data: &[u8],
    lock: &Arc<Mutex<()>>,
) -> Result<(), OfflineKeyError> {
    let _guard = lock.lock().await;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    }
    let mut file = fs::File::create(path)
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    file.write_all(data)
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    file.flush()
        .await
        .map_err(|err| OfflineKeyError::Io(err.to_string()))?;
    Ok(())
}

pub fn split_offline_key(raw: &str) -> Result<(String, String), OfflineKeyError> {
    let mut parts = raw.splitn(2, OFFLINE_KEY_SEPARATOR);
    let rsa_part = parts
        .next()
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥格式不正确".into()))?;
    let aes_part = parts
        .next()
        .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥缺少时间戳部分".into()))?;
    Ok((rsa_part.to_string(), aes_part.to_string()))
}

pub fn combine_offline_key(rsa: &str, aes: &str) -> String {
    format!("{rsa}{OFFLINE_KEY_SEPARATOR}{aes}")
}

pub fn decrypt_license_payload(encoded: &str) -> Result<OfflineLicensePayload, OfflineKeyError> {
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

pub fn decrypt_server_time(encoded: &str) -> Result<chrono::DateTime<chrono::Utc>, OfflineKeyError> {
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

    chrono::DateTime::parse_from_rfc3339(&time_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| OfflineKeyError::InvalidData("服务器时间格式无效".into()))
}

pub fn encrypt_current_time(now: chrono::DateTime<chrono::Utc>) -> Result<String, OfflineKeyError> {
    use rand::rngs::OsRng;
    use rand::RngCore;

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

pub async fn try_validate_from_env(lock: &Arc<Mutex<()>>) -> Result<OfflineKeyValidationResult, OfflineKeyError> {
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
        .with_timezone(&chrono::Utc);

    let now = chrono::Utc::now();
    if now > expires_at {
        return Ok(build_invalid_result(
            "离线密钥已过期".into(),
            Some(payload.expires_at.clone()),
            Some(payload),
        ));
    }

    let device_id = crate::license::device::get_device_id()
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

pub mod device {
    use super::*;
    use if_addrs::IfAddr;
    use sha2::Digest;
    use uuid::Uuid;

    pub fn get_device_id() -> Result<String> {
        if let Ok(machine_id) = get_machine_uid() {
            if let Some(id) = hash_identifier(machine_id.as_bytes()) {
                return Ok(id);
            }
        }

        let mut hasher = Sha256::new();

        if let Ok(hostname) = hostname::get() {
            hasher.update(hostname.to_string_lossy().as_bytes());
        }

        if let Ok(interfaces) = if_addrs::get_if_addrs() {
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

    fn get_machine_uid() -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            use winreg::enums::HKEY_LOCAL_MACHINE;
            use winreg::RegKey;

            let key = RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey("SOFTWARE\\Microsoft\\Cryptography")
                .context("无法打开注册表以读取 MachineGuid")?;
            let guid: String = key.get_value("MachineGuid").context("无法读取 MachineGuid")?;
            return Ok(guid);
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(content) = std::fs::read_to_string("/etc/machine-id") {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }

            if let Ok(content) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }

            Err(anyhow!("无法确定机器唯一标识"))
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
}
