use crate::models::message::Message;
use crate::plugins::plugin::Plugin;
use std::collections::HashMap;

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
    }

    pub fn initialize_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for (name, plugin) in &self.plugins {
            println!("Initializing plugin: {}", name);
            plugin.initialize()?;
        }
        Ok(())
    }

    pub fn shutdown_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for (name, plugin) in &self.plugins {
            println!("Shutting down plugin: {}", name);
            plugin.shutdown()?;
        }
        Ok(())
    }

    pub fn on_message_received(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        for plugin in self.plugins.values() {
            plugin.on_message_received(message)?;
        }
        Ok(())
    }

    pub fn on_message_sent(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        for plugin in self.plugins.values() {
            plugin.on_message_sent(message)?;
        }
        Ok(())
    }

    pub fn on_message_failed(&self, message: &Message, error: &str) -> Result<(), Box<dyn std::error::Error>> {
        for plugin in self.plugins.values() {
            plugin.on_message_failed(message, error)?;
        }
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.get(name)
    }

    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}
