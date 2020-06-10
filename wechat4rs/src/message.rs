use crate::core::{WechatEncryptError, WechatError};
use log::{info};

/// 事件消息
#[derive(Debug, PartialEq, Clone)]
pub enum EventMessage {
    /// 订阅
    Subscribe {},
    /// 取消订阅
    Unsubscribe {},
    /// 扫码关注
    QrSubscribe {
        /// 事件KEY值，qrscene_为前缀，后面为二维码的参数值
        event_key: String,
        /// 二维码的ticket，可用来换取二维码图片
        ticket: String,
    },
    /// 用户已关注时的事件推送
    Scan {
        /// 事件KEY值，是一个32位无符号整数，即创建二维码时的二维码scene_id
        event_key: String,
        /// 二维码的ticket，可用来换取二维码图片
        ticket: String,
    },
    /// 上报地理位置事件
    Location {
        /// 地理位置纬度
        lat: f64,
        /// 地理位置经度
        lng: f64,
        /// 地理位置精度
        precesion: f64,
    },
    /// 自定义菜单事件
    Click {
        /// 事件KEY值，与自定义菜单接口中KEY值对应
        event_key: String,
    },
    /// 点击菜单跳转链接时的事件推送
    View { event_key: String },
}

#[derive(Debug, PartialEq, Clone)]
pub struct MessageInfo {
    /// 开发者微信号
    pub to_user_name: String,
    /// 发送方帐号（一个OpenID）
    pub from_user_name: String,
    /// 消息创建时间 （整型）
    pub create_time: u64,
    /// 消息id，64位整型
    pub msg_id: Option<u64>,
}

/// 微信回调消息
#[derive(Debug, PartialEq, Clone)]
pub enum CallbackMessage {
    /// 文本消息
    Text {
        info: MessageInfo,
        /// 文本消息内容
        content: String,
        /// 菜单消息
        biz_msg_menu_id: Option<String>,
    },
    /// 图片消息
    Image {
        info: MessageInfo,
        /// 图片链接（由系统生成）
        pic_url: String,
        /// 图片消息媒体id，可以调用获取临时素材接口拉取数据。
        media_id: String,
    },
    /// 语音消息
    Voice {
        info: MessageInfo,
        /// 语音消息媒体id，可以调用获取临时素材接口拉取数据
        media_id: String,
        /// 语音格式，如amr，speex等
        format: String,
        /// 语音识别结果，UTF8编码
        recognition: Option<String>,
    },
    /// 视频消息
    Video {
        info: MessageInfo,
        /// 视频消息媒体id，可以调用获取临时素材接口拉取数据
        media_id: String,
        /// 视频消息缩略图的媒体id，可以调用多媒体文件下载接口拉取数据
        thumb_media_id: String,
    },
    /// 小视频消息
    ShortVideo {
        info: MessageInfo,
        /// 视频消息媒体id，可以调用获取临时素材接口拉取数据。
        media_id: String,
        /// 视频消息缩略图的媒体id，可以调用获取临时素材接口拉取数据
        thumb_media_id: String,
    },
    /// 地理位置消息
    Location {
        info: MessageInfo,
        /// 地理位置纬度
        x: f64,
        /// 地理位置经度
        y: f64,
        /// 地图缩放大小
        scale: u64,
        /// 地理位置信息
        label: String,
    },
    /// 链接消息
    Link {
        info: MessageInfo,
        /// 消息标题
        title: String,
        /// 消息描述
        description: String,
        /// 消息链接
        url: String,
    },
    /// 回调事件
    Event {
        info: MessageInfo,
        event: EventMessage,
    },
}

pub fn from_xml(xml: &str) -> Result<CallbackMessage, WechatError> {
    use sxd_document::parser;
    use sxd_xpath::evaluate_xpath;

    let package = parser::parse(xml).unwrap();
    let doc = package.as_document();

    let get_string = |path: &str| -> Result<String, WechatError> {
        let path = format!("/xml/{}", path);
        Ok(evaluate_xpath(&doc, &path)?.string())
    };

    let get_f64_option = |path: &str| -> Result<Option<f64>, WechatError> {
        let path = format!("/xml/{}", path);
        let path = evaluate_xpath(&doc, &path)?;
        let n = path.number();
        if n.is_nan() {
            Ok(None)
        } else {
            Ok(Some(n))
        }
    };
    let get_f64 = |path: &str| -> Result<f64, WechatError> {
        let n = get_f64_option(path)?.unwrap_or(0f64);
        Ok(n)
    };

    let get_u64 = |path: &str| -> Result<u64, WechatError> { Ok(get_f64(path)? as u64) };

    let get_u64_option = |path: &str| -> Result<Option<u64>, WechatError> {
        let n = get_f64_option(path)?;
        let n = match n {
            Some(n) => Some(n as u64),
            None => None,
        };
        Ok(n)
    };

    let get_string_optional = |path: &str| -> Result<Option<String>, WechatError> {
        get_string(path)
            .map(|result| -> Option<String> {
                if result.is_empty() {
                    None
                } else {
                    Some(result)
                }
            })
            .map(|r| Ok(r))?
    };
    let msg_type = get_string("MsgType")?;

    let info = MessageInfo {
        to_user_name: get_string("ToUserName")?,
        from_user_name: get_string("FromUserName")?,
        create_time: get_u64("CreateTime")?,
        msg_id: get_u64_option("MsgId")?,
    };

    let msg = match msg_type.as_str() {
        "text" => CallbackMessage::Text {
            info,
            content: get_string("Content")?,
            biz_msg_menu_id: get_string_optional("bizmsgmenuid")?,
        },
        "image" => CallbackMessage::Image {
            info,
            pic_url: get_string("PicUrl")?,
            media_id: get_string("MediaId")?,
        },
        "voice" => CallbackMessage::Voice {
            info,
            media_id: get_string("MediaId")?,
            format: get_string("Format")?,
            recognition: get_string_optional("Recognition")?,
        },
        "video" => CallbackMessage::Video {
            info,
            media_id: get_string("MediaId")?,
            thumb_media_id: get_string("ThumbMediaId")?,
        },
        "shortvideo" => CallbackMessage::ShortVideo {
            info,
            media_id: get_string("MediaId")?,
            thumb_media_id: get_string("ThumbMediaId")?,
        },
        "location" => CallbackMessage::Location {
            info,
            x: get_f64("Location_X")?,
            y: get_f64("Location_Y")?,
            scale: get_u64("Scale")?,
            label: get_string("Label")?,
        },
        "link" => CallbackMessage::Link {
            info,
            title: get_string("Title")?,
            description: get_string("Description")?,
            url: get_string("Url")?,
        },
        "event" => {
            let event = get_string("Event")?;
            let event_key = get_string("EventKey")?;
            let event = match event.as_str() {
                "subscribe" => {
                    if event_key.is_empty() {
                        EventMessage::Subscribe {}
                    } else {
                        EventMessage::QrSubscribe {
                            event_key,
                            ticket: get_string("Ticket")?,
                        }
                    }
                }
                "SCAN" => EventMessage::Scan {
                    event_key,
                    ticket: get_string("Ticket")?,
                },
                "LOCATION" => EventMessage::Location {
                    lat: get_f64("Latitude")?,
                    lng: get_f64("Longitude")?,
                    precesion: get_f64("Precision")?,
                },
                "CLICK" => EventMessage::Click { event_key },
                "VIEW" => EventMessage::View { event_key },
                e @ _ => {
                    print!("{}", e);
                    unreachable!()
                }
            };
            CallbackMessage::Event { info, event }
        }
        _ => {
            todo!();
        }
    };

    Ok(msg)
}

/// 图文消息信息，注意，如果图文数超过限制，则将只发限制内的条数
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ReplyArticle {
    /// 图文消息标题
    title: String,
    /// 图文消息描述
    description: String,
    /// 图片链接，支持JPG、PNG格式，较好的效果为大图360*200，小图200*200
    pic_url: String,
    /// 点击图文消息跳转链接
    url: String,
}
/// 被动回复用户消息
#[derive(Debug, PartialEq, Clone)]
pub enum ReplyMessage {
    /// 回复文本消息
    Text {
        info: MessageInfo,
        ///回复的消息内容（换行：在content中能够换行，微信客户端就支持换行显示）
        content: String,
    },
    /// 回复图片消息
    Image {
        info: MessageInfo,
        /// 通过素材管理中的接口上传多媒体文件，得到的id。
        media_id: String,
    },
    /// 回复语音消息
    Voice {
        info: MessageInfo,
        /// 通过素材管理中的接口上传多媒体文件，得到的id
        media_id: String,
    },
    /// 回复视频消息
    Video {
        info: MessageInfo,
        /// 通过素材管理中的接口上传多媒体文件，得到的id
        media_id: String,
        /// 视频消息的标题
        title: Option<String>,
        /// 视频消息的描述
        description: Option<String>,
    },
    /// 回复音乐消息
    Music {
        info: MessageInfo,
        /// 缩略图的媒体id，通过素材管理中的接口上传多媒体文件，得到的id
        thumb_media_id: String,
        /// 音乐标题
        title: Option<String>,
        /// 音乐描述
        description: Option<String>,
        /// 音乐链接
        music_url: Option<String>,
        /// 高质量音乐链接，WIFI环境优先使用该链接播放音乐
        hq_music_url: Option<String>,
    },
    News {
        info: MessageInfo,
        articles: Vec<ReplyArticle>,
    },
}

use std::collections::VecDeque;
struct XmlWriter {
    w: Vec<u8>,
    stack: VecDeque<String>,
}
use std::io::Write;
impl XmlWriter {
    pub fn new() -> Self {
        Self {
            w: Vec::new(),
            stack: VecDeque::new(),
        }
    }
    pub fn text(&mut self, txt: &String) -> Result<&mut Self, std::io::Error> {
        use crate::core::utils::str_ext::{SplitKeepingDelimiterExt, SplitType};

        for item in txt
            .as_str()
            .split_keeping_delimiter(|c| c == '<' || c == '>' || c == '&')
        {
            match item {
                SplitType::Match(t) => self.w.write_all(t.as_bytes())?,
                SplitType::Delimiter("<") => self.w.write_all("&lt;".as_bytes())?,
                SplitType::Delimiter(">") => self.w.write_all("&gt;".as_bytes())?,
                SplitType::Delimiter("&") => self.w.write_all("&amp;".as_bytes())?,
                SplitType::Delimiter(..) => unreachable!(),
            }
        }

        Ok(self)
    }
    pub fn begin_tag(&mut self, tag: &str) -> Result<&mut Self, std::io::Error> {
        self.w.write_all(&['<' as u8])?;
        self.w.write_all(tag.as_bytes())?;
        self.w.write_all(&['>' as u8])?;
        self.stack.push_back(tag.to_string());
        Ok(self)
    }

    pub fn end_tag(&mut self) -> Result<&mut Self, std::io::Error> {
        let tag = self.stack.pop_back().expect("end of stack");
        self.w.write_all(&['<' as u8, '/' as u8])?;
        self.w.write_all(tag.as_bytes())?;
        self.w.write_all(&['>' as u8])?;
        Ok(self)
    }

    pub fn element(&mut self, tag: &str, text: &String) -> Result<&mut Self, std::io::Error> {
        self.begin_tag(tag)?.text(text)?.end_tag()
    }

    pub fn element_optional(
        &mut self,
        tag: &str,
        text: &Option<String>,
    ) -> Result<&mut Self, std::io::Error> {
        match text {
            Some(v) => self.begin_tag(tag)?.text(v)?.end_tag(),
            None => Ok(self),
        }
    }

    pub fn to_string(&mut self) -> Result<String, std::string::FromUtf8Error> {
        let v = self.w.clone();
        let xml = String::from_utf8(v)?;
        Ok(xml)
    }
}

fn format_xml<'d, F>(msg_type: &str, info: &MessageInfo, f: F) -> Result<String, WechatError>
where
    F: Fn(&mut XmlWriter) -> Result<(), WechatError>,
{
    let mut xml = XmlWriter::new();

    xml.begin_tag("xml")?
        .element("ToUserName", &info.to_user_name)?
        .element("FromUserName", &info.from_user_name)?
        .element("CreateTime", &info.create_time.to_string())?;

    if let Some(id) = info.msg_id {
        xml.element("MsgId", &id.to_string())?;
    }
    xml.element("MsgType", &msg_type.into())?;

    f(&mut xml)?;

    xml.end_tag()?;

    let xml = xml.to_string()?;
    Ok(xml)
}

impl ReplyMessage {
    pub(crate) fn to_xml(&self) -> Result<String, WechatError> {
        let xml = match &self {
            ReplyMessage::Text { info, content } => format_xml("text", &info, |w| {
                w.element("Content", &content)?;
                Ok(())
            })?,
            ReplyMessage::Image { info, media_id } => format_xml("image", &info, |w| {
                w.begin_tag("Image")?
                    .element("MediaId", media_id)?
                    .end_tag()?;
                Ok(())
            })?,
            ReplyMessage::Voice { info, media_id } => format_xml("voice", &info, |w| {
                w.begin_tag("Voice")?
                    .element("MediaId", media_id)?
                    .end_tag()?;
                Ok(())
            })?,
            ReplyMessage::Video {
                info,
                media_id,
                title,
                description,
            } => format_xml("video", &info, |w| {
                w.begin_tag("Video")?
                    .element("MediaId", media_id)?
                    .element_optional("Title", title)?
                    .element_optional("Description", description)?
                    .end_tag()?;
                Ok(())
            })?,
            ReplyMessage::Music {
                info,
                thumb_media_id,
                title,
                description,
                music_url,
                hq_music_url,
            } => format_xml("music", &info, |w| {
                w.begin_tag("Music")?
                    .element_optional("Title", title)?
                    .element_optional("Description", description)?
                    .element_optional("MusicUrl", music_url)?
                    .element_optional("HQMusicUrl", hq_music_url)?
                    .element("ThumbMediaId", thumb_media_id)?
                    .end_tag()?;
                Ok(())
            })?,
            ReplyMessage::News { info, articles } => format_xml("news", &info, |w| {
                w.element("ArticleCount", &articles.len().to_string())?
                    .begin_tag("Articles")?;
                for article in articles {
                    w.begin_tag("Item")?
                        .element("Title", &article.title)?
                        .element("Description", &article.description)?
                        .element("PicUrl", &article.pic_url)?
                        .element("Url", &article.url)?
                        .end_tag()?;
                }
                w.end_tag()?;
                Ok(())
            })?,
        };
        Ok(xml)
    }
}

#[cfg(test)]
mod callback_message_tests {
    use super::from_xml;
    use super::*;

    #[test]
    fn test_text() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>1348831860</CreateTime>
  <MsgType><![CDATA[text]]></MsgType>
  <Content><![CDATA[this is a test]]></Content>
  <MsgId>1234567890123456</MsgId>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Text {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 1348831860,
                    msg_id: Some(1234567890123456),
                },
                content: "this is a test".into(),
                biz_msg_menu_id: None,
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_text_widhbizmsgmenuid() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
<ToUserName><![CDATA[ToUser]]></ToUserName>
<FromUserName><![CDATA[FromUser]]></FromUserName>
<CreateTime>1500000000</CreateTime>
<MsgType><![CDATA[text]]></MsgType>
<Content><![CDATA[满意]]></Content>
<MsgId>1234567890123456</MsgId>
<bizmsgmenuid>101</bizmsgmenuid>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Text {
                info: MessageInfo {
                    to_user_name: "ToUser".into(),
                    from_user_name: "FromUser".into(),
                    create_time: 1500000000,
                    msg_id: Some(1234567890123456),
                },
                content: "满意".into(),
                biz_msg_menu_id: Some("101".into()),
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_image() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>1348831860</CreateTime>
  <MsgType><![CDATA[image]]></MsgType>
  <PicUrl><![CDATA[this is a url]]></PicUrl>
  <MediaId><![CDATA[media_id]]></MediaId>
  <MsgId>1234567890123456</MsgId>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Image {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 1348831860,
                    msg_id: Some(1234567890123456),
                },
                pic_url: "this is a url".into(),
                media_id: "media_id".into(),
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_voice_widhout_recognition() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>1357290913</CreateTime>
  <MsgType><![CDATA[voice]]></MsgType>
  <MediaId><![CDATA[media_id]]></MediaId>
  <Format><![CDATA[Format]]></Format>
  <MsgId>1234567890123456</MsgId>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Voice {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 1357290913,
                    msg_id: Some(1234567890123456),
                },
                media_id: "media_id".into(),
                format: "Format".into(),
                recognition: None,
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_voice_widh_recognition() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>1357290913</CreateTime>
  <MsgType><![CDATA[voice]]></MsgType>
  <MediaId><![CDATA[media_id]]></MediaId>
  <Format><![CDATA[Format]]></Format>
  <Recognition><![CDATA[腾讯微信团队]]></Recognition>
  <MsgId>1234567890123456</MsgId>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Voice {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 1357290913,
                    msg_id: Some(1234567890123456),
                },
                media_id: "media_id".into(),
                format: "Format".into(),
                recognition: Some("腾讯微信团队".into()),
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_vedio() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>1357290913</CreateTime>
  <MsgType><![CDATA[video]]></MsgType>
  <MediaId><![CDATA[media_id]]></MediaId>
  <ThumbMediaId><![CDATA[thumb_media_id]]></ThumbMediaId>
  <MsgId>1234567890123456</MsgId>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Video {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 1357290913,
                    msg_id: Some(1234567890123456),
                },
                media_id: "media_id".into(),
                thumb_media_id: "thumb_media_id".into(),
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_shortvedio() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>1357290913</CreateTime>
  <MsgType><![CDATA[shortvideo]]></MsgType>
  <MediaId><![CDATA[media_id]]></MediaId>
  <ThumbMediaId><![CDATA[thumb_media_id]]></ThumbMediaId>
  <MsgId>1234567890123456</MsgId>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::ShortVideo {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 1357290913,
                    msg_id: Some(1234567890123456),
                },
                media_id: "media_id".into(),
                thumb_media_id: "thumb_media_id".into(),
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_location() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>1351776360</CreateTime>
  <MsgType><![CDATA[location]]></MsgType>
  <Location_X>23.134521</Location_X>
  <Location_Y>113.358803</Location_Y>
  <Scale>20</Scale>
  <Label><![CDATA[位置信息]]></Label>
  <MsgId>1234567890123456</MsgId>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Location {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 1351776360,
                    msg_id: Some(1234567890123456),
                },
                x: 23.134521,
                y: 113.358803,
                scale: 20,
                label: "位置信息".into(),
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_link() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>1351776360</CreateTime>
  <MsgType><![CDATA[link]]></MsgType>
  <Title><![CDATA[公众平台官网链接]]></Title>
  <Description><![CDATA[公众平台官网链接]]></Description>
  <Url><![CDATA[url]]></Url>
  <MsgId>1234567890123456</MsgId>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Link {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 1351776360,
                    msg_id: Some(1234567890123456),
                },
                title: "公众平台官网链接".into(),
                description: "公众平台官网链接".into(),
                url: "url".into(),
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_event_subscribe() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[FromUser]]></FromUserName>
  <CreateTime>123456789</CreateTime>
  <MsgType><![CDATA[event]]></MsgType>
  <Event><![CDATA[subscribe]]></Event>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Event {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "FromUser".into(),
                    create_time: 123456789,
                    msg_id: None,
                },
                event: EventMessage::Subscribe {}
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_event_qr_subscribe() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[FromUser]]></FromUserName>
  <CreateTime>123456789</CreateTime>
  <MsgType><![CDATA[event]]></MsgType>
  <Event><![CDATA[subscribe]]></Event>
  <EventKey><![CDATA[qrscene_123123]]></EventKey>
  <Ticket><![CDATA[TICKET]]></Ticket>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Event {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "FromUser".into(),
                    create_time: 123456789,
                    msg_id: None,
                },
                event: EventMessage::QrSubscribe {
                    event_key: "qrscene_123123".into(),
                    ticket: "TICKET".into(),
                }
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_event_scan() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[FromUser]]></FromUserName>
  <CreateTime>123456789</CreateTime>
  <MsgType><![CDATA[event]]></MsgType>
  <Event><![CDATA[SCAN]]></Event>
  <EventKey><![CDATA[SCENE_VALUE]]></EventKey>
  <Ticket><![CDATA[TICKET]]></Ticket>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Event {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "FromUser".into(),
                    create_time: 123456789,
                    msg_id: None,
                },
                event: EventMessage::Scan {
                    event_key: "SCENE_VALUE".into(),
                    ticket: "TICKET".into(),
                }
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_event_location() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[fromUser]]></FromUserName>
  <CreateTime>123456789</CreateTime>
  <MsgType><![CDATA[event]]></MsgType>
  <Event><![CDATA[LOCATION]]></Event>
  <Latitude>23.137466</Latitude>
  <Longitude>113.352425</Longitude>
  <Precision>119.385040</Precision>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Event {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "fromUser".into(),
                    create_time: 123456789,
                    msg_id: None,
                },
                event: EventMessage::Location {
                    lat: 23.137466,
                    lng: 113.352425,
                    precesion: 119.385040,
                }
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_event_click() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[FromUser]]></FromUserName>
  <CreateTime>123456789</CreateTime>
  <MsgType><![CDATA[event]]></MsgType>
  <Event><![CDATA[CLICK]]></Event>
  <EventKey><![CDATA[EVENTKEY]]></EventKey>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Event {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "FromUser".into(),
                    create_time: 123456789,
                    msg_id: None,
                },
                event: EventMessage::Click {
                    event_key: "EVENTKEY".into(),
                }
            },
            msg
        );
        Ok(())
    }

    #[test]
    fn test_event_view() -> Result<(), WechatError> {
        let msg = from_xml(
            r#"<xml>
  <ToUserName><![CDATA[toUser]]></ToUserName>
  <FromUserName><![CDATA[FromUser]]></FromUserName>
  <CreateTime>123456789</CreateTime>
  <MsgType><![CDATA[event]]></MsgType>
  <Event><![CDATA[VIEW]]></Event>
  <EventKey><![CDATA[www.qq.com]]></EventKey>
</xml>"#,
        )?;
        assert_eq!(
            CallbackMessage::Event {
                info: MessageInfo {
                    to_user_name: "toUser".into(),
                    from_user_name: "FromUser".into(),
                    create_time: 123456789,
                    msg_id: None,
                },
                event: EventMessage::View {
                    event_key: "www.qq.com".into(),
                }
            },
            msg
        );
        Ok(())
    }
}

#[cfg(test)]
mod reply_message_testes {
    use super::*;

    fn get_info() -> MessageInfo {
        MessageInfo {
            to_user_name: "to user".into(),
            from_user_name: "my_id".into(),
            create_time: 123456789876,
            msg_id: None,
        }
    }

    #[test]
    fn test_text() -> Result<(), WechatError> {
        let msg = ReplyMessage::Text {
            info: get_info(),
            content: "reply message".into(),
        };
        assert_eq!(
            r#"<xml><ToUserName>to user</ToUserName><FromUserName>my_id</FromUserName><CreateTime>123456789876</CreateTime><MsgType>text</MsgType><Content>reply message</Content></xml>"#,
            msg.to_xml()?
        );
        Ok(())
    }

    #[test]
    fn test_image() -> Result<(), WechatError> {
        let msg = ReplyMessage::Image {
            info: get_info(),
            media_id: "media_id_test".into(),
        };
        assert_eq!(
            r#"<xml><ToUserName>to user</ToUserName><FromUserName>my_id</FromUserName><CreateTime>123456789876</CreateTime><MsgType>image</MsgType><Image><MediaId>media_id_test</MediaId></Image></xml>"#,
            msg.to_xml()?
        );
        Ok(())
    }

    #[test]
    fn test_voice() -> Result<(), WechatError> {
        let msg = ReplyMessage::Voice {
            info: get_info(),
            media_id: "media_id_test".into(),
        };
        assert_eq!(
            r#"<xml><ToUserName>to user</ToUserName><FromUserName>my_id</FromUserName><CreateTime>123456789876</CreateTime><MsgType>voice</MsgType><Voice><MediaId>media_id_test</MediaId></Voice></xml>"#,
            msg.to_xml()?
        );
        Ok(())
    }

    #[test]
    fn test_vedio() -> Result<(), WechatError> {
        let msg = ReplyMessage::Video {
            info: get_info(),
            media_id: "media_id_test".into(),
            title: Some("test vedio".into()),
            description: Some("测试描述".into()),
        };
        assert_eq!(
            r#"<xml><ToUserName>to user</ToUserName><FromUserName>my_id</FromUserName><CreateTime>123456789876</CreateTime><MsgType>video</MsgType><Video><MediaId>media_id_test</MediaId><Title>test vedio</Title><Description>测试描述</Description></Video></xml>"#,
            msg.to_xml()?
        );
        Ok(())
    }

    #[test]
    fn test_music() -> Result<(), WechatError> {
        let msg = ReplyMessage::Music {
            info: get_info(),
            title: Some("test vedio".into()),
            description: Some("测试描述 1>2 && 2<3".into()),
            music_url: Some("music url..".into()),
            hq_music_url: None,
            thumb_media_id: "thumb media id".into(),
        };
        assert_eq!(
            r#"<xml><ToUserName>to user</ToUserName><FromUserName>my_id</FromUserName><CreateTime>123456789876</CreateTime><MsgType>music</MsgType><Music><Title>test vedio</Title><Description>测试描述 1&gt;2 &amp;&amp; 2&lt;3</Description><MusicUrl>music url..</MusicUrl><ThumbMediaId>thumb media id</ThumbMediaId></Music></xml>"#,
            msg.to_xml()?
        );
        Ok(())
    }

    #[test]
    fn test_news() -> Result<(), WechatError> {
        let article1 = ReplyArticle {
            title: "article title 1".into(),
            description: "description 1".into(),
            pic_url: "pic url 1".into(),
            url: "url 1".into(),
        };

        let article2 = ReplyArticle {
            title: "article title 2".into(),
            description: "description 2".into(),
            pic_url: "pic url 2".into(),
            url: "url 2".into(),
        };

        let msg = ReplyMessage::News {
            info: get_info(),
            articles: vec![article1, article2],
        };

        assert_eq!(
            r#"<xml><ToUserName>to user</ToUserName><FromUserName>my_id</FromUserName><CreateTime>123456789876</CreateTime><MsgType>news</MsgType><ArticleCount>2</ArticleCount><Articles><Item><Title>article title 1</Title><Description>description 1</Description><PicUrl>pic url 1</PicUrl><Url>url 1</Url></Item><Item><Title>article title 2</Title><Description>description 2</Description><PicUrl>pic url 2</PicUrl><Url>url 2</Url></Item></Articles></xml>"#,
            msg.to_xml()?
        );
        Ok(())
    }

    //
}

/// 微信加密解密
pub mod crypt {

    use super::*;

    use std::io::Cursor;

    use base64;
    use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
    use openssl::symm;
    use rand::thread_rng;
    use rand::Rng;
    use serde::Deserialize;

    pub fn decode_aes_key(key: &String) -> Result<Option<Vec<u8>>, WechatEncryptError> {
        if key.is_empty() {
            return Ok(None);
        }
        let config = base64::STANDARD.decode_allow_trailing_bits(true);
        let key = base64::decode_config(key, config)?;
        Ok(Some(key))
    }

    #[derive(Deserialize, Debug)]
    pub struct VerifyInfo {
        pub signature: String,
        pub timestamp: i64,
        pub nonce: String,
        /// 消息签名
        pub msg_signature: Option<String>,
        /// 消息加密方法, aes
        pub encrypt_type: Option<String>,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct PrpCrypto {
        key: Vec<u8>,
    }

    impl PrpCrypto {
        pub fn new(key: &[u8]) -> PrpCrypto {
            PrpCrypto { key: key.to_vec() }
        }

        fn get_random_string() -> String {
            if cfg!(test) {
                "1234567890123456".to_owned()
            } else {
                use rand::distributions::Alphanumeric;
                thread_rng().sample_iter(&Alphanumeric).take(16).collect()
            }
        }

        pub fn encrypt(&self, plaintext: &str, app_id: &str) -> Result<String, WechatEncryptError> {
            let mut wtr = PrpCrypto::get_random_string().into_bytes();
            wtr.write_u32::<NativeEndian>((plaintext.len() as u32).to_be())?;
            wtr.extend(plaintext.bytes());
            wtr.extend(app_id.bytes());

            let cipher = openssl::symm::Cipher::aes_256_cbc();
            let encrypted = symm::encrypt(cipher, &self.key, Some(&self.key[..16]), &wtr)?;
            let b64encoded = base64::encode(&encrypted);
            Ok(b64encoded)
        }

        pub fn decrypt(&self, ciphertext: &str, _id: &str) -> Result<String, WechatEncryptError> {
            let b64decoded = base64::decode(ciphertext)?;
            let cipher = openssl::symm::Cipher::aes_256_cbc();
            let text = symm::decrypt(cipher, &self.key, Some(&self.key[..16]), &b64decoded)?;
            let mut rdr = Cursor::new(text[16..20].to_vec());
            let content_length = u32::from_be(rdr.read_u32::<NativeEndian>()?) as usize;
            let content = &text[20..content_length + 20];
            let from_id = &text[content_length + 20..];
            if from_id != _id.as_bytes() {
                return Err(WechatEncryptError::InvalidAppId);
            }
            let content_string = String::from_utf8(content.to_vec())?;
            Ok(content_string)
        }
    }

    pub trait WeChatCrypto {
       fn get_signature( &self, timestamp: i64, nonce: &str, encrypted: &str,) -> Result<String, WechatEncryptError>;

        fn verify_base(&self, verify_info: &VerifyInfo) -> Result<(), WechatEncryptError>;
        fn verify_message( &self, verify_info: &VerifyInfo, msg: &String,) -> Result<(), WechatEncryptError>;

        fn decrypt_echostr( &self, verify_info: &VerifyInfo, echo_str: &String,) -> Result<String, WechatEncryptError>; 
        fn encrypt_message( &self, msg: &str, timestamp: i64, nonce: &str,) -> Result<String, WechatEncryptError>; 
        fn decrypt_message( &self, verify_info: &VerifyInfo, xml: &str,) -> Result<String, WechatEncryptError>;
    }

    use crate::core::WechatConfig;

    impl WeChatCrypto for WechatConfig {
        fn get_signature( &self, timestamp: i64, nonce: &str, encrypted: &str,) -> Result<String, WechatEncryptError> {
            let mut data = vec![
                self.token_string(),
                timestamp.to_string(),
                nonce.to_owned(),
                encrypted.to_owned(),
            ];
            data.sort();
            let data_str = data.join("");
            let mut hasher = openssl::sha::Sha1::new();
            hasher.update(data_str.as_bytes());
            let signature = hasher.finish();
            Ok(hex::encode(signature))
        }

        /// 校验基础的消息头
        fn verify_base(&self, verify_info: &VerifyInfo) -> Result<(), WechatEncryptError> {
            let real_signature =
                self.get_signature(verify_info.timestamp, &verify_info.nonce, &"")?;

            if verify_info.signature != real_signature {
                info!(
                    "消息头签名校验失败: acture_signature:{}, {:?}",
                    real_signature, verify_info,
                );
                return Err(WechatEncryptError::InvalidSignature(
                    "头签名校验失败".into(),
                ));
            }

            Ok(())
        }

        /// 校验消息内容
        fn verify_message( &self, verify_info: &VerifyInfo, msg: &String,) -> Result<(), WechatEncryptError> {
            match &verify_info.msg_signature {
                None => Ok(()),
                Some(signature) => {
                    let real_signature =
                        self.get_signature(verify_info.timestamp, &verify_info.nonce, msg)?;

                    if signature != &real_signature {
                        info!(
                            "消息内容签名校验失败: verify:{:?}, acture_signature:{}, {}",
                            verify_info, real_signature, msg,
                        );
                        return Err(WechatEncryptError::InvalidSignature(
                            "消息签名校验失败".into(),
                        ));
                    }
                    Ok(())
                }
            }
        }

        fn decrypt_echostr( &self, verify_info: &VerifyInfo, echo_str: &String,) -> Result<String, WechatEncryptError> {
            // 校验消息头
            self.verify_base(verify_info)?;
            // 明文消息
            if let None = verify_info.encrypt_type {
                return Ok(echo_str.into());
            }
            self.verify_message(verify_info, echo_str)?;
            // 尝试解密
            let key =         match &self.key {
                Some(key) => key,
                None => return Ok(echo_str.clone()),
            };

            let prp = PrpCrypto::new(key);
            let msg = prp.decrypt(echo_str, &self.app_id);
            match msg {
                Ok(msg) => Ok(msg),
                Err(e) => Err(WechatEncryptError::InvalidSignature(format!(
                    "无法解密: {:?}",
                    e
                ))),
            }
        }

        fn encrypt_message(
            &self,
            msg: &str,
            timestamp: i64,
            nonce: &str,
        ) -> Result<String, WechatEncryptError> {
            let key = match &self.key {
                Some(key) => key,
                None => return Err(WechatEncryptError::InvalidConfig),
            };
            let prp = PrpCrypto::new(&key);
            let encrypted_msg = prp.encrypt(msg, &self.app_id)?;
            let signature = self.get_signature(timestamp, nonce, &encrypted_msg)?;
            let msg = XmlWriter::new()
                .begin_tag("xml")?
                .element("Encrypt", &encrypted_msg)?
                .element("MsgSignature", &signature)?
                .element("TimeStamp", &timestamp.to_string())?
                .element("Nonce", &nonce.to_string())?
                .end_tag()?
                .to_string()?;
            Ok(msg)
        }

        fn decrypt_message(
            &self,
            verify_info: &VerifyInfo,
            xml: &str,
        ) -> Result<String, WechatEncryptError> {
            self.verify_base(verify_info)?;

            if let None = verify_info.encrypt_type {
                return Ok(xml.into());
            }

            use sxd_document::parser;
            use sxd_xpath::evaluate_xpath;

            let package = parser::parse(xml).unwrap();
            let doc = package.as_document();

            let encrypted_msg = evaluate_xpath(&doc, "/xml/Encrypt")?.string();
            self.verify_message(verify_info, &encrypted_msg)?;
            match &self.key {
                Some(key) => {
            let prp = PrpCrypto::new(key);
            let msg = prp.decrypt(&encrypted_msg, &self.app_id)?;
            Ok(msg)
                },
                None => {
                    Ok(encrypted_msg)
                }
            }
        }
    }
}
#[cfg(test)]
mod crypt_tests {
    use super::crypt::*;
    use super::*;
    use crate::core::*;
    use base64;

    #[test]
    fn test_prpcrypto_encrypt() -> Result<(), WechatEncryptError> {
        let encoding_aes_key = "kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aQ=";
        let key = base64::decode(encoding_aes_key)?;
        let prp = PrpCrypto::new(&key);
        let encrypted = prp.encrypt("test", "rust")?;
        assert_eq!("9s4gMv99m88kKTh/H8IdkNiFGeG9pd7vNWl50fGRWXY=", &encrypted);
        Ok(())
    }

    #[test]
    fn test_prpcrypto_decrypt() {
        let encoding_aes_key = "kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aQ=";
        let key = base64::decode(encoding_aes_key).unwrap();
        let prp = PrpCrypto::new(&key);
        let decrypted = prp
            .decrypt("9s4gMv99m88kKTh/H8IdkNiFGeG9pd7vNWl50fGRWXY=", "rust")
            .unwrap();
        assert_eq!("test", &decrypted);
    }

    #[test]
    fn test_get_signature() {
        let config = WechatConfig::new(
            Some("test".into()),
            WechatConfig::decode_aes_key(&"kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aQ=".into()).unwrap(),
            "test".into(),
            None,
        );
        let signature = config.get_signature(123456i64, "test", "rust").unwrap();
        assert_eq!("d6056f2bb3ad3e30f4afa5ef90cc9ddcdc7b7b27", &signature);
    }

    #[test]
    fn test_check_signature_should_ok() {
        let signature = "97f44b51ccbee5533bf61e753557d165ea0f4566";
        let timestamp = 1411443780;
        let nonce = "437374425";
        let echo_str = "4ByGGj+sVCYcvGeQYhaKIk1o0pQRNbRjxybjTGblXrBaXlTXeOo1+bXFXDQQb1o6co6Yh9Bv41n7hOchLF6p+Q==";

        let config = WechatConfig::new(
            Some("123456".into()),
            WechatConfig::decode_aes_key(&"kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aQ=".into()).unwrap(),
            "wx49f0ab532d5d035a".into(),
            None,
        );
        let verify_info = VerifyInfo {
            signature: signature.into(),
            timestamp,
            nonce: nonce.into(),
            encrypt_type: Some("aes".into()),
            msg_signature: None,
        };
        match config.decrypt_echostr(&verify_info, &echo_str.into()) {
            Ok(echostr) => assert_eq!("5927782489442352469".to_string(), echostr),
            Err(e) => panic!(format!("Check signature failed:{:?}", e)),
        }
    }

    #[test]
    fn test_check_signature_should_fail() {
        let signature = "dd6b9c95b495b3f7e2901bfbc76c664930ffdb96";
        let timestamp = 1411443780;
        let nonce = "437374424";
        let echo_str = "4ByGGj+sVCYcvGeQYhaKIk1o0pQRNbRjxybjTGblXrBaXlTXeOo1+bXFXDQQb1o6co6Yh9Bv41n7hOchLF6p+Q==";
        let verify_info = VerifyInfo {
            signature: signature.into(),
            timestamp,
            nonce: nonce.into(),
            encrypt_type: Some("aes".into()),
            msg_signature: None,
        };
        let config = WechatConfig::new(
            Some("123456".into()),
            WechatConfig::decode_aes_key(&"kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aQ=".into()).unwrap(),
            "wx49f0ab532d5d035a".into(),
            None,
        );
        match config.decrypt_echostr(&verify_info, &echo_str.into()) {
            Ok(_) => panic!("Check signature should failed"),
            Err(_) => {}
        }
    }

    #[test]
    fn test_encrypt_message() {
        let timestamp = 1411525903;
        let nonce = "461056294";
        let msg = "<xml>\n\
            <MsgType><![CDATA[text]]></MsgType>\n\
            <Content><![CDATA[test]]></Content>\n\
            <FromUserName><![CDATA[wx49f0ab532d5d035a]]></FromUserName>\n\
            <ToUserName><![CDATA[messense]]></ToUserName>\n\
            <AgentID>1</AgentID>\n\
            <CreateTime>1411525903</CreateTime>\n\
            </xml>";
        let expected = "<xml>\
            <Encrypt>9s4gMv99m88kKTh/H8IdkOiMg6bisoy3ypwy9H4hvSPe9nsGaqyw5hhSjdYbcrKk+j3nba4HMOTzHrluLBYqxgNcBqGsL8GqxlhZgURnAtObvesEl5nZ+uBE8bviY0LWke8Zy9V/QYKxNV2FqllNXcfmstttyIkMKCCmVbCFM2JTF5wY0nFhHZSjPUL2Q1qvSUCUld+/WIXrx0oyKQmpB6o8NRrrNrsDf03oxI1p9FxUgMnwKKZeOA/uu+2IEvEBtb7muXsVbwbgX05UPPJvFurDXafG0RQyPR+mf1nDnAtQmmNOuiR5MIkdQ39xn1vWwi1O5oazPoQJz0nTYjxxEE8kv3kFxtAGVRe3ypD3WeK2XeFYFMNMpatF9XiKzHo3</Encrypt>\
            <MsgSignature>407518b7649e86ef23978113f92d27afa9296533</MsgSignature>\
            <TimeStamp>1411525903</TimeStamp>\
            <Nonce>461056294</Nonce>\
            </xml>";
        let config = WechatConfig::new(
            Some("123456".into()),
            WechatConfig::decode_aes_key(&"kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aQ=".into()).unwrap(),
            "wx49f0ab532d5d035a".into(),
            None,
        );
        let encrypted = config.encrypt_message(msg, timestamp, nonce).unwrap();
        assert_eq!(expected, &encrypted);
    }

    #[test]
    fn test_decrypt_message() {
        let xml = "<xml><ToUserName><![CDATA[wx49f0ab532d5d035a]]></ToUserName>\n\
            <Encrypt><![CDATA[RgqEoJj5A4EMYlLvWO1F86ioRjZfaex/gePD0gOXTxpsq5Yj4GNglrBb8I2BAJVODGajiFnXBu7mCPatfjsu6IHCrsTyeDXzF6Bv283dGymzxh6ydJRvZsryDyZbLTE7rhnus50qGPMfp2wASFlzEgMW9z1ef/RD8XzaFYgm7iTdaXpXaG4+BiYyolBug/gYNx410cvkKR2/nPwBiT+P4hIiOAQqGp/TywZBtDh1yCF2KOd0gpiMZ5jSw3e29mTvmUHzkVQiMS6td7vXUaWOMZnYZlF3So2SjHnwh4jYFxdgpkHHqIrH/54SNdshoQgWYEvccTKe7FS709/5t6NMxuGhcUGAPOQipvWTT4dShyqio7mlsl5noTrb++x6En749zCpQVhDpbV6GDnTbcX2e8K9QaNWHp91eBdCRxthuL0=]]></Encrypt>\n\
            <AgentID><![CDATA[1]]></AgentID>\n\
            </xml>";
        let expected = "<xml><ToUserName><![CDATA[wx49f0ab532d5d035a]]></ToUserName>\n\
            <FromUserName><![CDATA[messense]]></FromUserName>\n\
            <CreateTime>1411525903</CreateTime>\n\
            <MsgType><![CDATA[text]]></MsgType>\n\
            <Content><![CDATA[test]]></Content>\n\
            <MsgId>4363689963896700987</MsgId>\n\
            <AgentID>1</AgentID>\n\
            </xml>";

        let signature = "6c729cc5480fab0c2e594b7e25a93d2dbef6ab97";
        let timestamp = 1411525903;
        let nonce = "461056294";
        let config = WechatConfig::new(
            Some("123456".into()),
            WechatConfig::decode_aes_key(&"kWxPEV2UEDyxWpmPdKC3F4dgPDmOvfKX1HGnEUDS1aQ=".into()).unwrap(),
            "wx49f0ab532d5d035a".into(),
            None,
        );
        let verify_info = VerifyInfo {
            signature: signature.into(),
            timestamp,
            nonce: nonce.into(),
            msg_signature: Some("74d92dfeb87ba7c714f89d98870ae5eb62dff26d".into()),
            encrypt_type: Some("aes".into()),
        };
        let decrypted = config.decrypt_message(&verify_info, xml).unwrap();
        assert_eq!(expected, &decrypted);
    }

    #[test]
    fn test_decrypt_message2() {
        let xml = r#"<xml>
        <ToUserName><![CDATA[gh_f91a47ec7ff6]]></ToUserName>
        <FromUserName><![CDATA[oseZYwXU64cWTJuTV4UkS-DTu9OQ]]></FromUserName>
        <CreateTime>1592558813</CreateTime>
        <MsgType><![CDATA[text]]></MsgType>
        <Content><![CDATA[好的]]></Content>
        <MsgId>22799962246505739</MsgId>
        <Encrypt><![CDATA[YsZjOA0RLvxvjZ8Xq38yC2YZgxw20MS/UCS13eiWaznQawh8JHGyonUKFLKC9cSgpxDpP9IHQ5+Vl9exTBSMgCzI19P1z0YpByB5rLfHMQWsyvm/H5uwH16lf2BgooZZRoEyzTDLXQFqjwiUSP7Iw8IzdtMp1Ux3f9glW5D/I5H3sGmxbmxf0N/2I5DKKWAlQZSfEnzouKcpyD9DJeY8FfKcQAlJFs/FKGs7g6UdXlxHwmgK3+ZOf7+FL8nFVOQzVpCLuOfRJnMQ//+Bp8aXoTbLiaW6haYuKf7CpPihQJ9/XFTgirBRB2V3jNFisVzwL9XeJ6r/H8Pt8GyGeQ6Hdpl4RVJY4gOTYvZpNvcz0WsKtJkh04tC7zj6tO8/cR4wsJxzTvDpMtBSpukNcFuR7BQtKKTlAkYulnoj8dAfHFc=]]></Encrypt>
    </xml>"#;
        let expected = "<xml><ToUserName><![CDATA[gh_f91a47ec7ff6]]></ToUserName>\n<FromUserName><![CDATA[oseZYwXU64cWTJuTV4UkS-DTu9OQ]]></FromUserName>\n<CreateTime>1592558813</CreateTime>\n<MsgType><![CDATA[text]]></MsgType>\n<Content><![CDATA[好的]]></Content>\n<MsgId>22799962246505739</MsgId>\n</xml>";
        let signature = "3af8f544dbc8c4e096c492984cbd1175c86a95c1";
        let msg_signature = "060c0da85ba7c4c2bf2a69716debec5858145826";
        let timestamp = 1592558813;
        let nonce = "1681763772";
        // let encrypt_type=aes
        let verify_info = VerifyInfo {
            signature: signature.into(),
            timestamp,
            nonce: nonce.into(),
            msg_signature: Some(msg_signature.into()),
            encrypt_type: Some("aes".into()),
        };
        let config = WechatConfig::new(
            Some("testtoken123456".into()),
            WechatConfig::decode_aes_key(&"znpfGFxELvUSxh0Gx4rJenvVQRrAhdTsioG08XR4z3S=".into()).unwrap(),
            "wx11853b05910e1b6b".into(),
            None,
        );
        // openid=
        let decrypted = config.decrypt_message(&verify_info, xml).unwrap();
        assert_eq!(expected, &decrypted);
    }
}
