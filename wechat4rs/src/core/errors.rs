use actix_web::error::ResponseError;
use http::StatusCode;
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub struct NoError;
impl Display for NoError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum WechatEncryptError {
    #[error("签名无效, {0}")]
    InvalidSignature(String),
    #[error("appId无效")]
    InvalidAppId,
    #[error("配置信息无效")]
    InvalidConfig,
    #[error("API请求错误:{msg:?}")]
    ApiRequestError {
        msg: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum WechatError {
    #[error("error on parse")]
    ParseError(String),
    #[error("error on parse")]
    EncryptError { source: WechatEncryptError },
}

impl From<bb8::RunError<redis::RedisError>> for WechatError {
    fn from(e: bb8::RunError<redis::RedisError>) -> Self {
        WechatError::EncryptError {
            source: WechatEncryptError::ApiRequestError {
                msg: format!("{:?}", e),
                source: Box::new(NoError),
            },
        }
    }
}

impl From<redis::RedisError> for WechatError {
    fn from(e: redis::RedisError) -> Self {
        WechatError::EncryptError {
            source: WechatEncryptError::ApiRequestError {
                msg: format!("{:?}", e),
                source: Box::new(NoError),
            },
        }
    }
}

impl From<serde_json::error::Error> for WechatError {
    fn from(e: serde_json::error::Error) -> Self {
        WechatError::EncryptError {
            source: WechatEncryptError::ApiRequestError {
                msg: format!("{:?}", e),
                source: Box::new(NoError),
            },
        }
    }
}

impl From<sxd_xpath::Error> for WechatError {
    fn from(_: sxd_xpath::Error) -> Self {
        todo!()
    }
}

impl From<reqwest::Error> for WechatError {
    fn from(e: reqwest::Error) -> Self {
        WechatError::EncryptError { source: e.into() }
    }
}

impl From<std::io::Error> for WechatError {
    fn from(_: std::io::Error) -> Self {
        todo!()
    }
}

impl From<std::string::FromUtf8Error> for WechatError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        todo!()
    }
}

impl From<WechatEncryptError> for WechatError {
    fn from(e: WechatEncryptError) -> Self {
        WechatError::EncryptError { source: e }
    }
}

impl ResponseError for WechatError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

impl From<openssl::error::ErrorStack> for WechatEncryptError {
    fn from(e: openssl::error::ErrorStack) -> Self {
        WechatEncryptError::InvalidSignature(format!("{:?}", e))
    }
}

impl From<base64::DecodeError> for WechatEncryptError {
    fn from(e: base64::DecodeError) -> Self {
        WechatEncryptError::InvalidSignature(e.to_string())
    }
}

impl From<std::io::Error> for WechatEncryptError {
    fn from(_: std::io::Error) -> Self {
        todo!()
    }
}

impl From<std::string::FromUtf8Error> for WechatEncryptError {
    fn from(_: std::string::FromUtf8Error) -> Self {
        todo!()
    }
}

impl From<sxd_xpath::Error> for WechatEncryptError {
    fn from(_: sxd_xpath::Error) -> Self {
        todo!()
    }
}

impl From<reqwest::Error> for WechatEncryptError {
    fn from(e: reqwest::Error) -> Self {
        WechatEncryptError::ApiRequestError {
            msg: format!("{:?}", e),
            source: Box::new(e),
        }
    }
}

impl From<WechatEncryptError> for std::io::Error {
    fn from(e: WechatEncryptError) -> Self {
        use std::io::ErrorKind;
        match &e {
            WechatEncryptError::InvalidAppId => std::io::Error::new(ErrorKind::InvalidData, e),
            WechatEncryptError::InvalidSignature(_) => {
                std::io::Error::new(ErrorKind::InvalidData, e)
            }
            WechatEncryptError::InvalidConfig => std::io::Error::new(ErrorKind::InvalidData, e),
            WechatEncryptError::ApiRequestError { msg: _, source: _ } => {
                std::io::Error::new(ErrorKind::InvalidData, e)
            }
        }
    }
}

impl ResponseError for WechatEncryptError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
