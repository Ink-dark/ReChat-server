use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Image,
    File,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Sending,
    Sent,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub message_type: MessageType,
    pub content: String,
    pub recipient: String,
    pub status: MessageStatus,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub retry_count: u32,
}

impl Message {
    pub fn new(message_type: MessageType, content: String, recipient: String) -> Self {
        let now = SystemTime::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type,
            content,
            recipient,
            status: MessageStatus::Pending,
            created_at: now,
            updated_at: now,
            retry_count: 0,
        }
    }

    pub fn update_status(&mut self, status: MessageStatus) {
        self.status = status;
        self.updated_at = SystemTime::now();
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.updated_at = SystemTime::now();
    }
}
