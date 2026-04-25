use actix_web::{web, Scope};

pub mod endpoints;

pub fn routes() -> Scope {
    web::scope("/api")
        .service(web::resource("/messages").route(web::post().to(endpoints::messages::create_message)))
        .service(web::resource("/messages/{id}").route(web::get().to(endpoints::messages::get_message)))
        .service(web::resource("/health").route(web::get().to(endpoints::messages::health_check)))
}
