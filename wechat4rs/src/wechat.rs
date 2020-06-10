use crate::core::errors::{WechatEncryptError, WechatError};
use crate::core::*;
use crate::message::*;
use log::info;
use serde::Deserialize;

use crate::message::crypt::{VerifyInfo, WeChatCrypto};

use async_trait::async_trait;
use std::marker::{Send, Sync};

#[allow(unused_variables)]
#[async_trait]
pub trait WechatCallBackHandler: Send + Sync {
    /// 处理微信回调响应事件
    async fn handler_callback(
        &self,
        wechat: &Wechat,
        prev_result: Option<ReplyMessage>,
        message: &CallbackMessage,
    ) -> Result<Option<ReplyMessage>, WechatError> {
        Ok(prev_result)
    }
}

/// Saas配置解析
#[async_trait]
pub trait WechatSaasResolver: Send + Sync {
    async fn resolve_config(
        &self,
        wechat: &Wechat,
        context: &SaasContext,
    ) -> Result<WechatConfig, WechatError>;
}

pub struct Wechat {
    saas_resolver: Box<dyn WechatSaasResolver>,
    callback_handlers: Vec<Box<dyn WechatCallBackHandler>>,
}

impl Wechat {
    pub fn new(saas_resolver: Box<dyn WechatSaasResolver>) -> Self {
        Wechat {
            saas_resolver,
            callback_handlers: Vec::new(),
        }
    }

    pub fn registry_callback(&mut self, callback: Box<dyn WechatCallBackHandler>) {
        self.callback_handlers.push(callback);
    }

    pub fn get_aes_key(key: String) -> Result<Vec<u8>, WechatEncryptError> {
        let key = base64::decode(&key)?;
        Ok(key)
    }
}

#[derive(Deserialize, Debug)]
pub struct EchoStrReq {
    echostr: String,
}

// call back msg
impl Wechat {
    /// 开发者提交信息后，微信服务器将发送GET请求到填写的服务器地址URL上：
    /// 开发者通过检验signature对请求进行校验（下面有校验方式）。若确认此次GET请求来自微信服务器，请原样返回echostr参数内容，则接入生效，成为开发者
    pub async fn handle_echo(
        &self,
        verify_info: &VerifyInfo,
        req: &EchoStrReq,
        context: &SaasContext,
    ) -> Result<String, WechatError> {
        info!("handler echo: {:?}", req);
        let config = self.saas_resolver.resolve_config(&self, &context).await?;
        let msg = config.decrypt_echostr(verify_info, &req.echostr)?;
        info!("msg:{}", msg);
        Ok(msg)
    }

    /// 处理微信消息回调
    pub async fn handle_callback(
        &self,
        verify_info: &VerifyInfo,
        request_body: &String,
        context: &SaasContext,
    ) -> Result<String, WechatError> {
        info!("handler callback: {:?} {}", verify_info, request_body);
        let config = self.saas_resolver.resolve_config(&self, &context).await?;
        let xml = config.decrypt_message(verify_info, request_body)?;
        let message = crate::message::from_xml(&xml)?;
        let mut prev_result = None;
        for handler in self.callback_handlers.iter() {
            prev_result = handler
                .handler_callback(self, prev_result, &message)
                .await?;
        }
        let xml = match prev_result {
            None => "".to_string(),
            Some(msg) => msg.to_xml()?,
        };
        Ok(xml)
    }
}
