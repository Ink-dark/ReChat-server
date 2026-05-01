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

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageListQuery {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub status: Option<String>,
}

pub async fn list_messages(query: web::Query<MessageListQuery>) -> impl Responder {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50).min(200);
    let status = query.status.as_deref();

    let result = crate::REPO.with(|repo| {
        repo.borrow()
            .as_ref()
            .map(|r| r.list(status, offset, limit))
    });
    match result {
        Some(Ok(messages)) => {
            let items: Vec<MessageResponse> =
                messages.into_iter().map(MessageResponse::from).collect();
            HttpResponse::Ok().json(serde_json::json!({
                "messages": items,
                "offset": offset,
                "limit": limit,
            }))
        }
        Some(Err(e)) => {
            tracing::error!(error = %e, "Failed to list messages");
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
        None => {
            tracing::error!("Repository not initialized during list call");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Repository not initialized"}))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchMessageRequest {
    pub status: String,
}

pub async fn update_message(
    id: web::Path<String>,
    body: web::Json<PatchMessageRequest>,
) -> impl Responder {
    let new_status = match body.status.as_str() {
        "Pending" => crate::models::message::MessageStatus::Pending,
        "Sending" => crate::models::message::MessageStatus::Sending,
        "Sent" => crate::models::message::MessageStatus::Sent,
        "Canceled" | "Failed" => crate::models::message::MessageStatus::Failed,
        _ => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"error": "Invalid status. Use Pending, Sending, Sent, Failed, or Canceled"}));
        }
    };

    let result = crate::REPO.with(|repo| {
        repo.borrow()
            .as_ref()
            .map(|r| r.update_message_status(&id, &new_status))
    });
    match result {
        Some(Ok(())) => {
            let msg_result = crate::REPO.with(|repo| repo.borrow().as_ref().map(|r| r.get(&id)));
            match msg_result {
                Some(Ok(Some(msg))) => HttpResponse::Ok().json(MessageResponse::from(msg)),
                _ => HttpResponse::Ok().json(serde_json::json!({"status": "updated"})),
            }
        }
        Some(Err(e)) => {
            tracing::error!(error = %e, message_id = %id.into_inner(), "Failed to update message");
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
        None => {
            tracing::error!("Repository not initialized during update call");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Repository not initialized"}))
        }
    }
}

pub async fn delete_message(id: web::Path<String>) -> impl Responder {
    let result = crate::REPO.with(|repo| repo.borrow().as_ref().map(|r| r.delete(&id)));
    match result {
        Some(Ok(true)) => HttpResponse::Ok().json(serde_json::json!({"deleted": true})),
        Some(Ok(false)) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Message not found"}))
        }
        Some(Err(e)) => {
            tracing::error!(error = %e, message_id = %id.into_inner(), "Failed to delete message");
            HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
        }
        None => {
            tracing::error!("Repository not initialized during delete call");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Repository not initialized"}))
        }
    }
}

pub async fn get_stats() -> impl Responder {
    let result = crate::REPO.with(|repo| {
        let r = repo.borrow();
        let repo_ref = r.as_ref()?;
        let pending = repo_ref.count_by_status("Pending").unwrap_or(0);
        let sending = repo_ref.count_by_status("Sending").unwrap_or(0);
        let sent = repo_ref.count_by_status("Sent").unwrap_or(0);
        let failed = repo_ref.count_by_status("Failed").unwrap_or(0);
        Some((pending, sending, sent, failed))
    });
    match result {
        Some((pending, sending, sent, failed)) => HttpResponse::Ok().json(serde_json::json!({
            "pending": pending,
            "sending": sending,
            "sent": sent,
            "failed": failed,
            "total": pending + sending + sent + failed,
        })),
        None => {
            tracing::error!("Repository not initialized during stats call");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Repository not initialized"}))
        }
    }
}
