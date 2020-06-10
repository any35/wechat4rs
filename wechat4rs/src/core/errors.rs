use actix_web::error::ResponseError;
use http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WechatEncryptError {
    #[error("签名无效, {0}")]
    InvalidSignature(String),
    #[error("appId无效")]
    InvalidAppId,
    #[error("配置信息无效")]
    InvalidConfig,
}

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum WechatError {
    #[error("error on parse")]
    ParseError(String),
    #[error("error on parse")]
    EncryptError { source: WechatEncryptError },
}

impl From<sxd_xpath::Error> for WechatError {
    fn from(_: sxd_xpath::Error) -> Self {
        todo!()
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

impl From<WechatEncryptError> for std::io::Error {
    fn from(e: WechatEncryptError) -> Self {
        use std::io::ErrorKind;
        match e {
            WechatEncryptError::InvalidAppId => std::io::Error::new(ErrorKind::InvalidData, e),
            WechatEncryptError::InvalidSignature(_) => {
                std::io::Error::new(ErrorKind::InvalidData, e)
            }
            WechatEncryptError::InvalidConfig => std::io::Error::new(ErrorKind::InvalidData, e),
        }
    }
}

impl ResponseError for WechatEncryptError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}
