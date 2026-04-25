use crate::models::message::{Message, MessageStatus};
use rusqlite::{Connection, Result};

pub struct MessageRepository {
    conn: Connection,
}

impl MessageRepository {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                message_type TEXT NOT NULL,
                content TEXT NOT NULL,
                recipient TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                retry_count INTEGER NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn save(&self, message: &Message) -> Result<()> {
        let created_at_secs = message.created_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let updated_at_secs = message.updated_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let created_at_str = created_at_secs.to_string();
        let updated_at_str = updated_at_secs.to_string();
        let retry_count_str = message.retry_count.to_string();
        self.conn.execute(
            "INSERT OR REPLACE INTO messages 
             (id, message_type, content, recipient, status, created_at, updated_at, retry_count) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            &[
                &message.id,
                &format!("{:?}", message.message_type),
                &message.content,
                &message.recipient,
                &format!("{:?}", message.status),
                &created_at_str,
                &updated_at_str,
                &retry_count_str,
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message_type, content, recipient, status, created_at, updated_at, retry_count 
             FROM messages WHERE id = ?",
        )?;
        let mut rows = stmt.query(&[id])?;
        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let message_type_str: String = row.get(1)?;
            let content: String = row.get(2)?;
            let recipient: String = row.get(3)?;
            let status_str: String = row.get(4)?;
            let created_at_secs: i64 = row.get(5)?;
            let updated_at_secs: i64 = row.get(6)?;
            let retry_count: u32 = row.get(7)?;

            let message_type = match message_type_str.as_str() {
                "Text" => crate::models::message::MessageType::Text,
                "Image" => crate::models::message::MessageType::Image,
                "File" => crate::models::message::MessageType::File,
                _ => return Err(rusqlite::Error::InvalidColumnType(1, "Text".to_string(), rusqlite::types::Type::Text)),
            };

            let status = match status_str.as_str() {
                "Pending" => MessageStatus::Pending,
                "Sending" => MessageStatus::Sending,
                "Sent" => MessageStatus::Sent,
                "Failed" => MessageStatus::Failed,
                _ => return Err(rusqlite::Error::InvalidColumnType(4, "Text".to_string(), rusqlite::types::Type::Text)),
            };

            let created_at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(created_at_secs as u64);
            let updated_at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(updated_at_secs as u64);

            Ok(Some(Message {
                id,
                message_type,
                content,
                recipient,
                status,
                created_at,
                updated_at,
                retry_count,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_pending_messages(&self, limit: usize) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message_type, content, recipient, status, created_at, updated_at, retry_count 
             FROM messages WHERE status = 'Pending' OR status = 'Sending' LIMIT ?",
        )?;
        let mut rows = stmt.query(&[&limit])?;
        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let message_type_str: String = row.get(1)?;
            let content: String = row.get(2)?;
            let recipient: String = row.get(3)?;
            let status_str: String = row.get(4)?;
            let created_at_secs: i64 = row.get(5)?;
            let updated_at_secs: i64 = row.get(6)?;
            let retry_count: u32 = row.get(7)?;

            let message_type = match message_type_str.as_str() {
                "Text" => crate::models::message::MessageType::Text,
                "Image" => crate::models::message::MessageType::Image,
                "File" => crate::models::message::MessageType::File,
                _ => continue,
            };

            let status = match status_str.as_str() {
                "Pending" => MessageStatus::Pending,
                "Sending" => MessageStatus::Sending,
                "Sent" => MessageStatus::Sent,
                "Failed" => MessageStatus::Failed,
                _ => continue,
            };

            let created_at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(created_at_secs as u64);
            let updated_at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(updated_at_secs as u64);

            messages.push(Message {
                id,
                message_type,
                content,
                recipient,
                status,
                created_at,
                updated_at,
                retry_count,
            });
        }

        Ok(messages)
    }
}
