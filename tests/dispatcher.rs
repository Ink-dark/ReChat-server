use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use rechat_sender::core::adapter::{Adapter, AdapterManager};
use rechat_sender::core::dispatcher::MessageDispatcher;
use rechat_sender::core::message::MessageRepository;
use rechat_sender::models::message::{Message, MessageStatus, MessageType};

struct MockAdapter {
    name: String,
    should_fail: AtomicBool,
    call_count: AtomicU32,
}

impl MockAdapter {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail: AtomicBool::new(false),
            call_count: AtomicU32::new(0),
        }
    }

    fn set_should_fail(&self, fail: bool) {
        self.should_fail.store(fail, Ordering::SeqCst);
    }

    fn call_count(&self) -> u32 {
        self.call_count.load(Ordering::SeqCst)
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

    fn send_message(&self, _message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        if self.should_fail.load(Ordering::SeqCst) {
            Err(Box::new(std::io::Error::other("mock send failure")))
        } else {
            Ok(())
        }
    }

    fn receive_message(&self) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        Ok(None)
    }
}

fn make_pending_message(recipient: &str, content: &str) -> Message {
    let mut msg = Message::new(
        MessageType::Text,
        content.to_string(),
        recipient.to_string(),
    );
    // Override created_at to ensure ordering in get_pending_messages
    msg.created_at = std::time::UNIX_EPOCH;
    msg
}

fn setup_repo(db_path: &str) -> MessageRepository {
    MessageRepository::new(db_path).expect("Failed to create test repo")
}

async fn wait_for_status(
    repo: &MessageRepository,
    msg_id: &str,
    expected: MessageStatus,
    timeout_secs: u64,
) -> bool {
    let start = std::time::Instant::now();
    loop {
        if let Ok(Some(msg)) = repo.get(msg_id)
            && msg.status == expected
        {
            return true;
        }
        if start.elapsed().as_secs() >= timeout_secs {
            return false;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatcher_sends_pending_message() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_str().unwrap().to_string();

    let repo = setup_repo(&db_path);
    let msg = make_pending_message("mock_backend", "hello world");
    repo.save(&msg).unwrap();
    assert_eq!(
        repo.get(&msg.id).unwrap().unwrap().status,
        MessageStatus::Pending
    );

    let mock = Arc::new(MockAdapter::new("mock_backend"));
    let mut manager = AdapterManager::new();
    manager.add_adapter(mock.clone());
    let manager = Arc::new(manager);

    let dispatcher = MessageDispatcher::new(
        manager,
        db_path.clone(),
        0, // max_retries
        1, // retry_interval (1s, but 0 retries = no wait needed)
        10,
        5,
    );
    let shutdown = dispatcher.shutdown_handle();
    dispatcher.start();

    let ok = wait_for_status(&repo, &msg.id, MessageStatus::Sent, 10).await;
    shutdown.store(true, Ordering::Relaxed);

    assert!(ok, "Message should transition to Sent");
    assert!(mock.call_count() >= 1, "Adapter should have been called");
    let final_msg = repo.get(&msg.id).unwrap().unwrap();
    assert_eq!(final_msg.status, MessageStatus::Sent);
    assert_eq!(final_msg.retry_count, 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatcher_skips_canceled_message() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_str().unwrap().to_string();

    let repo = setup_repo(&db_path);
    let mut msg = make_pending_message("mock_backend", "should be canceled");
    msg.status = MessageStatus::Canceled;
    repo.save(&msg).unwrap();

    let mock = Arc::new(MockAdapter::new("mock_backend"));
    let mut manager = AdapterManager::new();
    manager.add_adapter(mock.clone());
    let manager = Arc::new(manager);

    let dispatcher = MessageDispatcher::new(manager, db_path.clone(), 0, 1, 10, 5);
    let shutdown = dispatcher.shutdown_handle();
    dispatcher.start();

    // Wait briefly — canceled messages should not be attempted
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    shutdown.store(true, Ordering::Relaxed);

    assert_eq!(
        mock.call_count(),
        0,
        "Mock adapter should NOT be called for canceled messages"
    );
    let final_msg = repo.get(&msg.id).unwrap().unwrap();
    assert_eq!(
        final_msg.status,
        MessageStatus::Canceled,
        "Canceled message should stay Canceled"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatcher_marks_failed_after_retries() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_str().unwrap().to_string();

    let repo = setup_repo(&db_path);
    let msg = make_pending_message("mock_backend", "will fail");
    repo.save(&msg).unwrap();

    let mock = Arc::new(MockAdapter::new("mock_backend"));
    mock.set_should_fail(true);
    let mut manager = AdapterManager::new();
    manager.add_adapter(mock.clone());
    let manager = Arc::new(manager);

    let max_retries: u32 = 2;
    let dispatcher = MessageDispatcher::new(
        manager,
        db_path.clone(),
        max_retries,
        1, // retry every 1s
        10,
        5,
    );
    let shutdown = dispatcher.shutdown_handle();
    dispatcher.start();

    let ok = wait_for_status(&repo, &msg.id, MessageStatus::Failed, 15).await;
    shutdown.store(true, Ordering::Relaxed);

    assert!(ok, "Message should transition to Failed after retries");
    // Called once per attempt: attempt 0, 1, 2 = 3 calls (max_retries + 1)
    assert_eq!(
        mock.call_count(),
        max_retries + 1,
        "Adapter should be called max_retries+1 times"
    );
    let final_msg = repo.get(&msg.id).unwrap().unwrap();
    assert_eq!(final_msg.status, MessageStatus::Failed);
    assert_eq!(
        final_msg.retry_count,
        max_retries + 1,
        "retry_count should be max_retries+1"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatcher_try_claim_atomicity() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_str().unwrap().to_string();

    let repo = setup_repo(&db_path);
    let msg = make_pending_message("mock_backend", "claim me");
    repo.save(&msg).unwrap();

    // First claim should succeed
    assert!(
        repo.try_claim_message(&msg.id).unwrap(),
        "First claim should succeed"
    );
    let claimed = repo.get(&msg.id).unwrap().unwrap();
    assert_eq!(claimed.status, MessageStatus::Sending);

    // Cancel via normal update
    repo.update_message_status(&msg.id, &MessageStatus::Canceled)
        .unwrap();

    // try_claim should fail for canceled message
    assert!(
        !repo.try_claim_message(&msg.id).unwrap(),
        "Claim should fail for canceled message"
    );

    // Another claim on an already Sent message
    let msg2 = make_pending_message("mock_backend", "already sent");
    repo.save(&msg2).unwrap();
    repo.update_message_status(&msg2.id, &MessageStatus::Sent)
        .unwrap();
    assert!(
        !repo.try_claim_message(&msg2.id).unwrap(),
        "Claim should fail for already sent message"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dispatcher_shutdown_stops_processing() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db_path = tmp.path().to_str().unwrap().to_string();

    let repo = setup_repo(&db_path);

    let mock = Arc::new(MockAdapter::new("mock_backend"));
    let mut manager = AdapterManager::new();
    manager.add_adapter(mock.clone());
    let manager = Arc::new(manager);

    let dispatcher = MessageDispatcher::new(manager, db_path.clone(), 0, 1, 10, 5);
    let shutdown = dispatcher.shutdown_handle();
    shutdown.store(true, Ordering::Relaxed);
    dispatcher.start();

    // After shutdown, save a pending message and wait
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let msg = make_pending_message("mock_backend", "after shutdown");
    repo.save(&msg).unwrap();

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    assert_eq!(
        mock.call_count(),
        0,
        "No messages should be sent after shutdown"
    );
    let final_msg = repo.get(&msg.id).unwrap().unwrap();
    assert_eq!(
        final_msg.status,
        MessageStatus::Pending,
        "Message saved after shutdown should stay Pending"
    );
}
