use crate::models::message::{Message, MessageType};

pub trait MessageSender {
    fn send(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>>;
    fn supports_type(&self, message_type: &MessageType) -> bool;
}

pub struct DefaultMessageSender;

impl MessageSender for DefaultMessageSender {
    fn send(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        // 默认消息发送实现
        println!("Sending message: {:?}", message);
        Ok(())
    }

    fn supports_type(&self, _message_type: &MessageType) -> bool {
        true
    }
}

pub struct TextMessageSender;

impl MessageSender for TextMessageSender {
    fn send(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        if let MessageType::Text = message.message_type {
            println!("Sending text message: {}", message.content);
            Ok(())
        } else {
            Err(Box::from("This sender only supports text messages"))
        }
    }

    fn supports_type(&self, message_type: &MessageType) -> bool {
        matches!(message_type, MessageType::Text)
    }
}

pub struct ImageMessageSender;

impl MessageSender for ImageMessageSender {
    fn send(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        if let MessageType::Image = message.message_type {
            println!("Sending image message: {}", message.content);
            Ok(())
        } else {
            Err(Box::from("This sender only supports image messages"))
        }
    }

    fn supports_type(&self, message_type: &MessageType) -> bool {
        matches!(message_type, MessageType::Image)
    }
}

pub struct FileMessageSender;

impl MessageSender for FileMessageSender {
    fn send(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        if let MessageType::File = message.message_type {
            println!("Sending file message: {}", message.content);
            Ok(())
        } else {
            Err(Box::from("This sender only supports file messages"))
        }
    }

    fn supports_type(&self, message_type: &MessageType) -> bool {
        matches!(message_type, MessageType::File)
    }
}
