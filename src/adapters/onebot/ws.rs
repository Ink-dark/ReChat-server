use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

use super::protocol::{ActionRequest, ActionResponse, MessageEvent, OneBotEvent};
use crate::core::broadcaster::{BroadcastMessage, BroadcastMessageData, MessageBroadcaster};
use crate::models::message::{Message, MessageStatus, MessageType};

/// Sender handle for sending OneBot actions back to NapCat.
/// The adapter.rs in Phase 3 will hold a clone of this.
pub type OneBotSender = Arc<Mutex<Option<mpsc::UnboundedSender<ActionRequest>>>>;

pub async fn onebot_ws(
    req: HttpRequest,
    stream: web::Payload,
    broadcaster: web::Data<MessageBroadcaster>,
    onebot_sender: web::Data<OneBotSender>,
) -> Result<HttpResponse, Error> {
    tracing::info!(
        peer = ?req.peer_addr(),
        "OneBot client connected"
    );

    let (res, mut session, msg_stream) = match actix_ws::handle(&req, stream) {
        Ok(result) => result,
        Err(e) => {
            tracing::error!(error = %e, "OneBot WebSocket handshake failed");
            return Err(actix_web::error::ErrorBadRequest(e));
        }
    };

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<ActionRequest>();
    // Store the sender so the adapter can use it later
    {
        let mut sender = onebot_sender.lock().unwrap();
        *sender = Some(action_tx);
    }

    // Forward OneBot actions to the WS
    let mut session_clone = session.clone();
    actix_web::rt::spawn(async move {
        while let Some(action) = action_rx.recv().await {
            match serde_json::to_string(&action) {
                Ok(json) => {
                    if let Err(e) = session_clone.text(json).await {
                        tracing::warn!(error = %e, "Failed to send OneBot action");
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to serialize OneBot action");
                }
            }
        }
    });

    let broadcaster_clone = broadcaster.get_ref().clone();

    let mut stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    actix_web::rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            let text = match msg {
                Ok(AggregatedMessage::Text(t)) => t,
                Ok(AggregatedMessage::Ping(bytes)) => {
                    let _ = session.pong(&bytes).await;
                    continue;
                }
                Ok(AggregatedMessage::Close(_)) | Err(_) => {
                    tracing::info!("OneBot client disconnected");
                    break;
                }
                _ => continue,
            };

            match serde_json::from_str::<ActionResponse>(&text) {
                Ok(resp) => {
                    if resp.status == "failed" {
                        tracing::warn!(
                            retcode = resp.retcode,
                            echo = ?resp.echo,
                            "OneBot action failed"
                        );
                    }
                }
                Err(_) => match serde_json::from_str::<OneBotEvent>(&text) {
                    Ok(event) => handle_onebot_event(&broadcaster_clone, event),
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to parse OneBot message: {}", text.chars().take(200).collect::<String>());
                    }
                },
            }
        }

        let _ = session.close(None).await;
    });

    Ok(res)
}

fn handle_onebot_event(broadcaster: &MessageBroadcaster, event: OneBotEvent) {
    match event {
        OneBotEvent::Message(msg_event) => handle_message_event(broadcaster, msg_event),
        OneBotEvent::Notice(notice) => {
            tracing::debug!(
                notice_type = %notice.notice_type,
                "OneBot notice event"
            );
        }
        OneBotEvent::Request(request) => {
            tracing::debug!(
                request_type = %request.request_type,
                "OneBot request event"
            );
        }
        OneBotEvent::Meta(meta) => {
            tracing::debug!(
                meta_type = %meta.meta_event_type,
                "OneBot meta event"
            );
        }
    }
}

fn handle_message_event(broadcaster: &MessageBroadcaster, event: MessageEvent) {
    let platform = "qq".to_string();
    let conversation = if let Some(gid) = event.group_id {
        format!("group_{}", gid)
    } else {
        format!("private_{}", event.user_id)
    };

    let raw_text = super::protocol::MessageSegment::segments_to_text(&event.message);

    let message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        message_type: MessageType::Text,
        content: raw_text.clone(),
        recipient: conversation.clone(),
        status: MessageStatus::Pending,
        created_at: std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(event.time as u64))
            .unwrap_or(std::time::SystemTime::now()),
        updated_at: std::time::SystemTime::now(),
        retry_count: 0,
    };

    crate::REPO.with(|repo| {
        if let Some(r) = repo.borrow().as_ref()
            && let Err(e) = r.save(&message)
        {
            tracing::error!(error = %e, "Failed to save OneBot message to database");
        }
    });

    let created_at = event.time as u64;
    let broadcast_msg = BroadcastMessage {
        msg_type: "new_message".into(),
        data: BroadcastMessageData {
            id: message.id.clone(),
            platform: platform.clone(),
            conversation: conversation.clone(),
            conversation_name: None,
            content: raw_text,
            message_type: "Text".into(),
            sender: event.sender.map(|s| crate::core::broadcaster::SenderInfo {
                id: s.user_id.to_string(),
                name: s.card.unwrap_or(s.nickname),
            }),
            created_at,
        },
    };

    broadcaster.broadcast_message(&platform, &conversation, &broadcast_msg);
    tracing::info!(
        message_id = %message.id,
        conversation = %conversation,
        "OneBot message received and broadcast"
    );
}
