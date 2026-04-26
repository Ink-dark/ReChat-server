#![cfg_attr(feature = "windows-gui", windows_subsystem = "windows")]

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
    core::logging::init();

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
            tracing::warn!(
                config_path = %config_path,
                error = %e,
                "Failed to load config, using default"
            );
            core::config::Config::default()
        })
    } else {
        core::config::Config::default()
    };

    let db_path = config.database.path.clone();

    let adapter_manager = Arc::new(core::adapter::AdapterManager::new());
    let plugin_manager = Arc::new(core::plugin::PluginManager::new());
    let broadcaster = core::broadcaster::MessageBroadcaster::new();

    if let Err(e) = adapter_manager.start_all() {
        tracing::error!(error = %e, "Failed to start adapters");
    }
    if let Err(e) = plugin_manager.initialize_all() {
        tracing::error!(error = %e, "Failed to initialize plugins");
    }

    tracing::info!(
        host = %config.server.host,
        port = config.server.port,
        workers = config.server.workers,
        "Starting ReChat sender server"
    );

    HttpServer::new(move || {
        REPO.with(|repo| {
            if repo.borrow().is_none() {
                match core::message::MessageRepository::new(&db_path) {
                    Ok(r) => *repo.borrow_mut() = Some(r),
                    Err(e) => {
                        tracing::error!(error = %e, path = %db_path, "Failed to initialize message repository");
                    }
                }
            }
        });

        App::new()
            .app_data(actix_web::web::Data::new(adapter_manager.clone()))
            .app_data(actix_web::web::Data::new(plugin_manager.clone()))
            .app_data(actix_web::web::Data::new(broadcaster.clone()))
            .service(api::ws_routes())
            .service(api::routes())
            .service(web::routes())
    })
    .workers(config.server.workers)
    .bind((config.server.host, config.server.port))?
    .run()
    .await
}
