use crate::core::broadcaster::MessageBroadcaster;
use crate::models::message::{Message, MessageType};
use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub message_type: String,
    pub content: String,
    pub recipient: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub message_type: String,
    pub content: String,
    pub recipient: String,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub retry_count: u32,
}

fn system_time_to_secs(t: std::time::SystemTime) -> u64 {
    match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(e) => {
            tracing::warn!(error = %e, "System time before UNIX epoch, returning 0");
            0
        }
    }
}

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            message_type: format!("{:?}", message.message_type),
            content: message.content,
            recipient: message.recipient,
            status: format!("{:?}", message.status),
            created_at: system_time_to_secs(message.created_at),
            updated_at: system_time_to_secs(message.updated_at),
            retry_count: message.retry_count,
        }
    }
}

pub async fn create_message(
    req: web::Json<CreateMessageRequest>,
    broadcaster: web::Data<MessageBroadcaster>,
) -> impl Responder {
    let message_type = match req.message_type.as_str() {
        "Text" => MessageType::Text,
        "Image" => MessageType::Image,
        "File" => MessageType::File,
        _ => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid message type"}));
        }
    };

    let message = Message::new(message_type, req.content.clone(), req.recipient.clone());
    let result = crate::REPO.with(|repo| repo.borrow().as_ref().map(|r| r.save(&message)));
    match result {
        Some(Ok(_)) => {
            let now = message
                .created_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let broadcast_msg = crate::core::broadcaster::BroadcastMessage {
                msg_type: "new_message".into(),
                data: crate::core::broadcaster::BroadcastMessageData {
                    id: message.id.clone(),
                    platform: message.recipient.clone(),
                    conversation: message.recipient.clone(),
                    conversation_name: None,
                    content: message.content.clone(),
                    message_type: format!("{:?}", message.message_type),
                    sender: None,
                    created_at: now,
                },
            };
            broadcaster.broadcast_message(&message.recipient, &message.recipient, &broadcast_msg);
            HttpResponse::Created().json(MessageResponse::from(message))
        }
        Some(Err(e)) => {
            tracing::error!(error = %e, "Failed to save message via HTTP API");
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
        None => {
            tracing::error!("Repository not initialized during HTTP API call");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Repository not initialized"}))
        }
    }
}

pub async fn get_message(id: web::Path<String>) -> impl Responder {
    let result = crate::REPO.with(|repo| repo.borrow().as_ref().map(|r| r.get(&id)));
    match result {
        Some(Ok(Some(message))) => HttpResponse::Ok().json(MessageResponse::from(message)),
        Some(Ok(None)) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Message not found"}))
        }
        Some(Err(e)) => {
            tracing::error!(error = %e, message_id = %id.into_inner(), "Failed to get message");
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
        None => {
            tracing::error!("Repository not initialized during HTTP API get call");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Repository not initialized"}))
        }
    }
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}
