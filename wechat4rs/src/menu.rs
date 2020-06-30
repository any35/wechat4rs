/// 自定义菜单
///
use crate::req_utils::*;
use crate::{SaasContext, Wechat, WechatError, WechatResult};
use async_trait::async_trait;
use maplit::hashmap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::vec::Vec;
use serde_repr::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Button {
    /// 点击推事件用户点击click类型按钮后，微信服务器会通过消息接口推送消息类型为event的结构给开发者
    /// （参考消息接口指南），并且带上按钮中开发者填写的key值，开发者可以通过自定义的key值与用户进行
    /// 交互；
    #[serde(rename = "click")]
    Click { name: String, key: String },
    /// 跳转URL用户点击view类型按钮后，微信客户端将会打开开发者在按钮中填写的网页URL，可与网页授权
    /// 获取用户基本信息接口结合，获得用户基本信息
    #[serde(rename = "view")]
    View { name: String, url: String },
    /// 小程序
    #[serde(rename = "miniprogram")]
    MiniProgram {
        name: String,
        url: String,
        appid: String,
        pagepath: String,
    },
    /// 扫码推事件用户点击按钮后，微信客户端将调起扫一扫工具，完成扫码操作后显示扫描结果（如果是URL，
    /// 将进入URL），且会将扫码的结果传给开发者，开发者可以下发消息。
    #[serde(rename = "scancode_push")]
    ScanCodePush { name: String, key: String },
    /// 扫码推事件且弹出“消息接收中”提示框用户点击按钮后，微信客户端将调起扫一扫工具，完成扫码操作后，
    /// 将扫码的结果传给开发者，同时收起扫一扫工具，然后弹出“消息接收中”提示框，随后可能会收到开发者下发的消息。
    #[serde(rename = "scancode_waitmsg")]
    ScanCodeWaitMsg { name: String, key: String },
    /// 弹出系统拍照发图用户点击按钮后，微信客户端将调起系统相机，完成拍照操作后，会将拍摄的相片发送给
    /// 开发者，并推送事件给开发者，同时收起系统相机，随后可能会收到开发者下发的消息。
    #[serde(rename = "pic_sysphoto")]
    PicSysPhoto { name: String, key: String },
    /// 弹出拍照或者相册发图用户点击按钮后，微信客户端将弹出选择器供用户选择“拍照”或者“从手机相册选择”。
    /// 用户选择后即走其他两种流程。
    #[serde(rename = "pic_photo_or_album")]
    PicPhotoOrAlbum { name: String, key: String },
    /// 弹出微信相册发图器用户点击按钮后，微信客户端将调起微信相册，完成选择操作后，将选择的相片发送给
    /// 开发者的服务器，并推送事件给开发者，同时收起相册，随后可能会收到开发者下发的消息。
    #[serde(rename = "pic_weixin")]
    PicWeixin { name: String, key: String },
    /// 弹出地理位置选择器用户点击按钮后，微信客户端将调起地理位置选择工具，完成选择操作后，将选择的地理
    /// 位置发送给开发者的服务器，同时收起位置选择工具，随后可能会收到开发者下发的消息。
    #[serde(rename = "location_select")]
    LocationSelect { name: String, key: String },
    /// 下发消息（除文本消息）用户点击media_id类型按钮后，微信服务器会将开发者填写的永久素材id对应的素材
    /// 下发给用户，永久素材类型可以是图片、音频、视频、图文消息。请注意：永久素材id必须是在“素材管理/新增
    /// 永久素材”接口上传后获得的合法id。
    ///
    /// Note:
    ///   是专门给第三方平台旗下未微信认证（具体而言，是资质认证未通过）的订阅号准备的事件类型，
    ///   它们是没有事件推送的，能力相对受限，其他类型的公众号不必使用。
    #[serde(rename = "media_id")]
    MediaId { name: String, media_id: String },
    /// 跳转图文消息URL用户点击view_limited类型按钮后，微信客户端将打开开发者在按钮中填写的永久素材id对应
    /// 的图文消息URL，永久素材类型只支持图文消息。请注意：永久素材id必须是在“素材管理/新增永久素材”接口上
    /// 传后获得的合法id。​
    ///
    /// Note:
    ///   是专门给第三方平台旗下未微信认证（具体而言，是资质认证未通过）的订阅号准备的事件类型，
    ///   它们是没有事件推送的，能力相对受限，其他类型的公众号不必使用。
    #[serde(rename = "view_limited")]
    ViewLimited { name: String, media_id: String },

}


/// 菜单条目
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum MenuItem<Btn> {
    /// 按钮
    Item(Btn),
    /// 子菜单,最多包含5个二级菜单
    SubMenu {
        name: String,
        sub_button: Vec<Btn>,
    },
}

/// 微信菜单, 最多包括3个一级菜单
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Menu<Btn> {
    button: Vec<MenuItem<Btn>>,
}

/// 通过API设置的菜单
pub type ApiMenu = Menu<Button>;
/// 通过公众号后台设置的菜单
pub type CustomMenu = Menu<CustomButton>;

/// 菜单查询结果
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SelfMenuInfo {
    /// 菜单是否开启，0代表未开启，1代表开启
    #[serde(rename = "is_menu_open")]
    open: bool,
    /// 菜单
    #[serde(rename = "selfmenu_info")]
    menu: SelfMenuInfoMenu,
}

impl SelfMenuInfo {
    fn from_json(json: &String) -> Result<Self, WechatError> {
        let json = serde_json::from_str(json)?;
        let json = normalize_json(json);
        if let Value::Object(mut obj) = json {
            let open = obj.get("is_menu_open")
                .map(|x| x.as_i64())
                .flatten()
                .map(|x| if x == 0 {false} else {true} )
                .unwrap_or(false);
            let selfmenu_info = obj.remove("selfmenu_info").unwrap_or(Value::Null);
            if let Ok(menu) = serde_json::from_value::<ApiMenu> (selfmenu_info.clone()) {
                return Ok(SelfMenuInfo {
                    open,
                    menu: SelfMenuInfoMenu::ApiMenu(menu),
                });
            }

            let menu = serde_json::from_value::<CustomMenu> (selfmenu_info)?; 
            return Ok(SelfMenuInfo{
                open,
                menu: SelfMenuInfoMenu::CustomMenu(menu),
            })
        }
        Err(WechatError::ParseError(format!("json 解析失败:{}", json)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
enum SelfMenuInfoMenu {
    ApiMenu(ApiMenu),
    CustomMenu(CustomMenu),
}

/// 通过UI设置的菜单
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum CustomButton {
    #[serde(rename = "news")]
    News {
        name: String,
        value:String,
        news_info: Vec<ButtonNewsItem>,
    },
    #[serde(rename = "view")]
    View {
        name: String,
        url: String,
    },
    #[serde(rename = "video")]
    Video {
        name:String,
        value: String,
    },
    #[serde(rename = "voice")]
    Voice {
        name:String,
        value:String,
    },
    #[serde(rename = "text")]
    Text {
        name: String,
        value: String,
    },
    #[serde(rename = "img")]
    Img {
        name:String,
        value:String,
    }
}
 // 男（1）女（2）
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum Gender {
    Man = 1,
    Woman = 2,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum ClientPlatform {
    IOS = 1,
     Android = 2,
     Others = 3
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// 个性化菜单
pub struct ConditionalMenuRule {
    /// 用户标签的id，可通过用户标签管理接口获取
    pub tag_id: Option<String>,
    /// 性别：男（1）女（2），不填则不做匹配
    pub sex: Option<Gender>, 
    /// 客户端版本，当前只具体到系统型号：IOS(1), Android(2),Others(3)，不填则不做匹配
    pub country:Option<String>,
    /// 国家信息，是用户在微信中设置的地区，具体请参考地区信息表
    pub province:Option<String>,
    /// 省份信息，是用户在微信中设置的地区，具体请参考地区信息表
    pub city:Option<String>,
    /// 城市信息，是用户在微信中设置的地区，具体请参考地区信息表
    pub client_platform_type: Option<ClientPlatform>,
    /// 语言信息，是用户在微信中设置的语言，具体请参考语言表： 
    /// 1、简体中文 "zh_CN" 2、繁体中文TW "zh_TW" 3、繁体中文HK "zh_HK" 
    /// 4、英文 "en" 5、印尼 "id" 6、马来 "ms" 7、西班牙 "es" 8、韩国 "ko" 
    /// 9、意大利 "it" 10、日本 "ja" 11、波兰 "pl" 12、葡萄牙 "pt" 
    /// 13、俄国 "ru" 14、泰文 "th" 15、越南 "vi" 16、阿拉伯语 "ar" 
    /// 17、北印度 "hi" 18、希伯来 "he" 19、土耳其 "tr" 20、德语 "de" 
    /// 21、法语 "fr"
    pub language: Option<String>, 
}

impl Default for ConditionalMenuRule {
    fn default() ->Self {
        ConditionalMenuRule {
            tag_id: None,
            sex: None,
            country:None,
            province:None,
            city: None,
            client_platform_type:None,
            language:None,
        }
    }
}
/// 个性化菜单, 最多包括3个一级菜单
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConditionalMenu {
    button: Vec<MenuItem<Button>>,
    matchrule: ConditionalMenuRule,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConditionalMenuItem {
    button: Vec<MenuItem<Button>>,
    matchrule: Option<ConditionalMenuRule>,
    menuid: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// 个性菜单列表
pub struct ConditionalMenuList {
    menu: ConditionalMenuItem,
    conditionalmenu: Option<Vec<ConditionalMenuItem>>,
}


use crate::core::utils::int2bool_format::bool_from_int;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ButtonNewsItem {
    title:String,
    author:String,
    digest:String,
    #[serde(deserialize_with = "bool_from_int")]
    show_cover :bool, // to u8
    cover_url:String,
    content_url:String,
    source_url:String,
}

fn normalize_json(v:Value) -> Value {
    match v {
        Value::Object(mut obj) => {
            if let Some(list) = obj.remove("list") {
                return normalize_json(list);
            }
            let obj = obj.into_iter()
                .map(|kv| (kv.0, normalize_json(kv.1)))
                .collect();
            Value::Object(obj)
        },
        Value::Array(list) => list.into_iter().map(|x| normalize_json(x)).collect(),
        _=>v
    }
}

/// 微信菜单服务
#[async_trait]
trait WechatMenu {
    /// 创建菜单
    async fn create_menu(&self, context: &SaasContext, menu: &ApiMenu) -> WechatResult<()>;

    /// 查询当前菜单
    async fn get_menu(&self, context: &SaasContext) -> WechatResult<SelfMenuInfo> ;

    /// 删除当前使用的自定义菜单
    async fn delete_all_menu(&self, context:&SaasContext) -> WechatResult<()> ;

    /// 创建个性化菜单, 返回菜单id
    async fn create_conditional_menu(&self, context: &SaasContext, menu:&ConditionalMenu) -> WechatResult<i64> ;

    /// 删除个性化菜单
    async fn delete_conditional_menu(&self, context: &SaasContext, menu_id:i64) -> WechatResult<()> ;

    /// 测试个性化菜单匹配结果
    /// openid可以是粉丝的OpenID，也可以是粉丝的微信号。
    async fn get_conditional_menu_of_user(&self, context: &SaasContext, openid:&String) ->WechatResult<ApiMenu> ;

    /// 查询全部自定义菜单
    /// 使用接口创建自定义菜单后，开发者还可使用接口查询自定义菜单的结构。另外请注意，在设置了个性化菜单后，使用本自定义菜单查询接口可以获取默认菜单和全部个性化菜单信息。
    async fn get_conditional_menu_list(&self, context: &SaasContext) -> WechatResult<ConditionalMenuList>;
}


#[async_trait]
impl WechatMenu for Wechat {
    async fn create_menu(&self, context: &SaasContext, menu: &ApiMenu) -> WechatResult<()> {
        self.api_post(context, "cgi-bin/menu/create", None, menu)
            .await
    }

    async fn get_menu(&self, context: &SaasContext) -> WechatResult<SelfMenuInfo> {
        self.api_get(context, "cgi-bin/get_current_selfmenu_info", None)
            .await
    }

    async fn delete_all_menu(&self, context:&SaasContext) -> WechatResult<()> {
        self.api_get(context, "cgi-bin/menu/delete", None)
            .await
    }

    async fn create_conditional_menu(&self, context: &SaasContext, menu:&ConditionalMenu) -> WechatResult<i64> {
        #[derive(Deserialize)]
        struct ConditionalMenuResult {
            menuid: i64,
        }
        let result : ConditionalMenuResult = self.api_post(context, "cgi-bin/menu/addconditional", None, menu)
            .await?;
        Ok(result.menuid)
    }

    /// 删除个性化菜单
    async fn delete_conditional_menu(&self, context: &SaasContext, menu_id:i64) -> WechatResult<()> {
        self.api_post(context, "cgi-bin/menu/delconditional", None, &json!({ "menuid": menu_id }))
            .await
    }

    /// 测试个性化菜单匹配结果
    /// openid可以是粉丝的OpenID，也可以是粉丝的微信号。
    async fn get_conditional_menu_of_user(&self, context: &SaasContext, openid:&String) -> WechatResult<ApiMenu> {
        self.api_post(context, "cgi-bin/menu/trymatch", None, &json!({"user_id": openid}))
            .await
    }

    /// 查询全部自定义菜单
    /// 使用接口创建自定义菜单后，开发者还可使用接口查询自定义菜单的结构。另外请注意，在设置了个性化菜单后，使用本自定义菜单查询接口可以获取默认菜单和全部个性化菜单信息。
    async fn get_conditional_menu_list(&self, context: &SaasContext) -> WechatResult<ConditionalMenuList> {
        self.api_get(context, "cgi-bin/menu/get", None)
            .await
    }

}

mod test {
    use super::*;
    #[test]
    fn menu_serde() {
        let menu = ApiMenu {
            button: vec![
                MenuItem::Item(Button::Click {
                    name: "最新活动".into(),
                    key: "latest_news".into(),
                }),
                MenuItem::SubMenu {
                    name: "更多".into(),
                    sub_button: vec![Button::View {
                        name: "百度".into(),
                        url: "https://baidu.com".into(),
                    }],
                },
            ],
        };
        let menu_str = serde_json::to_string(&menu).unwrap();

        assert_eq!(
            menu_str,
            r#"{"button":[{"type":"click","name":"最新活动","key":"latest_news"},{"name":"更多","sub_button":[{"type":"view","name":"百度","url":"https://baidu.com"}]}]}"#
        );
    }

    #[test]
    fn button_click_serde() {
        let btn = Button::Click {
            name: "今日歌曲".into(),
            key: "V1001_TODAY_MUSIC".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"click","name":"今日歌曲","key":"V1001_TODAY_MUSIC"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_view_serde() {
        let btn = Button::View {
            name: "搜索".into(),
            url: "http://www.soso.com/".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"view","name":"搜索","url":"http://www.soso.com/"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_miniprogram_serde() {
        let btn = Button::MiniProgram {
            name: "wxa".into(),
            url: "http://mp.weixin.qq.com".into(),
            appid: "wx286b93c14bbf93aa".into(),
            pagepath: "pages/lunar/index".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"miniprogram","name":"wxa","url":"http://mp.weixin.qq.com","appid":"wx286b93c14bbf93aa","pagepath":"pages/lunar/index"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_scancode_push_serde() {
        let btn = Button::ScanCodePush {
            name: "扫码推事件".into(),
            key: "rselfmenu_0_1".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"scancode_push","name":"扫码推事件","key":"rselfmenu_0_1"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_scancode_waitmsg_serde() {
        let btn = Button::ScanCodeWaitMsg {
            name: "扫码带提示".into(),
            key: "rselfmenu_0_0".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"scancode_waitmsg","name":"扫码带提示","key":"rselfmenu_0_0"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_pic_sysphoto_serde() {
        let btn = Button::PicSysPhoto {
            name: "系统拍照发图".into(),
            key: "rselfmenu_1_0".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"pic_sysphoto","name":"系统拍照发图","key":"rselfmenu_1_0"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_pic_photo_or_album_serde() {
        let btn = Button::PicPhotoOrAlbum {
            name: "拍照或者相册发图".into(),
            key: "rselfmenu_1_1".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"pic_photo_or_album","name":"拍照或者相册发图","key":"rselfmenu_1_1"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_pic_weixin_serde() {
        let btn = Button::PicWeixin {
            name: "微信相册发图".into(),
            key: "rselfmenu_1_2".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"pic_weixin","name":"微信相册发图","key":"rselfmenu_1_2"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_location_select_serde() {
        let btn = Button::LocationSelect {
            name: "发送位置".into(),
            key: "rselfmenu_2_0".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"location_select","name":"发送位置","key":"rselfmenu_2_0"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_media_id_serde() {
        let btn = Button::MediaId {
            name: "图片".into(),
            media_id: "MEDIA_ID1".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"media_id","name":"图片","media_id":"MEDIA_ID1"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn button_view_limited_serde() {
        let btn = Button::ViewLimited {
            name: "图文消息".into(),
            media_id: "MEDIA_ID2".into(),
        };
        let json = serde_json::to_string(&btn).unwrap();
        assert_eq!(
            json,
            r#"{"type":"view_limited","name":"图文消息","media_id":"MEDIA_ID2"}"#
        );
        let btn2: Button = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(btn, btn2);
    }

    #[test]
    fn custom_menu_serde1() {

        let json = r#" { "name": "news", "type": "news", "value": "KQb_w_Tiz-nSdVLoTV35Psmty8hGBulGhEdbb9SKs-o", "news_info": [
        { "author": "JIMZHENG", "content_url": "http://mp.weixin.qq.com/s?__biz=MjM5ODUwNTM3Ng==&mid=204013432&idx=1&sn=80ce6d9abcb832237bf86c87e50fda15#rd", "cover_url": "http://mmbiz.qpic.cn/mmbiz/GE7et87vE9vicuCibqXsX9GPPLuEtBfXfK0HKuBIa1A1cypS0uY1wickv70iaY1gf3I1DTszuJoS3lAVLvhTcm9sDA/0", "digest": "text", "show_cover": 0, "source_url": "", "title": "MULTI_NEWS" },
        { "author": "JIMZHENG", "content_url": "http://mp.weixin.qq.com/s?__biz=MjM5ODUwNTM3Ng==&mid=204013432&idx=2&sn=8226843afb14ecdecb08d9ce46bc1d37#rd", "cover_url": "http://mmbiz.qpic.cn/mmbiz/GE7et87vE9vicuCibqXsX9GPPLuEtBfXfKnmnpXYgWmQD5gXUrEApIYBCgvh2yHsu3ic3anDUGtUCHwjiaEC5bicd7A/0", "digest": "MULTI_NEWS1", "show_cover": 1, "source_url": "", "title": "MULTI_NEWS1" }
        ] }"#;
        let info:CustomButton = serde_json::from_str(json).unwrap();
        assert_eq!(info, CustomButton::News{
            name:"news".into(),
            value:"KQb_w_Tiz-nSdVLoTV35Psmty8hGBulGhEdbb9SKs-o".into(),
            news_info: vec![
                ButtonNewsItem{
                    title:"MULTI_NEWS".into(),
                    author:"JIMZHENG".into(),
                    digest:"text".into(),
                    show_cover:false,
                    cover_url:"http://mmbiz.qpic.cn/mmbiz/GE7et87vE9vicuCibqXsX9GPPLuEtBfXfK0HKuBIa1A1cypS0uY1wickv70iaY1gf3I1DTszuJoS3lAVLvhTcm9sDA/0".into(),
                    content_url:"http://mp.weixin.qq.com/s?__biz=MjM5ODUwNTM3Ng==&mid=204013432&idx=1&sn=80ce6d9abcb832237bf86c87e50fda15#rd".into(),
                    source_url:"".into(),
                },
                ButtonNewsItem{
                    title:"MULTI_NEWS1".into(),
                    author:"JIMZHENG".into(),
                    digest:"MULTI_NEWS1".into(),
                    show_cover:true,
                    cover_url:"http://mmbiz.qpic.cn/mmbiz/GE7et87vE9vicuCibqXsX9GPPLuEtBfXfKnmnpXYgWmQD5gXUrEApIYBCgvh2yHsu3ic3anDUGtUCHwjiaEC5bicd7A/0".into(),
                    content_url:"http://mp.weixin.qq.com/s?__biz=MjM5ODUwNTM3Ng==&mid=204013432&idx=2&sn=8226843afb14ecdecb08d9ce46bc1d37#rd".into(),
                    source_url:"".into(),
                }, 
            ]
        });

        let json = r#"{ "name": "video", "type": "video", "value": "http://61.182.130.30/vweixinp.tc.qq.com/1007_114bcede9a2244eeb5ab7f76d951df5f.f10.mp4?vkey=77A42D0C2015FBB0A3653D29C571B5F4BBF1D243FBEF17F09C24FF1F2F22E30881BD350E360BC53F&sha=0&save=1" }"#;
        let info:CustomButton = serde_json::from_str(json).unwrap();
        assert_eq!(info, CustomButton::Video {
            name:"video".into(),
            value:"http://61.182.130.30/vweixinp.tc.qq.com/1007_114bcede9a2244eeb5ab7f76d951df5f.f10.mp4?vkey=77A42D0C2015FBB0A3653D29C571B5F4BBF1D243FBEF17F09C24FF1F2F22E30881BD350E360BC53F&sha=0&save=1".into(),
        });

        let json = r#"{ "name": "voice", "type": "voice", "value": "nTXe3aghlQ4XYHa0AQPWiQQbFW9RVtaYTLPC1PCQx11qc9UB6CiUPFjdkeEtJicn" }"#;
        let info:CustomButton = serde_json::from_str(json).unwrap();
        assert_eq!(info, CustomButton::Voice {
            name:"voice".into(),
            value:"nTXe3aghlQ4XYHa0AQPWiQQbFW9RVtaYTLPC1PCQx11qc9UB6CiUPFjdkeEtJicn".into(),
        });

        let json = r#"{ "name": "text", "type": "text", "value": "This is text!" }"#;
        let info:CustomButton = serde_json::from_str(json).unwrap();
        assert_eq!(info, CustomButton::Text {
            name:"text".into(),
            value:"This is text!".into(),
        });

        let json = r#"{ "name": "photo", "type": "img", "value": "ax5Whs5dsoomJLEppAvftBUuH7CgXCZGFbFJifmbUjnQk_ierMHY99Y5d2Cv14RD" }"#;
        let info:CustomButton = serde_json::from_str(json).unwrap();
        assert_eq!(info, CustomButton::Img {
            name:"photo".into(),
            value:"ax5Whs5dsoomJLEppAvftBUuH7CgXCZGFbFJifmbUjnQk_ierMHY99Y5d2Cv14RD".into(),
        });

    }

    fn get_custom_menu_json() -> &'static str {
        r#"
{ "button": [
    { "name": "button", "sub_button": [
        { "name": "view_url", "type": "view", "url": "http://www.qq.com" },
        { "name": "news", "type": "news", "value": "KQb_w_Tiz-nSdVLoTV35Psmty8hGBulGhEdbb9SKs-o", "news_info": [
            { "author": "JIMZHENG", "content_url": "http://mp.weixin.qq.com/s?__biz=MjM5ODUwNTM3Ng==&mid=204013432&idx=1&sn=80ce6d9abcb832237bf86c87e50fda15#rd", "cover_url": "http://mmbiz.qpic.cn/mmbiz/GE7et87vE9vicuCibqXsX9GPPLuEtBfXfK0HKuBIa1A1cypS0uY1wickv70iaY1gf3I1DTszuJoS3lAVLvhTcm9sDA/0", "digest": "text", "show_cover": 0, "source_url": "", "title": "MULTI_NEWS" },
            { "author": "JIMZHENG", "content_url": "http://mp.weixin.qq.com/s?__biz=MjM5ODUwNTM3Ng==&mid=204013432&idx=2&sn=8226843afb14ecdecb08d9ce46bc1d37#rd", "cover_url": "http://mmbiz.qpic.cn/mmbiz/GE7et87vE9vicuCibqXsX9GPPLuEtBfXfKnmnpXYgWmQD5gXUrEApIYBCgvh2yHsu3ic3anDUGtUCHwjiaEC5bicd7A/0", "digest": "MULTI_NEWS1", "show_cover": 1, "source_url": "", "title": "MULTI_NEWS1" }
        ] },
        { "name": "video", "type": "video", "value": "http://61.182.130.30/vweixinp.tc.qq.com/1007_114bcede9a2244eeb5ab7f76d951df5f.f10.mp4?vkey=77A42D0C2015FBB0A3653D29C571B5F4BBF1D243FBEF17F09C24FF1F2F22E30881BD350E360BC53F&sha=0&save=1" },
        { "name": "voice", "type": "voice", "value": "nTXe3aghlQ4XYHa0AQPWiQQbFW9RVtaYTLPC1PCQx11qc9UB6CiUPFjdkeEtJicn" }
    ] },
    { "name": "text", "type": "text", "value": "This is text!" },
    { "name": "photo", "type": "img", "value": "ax5Whs5dsoomJLEppAvftBUuH7CgXCZGFbFJifmbUjnQk_ierMHY99Y5d2Cv14RD" }
] }"#
    }

    fn get_custom_menu() -> CustomMenu {
        CustomMenu {
            button: vec![
                MenuItem::SubMenu{
                    name:"button".into(),
                    sub_button: vec![
                        CustomButton::View {
                            name:"view_url".into(),
                            url:"http://www.qq.com".into(),
                        },
                        CustomButton::News {
                            name:"news".into(),
                            value:"KQb_w_Tiz-nSdVLoTV35Psmty8hGBulGhEdbb9SKs-o".into(),
                            news_info: vec![
                                ButtonNewsItem{
                                    title:"MULTI_NEWS".into(),
                                    author:"JIMZHENG".into(),
                                    digest:"text".into(),
                                    show_cover:false,
                                    cover_url:"http://mmbiz.qpic.cn/mmbiz/GE7et87vE9vicuCibqXsX9GPPLuEtBfXfK0HKuBIa1A1cypS0uY1wickv70iaY1gf3I1DTszuJoS3lAVLvhTcm9sDA/0".into(),
                                    content_url:"http://mp.weixin.qq.com/s?__biz=MjM5ODUwNTM3Ng==&mid=204013432&idx=1&sn=80ce6d9abcb832237bf86c87e50fda15#rd".into(),
                                    source_url:"".into(),
                                },
                                ButtonNewsItem{
                                    title:"MULTI_NEWS1".into(),
                                    author:"JIMZHENG".into(),
                                    digest:"MULTI_NEWS1".into(),
                                    show_cover:true,
                                    cover_url:"http://mmbiz.qpic.cn/mmbiz/GE7et87vE9vicuCibqXsX9GPPLuEtBfXfKnmnpXYgWmQD5gXUrEApIYBCgvh2yHsu3ic3anDUGtUCHwjiaEC5bicd7A/0".into(),
                                    content_url:"http://mp.weixin.qq.com/s?__biz=MjM5ODUwNTM3Ng==&mid=204013432&idx=2&sn=8226843afb14ecdecb08d9ce46bc1d37#rd".into(),
                                    source_url:"".into(),
                                }, 
                            ]
                        },
                        CustomButton::Video {
                            name:"video".into(),
                            value:"http://61.182.130.30/vweixinp.tc.qq.com/1007_114bcede9a2244eeb5ab7f76d951df5f.f10.mp4?vkey=77A42D0C2015FBB0A3653D29C571B5F4BBF1D243FBEF17F09C24FF1F2F22E30881BD350E360BC53F&sha=0&save=1".into(),
                        },
                        CustomButton::Voice {
                            name:"voice".into(),
                            value:"nTXe3aghlQ4XYHa0AQPWiQQbFW9RVtaYTLPC1PCQx11qc9UB6CiUPFjdkeEtJicn".into(),
                        }
                    ],
                },
                MenuItem::Item( CustomButton::Text {
                    name:"text".into(),
                    value:"This is text!".into(),
                }),
                MenuItem::Item(CustomButton::Img {
                    name:"photo".into(),
                    value:"ax5Whs5dsoomJLEppAvftBUuH7CgXCZGFbFJifmbUjnQk_ierMHY99Y5d2Cv14RD".into(),
                }),
            ]
        }
    }

    #[test]
    fn custom_menu_serde2() {
        let json = get_custom_menu_json();
        let menu:CustomMenu = serde_json::from_str(json).unwrap();
        assert_eq!(menu, get_custom_menu());
    }

    #[test]
    fn self_menu_serde() {
        let json = format!(r#"
{{ 
   "is_menu_open": 1, 
   "selfmenu_info": {} 
}}"#, get_custom_menu_json());

        let info = SelfMenuInfo::from_json(&json.into()).unwrap();
        let result = SelfMenuInfo {
            open:true,
            menu:SelfMenuInfoMenu::CustomMenu(get_custom_menu())
        };
        assert_eq!(info, result);
    }

    #[test]
    fn api_menu_serde() {
        let json = r#"
{ 
   "is_menu_open": 1, 
   "selfmenu_info": { "button": [ 
        { "type": "click", "name": "今日歌曲", "key": "V1001_TODAY_MUSIC" }, 
        { "name": "菜单", "sub_button": { "list": [ 
            { "type": "view", "name": "搜索", "url": "http://www.soso.com/" }, 
            { "type": "view", "name": "视频", "url": "http://v.qq.com/" }, 
            { "type": "click", "name": "赞一下我们", "key": "V1001_GOOD" }
        ] } }
    ] }
}"#;
        let info = SelfMenuInfo::from_json(&json.into()).unwrap();
        let result = SelfMenuInfo {
            open:true,
            menu:SelfMenuInfoMenu::ApiMenu(ApiMenu {
                button:vec![
                    MenuItem::Item(Button::Click{
                        name:"今日歌曲".into(),
                        key:"V1001_TODAY_MUSIC".into(),
                    }),
                    MenuItem::SubMenu {
                        name: "菜单".into(),
                        sub_button: vec![
                            Button::View {
                                name:"搜索".into(),
                                url:"http://www.soso.com/".into(),
                            },
                            Button::View {
                                name:"视频".into(),
                                url:"http://v.qq.com/".into(),
                            },
                            Button::Click {
                                name:"赞一下我们".into(),
                                key:"V1001_GOOD".into(),
                            }
                        ]
                    }
                ]
            })
        };
        assert_eq!(info, result);
    }

    #[test]
    fn conditional_menu_list() {
        let json = r#"
{ "menu": {
    "button": [
        { "type": "click", "name": "今日歌曲", "key": "V1001_TODAY_MUSIC", "sub_button": [ ] }, 
        { "type": "click", "name": "歌手简介", "key": "V1001_TODAY_SINGER", "sub_button": [ ] }, 
        { "name": "菜单", "sub_button": [
            { "type": "view", "name": "搜索", "url": "http://www.soso.com/", "sub_button": [ ] }, 
            { "type": "view", "name": "视频", "url": "http://v.qq.com/", "sub_button": [ ] }, 
            { "type": "click", "name": "赞一下我们", "key": "V1001_GOOD", "sub_button": [ ] }
        ] }
    ]
} }"#;
        let menu:ConditionalMenuList = serde_json::from_str(json).unwrap();
        assert_eq!(menu, ConditionalMenuList {
            menu: ConditionalMenuItem {
                button: vec![
                    MenuItem::Item(Button::Click{name:"今日歌曲".into(), key:"V1001_TODAY_MUSIC".into()}),
                    MenuItem::Item(Button::Click{name:"歌手简介".into(), key:"V1001_TODAY_SINGER".into()}),
                    MenuItem::SubMenu {
                        name: "菜单".into(),
                        sub_button:vec![
                            Button::View{name: "搜索".into(), url:"http://www.soso.com/".into()},
                            Button::View{name: "视频".into(), url:"http://v.qq.com/".into()},
                            Button::Click{name: "赞一下我们".into(), key:"V1001_GOOD".into()},
                        ],
                    },
                ],
                matchrule:None,
                menuid:None,
            },
            conditionalmenu: None,
        });
    }

    #[test]
    fn conditional_menu_list2() {
        let json = r#"
{
    "menu": {
        "button": [ { "type": "click", "name": "今日歌曲", "key": "V1001_TODAY_MUSIC", "sub_button": [ ] } ], 
        "menuid": 208396938
    }, 
    "conditionalmenu": [
        {
            "button": [
                { "type": "click", "name": "今日歌曲", "key": "V1001_TODAY_MUSIC", "sub_button": [ ] }, 
                { "name": "菜单", "sub_button": [
                        { "type": "view", "name": "搜索", "url": "http://www.soso.com/", "sub_button": [ ] }, 
                        { "type": "view", "name": "视频", "url": "http://v.qq.com/", "sub_button": [ ] }, 
                        { "type": "click", "name": "赞一下我们", "key": "V1001_GOOD", "sub_button": [ ] }
                ] }
            ], 
            "matchrule": {
                "group_id": 2, 
                "sex": 1, 
                "country": "中国", 
                "province": "广东", 
                "city": "广州", 
                "client_platform_type": 2
            }, 
            "menuid": 208396993
        }
    ]
}"#;
        let menu:ConditionalMenuList = serde_json::from_str(json).unwrap();
        assert_eq!(menu, ConditionalMenuList {
            menu: ConditionalMenuItem {
                button: vec![
                  MenuItem::Item(Button::Click{name:"今日歌曲".into(), key:"V1001_TODAY_MUSIC".into()}),
                ],
                matchrule:None,
                menuid: Some(208396938),
            },
            conditionalmenu: Some(vec![ 
                ConditionalMenuItem {
                    button: vec![
                        MenuItem::Item(Button::Click{name:"今日歌曲".into(), key:"V1001_TODAY_MUSIC".into()}),
                        MenuItem::SubMenu {
                            name: "菜单".into(),
                            sub_button:vec![
                                Button::View{name: "搜索".into(), url:"http://www.soso.com/".into()},
                                Button::View{name: "视频".into(), url:"http://v.qq.com/".into()},
                                Button::Click{name: "赞一下我们".into(), key:"V1001_GOOD".into()},
                            ],
                        },
                    ],
                    matchrule: Some(ConditionalMenuRule {
                        // todo: group_id????
                        sex: Some(Gender::Man),
                        country: Some("中国".into()),
                        province: Some("广东".into()),
                        city: Some("广州".into()),
                        client_platform_type: Some(ClientPlatform::Android),
                        ..ConditionalMenuRule::default()
                    }),
                    menuid: Some(208396993),
                }
            ]),
        });

    }
}

