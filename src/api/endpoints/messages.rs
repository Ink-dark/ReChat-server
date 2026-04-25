use actix_web::{web, HttpResponse, Responder};
use crate::models::message::{Message, MessageType};
use crate::REPO;
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

impl From<Message> for MessageResponse {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            message_type: format!("{:?}", message.message_type),
            content: message.content,
            recipient: message.recipient,
            status: format!("{:?}", message.status),
            created_at: message.created_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            updated_at: message.updated_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            retry_count: message.retry_count,
        }
    }
}

pub async fn create_message(
    req: web::Json<CreateMessageRequest>,
) -> impl Responder {
    let message_type = match req.message_type.as_str() {
        "Text" => MessageType::Text,
        "Image" => MessageType::Image,
        "File" => MessageType::File,
        _ => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid message type"})),
    };

    let message = Message::new(message_type, req.content.clone(), req.recipient.clone());
    match crate::REPO.with(|repo| {
        repo.borrow().as_ref().unwrap().save(&message)
    }) {
        Ok(_) => HttpResponse::Created().json(MessageResponse::from(message)),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn get_message(
    id: web::Path<String>,
) -> impl Responder {
    match crate::REPO.with(|repo| {
        repo.borrow().as_ref().unwrap().get(&id)
    }) {
        Ok(Some(message)) => HttpResponse::Ok().json(MessageResponse::from(message)),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({"error": "Message not found"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}
