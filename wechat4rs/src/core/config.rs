use super::errors::WechatEncryptError;
use crate::core::utils::iso_date_format;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WechatToken {
    pub token: String,
    #[serde(with = "iso_date_format")]
    pub expire_at: DateTime<Utc>,
}

impl WechatToken {
    pub fn new_relative(token: String, expire_in_seconds: i32) -> Self {
        use chrono::Duration;
        WechatToken {
            token,
            expire_at: chrono::Utc::now() + Duration::seconds(expire_in_seconds as i64),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WechatConfig {
    pub key: Option<Vec<u8>>,
    pub app_id: String,
    pub app_secret: String,
}

impl WechatConfig {
    pub fn decode_aes_key(key: &String) -> Result<Option<Vec<u8>>, WechatEncryptError> {
        crate::message::crypt::decode_aes_key(key)
    }
    pub fn new(key: Option<Vec<u8>>, app_id: String, app_secret: String) -> Self {
        WechatConfig {
            key,
            app_id,
            app_secret,
        }
    }
}

impl Default for WechatConfig {
    fn default() -> Self {
        Self {
            key: None,
            app_id: "".into(),
            app_secret: "".into(),
        }
    }
}

#[derive(Copy, Debug, Clone)]
pub struct SaasContext {
    pub id: u64,
}

impl SaasContext {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}
