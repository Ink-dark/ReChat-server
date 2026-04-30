use actix_web::{HttpResponse, Responder, web};

const INDEX_HTML: &str = include_str!("templates/index.html");
const APP_CSS: &str = include_str!("templates/app.css");
const APP_JS: &str = include_str!("templates/app.js");

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(index))
        .route("/app.css", web::get().to(app_css))
        .route("/app.js", web::get().to(app_js));
}

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX_HTML)
}

async fn app_css() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css; charset=utf-8")
        .body(APP_CSS)
}

async fn app_js() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/javascript; charset=utf-8")
        .body(APP_JS)
}
