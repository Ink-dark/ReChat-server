use crate::models::message::{Message, MessageStatus, MessageType};
use rusqlite::{Connection, Result};

pub struct MessageRepository {
    conn: Connection,
}

fn read_message_row(
    row: (String, String, String, String, String, i64, i64, u32),
) -> Option<Message> {
    let (id, mt, content, recipient, status_str, created_at_secs, updated_at_secs, retry_count) =
        row;
    let message_type = match mt.as_str() {
        "Text" => MessageType::Text,
        "Image" => MessageType::Image,
        "File" => MessageType::File,
        "Video" => MessageType::Video,
        "Audio" => MessageType::Audio,
        _ => return None,
    };
    let status = match status_str.as_str() {
        "Pending" => MessageStatus::Pending,
        "Sending" => MessageStatus::Sending,
        "Sent" => MessageStatus::Sent,
        "Failed" => MessageStatus::Failed,
        "Canceled" => MessageStatus::Canceled,
        _ => return None,
    };
    let created_at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(created_at_secs as u64);
    let updated_at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(updated_at_secs as u64);
    Some(Message {
        id,
        message_type,
        content,
        recipient,
        status,
        created_at,
        updated_at,
        retry_count,
    })
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
        let created_at_secs = message
            .created_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let updated_at_secs = message
            .updated_at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        if created_at_secs == 0 {
            tracing::warn!(
                message_id = %message.id,
                "Message has zero created_at timestamp, possible system time anomaly"
            );
        }
        let retry_count = message.retry_count;
        self.conn.execute(
            "INSERT OR REPLACE INTO messages 
             (id, message_type, content, recipient, status, created_at, updated_at, retry_count) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                message.id,
                format!("{:?}", message.message_type),
                message.content,
                message.recipient,
                format!("{:?}", message.status),
                created_at_secs,
                updated_at_secs,
                retry_count,
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message_type, content, recipient, status, created_at, updated_at, retry_count 
             FROM messages WHERE id = ?",
        )?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let message_type_str: String = row.get(1)?;
            let content: String = row.get(2)?;
            let recipient: String = row.get(3)?;
            let status_str: String = row.get(4)?;
            let created_at_secs: i64 = row.get(5)?;
            let updated_at_secs: i64 = row.get(6)?;
            let retry_count: u32 = row.get(7)?;

            if created_at_secs <= 0 {
                tracing::warn!(
                    message_id = %id,
                    created_at = created_at_secs,
                    "Message has invalid created_at timestamp"
                );
            }

            let message_type = match message_type_str.as_str() {
                "Text" => crate::models::message::MessageType::Text,
                "Image" => crate::models::message::MessageType::Image,
                "File" => crate::models::message::MessageType::File,
                "Video" => crate::models::message::MessageType::Video,
                "Audio" => crate::models::message::MessageType::Audio,
                _ => return Err(rusqlite::Error::InvalidQuery),
            };

            let status = match status_str.as_str() {
                "Pending" => MessageStatus::Pending,
                "Sending" => MessageStatus::Sending,
                "Sent" => MessageStatus::Sent,
                "Failed" => MessageStatus::Failed,
                "Canceled" => MessageStatus::Canceled,
                _ => return Err(rusqlite::Error::InvalidQuery),
            };

            let created_at =
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(created_at_secs as u64);
            let updated_at =
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(updated_at_secs as u64);

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
             FROM messages WHERE status = 'Pending' OR status = 'Sending' ORDER BY created_at ASC LIMIT ?",
        )?;
        let mut rows = stmt.query([&limit])?;
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
                "Video" => crate::models::message::MessageType::Video,
                "Audio" => crate::models::message::MessageType::Audio,
                _ => continue,
            };

            let status = match status_str.as_str() {
                "Pending" => MessageStatus::Pending,
                "Sending" => MessageStatus::Sending,
                "Sent" => MessageStatus::Sent,
                "Failed" => MessageStatus::Failed,
                "Canceled" => MessageStatus::Canceled,
                _ => continue,
            };

            let created_at =
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(created_at_secs as u64);
            let updated_at =
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(updated_at_secs as u64);

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

    pub fn update_message_status(&self, id: &str, status: &MessageStatus) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn.execute(
            "UPDATE messages SET status = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![format!("{:?}", status), now, id],
        )?;
        Ok(())
    }

    /// Atomically claim a message for sending: only succeeds if status is Pending or Sending.
    /// Returns true if claimed, false if the message was already canceled/failed/sent.
    pub fn try_claim_message(&self, id: &str) -> Result<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let affected = self.conn.execute(
            "UPDATE messages SET status = 'Sending', updated_at = ?1 WHERE id = ?2 AND status IN ('Pending', 'Sending')",
            rusqlite::params![now, id],
        )?;
        Ok(affected > 0)
    }

    pub fn increment_retry(&self, id: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn.execute(
            "UPDATE messages SET retry_count = retry_count + 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id],
        )?;
        Ok(())
    }

    pub fn list(&self, status: Option<&str>, offset: usize, limit: usize) -> Result<Vec<Message>> {
        let mut messages = Vec::new();
        if let Some(s) = status {
            let mut stmt = self.conn.prepare(
                "SELECT id, message_type, content, recipient, status, created_at, updated_at, retry_count FROM messages WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
            )?;
            let rows = stmt.query_map(rusqlite::params![s, limit, offset], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, u32>(7)?,
                ))
            })?;
            for row in rows {
                if let Some(msg) = read_message_row(row?) {
                    messages.push(msg);
                }
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, message_type, content, recipient, status, created_at, updated_at, retry_count FROM messages ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![limit, offset], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, u32>(7)?,
                ))
            })?;
            for row in rows {
                if let Some(msg) = read_message_row(row?) {
                    messages.push(msg);
                }
            }
        }
        Ok(messages)
    }

    pub fn count_by_status(&self, status: &str) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE status = ?1",
            rusqlite::params![status],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        let affected = self
            .conn
            .execute("DELETE FROM messages WHERE id = ?1", rusqlite::params![id])?;
        Ok(affected > 0)
    }
}
