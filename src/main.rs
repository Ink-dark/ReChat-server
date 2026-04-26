use actix_web::{App, HttpServer};
use clap::{App as ClapApp, Arg};
use std::path::Path;
use std::sync::Arc;

use rechat_sender::REPO;
use rechat_sender::api;
use rechat_sender::core;
use rechat_sender::web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = ClapApp::new("rechat-sender")
        .version("0.1.0")
        .about("ReChat message sender server")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("Path to configuration file (JSON)"),
        )
        .get_matches();

    let config = if let Some(config_path) = matches.value_of("config") {
        core::config::Config::load(Path::new(config_path)).unwrap_or_else(|e| {
            eprintln!(
                "Warning: failed to load config from {}: {}. Using default config.",
                config_path, e
            );
            core::config::Config::default()
        })
    } else {
        core::config::Config::default()
    };

    let db_path = config.database.path.clone();

    let adapter_manager = Arc::new(core::adapter::AdapterManager::new());
    let plugin_manager = Arc::new(core::plugin::PluginManager::new());

    if let Err(e) = adapter_manager.start_all() {
        eprintln!("Failed to start adapters: {}", e);
    }
    if let Err(e) = plugin_manager.initialize_all() {
        eprintln!("Failed to initialize plugins: {}", e);
    }

    HttpServer::new(move || {
        REPO.with(|repo| {
            if repo.borrow().is_none() {
                match core::message::MessageRepository::new(&db_path) {
                    Ok(r) => *repo.borrow_mut() = Some(r),
                    Err(e) => eprintln!("Failed to initialize message repository: {}", e),
                }
            }
        });

        App::new()
            .app_data(actix_web::web::Data::new(adapter_manager.clone()))
            .app_data(actix_web::web::Data::new(plugin_manager.clone()))
            .service(api::routes())
            .service(web::routes())
    })
    .workers(config.server.workers)
    .bind((config.server.host, config.server.port))?
    .run()
    .await
}
