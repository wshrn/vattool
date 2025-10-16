use std::path::PathBuf;
use std::sync::Arc;

use aes_gcm::{aead::generic_array::GenericArray, aead::Aead, Aes256Gcm, Nonce};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rand::rngs::OsRng;
use rand::RngCore;
use rsa::{pkcs1::DecodeRsaPrivateKey, Oaep, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::task;

use crate::device;

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

lazy_static::lazy_static! {
  static ref PRIVATE_KEY: RsaPrivateKey = RsaPrivateKey::from_pkcs1_pem(OFFLINE_RSA_PRIVATE_KEY)
    .expect("invalid offline private key");
  static ref AES_KEY: Vec<u8> = general_purpose::STANDARD
    .decode(OFFLINE_AES_KEY_B64)
    .expect("invalid offline AES key");
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
  pub fn from_value(value: &Value) -> Result<Self, OfflineKeyError> {
    serde_json::from_value(value.clone()).map_err(|_| OfflineKeyError::InvalidData("离线密钥数据不完整".into()))
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
  #[error("{0}")]
  InvalidData(String),
  #[error("{0}")]
  Crypto(String),
  #[error(transparent)]
  Io(#[from] anyhow::Error),
}

fn build_invalid_result(reason: impl Into<String>, expires_at: Option<String>, payload: Option<OfflineLicensePayload>) -> OfflineKeyValidationResult {
  OfflineKeyValidationResult {
    is_valid: false,
    reason: Some(reason.into()),
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
    .ok_or_else(|| OfflineKeyError::InvalidData("离线密钥格式错误".into()))?;
  Ok((rsa_part.to_string(), aes_part.to_string()))
}

fn combine_offline_key(rsa_part: &str, aes_part: &str) -> String {
  format!("{rsa_part}{OFFLINE_KEY_SEPARATOR}{aes_part}")
}

async fn read_file_with_lock(path: PathBuf, lock: &Arc<Mutex<()>>) -> Result<String, OfflineKeyError> {
  let lock = lock.clone();
  let content = task::spawn_blocking(move || {
    let _guard = lock.lock();
    std::fs::read_to_string(path).map_err(|err| OfflineKeyError::Io(err.into()))
  })
  .await
  .map_err(|err| OfflineKeyError::Io(err.into()))??;
  Ok(content)
}

async fn write_file_with_lock(path: PathBuf, data: Vec<u8>, lock: &Arc<Mutex<()>>) -> Result<(), OfflineKeyError> {
  let lock = lock.clone();
  task::spawn_blocking(move || {
    let _guard = lock.lock();
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, data)?;
    Ok::<(), std::io::Error>(())
  })
  .await
  .map_err(|err| OfflineKeyError::Io(err.into()))??;
  Ok(())
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
  let cipher = Aes256Gcm::new(GenericArray::from_slice(&AES_KEY));
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
  let cipher = Aes256Gcm::new(GenericArray::from_slice(&AES_KEY));
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
    return Ok(build_invalid_result("离线密钥文件不存在", None, None));
  }

  let raw = read_file_with_lock(env_path.clone(), lock).await?;
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

  let device_id = device::generate_device_id().map_err(|err| OfflineKeyError::InvalidData(err.to_string()))?;
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
  write_file_with_lock(env_path, updated_raw.into_bytes(), lock).await?;

  Ok(OfflineKeyValidationResult {
    is_valid: true,
    reason: None,
    expires_at: Some(payload.expires_at.clone()),
    payload: Some(payload),
  })
}
