use rechat_sender::core::adapter::Adapter;
use rechat_sender::core::adapter::AdapterManager;
use rechat_sender::core::dispatcher::MessageDispatcher;
use rechat_sender::core::message::MessageRepository;
use rechat_sender::models::message::{Message, MessageStatus, MessageType};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;

struct MockAdapter {
    name: String,
    fail_count: AtomicU32,
    should_fail: bool,
}

impl MockAdapter {
    fn new(name: &str, should_fail: bool) -> Self {
        Self {
            name: name.into(),
            fail_count: AtomicU32::new(0),
            should_fail,
        }
    }
}

impl Adapter for MockAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn send_message(
        &self,
        _message: &Message,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.should_fail {
            self.fail_count.fetch_add(1, Ordering::SeqCst);
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "simulated failure",
            )))
        } else {
            Ok(())
        }
    }

    fn receive_message(
        &self,
    ) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        Ok(None)
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatcher_sends_pending_message() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap().to_string();

    let repo = MessageRepository::new(&db_path).unwrap();

    let msg = Message::new(
        MessageType::Text,
        "Test dispatcher".into(),
        "test_platform".into(),
    );
    repo.save(&msg).unwrap();
    // Close connection so dispatcher can open its own
    drop(repo);

    let mut am = AdapterManager::new();
    let mock = Arc::new(MockAdapter::new("test_platform", false));
    am.add_adapter(mock.clone());
    let am = Arc::new(am);

    let dispatcher = MessageDispatcher::new(am, db_path, 0, 1, 1, 1);
    let shutdown = dispatcher.shutdown_handle();
    dispatcher.start();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    shutdown.store(true, Ordering::Relaxed);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let repo2 =
        MessageRepository::new(&temp_db.path().to_str().unwrap()).unwrap();
    let updated = repo2.get(&msg.id).unwrap().unwrap();
    assert_eq!(updated.status, MessageStatus::Sent);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatcher_marks_as_failed_after_max_retries() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap().to_string();

    let repo = MessageRepository::new(&db_path).unwrap();

    let msg = Message::new(
        MessageType::Text,
        "Should fail eventually".into(),
        "bad_platform".into(),
    );
    repo.save(&msg).unwrap();
    drop(repo);

    let mut am = AdapterManager::new();
    let mock = Arc::new(MockAdapter::new("bad_platform", true));
    am.add_adapter(mock.clone());
    let am = Arc::new(am);

    let dispatcher = MessageDispatcher::new(am, db_path, 1, 1, 1, 1);
    let shutdown = dispatcher.shutdown_handle();
    dispatcher.start();

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    shutdown.store(true, Ordering::Relaxed);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let repo2 =
        MessageRepository::new(&temp_db.path().to_str().unwrap()).unwrap();
    let updated = repo2.get(&msg.id).unwrap().unwrap();
    assert_eq!(updated.status, MessageStatus::Failed);
    assert!(updated.retry_count > 0);
}

#[test]
fn test_update_message_status() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let repo = MessageRepository::new(db_path).unwrap();

    let msg = Message::new(
        MessageType::Text,
        "Status test".into(),
        "user1".into(),
    );
    repo.save(&msg).unwrap();

    repo.update_message_status(&msg.id, &MessageStatus::Sent)
        .unwrap();

    let updated = repo.get(&msg.id).unwrap().unwrap();
    assert_eq!(updated.status, MessageStatus::Sent);
}

#[test]
fn test_increment_retry() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let repo = MessageRepository::new(db_path).unwrap();

    let msg = Message::new(
        MessageType::Text,
        "Retry test".into(),
        "user1".into(),
    );
    repo.save(&msg).unwrap();

    repo.increment_retry(&msg.id).unwrap();
    repo.increment_retry(&msg.id).unwrap();

    let updated = repo.get(&msg.id).unwrap().unwrap();
    assert_eq!(updated.retry_count, 2);
}
