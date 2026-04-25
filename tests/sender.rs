use rechat_sender::core::message::MessageRepository;
use rechat_sender::models::message::{Message, MessageType};

#[test]
fn test_message_creation() {
    let message = Message::new(MessageType::Text, "Hello".to_string(), "user1".to_string());
    assert_eq!(message.content, "Hello");
    assert_eq!(message.recipient, "user1");
    assert_eq!(message.retry_count, 0);
}

#[test]
fn test_message_repository() {
    // 创建临时数据库文件
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    // 初始化消息仓库
    let repo = MessageRepository::new(db_path).unwrap();

    // 创建测试消息
    let message = Message::new(
        MessageType::Text,
        "Test message".to_string(),
        "user1".to_string(),
    );

    // 保存消息
    repo.save(&message).unwrap();

    // 获取消息
    let retrieved_message = repo.get(&message.id).unwrap().unwrap();
    assert_eq!(retrieved_message.id, message.id);
    assert_eq!(retrieved_message.content, "Test message");
    assert_eq!(retrieved_message.recipient, "user1");
}
