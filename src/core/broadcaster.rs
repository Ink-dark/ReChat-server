use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize)]
pub struct SenderInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BroadcastMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: BroadcastMessageData,
}

#[derive(Debug, Clone, Serialize)]
pub struct BroadcastMessageData {
    pub id: String,
    pub platform: String,
    pub conversation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_name: Option<String>,
    pub content: String,
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<SenderInfo>,
    pub created_at: u64,
}

pub struct ClientSession {
    pub id: String,
    pub platforms: HashSet<String>,
    pub conversations: HashSet<String>,
    pub sender: mpsc::UnboundedSender<String>,
}

#[derive(Clone)]
pub struct MessageBroadcaster {
    sessions: Arc<Mutex<HashMap<String, ClientSession>>>,
}

impl Default for MessageBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBroadcaster {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, session: ClientSession) {
        self.sessions
            .lock()
            .unwrap()
            .insert(session.id.clone(), session);
    }

    pub fn unregister(&self, session_id: &str) {
        self.sessions.lock().unwrap().remove(session_id);
    }

    pub fn broadcast_message(&self, platform: &str, msg: &BroadcastMessage) {
        let mut stale_ids = Vec::new();
        {
            let sessions = self.sessions.lock().unwrap();
            for session in sessions.values() {
                if session.platforms.contains(platform)
                    && let Ok(json) = serde_json::to_string(msg)
                    && session.sender.send(json).is_err()
                {
                    stale_ids.push(session.id.clone());
                }
            }
        }
        for id in stale_ids {
            self.unregister(&id);
            tracing::debug!(session_id = %id, "Removed stale client session after send failure");
        }
    }

    pub fn subscribe(
        &self,
        session_id: &str,
        platforms: Vec<String>,
        conversations: Vec<String>,
    ) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session_id) {
            for p in platforms {
                s.platforms.insert(p);
            }
            for c in conversations {
                s.conversations.insert(c);
            }
        }
    }

    pub fn unsubscribe(
        &self,
        session_id: &str,
        platforms: Vec<String>,
        conversations: Vec<String>,
    ) {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(s) = sessions.get_mut(session_id) {
            for p in platforms {
                s.platforms.remove(&p);
            }
            for c in conversations {
                s.conversations.remove(&c);
            }
        }
    }

    pub fn broadcast_adapter_status(&self, platform: &str, status: &str) {
        let msg = BroadcastMessage {
            msg_type: "adapter_status".into(),
            data: BroadcastMessageData {
                id: String::new(),
                platform: platform.into(),
                conversation: String::new(),
                conversation_name: None,
                content: status.into(),
                message_type: "status".into(),
                sender: None,
                created_at: 0,
            },
        };
        let mut stale_ids = Vec::new();
        {
            let sessions = self.sessions.lock().unwrap();
            for session in sessions.values() {
                if (session.platforms.contains(platform) || session.platforms.is_empty())
                    && let Ok(json) = serde_json::to_string(&msg)
                    && session.sender.send(json).is_err()
                {
                    stale_ids.push(session.id.clone());
                }
            }
        }
        for id in stale_ids {
            self.unregister(&id);
            tracing::debug!(session_id = %id, "Removed stale client session after status broadcast failure");
        }
    }

    pub fn client_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }
}
