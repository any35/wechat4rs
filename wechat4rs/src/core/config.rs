use super::errors::WechatEncryptError;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WechatConfig {
    pub token: Option<String>,
    pub key: Option<Vec<u8>>,
    pub app_id: String,
    pub app_secret: Option<String>,
}

impl WechatConfig {
    pub fn decode_aes_key(key: &String) -> Result<Option<Vec<u8>>, WechatEncryptError> {
        crate::message::crypt::decode_aes_key(key)
    }
    pub fn new(
        token: Option<String>,
        key: Option<Vec<u8>>,
        app_id: String,
        app_secret: Option<String>,
    ) -> Self {
        WechatConfig {
            token,
            key,
            app_id,
            app_secret,
        }
    }

    pub fn token_string(&self) -> String {
        match &self.token {
            Some(token) => token.clone(),
            None => "".into(),
        }
    }
}

impl Default for WechatConfig {
    fn default() -> Self {
        Self {
            token: None,
            key: None,
            app_id: "".into(),
            app_secret: None,
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
