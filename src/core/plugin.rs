use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::message::Message;

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn process_message(&self, message: &mut Message) -> Result<bool, Box<dyn std::error::Error>>;
    fn process_event(
        &self,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub status: PluginStatus,
    pub stats: PluginStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PluginStatus {
    Disabled,
    Initializing,
    Enabled,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginStats {
    pub total_processed: u64,
    pub error_count: u64,
    pub processing_time_ms: u64,
}

impl Default for PluginStats {
    fn default() -> Self {
        Self {
            total_processed: 0,
            error_count: 0,
            processing_time_ms: 0,
        }
    }
}

pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn add_plugin(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn initialize_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for plugin in &self.plugins {
            plugin.initialize()?;
        }
        Ok(())
    }

    pub fn shutdown_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for plugin in &self.plugins {
            plugin.shutdown()?;
        }
        Ok(())
    }

    pub fn process_message(
        &self,
        message: &mut Message,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        for plugin in &self.plugins {
            if plugin.process_message(message)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn process_event(
        &self,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut result = data;
        for plugin in &self.plugins {
            result = plugin.process_event(event_type, result)?;
        }
        Ok(result)
    }

    pub fn get_plugin_info(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|plugin| PluginInfo {
                name: plugin.name().to_string(),
                version: plugin.version().to_string(),
                description: plugin.description().to_string(),
                status: PluginStatus::Enabled,
                stats: PluginStats::default(),
            })
            .collect()
    }
}
