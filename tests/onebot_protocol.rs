use rechat_sender::adapters::onebot::protocol as protocol_mod;
use rechat_sender::adapters::onebot::protocol::{
    ActionRequest, ActionResponse, MessageSegment, OneBotEvent,
};
use std::collections::HashMap;

#[test]
fn test_message_segment_text() {
    let seg = MessageSegment::text("Hello World");
    assert_eq!(seg.seg_type, "text");
    assert_eq!(seg.data.get("text").unwrap(), "Hello World");
}

#[test]
fn test_message_segment_image() {
    let seg = MessageSegment::image("https://example.com/pic.jpg");
    assert_eq!(seg.seg_type, "image");
    assert_eq!(seg.data.get("file").unwrap(), "https://example.com/pic.jpg");
}

#[test]
fn test_message_segment_at() {
    let seg = MessageSegment::at("12345678");
    assert_eq!(seg.seg_type, "at");
    assert_eq!(seg.data.get("qq").unwrap(), "12345678");
}

#[test]
fn test_message_segment_reply() {
    let seg = MessageSegment::reply(987654321);
    assert_eq!(seg.seg_type, "reply");
    assert_eq!(seg.data.get("id").unwrap(), "987654321");
}

#[test]
fn test_to_plain_text_text() {
    let seg = MessageSegment::text("你好");
    assert_eq!(seg.to_plain_text(), "你好");
}

#[test]
fn test_to_plain_text_at() {
    let seg = MessageSegment::at("12345");
    assert_eq!(seg.to_plain_text(), "@12345");
}

#[test]
fn test_to_plain_text_reply() {
    let seg = MessageSegment::reply(123456);
    assert_eq!(seg.to_plain_text(), "[Reply]");
}

#[test]
fn test_to_plain_text_unknown_type() {
    let mut data = HashMap::new();
    data.insert("url".into(), "https://a.b".into());
    let seg = MessageSegment {
        seg_type: "unknown_type".into(),
        data,
    };
    assert_eq!(seg.to_plain_text(), "[unknown_type]");
}

#[test]
fn test_segments_to_text() {
    let segments = vec![
        MessageSegment::text("Hello "),
        MessageSegment::at("12345"),
        MessageSegment::text(" welcome"),
    ];
    assert_eq!(
        MessageSegment::segments_to_text(&segments),
        "Hello @12345 welcome"
    );
}

#[test]
fn test_segments_to_text_empty() {
    assert_eq!(MessageSegment::segments_to_text(&[]), "");
}

#[test]
fn test_build_send_private_msg() {
    let seg = MessageSegment::text("Hi");
    let action = protocol_mod::build_send_msg_action("private", 123456789, &[seg], Some("echo123"));
    assert_eq!(action.action, "send_private_msg");
    assert_eq!(action.params["user_id"], 123456789);
    assert_eq!(action.echo.as_deref(), Some("echo123"));
}

#[test]
fn test_build_send_group_msg() {
    let seg = MessageSegment::text("Group message");
    let action = protocol_mod::build_send_msg_action("group", 987654321, &[seg], None);
    assert_eq!(action.action, "send_group_msg");
    assert_eq!(action.params["group_id"], 987654321);
    assert!(action.echo.is_none());
}

#[test]
fn test_build_delete_msg() {
    let action = protocol_mod::build_delete_msg_action(42);
    assert_eq!(action.action, "delete_msg");
    assert_eq!(action.params["message_id"], 42);
    assert!(action.echo.is_none());
}

#[test]
fn test_action_request_serialization() {
    let action = ActionRequest {
        action: "send_private_msg".into(),
        params: serde_json::json!({"user_id": 123, "message": "Hi"}),
        echo: Some("trace1".into()),
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("send_private_msg"));
    assert!(json.contains("trace1"));
    // echo should be present
    assert!(json.contains("echo"));
}

#[test]
fn test_action_request_serialization_no_echo() {
    let action = ActionRequest {
        action: "get_version".into(),
        params: serde_json::json!({}),
        echo: None,
    };
    let json = serde_json::to_string(&action).unwrap();
    assert!(!json.contains("echo"));
}

#[test]
fn test_action_response_deserialization_ok() {
    let json = r#"{"status":"ok","retcode":0,"data":{},"echo":"trace1"}"#;
    let resp: ActionResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.status, "ok");
    assert_eq!(resp.retcode, 0);
    assert_eq!(resp.echo.as_deref(), Some("trace1"));
}

#[test]
fn test_action_response_deserialization_no_echo() {
    let json = r#"{"status":"failed","retcode":1404,"data":{"msg":"not found"}}"#;
    let resp: ActionResponse = serde_json::from_str(json).unwrap();
    assert_eq!(resp.status, "failed");
    assert_eq!(resp.retcode, 1404);
    assert!(resp.echo.is_none());
}

#[test]
fn test_onebot_event_deserialize_group_message() {
    let json = r#"{
        "time": 1715000000,
        "self_id": 123456789,
        "post_type": "message",
        "message_type": "group",
        "sub_type": "normal",
        "message_id": 789,
        "user_id": 111222333,
        "group_id": 999888777,
        "message": [{"type":"text","data":{"text":"Hello"}}],
        "raw_message": "Hello",
        "sender": {"user_id":111222333,"nickname":"User","card":"DisplayName","role":"member"}
    }"#;
    let event: OneBotEvent = serde_json::from_str(json).unwrap();
    match event {
        OneBotEvent::Message(msg) => {
            assert_eq!(msg.message_type, "group");
            assert_eq!(msg.user_id, 111222333);
            assert_eq!(msg.group_id, Some(999888777));
            assert_eq!(msg.message.len(), 1);
            assert_eq!(msg.raw_message, "Hello");
            let sender = msg.sender.unwrap();
            assert_eq!(sender.nickname, "User");
            assert_eq!(sender.card.as_deref(), Some("DisplayName"));
        }
        _ => panic!("Expected Message event"),
    }
}

#[test]
fn test_onebot_event_deserialize_private_message() {
    let json = r#"{
        "time": 1715000000,
        "self_id": 100000,
        "post_type": "message",
        "message_type": "private",
        "sub_type": "friend",
        "message_id": 1,
        "user_id": 200000,
        "message": [{"type":"text","data":{"text":"PM"}}],
        "raw_message": "PM",
        "sender": {"user_id":200000,"nickname":"Friend"}
    }"#;
    let event: OneBotEvent = serde_json::from_str(json).unwrap();
    match event {
        OneBotEvent::Message(msg) => {
            assert_eq!(msg.message_type, "private");
            assert!(msg.group_id.is_none());
        }
        _ => panic!("Expected Message event"),
    }
}

#[test]
fn test_onebot_event_deserialize_notice() {
    let json = r#"{
        "time": 1715000000,
        "self_id": 123456789,
        "post_type": "notice",
        "notice_type": "group_increase",
        "user_id": 111222333,
        "group_id": 999888777
    }"#;
    let event: OneBotEvent = serde_json::from_str(json).unwrap();
    match event {
        OneBotEvent::Notice(notice) => {
            assert_eq!(notice.notice_type, "group_increase");
            assert_eq!(notice.group_id, Some(999888777));
        }
        _ => panic!("Expected Notice event"),
    }
}

#[test]
fn test_onebot_event_deserialize_meta() {
    let json = r#"{
        "time": 1715000000,
        "self_id": 123456789,
        "post_type": "meta_event",
        "meta_event_type": "heartbeat",
        "status": {}
    }"#;
    let event: OneBotEvent = serde_json::from_str(json).unwrap();
    match event {
        OneBotEvent::Meta(meta) => {
            assert_eq!(meta.meta_event_type, "heartbeat");
        }
        _ => panic!("Expected Meta event"),
    }
}
