use crate::core::errors::{WechatEncryptError, WechatError};
use crate::core::token_provider::TokenProvider;
use crate::core::*;
use crate::message::*;
use log::info;
use serde::Deserialize;

use crate::message::crypt::VerifyInfo;

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

/// Saas版公众号配置解析器
/// Saas版本需要自定义实现从数据库或者Redis等地方加载配置的逻辑
/// 单机版本可用ConstSaasResolver
#[async_trait]
pub trait WechatSaasResolver: Send + Sync {
    /// 获取公众号配置信息
    async fn resolve_config(
        &self,
        wechat: &Wechat,
        context: &SaasContext,
    ) -> Result<WechatConfig, WechatError>;
}

/// 单微信配置
pub struct ConstSaasResolver {
    config: WechatConfig,
}
impl ConstSaasResolver {
    pub fn new(config: WechatConfig) -> Self {
        ConstSaasResolver { config }
    }
}

#[async_trait]
impl WechatSaasResolver for ConstSaasResolver {
    async fn resolve_config(
        &self,
        _wechat: &Wechat,
        _context: &SaasContext,
    ) -> Result<WechatConfig, WechatError> {
        Ok(self.config.clone())
    }
}

pub type WechatResult<T> = Result<T, WechatError>;

/// 微信公众平台SDK主类
pub struct Wechat {
    pub saas_resolver: Box<dyn WechatSaasResolver>,
    pub callback_handlers: Vec<Box<dyn WechatCallBackHandler>>,
    pub token_provider: Box<dyn TokenProvider>,
}

impl Wechat {
    pub fn new(
        saas_resolver: Box<dyn WechatSaasResolver>,
        token_provider: Box<dyn TokenProvider>,
    ) -> Self {
        Wechat {
            saas_resolver,
            callback_handlers: Vec::new(),
            token_provider,
        }
    }

    /// 注册自定义消息处理回调
    pub fn registry_callback(&mut self, callback: Box<dyn WechatCallBackHandler>) {
        self.callback_handlers.push(callback);
    }

    /// aes key的解码
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
        use crate::message::crypt::decrypt_echostr;
        info!("handler echo: {:?}", req);
        let config = self.saas_resolver.resolve_config(&self, &context).await?;
        let token = self.get_access_token(&context).await?;
        let msg = decrypt_echostr(&config, &token.token, verify_info, &req.echostr)?;
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
        use crate::message::crypt::decrypt_message;
        info!("handler callback: {:?} {}", verify_info, request_body);
        let config = self.saas_resolver.resolve_config(&self, &context).await?;
        let token = self.get_access_token(&context).await?;
        let xml = decrypt_message(&config, &token.token, verify_info, request_body)?;
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
