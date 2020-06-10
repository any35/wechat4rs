use actix_web::{
    get, post,
    web::{self, Bytes, Data, Path, Query},
    App, HttpResponse, HttpServer, Responder, Result,
};
use log::info;
use wechat4rs::{
    errors::{WechatEncryptError, WechatError},
    CallbackMessage, EchoStrReq, MessageInfo, ReplyMessage, SaasContext, VerifyInfo, Wechat,
    WechatCallBackHandler, WechatConfig, WechatSaasResolver,
};

/// 微信对接echo验证
#[get("/wechat-callback/{saas_id}/")]
async fn echo_str(
    wechat: Data<Wechat>,
    saas_id: Path<u64>,
    verify_info: Query<VerifyInfo>,
    req: Query<EchoStrReq>,
) -> Result<String, WechatError> {
    let context = SaasContext::new(saas_id.into_inner());
    let result = wechat.handle_echo(&verify_info, &req, &context).await?;
    Ok(result)
}

/// 微信回调消息接收入口
#[post("/wechat-callback/{saas_id}/")]
async fn wechat_callback(
    wechat: Data<Wechat>,
    saas_id: Path<u64>,
    info: Query<VerifyInfo>,
    body: Bytes,
) -> Result<String, WechatError> {
    let request_body = String::from_utf8(body.to_vec())?;
    let context = SaasContext::new(saas_id.into_inner());
    let result = wechat
        .handle_callback(&info, &request_body, &context)
        .await?;
    Ok(result)
}

use async_trait::async_trait;
struct EchoText;

#[async_trait]
impl WechatCallBackHandler for EchoText {
    async fn handler_callback(
        &self,
        wechat: &Wechat,
        prev_result: Option<ReplyMessage>,
        message: &CallbackMessage,
    ) -> Result<Option<ReplyMessage>, WechatError> {
        if let CallbackMessage::Text {
            info,
            content,
            biz_msg_menu_id: _,
        } = message
        {
            let info = info.clone();
            return Ok(Some(ReplyMessage::Text {
                info: MessageInfo {
                    from_user_name: info.to_user_name.clone(),
                    to_user_name: info.from_user_name.clone(),
                    ..info
                },
                content: format!("hello: {}", content),
            }));
        }
        Ok(prev_result)
    }
}

struct SaasResolve;
#[async_trait]
impl WechatSaasResolver for SaasResolve {
    async fn resolve_config(
        &self,
        _wechat: &Wechat,
        context: &SaasContext,
    ) -> Result<WechatConfig, WechatError> {
        let aes_key = wechat4rs::WechatConfig::decode_aes_key(
            &"znpfGFxELvUSxh0Gx4rJenvVQRrAhdTsioG08XR4z3S=".to_string(),
        )?;
        match context.id {
            1 => Ok(WechatConfig {
                token: Some("testtoken123456".into()),
                key: None,
                app_id: "wxc01451f1526a8a14".into(),
                app_secret: Some("d4624c36b6795d1d99dcf0547af5443d".into()),
            }),
            2 => Ok(WechatConfig {
                token: Some("testtoken123456".into()),
                key: aes_key,
                app_id: "wx11853b05910e1b6b".into(),
                app_secret: Some("wx11853b05910e1b6b".into()),
            }),
            _ => Err(WechatError::EncryptError {
                source: WechatEncryptError::InvalidAppId,
            }),
        }
    }
}

async fn init() -> anyhow::Result<()> {
    use actix_web::middleware::Logger;
    use env_logger::Env;

    use dotenv::dotenv;
    dotenv().ok();
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    info!("init");

    // let config_env: WechatConfig = envy::prefixed("WECHAT_").from_env()?;

    let mut wechat = wechat4rs::Wechat::new(Box::new(SaasResolve));
    wechat.registry_callback(Box::new(EchoText));
    let wechat = web::Data::new(wechat);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(wechat.clone())
            .service(echo_str)
            .service(wechat_callback)
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await?;

    Ok(())
}

#[actix_rt::main]
async fn main() -> Result<(), std::io::Error> {
    if let Err(err) = init().await {
        eprintln!("ERROR: {:#}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("because: {:?}", cause));
        std::process::exit(1);
    }
    Ok(())
}
