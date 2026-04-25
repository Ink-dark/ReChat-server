use actix_web::{App, HttpServer};
use clap::{App as ClapApp, Arg};
use std::sync::Arc;

// 从lib中导入模块
use rechat_sender::REPO;
use rechat_sender::api;
use rechat_sender::core;
use rechat_sender::web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 解析命令行参数
    let _matches = ClapApp::new("rechat-sender")
        .version("0.1.0")
        .about("ReChat message sender server")
        .arg(
            Arg::with_name("server")
                .long("server")
                .required(true)
                .help("Start the server"),
        )
        .get_matches();

    // 加载配置
    let config = core::config::Config::default();
    let db_path = config.database.path.clone();

    // 初始化适配器管理器
    let adapter_manager = Arc::new(core::adapter::AdapterManager::new());
    
    // 初始化插件管理器
    let plugin_manager = Arc::new(core::plugin::PluginManager::new());

    // 启动适配器
    if let Err(e) = adapter_manager.start_all() {
        eprintln!("Failed to start adapters: {}", e);
    }

    // 初始化插件
    if let Err(e) = plugin_manager.initialize_all() {
        eprintln!("Failed to initialize plugins: {}", e);
    }

    // 启动HTTP服务器
    HttpServer::new(move || {
        // 为每个线程初始化MessageRepository
        REPO.with(|repo| {
            if repo.borrow().is_none() {
                *repo.borrow_mut() = Some(core::message::MessageRepository::new(&db_path).unwrap());
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
