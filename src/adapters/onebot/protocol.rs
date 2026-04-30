use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ========== 动作 (ReChat → NapCat) ==========

#[derive(Debug, Serialize)]
pub struct ActionRequest {
    pub action: String,
    pub params: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActionResponse {
    pub status: String,
    pub retcode: i64,
    pub data: serde_json::Value,
    #[serde(default)]
    pub echo: Option<String>,
}

// ========== 事件 (NapCat → ReChat) ==========

#[derive(Debug, Deserialize)]
#[serde(tag = "post_type")]
pub enum OneBotEvent {
    #[serde(rename = "message")]
    Message(MessageEvent),
    #[serde(rename = "notice")]
    Notice(NoticeEvent),
    #[serde(rename = "request")]
    Request(RequestEvent),
    #[serde(rename = "meta_event")]
    Meta(MetaEvent),
}

// ========== 消息事件 ==========

#[derive(Debug, Deserialize)]
pub struct MessageEvent {
    pub time: i64,
    pub self_id: i64,
    pub message_type: String,
    pub sub_type: String,
    pub message_id: i64,
    pub user_id: i64,
    pub message: Vec<MessageSegment>,
    pub raw_message: String,
    pub sender: Option<Sender>,
    #[serde(default)]
    pub group_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Sender {
    pub user_id: i64,
    pub nickname: String,
    #[serde(default)]
    pub card: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
}

// ========== 消息段 ==========

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageSegment {
    #[serde(rename = "type")]
    pub seg_type: String,
    pub data: HashMap<String, String>,
}

impl MessageSegment {
    pub fn text(content: &str) -> Self {
        let mut data = HashMap::new();
        data.insert("text".into(), content.into());
        Self {
            seg_type: "text".into(),
            data,
        }
    }

    pub fn image(file: &str) -> Self {
        let mut data = HashMap::new();
        data.insert("file".into(), file.into());
        Self {
            seg_type: "image".into(),
            data,
        }
    }

    pub fn at(qq: &str) -> Self {
        let mut data = HashMap::new();
        data.insert("qq".into(), qq.into());
        Self {
            seg_type: "at".into(),
            data,
        }
    }

    pub fn reply(message_id: i64) -> Self {
        let mut data = HashMap::new();
        data.insert("id".into(), message_id.to_string());
        Self {
            seg_type: "reply".into(),
            data,
        }
    }

    /// 将消息段转换为纯文本（用于内部 Message 的 content 字段）
    pub fn to_plain_text(&self) -> String {
        match self.seg_type.as_str() {
            "text" => self.data.get("text").cloned().unwrap_or_default(),
            "image" => format!(
                "[Image: {}]",
                self.data.get("url").unwrap_or(&"<unknown>".into())
            ),
            "at" => {
                let qq = self.data.get("qq").map(|s| s.as_str()).unwrap_or("unknown");
                format!("@{}", qq)
            }
            "reply" => "[Reply]".into(),
            "face" => "[Face]".into(),
            "record" => "[Record]".into(),
            "video" => "[Video]".into(),
            _ => format!("[{}]", self.seg_type),
        }
    }

    /// 将消息段数组拼接为纯文本（用于内部 Message）
    pub fn segments_to_text(segments: &[MessageSegment]) -> String {
        segments
            .iter()
            .map(|s| s.to_plain_text())
            .collect::<Vec<_>>()
            .join("")
    }
}

// ========== 通知事件 ==========

#[derive(Debug, Deserialize)]
pub struct NoticeEvent {
    pub time: i64,
    pub self_id: i64,
    pub notice_type: String,
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub group_id: Option<i64>,
    #[serde(default)]
    pub operator_id: Option<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ========== 请求事件 ==========

#[derive(Debug, Deserialize)]
pub struct RequestEvent {
    pub time: i64,
    pub self_id: i64,
    pub request_type: String,
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub group_id: Option<i64>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub flag: Option<String>,
}

// ========== 元事件 ==========

#[derive(Debug, Deserialize)]
pub struct MetaEvent {
    pub time: i64,
    pub self_id: i64,
    pub meta_event_type: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ========== 消息构造辅助 ==========

/// 根据消息类型和会话 ID 构造 OneBot 动作请求
pub fn build_send_msg_action(
    message_type: &str,
    target_id: i64,
    segments: &[MessageSegment],
    echo: Option<&str>,
) -> ActionRequest {
    let (action, params) = match message_type {
        "private" => {
            let params = serde_json::json!({
                "user_id": target_id,
                "message": segments,
                "auto_escape": false,
            });
            ("send_private_msg".into(), params)
        }
        _ => {
            let params = serde_json::json!({
                "group_id": target_id,
                "message": segments,
                "auto_escape": false,
            });
            ("send_group_msg".into(), params)
        }
    };

    ActionRequest {
        action,
        params,
        echo: echo.map(|s| s.into()),
    }
}

/// 构造撤回消息动作
pub fn build_delete_msg_action(message_id: i64) -> ActionRequest {
    ActionRequest {
        action: "delete_msg".into(),
        params: serde_json::json!({"message_id": message_id}),
        echo: None,
    }
}
