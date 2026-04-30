use actix_web::web;

pub mod endpoints;

pub fn config(cfg: &mut web::ServiceConfig) {
    use crate::adapters;
    use crate::api::endpoints::{messages, ws_client};

    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/messages").route(web::post().to(messages::create_message)),
            )
            .service(
                web::resource("/messages/{id}").route(web::get().to(messages::get_message)),
            )
            .service(web::resource("/health").route(web::get().to(messages::health_check)))
    )
    .route("/ws/client", web::get().to(ws_client::ws_client))
    .route(
        "/onebot/v11/ws",
        web::get().to(adapters::onebot::ws::onebot_ws),
    );
}