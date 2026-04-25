use crate::core::message::MessageRepository;
use crate::models::message::{Message, MessageType};
use clap::{App, Arg, SubCommand};
use std::sync::{Arc, RwLock};

pub fn run(repo: Arc<RwLock<MessageRepository>>) {
    let matches = App::new("ReChat Sender CLI")
        .version("1.0")
        .author("ReChat Team")
        .about("Command line interface for ReChat Sender")
        .subcommand(
            SubCommand::with_name("send")
                .about("Send a message")
                .arg(
                    Arg::with_name("type")
                        .short("t")
                        .long("type")
                        .takes_value(true)
                        .required(true)
                        .help("Message type: text, image, file"),
                )
                .arg(
                    Arg::with_name("recipient")
                        .short("r")
                        .long("recipient")
                        .takes_value(true)
                        .required(true)
                        .help("Recipient"),
                )
                .arg(
                    Arg::with_name("content")
                        .short("c")
                        .long("content")
                        .takes_value(true)
                        .required(true)
                        .help("Message content"),
                ),
        )
        .subcommand(
            SubCommand::with_name("status")
                .about("Check message status")
                .arg(
                    Arg::with_name("id")
                        .short("i")
                        .long("id")
                        .takes_value(true)
                        .required(true)
                        .help("Message ID"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("send", Some(send_matches)) => {
            let message_type = match send_matches
                .value_of("type")
                .unwrap()
                .to_lowercase()
                .as_str()
            {
                "text" => MessageType::Text,
                "image" => MessageType::Image,
                "file" => MessageType::File,
                _ => {
                    println!("Invalid message type. Use text, image, or file.");
                    return;
                }
            };

            let recipient = send_matches.value_of("recipient").unwrap();
            let content = send_matches.value_of("content").unwrap();

            let message = Message::new(message_type, content.to_string(), recipient.to_string());
            match repo.write().unwrap().save(&message) {
                Ok(_) => println!("Message sent successfully. ID: {}", message.id),
                Err(e) => println!("Error sending message: {:?}", e),
            }
        }
        ("status", Some(status_matches)) => {
            let id = status_matches.value_of("id").unwrap();
            match repo.read().unwrap().get(id) {
                Ok(Some(message)) => {
                    println!("Message ID: {}", message.id);
                    println!("Type: {:?}", message.message_type);
                    println!("Recipient: {}", message.recipient);
                    println!("Content: {}", message.content);
                    println!("Status: {:?}", message.status);
                    println!("Created At: {:?}", message.created_at);
                    println!("Updated At: {:?}", message.updated_at);
                    println!("Retry Count: {}", message.retry_count);
                }
                Ok(None) => println!("Message not found"),
                Err(e) => println!("Error getting message: {:?}", e),
            }
        }
        _ => {
            println!("Use --help to see available commands");
        }
    }
}
