use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::message::Message;

pub trait Adapter: Send + Sync {
    fn name(&self) -> &str;
    fn start(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn stop(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn send_message(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>>;
    fn receive_message(&self) -> Result<Option<Message>, Box<dyn std::error::Error>>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdapterInfo {
    pub name: String,
    pub type_: String,
    pub status: AdapterStatus,
    pub stats: AdapterStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AdapterStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdapterStats {
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub error_count: u64,
    pub uptime_seconds: u64,
}

impl Default for AdapterStats {
    fn default() -> Self {
        Self {
            total_messages_sent: 0,
            total_messages_received: 0,
            error_count: 0,
            uptime_seconds: 0,
        }
    }
}

pub struct AdapterManager {
    adapters: Vec<Arc<dyn Adapter>>,
}

impl AdapterManager {
    pub fn new() -> Self {
        Self {
            adapters: vec!(),
        }
    }

    pub fn add_adapter(&mut self, adapter: Arc<dyn Adapter>) {
        self.adapters.push(adapter);
    }

    pub fn start_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for adapter in &self.adapters {
            adapter.start()?;
        }
        Ok(())
    }

    pub fn stop_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for adapter in &self.adapters {
            adapter.stop()?;
        }
        Ok(())
    }

    pub fn send_to_adapter(&self, adapter_name: &str, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        for adapter in &self.adapters {
            if adapter.name() == adapter_name {
                return adapter.send_message(message);
            }
        }
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Adapter '{}' not found", adapter_name),
        )))
    }

    pub fn broadcast_message(&self, message: &Message) -> Vec<Result<(), Box<dyn std::error::Error>>> {
        self.adapters
            .iter()
            .map(|adapter| adapter.send_message(message))
            .collect()
    }
}
