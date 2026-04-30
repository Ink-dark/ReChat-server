use actix_web::{Scope, web};
use crate::adapters;

pub mod endpoints;

pub fn routes() -> Scope {
    web::scope("/api")
        .service(
            web::resource("/messages").route(web::post().to(endpoints::messages::create_message)),
        )
        .service(
            web::resource("/messages/{id}").route(web::get().to(endpoints::messages::get_message)),
        )
        .service(web::resource("/health").route(web::get().to(endpoints::messages::health_check)))
}

pub fn ws_routes() -> Scope {
    web::scope("")
        .route("/ws/client", web::get().to(endpoints::ws_client::ws_client))
}

pub fn onebot_routes() -> Scope {
    web::scope("")
        .route("/onebot/v11/ws", web::get().to(adapters::onebot::ws::onebot_ws))
}
