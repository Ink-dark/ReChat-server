use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PluginStats {
    pub total_processed: u64,
    pub error_count: u64,
    pub processing_time_ms: u64,
}

pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
    initialized: Mutex<HashSet<String>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: vec![],
            initialized: Mutex::new(HashSet::new()),
        }
    }

    pub fn add_plugin(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn initialize_all(&self) {
        for plugin in &self.plugins {
            let name = plugin.name().to_string();
            if let Err(e) = plugin.initialize() {
                tracing::error!(
                    plugin = %name,
                    error = %e,
                    "Failed to initialize plugin"
                );
            } else {
                self.initialized.lock().unwrap().insert(name);
            }
        }
    }

    pub fn shutdown_all(&self) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.shutdown() {
                tracing::error!(
                    plugin = %plugin.name(),
                    error = %e,
                    "Failed to shutdown plugin"
                );
            }
        }
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
        let initialized = self.initialized.lock().unwrap();
        self.plugins
            .iter()
            .map(|plugin| {
                let name = plugin.name().to_string();
                PluginInfo {
                    name: name.clone(),
                    version: plugin.version().to_string(),
                    description: plugin.description().to_string(),
                    status: if initialized.contains(&name) {
                        PluginStatus::Enabled
                    } else {
                        PluginStatus::Disabled
                    },
                    stats: PluginStats::default(),
                }
            })
            .collect()
    }
}
