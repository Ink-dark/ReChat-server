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
            .route("/auth/verify", web::post().to(auth_verify))
    )
    .route("/ws/client", web::get().to(ws_client::ws_client))
    .route(
        "/onebot/v11/ws",
        web::get().to(adapters::onebot::ws::onebot_ws),
    );
}

async fn auth_verify(
    body: web::Json<serde_json::Value>,
    token: web::Data<String>,
) -> impl actix_web::Responder {
    let provided = body
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if token.get_ref() == provided {
        actix_web::HttpResponse::Ok().json(serde_json::json!({"valid": true}))
    } else {
        actix_web::HttpResponse::Unauthorized()
            .json(serde_json::json!({"valid": false, "error": "Invalid token"}))
    }
}
