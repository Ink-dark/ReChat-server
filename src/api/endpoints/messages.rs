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

pub async fn create_message(req: web::Json<CreateMessageRequest>) -> impl Responder {
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
        Some(Ok(_)) => HttpResponse::Created().json(MessageResponse::from(message)),
        Some(Err(e)) => {
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
        None => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Repository not initialized"})),
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
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
        None => HttpResponse::InternalServerError()
            .json(serde_json::json!({"error": "Repository not initialized"})),
    }
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}
