use crate::SaasContext;
use crate::{NoError, WechatEncryptError, WechatError, WechatToken};
use crate::{Wechat, WechatResult};
use lazy_static::*;
use maplit::hashmap;
use reqwest::get;
use reqwest::Client;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

lazy_static! {
    static ref WECHAT_API: Url = Url::parse("https://api.weixin.qq.com/").unwrap();
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ApiResult<T: Sized> {
    Error { errcode: i32, errmsg: String },
    Msg(T),
}

impl<T: Sized> ApiResult<T> {
    pub fn get_result(self) -> WechatResult<T> {
        match self {
            ApiResult::Error { errcode, errmsg } => Err(WechatError::EncryptError {
                source: WechatEncryptError::ApiRequestError {
                    msg: format!("code: {}, msg: {:?}", errcode, errmsg),
                    source: Box::new(NoError),
                },
            }),
            ApiResult::Msg(msg) => Ok(msg),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct GetAccessTokenResp {
    /// 获取到的凭证
    pub access_token: String,
    /// 凭证有效时间，单位：秒
    pub expires_in: i32,
}

pub fn get_url_with_token(
    url: &str,
    token: Option<String>,
    query: Option<HashMap<String, String>>,
) -> WechatResult<Url> {
    let mut u: Url = Url::options()
        .base_url(Some(&WECHAT_API))
        .parse(url)
        .map_err(|e| WechatError::ParseError(format!("{:?}", e)))?;
    {
        let mut query_kv = u.query_pairs_mut();
        if let Some(token) = token {
            query_kv.append_pair("access_token", token.as_str());
        }
        if let Some(kv) = query {
            for (k, v) in kv {
                query_kv.append_pair(k.as_str(), v.as_str());
            }
        }
    }
    Ok(u)
}
pub fn get_url(url: &str, query: Option<HashMap<String, String>>) -> WechatResult<Url> {
    get_url_with_token(url, None, query)
}

impl Wechat {
    /// 获取token
    pub async fn get_access_token(&self, context: &SaasContext) -> WechatResult<WechatToken> {
        let config = self.saas_resolver.resolve_config(self, context).await?;

        if let Some(token) = self.token_provider.get_token(self, context).await? {
            return Ok(token);
        }
        // get lock
        let resolver = self
            .token_provider
            .lock_token_resolver(self, context)
            .await?;
        // double check
        if let Some(token) = self.token_provider.get_token(self, context).await? {
            return Ok(token);
        }
        let url = get_url(
            "cgi-bin/token",
            Some(hashmap! {
                "grant_type".into() => "client_credential".into(),
                "appid".into() => config.app_id,
                "secret".into() => config.app_secret,
            }),
        )?;
        let resp = get(url)
            .await?
            .json::<ApiResult<GetAccessTokenResp>>()
            .await?
            .get_result()?;

        let token = WechatToken::new_relative(resp.access_token, resp.expires_in);

        self.token_provider
            .set_token(self, context, Some(token.clone()))
            .await?;

        // release lock
        self.token_provider
            .unlock_token_resolver(self, context, resolver)
            .await?;

        Ok(token)
    }

    pub(crate) async fn get_url(
        &self,
        context: &SaasContext,
        url: &str,
        query: Option<HashMap<String, String>>,
    ) -> WechatResult<Url> {
        let token = self.get_access_token(context).await?;
        get_url_with_token(url, Some(token.token), query)
    }
    pub(crate) async fn api_post<T: Serialize + ?Sized, R: DeserializeOwned>(
        &self,
        context: &SaasContext,
        url: &str,
        query: Option<HashMap<String, String>>,
        body: &T,
    ) -> WechatResult<R> {
        let url = self.get_url(context, url, query).await?;
        let client = Client::new();
        let result = client
            .post(url)
            .json(&body)
            .send()
            .await?
            .json::<ApiResult<R>>()
            .await?
            .get_result()?;
        Ok(result)
    }
    pub(crate) async fn api_get<R: DeserializeOwned>(
        &self,
        context: &SaasContext,
        url: &str,
        query: Option<HashMap<String, String>>,
    ) -> WechatResult<R> {
        let url = self.get_url(context, url, query).await?;
        let result = reqwest::get(url)
            .await?
            .json::<ApiResult<R>>()
            .await?
            .get_result()?;
        Ok(result)
    }

    // pub(crate) async fn apt_upload<R: DeserializeOwned>(
    //     &self,
    //     context: &SaasContext,
    //     url: &str,
    //     query: Option<HashMap<String, String>>,
    // ) -> Result<R, WechatError> {
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_apiresult_serde_error() {
        let json = r#"{"errcode":40013,"errmsg":"invalid appid"}"#;
        let r: ApiResult<GetAccessTokenResp> = serde_json::from_str(&json).unwrap();
        assert_eq!(
            ApiResult::<GetAccessTokenResp>::Error {
                errcode: 40013,
                errmsg: "invalid appid".into(),
            },
            r
        );
    }

    #[test]
    fn test_apiresult_serde_ok() {
        let json = r#"{"access_token":"ACCESS_TOKEN","expires_in":7200}"#;
        let r: ApiResult<GetAccessTokenResp> = serde_json::from_str(&json).unwrap();
        assert_eq!(
            ApiResult::Msg(GetAccessTokenResp {
                access_token: "ACCESS_TOKEN".into(),
                expires_in: 7200,
            }),
            r
        );
    }
}
