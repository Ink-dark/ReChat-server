use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::core::adapter::AdapterManager;
use crate::core::broadcaster::{ClientSession, MessageBroadcaster};
use crate::models::message::{Message, MessageType};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClientCommand {
    #[serde(rename = "type")]
    cmd_type: String,
    #[serde(default)]
    platforms: Option<Vec<String>>,
    #[serde(default)]
    conversations: Option<Vec<String>>,
    #[serde(default)]
    data: Option<ClientCommandData>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClientCommandData {
    platform: Option<String>,
    conversation: Option<String>,
    content: Option<String>,
    message_type: Option<String>,
}

pub async fn ws_client(
    req: HttpRequest,
    stream: web::Payload,
    broadcaster: web::Data<MessageBroadcaster>,
    adapter_manager: web::Data<Arc<AdapterManager>>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, msg_stream) = actix_ws::handle(&req, stream)?;

    let session_id = Uuid::new_v4().to_string();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let broadcaster_for_sender = broadcaster.get_ref().clone();
    let sid_for_sender = session_id.clone();
    let mut session_clone = session.clone();
    actix_web::rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = session_clone.text(msg).await {
                tracing::warn!(
                    session_id = %sid_for_sender,
                    error = %e,
                    "Failed to send message to client, closing connection"
                );
                broadcaster_for_sender.unregister(&sid_for_sender);
                let _ = session_clone.close(None).await;
                break;
            }
        }
        broadcaster_for_sender.unregister(&sid_for_sender);
    });

    let init_session = ClientSession {
        id: session_id.clone(),
        platforms: HashSet::new(),
        conversations: HashSet::new(),
        sender: tx,
    };
    broadcaster.get_ref().register(init_session);

    let broadcaster_clone = broadcaster.get_ref().clone();
    let adapter_clone = adapter_manager.get_ref().clone();
    let sid = session_id.clone();

    let mut stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    actix_web::rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                        handle_command(
                            &broadcaster_clone,
                            &adapter_clone,
                            &sid,
                            cmd,
                            &mut session,
                        )
                        .await;
                    }
                }
                Ok(AggregatedMessage::Ping(bytes)) => {
                    let _ = session.pong(&bytes).await;
                }
                Ok(AggregatedMessage::Close(_)) | Err(_) => {
                    broadcaster_clone.unregister(&sid);
                    let _ = session.close(None).await;
                    break;
                }
                _ => {}
            }
        }
        broadcaster_clone.unregister(&sid);
    });

    Ok(res)
}

async fn handle_command(
    broadcaster: &MessageBroadcaster,
    adapter_manager: &Arc<AdapterManager>,
    session_id: &str,
    cmd: ClientCommand,
    session: &mut actix_ws::Session,
) {
    match cmd.cmd_type.as_str() {
        "subscribe" => {
            let platforms = cmd.platforms.unwrap_or_default();
            let conversations = cmd.conversations.unwrap_or_default();
            broadcaster.subscribe(session_id, platforms, conversations);
        }
        "unsubscribe" => {
            let platforms = cmd.platforms.unwrap_or_default();
            let conversations = cmd.conversations.unwrap_or_default();
            broadcaster.unsubscribe(session_id, platforms, conversations);
        }
        "send_message" => {
            let data = match cmd.data {
                Some(d) => d,
                None => {
                    let err = serde_json::json!({"type": "error", "error": "Missing data field"});
                    let _ = session.text(err.to_string()).await;
                    return;
                }
            };

            let platform = data.platform.unwrap_or_default();
            let content = data.content.unwrap_or_default();
            let message_type = data.message_type.unwrap_or_else(|| "Text".into());

            let msg_type = match message_type.as_str() {
                "Image" => MessageType::Image,
                "File" => MessageType::File,
                _ => MessageType::Text,
            };

            let recipient = data.conversation.unwrap_or_default();
            let message = Message::new(msg_type, content.clone(), recipient.clone());

            crate::REPO.with(|repo| {
                if let Some(r) = repo.borrow().as_ref()
                    && let Err(e) = r.save(&message)
                {
                    tracing::error!(error = %e, "Failed to save message to database");
                }
            });

            if let Err(e) = adapter_manager.send_to_adapter(&platform, &message) {
                tracing::warn!(platform = %platform, error = %e, "No adapter found for platform");
            }

            let now = message
                .created_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let broadcast_msg = crate::core::broadcaster::BroadcastMessage {
                msg_type: "new_message".into(),
                data: crate::core::broadcaster::BroadcastMessageData {
                    id: message.id.clone(),
                    platform: platform.clone(),
                    conversation: message.recipient.clone(),
                    conversation_name: None,
                    content: message.content.clone(),
                    message_type: format!("{:?}", message.message_type),
                    sender: None,
                    created_at: now,
                },
            };
            broadcaster.broadcast_message(&platform, &broadcast_msg);

            let ack = serde_json::json!({
                "type": "ack",
                "status": "sent",
                "message_id": message.id
            });
            let _ = session.text(ack.to_string()).await;
        }
        _ => {
            let err = serde_json::json!({
                "type": "error",
                "error": format!("Unknown command type: {}", cmd.cmd_type)
            });
            let _ = session.text(err.to_string()).await;
        }
    }
}
