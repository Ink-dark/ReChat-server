use rechat_sender::core::broadcaster::{
    BroadcastMessage, BroadcastMessageData, ClientSession, MessageBroadcaster,
};
use std::collections::HashSet;
use tokio::sync::mpsc;

fn make_broadcast_msg(platform: &str, conversation: &str, content: &str) -> BroadcastMessage {
    BroadcastMessage {
        msg_type: "new_message".into(),
        data: BroadcastMessageData {
            id: uuid::Uuid::new_v4().to_string(),
            platform: platform.into(),
            conversation: conversation.into(),
            conversation_name: None,
            content: content.into(),
            message_type: "Text".into(),
            sender: None,
            created_at: 0,
        },
    }
}

fn make_session(
    platforms: Vec<&str>,
    conversations: Vec<&str>,
) -> (ClientSession, mpsc::UnboundedReceiver<String>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let session = ClientSession {
        id: uuid::Uuid::new_v4().to_string(),
        platforms: platforms.into_iter().map(|s| s.to_string()).collect(),
        conversations: conversations.into_iter().map(|s| s.to_string()).collect(),
        sender: tx,
    };
    (session, rx)
}

#[test]
fn test_register_and_count() {
    let bc = MessageBroadcaster::new();
    assert_eq!(bc.client_count(), 0);

    let (s1, _rx1) = make_session(vec!["qq"], vec![]);
    bc.register(s1);
    assert_eq!(bc.client_count(), 1);

    let (s2, _rx2) = make_session(vec!["wechat"], vec![]);
    bc.register(s2);
    assert_eq!(bc.client_count(), 2);
}

#[test]
fn test_unregister() {
    let bc = MessageBroadcaster::new();
    let (s1, _rx1) = make_session(vec!["qq"], vec![]);
    let id = s1.id.clone();
    bc.register(s1);
    assert_eq!(bc.client_count(), 1);

    bc.unregister(&id);
    assert_eq!(bc.client_count(), 0);

    bc.unregister("nonexistent");
    assert_eq!(bc.client_count(), 0);
}

#[test]
fn test_subscribe_and_receive() {
    let bc = MessageBroadcaster::new();
    let mut platforms = HashSet::new();
    platforms.insert("qq".to_string());
    let (tx, mut rx) = mpsc::unbounded_channel();
    let session_id = uuid::Uuid::new_v4().to_string();
    let session = ClientSession {
        id: session_id.clone(),
        platforms,
        conversations: HashSet::new(),
        sender: tx,
    };
    bc.register(session);

    bc.subscribe(&session_id, vec!["qq".into()], vec!["group_123".into()]);

    let msg = make_broadcast_msg("qq", "group_123", "Hello QQ");
    bc.broadcast_message("qq", "group_123", &msg);

    // The subscribed session should receive the message
    let received = rx.try_recv().unwrap();
    assert!(received.contains("Hello QQ"));
}

#[test]
fn test_broadcast_filters_by_platform() {
    let bc = MessageBroadcaster::new();
    let (tx_qq, mut rx_qq) = mpsc::unbounded_channel();
    let (tx_wx, mut rx_wx) = mpsc::unbounded_channel();

    let session_qq = ClientSession {
        id: uuid::Uuid::new_v4().to_string(),
        platforms: ["qq".into()].into(),
        conversations: HashSet::new(),
        sender: tx_qq,
    };
    let session_wx = ClientSession {
        id: uuid::Uuid::new_v4().to_string(),
        platforms: ["wechat".into()].into(),
        conversations: HashSet::new(),
        sender: tx_wx,
    };
    bc.register(session_qq);
    bc.register(session_wx);

    let msg = make_broadcast_msg("qq", "group_123", "QQ only");
    bc.broadcast_message("qq", "group_123", &msg);

    assert!(rx_qq.try_recv().is_ok()); // QQ should receive
    assert!(rx_wx.try_recv().is_err()); // WeChat should NOT
}

#[test]
fn test_broadcast_filters_by_conversation() {
    let bc = MessageBroadcaster::new();
    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();

    let s1 = ClientSession {
        id: uuid::Uuid::new_v4().to_string(),
        platforms: ["qq".into()].into(),
        conversations: ["group_123".into(), "group_456".into()].into(),
        sender: tx1,
    };
    let s2 = ClientSession {
        id: uuid::Uuid::new_v4().to_string(),
        platforms: ["qq".into()].into(),
        conversations: ["group_789".into()].into(),
        sender: tx2,
    };
    bc.register(s1);
    bc.register(s2);

    let msg = make_broadcast_msg("qq", "group_123", "To group 123");
    bc.broadcast_message("qq", "group_123", &msg);

    assert!(rx1.try_recv().is_ok()); // session 1 subscribed group_123
    assert!(rx2.try_recv().is_err()); // session 2 only has group_789
}

#[test]
fn test_broadcast_stale_session_cleanup() {
    let bc = MessageBroadcaster::new();
    let (tx, rx) = mpsc::unbounded_channel::<String>();
    let session_id = uuid::Uuid::new_v4().to_string();

    let session = ClientSession {
        id: session_id.clone(),
        platforms: ["qq".into()].into(),
        conversations: HashSet::new(),
        sender: tx,
    };
    bc.register(session);
    drop(rx); // close receiver — sender.send() will fail

    let msg = make_broadcast_msg("qq", "group_123", "Stale");
    bc.broadcast_message("qq", "group_123", &msg);

    // The stale session should have been cleaned up
    assert_eq!(bc.client_count(), 0);
}

#[test]
fn test_adapter_status_broadcast() {
    let bc = MessageBroadcaster::new();
    let (tx, mut rx) = mpsc::unbounded_channel();

    let session = ClientSession {
        id: uuid::Uuid::new_v4().to_string(),
        platforms: ["qq".into()].into(),
        conversations: HashSet::new(),
        sender: tx,
    };
    bc.register(session);

    bc.broadcast_adapter_status("qq", "", "Connected");

    let received = rx.try_recv().unwrap();
    assert!(received.contains("adapter_status"));
    assert!(received.contains("Connected"));
}

#[test]
fn test_adapter_status_respects_conversation_filter() {
    let bc = MessageBroadcaster::new();
    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let (tx2, mut rx2) = mpsc::unbounded_channel();

    let s1 = ClientSession {
        id: uuid::Uuid::new_v4().to_string(),
        platforms: ["qq".into()].into(),
        conversations: ["group_123".into()].into(),
        sender: tx1,
    };
    let s2 = ClientSession {
        id: uuid::Uuid::new_v4().to_string(),
        platforms: ["qq".into()].into(),
        conversations: HashSet::new(),
        sender: tx2,
    };
    bc.register(s1);
    bc.register(s2);

    // Broadcast with specific conversation — s1 should not get it (filtered out)
    bc.broadcast_adapter_status("qq", "group_999", "Connected");

    // s1 has group_123, not group_999 — should NOT receive
    assert!(rx1.try_recv().is_err());
    // s2 has empty conversations — should receive (empty = match all)
    assert!(rx2.try_recv().is_ok());
}

#[test]
fn test_subscribe_nonexistent_session() {
    let bc = MessageBroadcaster::new();
    // Should not panic
    bc.subscribe("nonexistent", vec!["qq".into()], vec![]);
    bc.unsubscribe("nonexistent", vec!["qq".into()], vec![]);
}

#[test]
fn test_subscribe_adds_platforms_and_conversations() {
    let bc = MessageBroadcaster::new();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let sid = uuid::Uuid::new_v4().to_string();

    let session = ClientSession {
        id: sid.clone(),
        platforms: HashSet::new(),
        conversations: HashSet::new(),
        sender: tx,
    };
    bc.register(session);

    bc.subscribe(&sid, vec!["qq".into()], vec!["group_1".into()]);

    // Now should receive messages on qq/group_1
    let msg = make_broadcast_msg("qq", "group_1", "Hello");
    bc.broadcast_message("qq", "group_1", &msg);
    assert!(rx.try_recv().is_ok());
}

#[test]
fn test_unsubscribe_removes_platform() {
    let bc = MessageBroadcaster::new();
    let mut platforms = HashSet::new();
    platforms.insert("qq".to_string());
    let (tx, mut rx) = mpsc::unbounded_channel();
    let sid = uuid::Uuid::new_v4().to_string();

    let session = ClientSession {
        id: sid.clone(),
        platforms,
        conversations: HashSet::new(),
        sender: tx,
    };
    bc.register(session);

    bc.unsubscribe(&sid, vec!["qq".into()], vec![]);

    let msg = make_broadcast_msg("qq", "group_1", "Should NOT receive");
    bc.broadcast_message("qq", "group_1", &msg);
    assert!(rx.try_recv().is_err());
}
