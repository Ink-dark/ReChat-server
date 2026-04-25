use crate::models::message::Message;

pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    
    fn on_message_received(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>>;
    fn on_message_sent(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>>;
    fn on_message_failed(&self, message: &Message, error: &str) -> Result<(), Box<dyn std::error::Error>>;
    
    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct DefaultPlugin;

impl Plugin for DefaultPlugin {
    fn name(&self) -> &str {
        "default"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Default plugin"
    }

    fn on_message_received(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        println!("[DefaultPlugin] Message received: {:?}", message);
        Ok(())
    }

    fn on_message_sent(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        println!("[DefaultPlugin] Message sent: {:?}", message);
        Ok(())
    }

    fn on_message_failed(&self, message: &Message, error: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("[DefaultPlugin] Message failed: {:?}, error: {}", message, error);
        Ok(())
    }

    fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[DefaultPlugin] Initialized");
        Ok(())
    }

    fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[DefaultPlugin] Shutdown");
        Ok(())
    }
}
