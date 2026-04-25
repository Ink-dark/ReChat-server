use actix_web::{App, HttpServer};

// 从lib中导入模块
use rechat_sender::api;
use rechat_sender::core;
use rechat_sender::web;
use rechat_sender::REPO;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载配置
    let config = core::config::Config::default();
    let db_path = config.database.path.clone();

    // 启动HTTP服务器
    HttpServer::new(move || {
        // 为每个线程初始化MessageRepository
        REPO.with(|repo| {
            if repo.borrow().is_none() {
                *repo.borrow_mut() = Some(core::message::MessageRepository::new(&db_path).unwrap());
            }
        });

        App::new()
            .service(api::routes())
            .service(web::routes())
    })
    .workers(1) // 只使用一个工作线程，避免SQLite连接的线程安全问题
    .bind((config.server.host, config.server.port))?
    .run()
    .await
}
