use crate::models::message::Message;
use redis::{Client, Commands};
use serde_json;

pub struct MessageQueue {
    client: Client,
    queue_name: String,
}

impl MessageQueue {
    pub fn new(url: &str, queue_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(url)?;
        Ok(Self {
            client,
            queue_name: queue_name.to_string(),
        })
    }

    pub fn push(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.client.get_connection()?;
        let message_json = serde_json::to_string(message)?;
        let _: () = conn.lpush(&self.queue_name, message_json)?;
        Ok(())
    }

    pub fn pop(&self) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        let mut conn = self.client.get_connection()?;
        let message_json: Option<String> = conn.rpop(&self.queue_name, None)?;
        if let Some(json) = message_json {
            let message: Message = serde_json::from_str(&json)?;
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    pub fn len(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let mut conn = self.client.get_connection()?;
        let len: usize = conn.llen(&self.queue_name)?;
        Ok(len)
    }

    pub fn is_empty(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let len = self.len()?;
        Ok(len == 0)
    }
}
