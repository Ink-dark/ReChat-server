use crate::models::message::Message;

pub trait ProtocolAdapter {
    fn serialize_message(&self, message: &Message) -> Result<String, Box<dyn std::error::Error>>;
    fn deserialize_message(&self, data: &str) -> Result<Message, Box<dyn std::error::Error>>;
    fn get_name(&self) -> &str;
}

pub struct JsonProtocolAdapter;

impl ProtocolAdapter for JsonProtocolAdapter {
    fn serialize_message(&self, message: &Message) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(message)?)
    }

    fn deserialize_message(&self, data: &str) -> Result<Message, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(data)?)
    }

    fn get_name(&self) -> &str {
        "json"
    }
}

pub struct PlainTextProtocolAdapter;

impl ProtocolAdapter for PlainTextProtocolAdapter {
    fn serialize_message(&self, message: &Message) -> Result<String, Box<dyn std::error::Error>> {
        Ok(format!("{}|{}|{}", message.id, message.recipient, message.content))
    }

    fn deserialize_message(&self, data: &str) -> Result<Message, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = data.split('|').collect();
        if parts.len() < 3 {
            return Err(Box::from("Invalid message format"));
        }

        let message = Message {
            id: parts[0].to_string(),
            message_type: crate::models::message::MessageType::Text,
            content: parts[2].to_string(),
            recipient: parts[1].to_string(),
            status: crate::models::message::MessageStatus::Pending,
            created_at: std::time::SystemTime::now(),
            updated_at: std::time::SystemTime::now(),
            retry_count: 0,
        };

        Ok(message)
    }

    fn get_name(&self) -> &str {
        "plaintext"
    }
}
