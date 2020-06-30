# 微信SDK for rust

库初始化中, 目前api还没完成, 请不要使用

## example
```rust
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

use async_trait::async_trait;
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
        let aes_key = wechat4rs::WechatConfig::decode_aes_key(
            &"znpfGFxELvUSxh0Gx4rJenvVQRrAhdTsioG08XR4z3S=".to_string(),
        )?;
        match context.id {
            1 => Ok(WechatConfig {
                key: None,
                app_id: "appid 1".into(),
                app_secret: "app id 1 secret".into(),
            }),
            2 => Ok(WechatConfig {
                key: aes_key,
                app_id: "appid 2".into(),
                app_secret: "appid 2 secret".into(),
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

```

## 关键特性
+ 支持单/多公众号管理

## 实现的API
接收消息
  + [x] 文本消息
  + [x] 图片消息
  + [x] 语音消息
  + [x] 视频消息
  + [x] 小视频消息
  + [x] 地理位置消息
  + [x] 链接消息
  + [x] 事件普通消息
    - [x] 关注/取消关注事件
    - [x] 扫描带参数二维码事件
    - [x] 上报地理位置事件
  + [x] 菜单事件消息
    - [x] 点击菜单拉取消息时的事件推送
    - [x] 点击菜单跳转链接时的事件推送
    - [x] 扫码推事件的事件推送
    - [x] 扫码推事件且弹出“消息接收中”提示框的事件推送
    - [x] 弹出系统拍照发图的事件推送
    - [x] 弹出拍照或者相册发图的事件推送
    - [x] 弹出微信相册发图器的事件推送
    - [x] 弹出地理位置选择器的事件推送
    - [x] 点击菜单跳转小程序的事件推送

回复消息
  + [x] 回复文本消息
  + [x] 回复图片消息
  + [x] 回复语音消息
  + [x] 回复视频消息
  + [x] 回复音乐消息
  + [x] 回复图文消息

客服消息
  + [x] 客服帐号管理
    - [x] 添加客服帐号
    - [x] 修改客服帐号
    - [x] 删除客服帐号
    - [ ] 设置客服帐号的头像
    - [x] 获取所有客服账号
  + [x] 客服接口-发消息
    - [x] 发送文本消息
    - [x] 发送图片消息
    - [x] 发送语音消息
    - [x] 发送视频消息
    - [x] 发送音乐消息
    - [x] 发送图文消息（点击跳转到外链）
    - [x] 发送图文消息（点击跳转到图文消息页面） 
    - [x] 发送菜单消息
    - [x] 发送卡券
    - [x] 发送小程序卡片（要求小程序与公众号已关联
    - [x] 以某个客服帐号来发消息（在微信6.0.2及以上版本中显示自定义头像）
  + [x] 客服接口-客服输入状态

自定义菜单
  + [x] 创建菜单
  + [x] 查询菜单
  + [x] 消除菜单
  + [x] 创建个性化菜单
  + [x] 删除个性化菜单
  + [x] 测试个性化菜单
  + [x] 查询个性化菜单

todo:

[ ] 群发接口和原创校验
[ ] 模板消息接口
[ ] 一次性订阅消息
[ ] 网页授权

