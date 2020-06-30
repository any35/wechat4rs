use actix_web::{
    get, post,
    web::{self, Bytes, Data, Path, Query},
    App, HttpResponse, HttpServer, Responder, Result,
};
use async_trait::async_trait;
use log::info;
use wechat4rs::{
    errors::{WechatEncryptError, WechatError},
    CallbackMessage, EchoStrReq, MessageInfo, ReplyMessage, SaasContext, VerifyInfo, Wechat,
    WechatCallBackHandler, WechatConfig, WechatSaasResolver,
};

/// 公众号对接echo验证,
/// 可以配置多个公众号回调,通过saas_id区分回调的公众号(u64), 下同
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
        .handle_callback(&info, &request_body, &context) // 调用消息处理入口
        .await?;
    Ok(result)
}

struct EchoText;

/// 处理消息回调[可选]
/// 该回调可以处理微信的消息回调, 并返回相应的处理结果
/// 此Demo返回 hello:{收到的消息}
#[async_trait]
impl WechatCallBackHandler for EchoText {
    async fn handler_callback(
        &self,
        _wechat: &Wechat,
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
        Ok(prev_result) //默认可以返回None
    }
}

/// 公众号配置信息解析器
///
/// 用于返回解析公众号配置信息
/// context中包含请求的上下文, 返回对应公众号的配置信息
/// 这些配置信息一般保存在redis或者数据库中
///
/// note:
///   如果只有一个公众号, 那可以使用ConstSaasResolver::new(config)始终返回固定的配置
struct SaasResolve;
#[async_trait]
impl WechatSaasResolver for SaasResolve {
    async fn resolve_config(
        &self,
        _wechat: &Wechat,
        context: &SaasContext,
    ) -> Result<WechatConfig, WechatError> {
        // 微信的配置aes key需要解码
        let aes_key = wechat4rs::WechatConfig::decode_aes_key(
            &"znpfGFxELvUSxh0Gx4rJenvVQRrAhdTsioG08XR4z3S=".to_string(),
        )?;
        match context.id {
            1 => Ok(WechatConfig {
                key: None,
                app_id: "wxc01451f1526a8a14".into(),
                app_secret: "d4624c36b6795d1d99dcf0547af5443d".into(),
            }),
            2 => Ok(WechatConfig {
                key: aes_key,
                app_id: "wx11853b05910e1b6b".into(),
                app_secret: "wx11853b05910e1b6b".into(),
            }),
            _ => Err(WechatError::EncryptError {
                source: WechatEncryptError::InvalidAppId,
            }),
        }
    }
}

async fn init() -> anyhow::Result<()> {
    use actix_web::middleware::Logger;
    use bb8_redis::{RedisConnectionManager, RedisPool};
    use env_logger::Env;
    use wechat4rs::token_provider::reids::RedisTokenProvider;

    use dotenv::dotenv;
    dotenv().ok();
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    info!("init");

    // 1. 为了使集群能正常分享公众号的access_token, 这里使用redis来保存token,
    // 如果是单机版可以使用MemoryTokenProvider
    // let config_env: WechatConfig = envy::prefixed("WECHAT_").from_env()?;
    let manager = RedisConnectionManager::new(dotenv::var("REDIS_URL")?)?;
    let pool = RedisPool::new(bb8::Pool::builder().build(manager).await?);
    let token_p = RedisTokenProvider::new(pool);

    // 2. 指定配置解析器
    let mut wechat = wechat4rs::Wechat::new(Box::new(SaasResolve), Box::new(token_p));
    // 3. [可选] 注册消息回调处理器, 用于处理微信回调的消息
    wechat.registry_callback(Box::new(EchoText));

    let wechat = web::Data::new(wechat);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(wechat.clone())
            .service(echo_str) // 微信注册echo str
            .service(wechat_callback) // 回调入口
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
