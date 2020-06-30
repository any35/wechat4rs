use crate::req_utils::*;
use crate::{SaasContext, Wechat, WechatError, WechatResult};
use async_trait::async_trait;
use maplit::hashmap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::vec::Vec;

/// 客服消息
///
/// 客服账号
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KfAccount {
    /// 完整客服账号，格式为：账号前缀@公众号微信号
    #[serde(rename = "kf_account")]
    pub account: String,
    /// 客服昵称，最长6个汉字或12个英文字符
    #[serde(rename = "kf_nick")]
    pub nickname: String,
    /// 客服工号
    #[serde(rename = "kf_id")]
    pub id: String,
    #[serde(rename = "kf_headimgurl")]
    pub headimageurl: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KfMsgMunuListItem {
    pub id: String,
    pub content: String,
}

/// 发送客服消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KfMessage {
    /// 文本消息
    Text { content: String },
    /// 图片消息
    Image { media_id: String },
    /// 语音消息
    Voice { media_id: String },
    /// 视频消息
    Video {
        media_id: String,
        thumb_media_id: String,
        title: Option<String>,
        description: Option<String>,
    },
    /// 音乐消息
    Music {
        title: Option<String>,
        description: Option<String>,
        /// 音乐链接
        musicurl: String,
        /// 高品质音乐链接，wifi环境优先使用该链接播放音乐
        hqmusicurl: String,
        thumb_media_id: String,
    },
    /// 图文消息（点击跳转到外链） 图文消息条数限制在1条以内，注意，如果图文数超过1，则将会返回错误码45008。
    News {
        title: Option<String>,
        description: Option<String>,
        url: Option<String>,
        picurl: Option<String>,
    },
    /// 图文消息（点击跳转到图文消息页面） 图文消息条数限制在1条以内，注意，如果图文数超过1，则将会返回错误码45008。
    MpNews { media_id: String },
    /// 菜单消息
    MsgMenu {
        head_content: String,
        tail_content: String,
        list: Vec<KfMsgMunuListItem>,
    },
    /// 卡券
    WxCard { card_id: String },
    /// 发送小程序卡片（要求小程序与公众号已关联）
    MiniProgramPage {
        title: Option<String>,
        appid: String,
        pagepath: String,
        thumb_media_id: String,
    },
}
impl KfMessage {
    fn get_msgtype(&self) -> &str {
        match self {
            KfMessage::Text { content: _ } => "text",
            KfMessage::Image { media_id: _ } => "image",
            KfMessage::Voice { media_id: _ } => "voice",
            KfMessage::Video {
                media_id: _,
                thumb_media_id: _,
                title: _,
                description: _,
            } => "video",
            KfMessage::Music {
                title: _,
                description: _,
                musicurl: _,
                hqmusicurl: _,
                thumb_media_id: _,
            } => "music",
            KfMessage::News {
                title: _,
                description: _,
                url: _,
                picurl: _,
            } => "news",
            KfMessage::MpNews { media_id: _ } => "mpnews",
            KfMessage::MsgMenu {
                head_content: _,
                tail_content: _,
                list: _,
            } => "msgmenu",
            KfMessage::WxCard { card_id: _ } => "wxcard",
            KfMessage::MiniProgramPage {
                title: _,
                appid: _,
                pagepath: _,
                thumb_media_id: _,
            } => "miniprogrampage",
        }
    }
}
/// 客服消息
///
#[async_trait]
pub trait Customservice {
    /// 添加客服帐号
    /// 开发者可以通过本接口为公众号添加客服账号，每个公众号最多添加10个客服账号。
    /// password: 客服账号登录密码，格式为密码明文的32位加密MD5值。该密码仅用于在公众平台官网的多客服功能中使用，若不使用多客服功能，则不必设置密码
    async fn add_account(
        &self,
        context: &SaasContext,
        account: &String,
        nickname: &String,
        password: Option<String>,
    ) -> WechatResult<()>;

    /// 修改客服帐号
    /// 开发者可以通过本接口为公众号修改客服账号。
    async fn update_account(
        &self,
        context: &SaasContext,
        account: &String,
        nickname: &String,
        password: Option<String>,
    ) -> WechatResult<()>;

    /// 删除客服帐号
    /// 开发者可以通过该接口为公众号删除客服帐号。
    async fn del_account(
        &self,
        context: &SaasContext,
        account: &String,
        nickname: &String,
        password: Option<String>,
    ) -> WechatResult<()>;

    /// 获取所有客服账号
    /// 开发者通过本接口，获取公众号中所设置的客服基本信息，包括客服工号、客服昵称、客服登录账号。
    async fn get_list(&self, context: &SaasContext) -> Result<Vec<KfAccount>, WechatError>;

    /// 客服接口-发消息
    async fn send_msg(
        &self,
        context: &SaasContext,
        touser: String,
        msg: KfMessage,
        kf_account: Option<String>,
    ) -> WechatResult<()>;

    /// 客服输入状态
    /// 此接口需要客服消息接口权限。
    /// 1. 如果不满足发送客服消息的触发条件，则无法下发输入状态。
    /// 2. 下发输入状态，需要客服之前30秒内跟用户有过消息交互。
    /// 3. 在输入状态中（持续15s），不可重复下发输入态。
    /// 4. 在输入状态中，如果向用户下发消息，会同时取消输入状态。
    /// "Typing"：对用户下发“正在输入"状态 "CancelTyping"：取消对用户的”正在输入"状态
    async fn typing(&self, context: &SaasContext, touser: String, typing: bool)
        -> WechatResult<()>;
}

#[async_trait]
impl Customservice for Wechat {
    async fn add_account(
        &self,
        context: &SaasContext,
        account: &String,
        nickname: &String,
        password: Option<String>,
    ) -> WechatResult<()> {
        let mut body = hashmap! {
            "kf_account" => account,
            "nickname" => nickname,
        };
        if let Some(password) = &password {
            body.insert("password", password);
        }
        self.api_post(context, "customservice/kfaccount/add", None, &body)
            .await
    }

    async fn update_account(
        &self,
        context: &SaasContext,
        account: &String,
        nickname: &String,
        password: Option<String>,
    ) -> WechatResult<()> {
        let mut body = hashmap! {
            "kf_account" => account,
            "nickname" => nickname,
        };
        if let Some(password) = &password {
            body.insert("password", password);
        }
        self.api_post(context, "customservice/kfaccount/update", None, &body)
            .await
    }

    async fn del_account(
        &self,
        context: &SaasContext,
        account: &String,
        nickname: &String,
        password: Option<String>,
    ) -> WechatResult<()> {
        let mut body = hashmap! {
            "kf_account" => account,
            "nickname" => nickname,
        };
        if let Some(password) = &password {
            body.insert("password", password);
        }
        self.api_post(context, "customservice/kfaccount/del", None, &body)
            .await
    }

    async fn get_list(&self, context: &SaasContext) -> Result<Vec<KfAccount>, WechatError> {
        self.api_get(context, "cgi-bin/customservice/getkflist", None)
            .await
    }

    /// 客服接口-发消息
    async fn send_msg(
        &self,
        context: &SaasContext,
        touser: String,
        msg: KfMessage,
        kf_account: Option<String>,
    ) -> WechatResult<()> {
        let msgtype = msg.get_msgtype();
        let mut msg_value = serde_json::to_value(msg.clone())?;
        if let KfMessage::News {
            title: _,
            description: _,
            url: _,
            picurl: _,
        } = &msg
        {
            msg_value = json! ({
                 "articles": [ msg_value ],
            });
        };
        let mut msg_value = json!({
            "touser": touser,
            "msgtype": msgtype.clone(),
            msgtype: msg_value,
        });
        if let Some(kf_account) = kf_account {
            msg_value["customservice"] = json!({
                "kf_account": kf_account,
            });
        }
        self.api_post(context, "cgi-bin/message/custom/send", None, &msg_value)
            .await
    }

    async fn typing(
        &self,
        context: &SaasContext,
        touser: String,
        typing: bool,
    ) -> WechatResult<()> {
        let body = json!({
            "touser": touser,
            "command": if typing {"Typing"} else {"CancelTyping"},
        });
        self.api_post(context, "cgi-bin/message/custom/typing", None, &body)
            .await
    }
}
