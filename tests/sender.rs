use rechat_sender::core::message::MessageRepository;
use rechat_sender::models::message::{Message, MessageType};

#[test]
fn test_message_creation() {
    let message = Message::new(MessageType::Text, "Hello".to_string(), "user1".to_string());
    assert_eq!(message.content, "Hello");
    assert_eq!(message.recipient, "user1");
    assert_eq!(message.retry_count, 0);
    assert!(!message.id.is_empty());
}

#[test]
fn test_message_repository() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let repo = MessageRepository::new(db_path).unwrap();

    let message = Message::new(
        MessageType::Text,
        "Test message".to_string(),
        "user1".to_string(),
    );

    repo.save(&message).unwrap();

    let retrieved = repo.get(&message.id).unwrap().unwrap();
    assert_eq!(retrieved.id, message.id);
    assert_eq!(retrieved.content, "Test message");
    assert_eq!(retrieved.recipient, "user1");
}

#[test]
fn test_get_nonexistent_message() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let repo = MessageRepository::new(db_path).unwrap();

    let result = repo.get("nonexistent-id").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_insert_or_replace() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let repo = MessageRepository::new(db_path).unwrap();

    let mut message = Message::new(
        MessageType::Text,
        "Original".to_string(),
        "user1".to_string(),
    );
    repo.save(&message).unwrap();

    message.content = "Updated".to_string();
    repo.save(&message).unwrap();

    let retrieved = repo.get(&message.id).unwrap().unwrap();
    assert_eq!(retrieved.content, "Updated");
}

#[test]
fn test_get_pending_messages() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let repo = MessageRepository::new(db_path).unwrap();

    let msg1 = Message::new(MessageType::Text, "Msg1".to_string(), "user1".to_string());
    let msg2 = Message::new(MessageType::Image, "Msg2".to_string(), "user2".to_string());
    repo.save(&msg1).unwrap();
    repo.save(&msg2).unwrap();

    let pending = repo.get_pending_messages(10).unwrap();
    assert_eq!(pending.len(), 2);
}
