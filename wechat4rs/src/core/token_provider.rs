use crate::{NoError, SaasContext, Wechat, WechatEncryptError, WechatError, WechatToken};
use async_trait::async_trait;
use log::{debug, info};
use smol::{Task, Timer};
use std::marker::{Send, Sync};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct TokenResolveGard {
    finish_flag: Arc<AtomicBool>,
}

impl TokenResolveGard {
    pub fn new(finish_flag: Arc<AtomicBool>) -> Self {
        TokenResolveGard { finish_flag }
    }
}

impl Drop for TokenResolveGard {
    fn drop(&mut self) {
        self.finish_flag.store(true, Ordering::SeqCst);
    }
}

#[async_trait]
pub trait TokenProvider: Send + Sync {
    /// 获取保存的token
    async fn get_token(
        &self,
        wechat: &Wechat,
        context: &SaasContext,
    ) -> Result<Option<WechatToken>, WechatError>;

    /// 保存token
    async fn set_token(
        &self,
        wechat: &Wechat,
        context: &SaasContext,
        token: Option<WechatToken>,
    ) -> Result<(), WechatError>;

    async fn lock_token_resolver(
        &self,
        wechat: &Wechat,
        context: &SaasContext,
    ) -> Result<TokenResolveGard, WechatError>;

    async fn unlock_token_resolver(
        &self,
        wechat: &Wechat,
        context: &SaasContext,
        gard: TokenResolveGard,
    ) -> Result<(), WechatError>;
}

pub mod memory {
    use super::*;
    use async_mutex::Mutex;
    use std::collections::HashMap;
    use std::sync::RwLock;

    pub struct MemoryTokenProvider {
        token_list: RwLock<HashMap<u64, WechatToken>>,
        rw_mutex: Arc<Mutex<()>>,
    }

    impl MemoryTokenProvider {
        pub fn new() -> Self {
            MemoryTokenProvider {
                token_list: RwLock::new(HashMap::new()),
                rw_mutex: Arc::new(Mutex::new(())),
            }
        }
    }

    impl Drop for MemoryTokenProvider {
        fn drop(&mut self) {
            todo!()
        }
    }

    #[allow(unused_variables)]
    #[async_trait]
    impl TokenProvider for MemoryTokenProvider {
        async fn get_token(
            &self,
            wechat: &Wechat,
            context: &SaasContext,
        ) -> Result<Option<WechatToken>, WechatError> {
            use chrono::Utc;
            let list = self.token_list.read().unwrap();
            let token = (*list).get(&context.id);
            Ok(match token {
                None => None,
                Some(token) => {
                    if token.expire_at < Utc::now() {
                        None
                    } else {
                        Some(token.clone())
                    }
                }
            })
        }
        async fn set_token(
            &self,
            wechat: &Wechat,
            context: &SaasContext,
            token: Option<WechatToken>,
        ) -> Result<(), WechatError> {
            let mut list = self.token_list.write().unwrap();
            if let Some(token) = token {
                info!("set token:{:?}, token:{:?}", context, token);
                (*list).insert(context.id, token);
            } else {
                info!("remove token:{:?}", context);
                (*list).remove(&context.id);
            }
            Ok(())
        }
        async fn lock_token_resolver(
            &self,
            wechat: &Wechat,
            context: &SaasContext,
        ) -> Result<TokenResolveGard, WechatError> {
            use std::sync::Arc;

            let context = context.clone();

            let rw_mutex = self.rw_mutex.clone();
            let lock = *rw_mutex.lock().await;
            let flag = Arc::new(AtomicBool::new(false));
            let gard = TokenResolveGard::new(flag.clone());
            Task::spawn(async move {
                use std::time::Duration;
                while !flag.load(Ordering::SeqCst) {
                    Timer::after(Duration::from_secs(10)).await;
                }
                drop(lock);
                debug!("token lock released, {:?}", context);
            })
            .detach();
            return Ok(gard);
        }

        async fn unlock_token_resolver(
            &self,
            wechat: &Wechat,
            context: &SaasContext,
            gard: TokenResolveGard,
        ) -> Result<(), WechatError> {
            debug!("begin release token lock:{:?}", context);
            drop(gard);
            Ok(())
        }

        //
    }
}

pub mod reids {
    use super::*;
    use bb8_redis::{
        redis::{cmd, AsyncCommands},
        RedisPool,
    };
    use std::sync::atomic::AtomicI64;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    fn get_key(context: &SaasContext, key: &str) -> String {
        format!("wechat::{}::{}", context.id, key)
    }

    fn get_token_key(context: &SaasContext) -> String {
        get_key(context, "token")
    }

    fn get_lock_key(context: &SaasContext) -> String {
        get_key(context, "lock")
    }

    pub struct RedisTokenProvider {
        redis_pool: RedisPool,
        seq: AtomicI64,
    }

    impl RedisTokenProvider {
        pub fn new(pool: RedisPool) -> Self {
            RedisTokenProvider {
                redis_pool: pool,
                seq: AtomicI64::new(1),
            }
        }
    }

    #[allow(unused_variables)]
    #[async_trait]
    impl TokenProvider for RedisTokenProvider {
        async fn get_token(
            &self,
            wechat: &Wechat,
            context: &SaasContext,
        ) -> Result<Option<WechatToken>, WechatError> {
            let mut conn = self.redis_pool.get().await?;
            let conn = conn.as_mut().unwrap();
            let key = get_token_key(context);

            let reply: String = cmd("GET").arg(key).query_async(conn).await?;
            if reply.is_empty() {
                return Ok(None);
            }

            let token: WechatToken = serde_json::from_str(&reply)?;

            Ok(Some(token))
        }
        async fn set_token(
            &self,
            wechat: &Wechat,
            context: &SaasContext,
            token: Option<WechatToken>,
        ) -> Result<(), WechatError> {
            let mut conn = self.redis_pool.get().await?;
            let conn = conn.as_mut().unwrap();
            let key = get_token_key(context);

            if let Some(token) = token {
                let value = serde_json::to_string(&token)?;
                cmd("SET").arg(key).arg(value).query_async(conn).await?;
            } else {
                cmd("DEL").arg(key).query_async(conn).await?;
            }
            Ok(())
        }
        async fn lock_token_resolver(
            &self,
            wechat: &Wechat,
            context: &SaasContext,
        ) -> Result<TokenResolveGard, WechatError> {
            use std::sync::Arc;

            let context = context.clone();
            let redis_pool = self.redis_pool.clone();
            let key = get_lock_key(&context);
            let seq = self.seq.fetch_add(1, Ordering::SeqCst);
            let seq = format!("{}", seq);
            for i in 0..=100i32 {
                let mut conn = redis_pool.get().await?;
                let conn = conn.as_mut().unwrap();
                let result: String = cmd("SET")
                    .arg(key.clone())
                    .arg(seq.clone())
                    .arg("EX 10")
                    .arg("NX")
                    .query_async(conn)
                    .await?;

                if "OK" == result.as_str() {
                    break;
                }
                Timer::after(Duration::from_secs(1)).await;
                if i > 99 {
                    return Err(WechatError::EncryptError {
                        source: WechatEncryptError::ApiRequestError {
                            msg: format!("无法获得token, 超时. {:?}", context),
                            source: Box::new(NoError),
                        },
                    });
                }
            }

            let flag = Arc::new(AtomicBool::new(false));
            let gard = TokenResolveGard::new(flag.clone());
            Task::spawn(async move {
                let r = Task::spawn(async move {
                    while !flag.load(Ordering::SeqCst) {
                        Timer::after(Duration::from_secs(1)).await;
                        let mut conn = redis_pool.get().await?;
                        let conn = conn.as_mut().unwrap();
                        cmd("EXPIRE")
                            .arg(key.clone())
                            .arg(10i32)
                            .query_async(conn)
                            .await?;
                    }
                    // del key
                    debug!("token lock released, {:?}", context);
                    let mut conn = redis_pool.get().await?;
                    let conn = conn.as_mut().unwrap();
                    cmd("DEL").arg(key).query_async(conn).await?;
                    Ok::<(), WechatError>(())
                })
                .await;

                if let Err(e) = r {
                    info!("{:?}", e);
                }
            })
            .detach();
            return Ok(gard);
        }

        async fn unlock_token_resolver(
            &self,
            wechat: &Wechat,
            context: &SaasContext,
            gard: TokenResolveGard,
        ) -> Result<(), WechatError> {
            debug!("begin release token lock:{:?}", context);
            drop(gard);
            Ok(())
        }

        //
    }
}
