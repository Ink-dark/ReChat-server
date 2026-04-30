use crate::core::adapter::Adapter;
use crate::core::broadcaster::MessageBroadcaster;
use crate::models::message::Message;

use super::protocol::{self, MessageSegment};
use super::ws::OneBotSender;

pub struct OneBotAdapter {
    name: String,
    sender: OneBotSender,
    broadcaster: MessageBroadcaster,
}

impl OneBotAdapter {
    pub fn new(name: String, sender: OneBotSender, broadcaster: MessageBroadcaster) -> Self {
        Self {
            name,
            sender,
            broadcaster,
        }
    }
}

impl Adapter for OneBotAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(adapter = %self.name, "OneBot adapter started");
        self.broadcaster
            .broadcast_adapter_status(&self.name, "Connected");
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(adapter = %self.name, "OneBot adapter stopped");
        self.broadcaster
            .broadcast_adapter_status(&self.name, "Disconnected");
        Ok(())
    }

    fn send_message(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        let segments = internal_message_to_segments(message);

        // Determine if group or private based on conversation ID format
        let (message_type, target_id) = if message.recipient.starts_with("group_") {
            let gid: i64 = message
                .recipient
                .strip_prefix("group_")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            ("group", gid)
        } else if message.recipient.starts_with("private_") {
            let uid: i64 = message
                .recipient
                .strip_prefix("private_")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            ("private", uid)
        } else {
            // Try to parse as numeric; default to group
            let id: i64 = message.recipient.parse().ok().unwrap_or(0);
            ("group", id)
        };

        let action =
            protocol::build_send_msg_action(message_type, target_id, &segments, Some(&message.id));

        let sender = self
            .sender
            .lock()
            .map_err(|e| format!("OneBot sender mutex poisoned: {}", e))?;

        sender
            .as_ref()
            .ok_or("OneBot sender not initialized")?
            .send(action)
            .map_err(|e| format!("Failed to send OneBot action: {}", e))?;

        tracing::info!(
            message_id = %message.id,
            conversation = %message.recipient,
            "OneBot message sent"
        );
        Ok(())
    }

    fn receive_message(&self) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        Ok(None)
    }
}

fn internal_message_to_segments(message: &Message) -> Vec<MessageSegment> {
    match message.message_type {
        crate::models::message::MessageType::Text => {
            vec![MessageSegment::text(&message.content)]
        }
        crate::models::message::MessageType::Image => {
            vec![MessageSegment::image(&message.content)]
        }
        crate::models::message::MessageType::File => {
            vec![MessageSegment::text(&format!(
                "[File: {}]",
                message.content
            ))]
        }
    }
}
