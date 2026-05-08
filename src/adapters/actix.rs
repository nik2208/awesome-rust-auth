use actix_web::{HttpResponse, Scope, get, post, web};

use crate::ui;

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().body("ok")
}

#[get("/auth/admin")]
async fn admin_ui() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ui::ADMIN_UI_HTML)
}

#[get("/auth/ui")]
async fn auth_ui() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ui::AUTH_UI_HTML)
}

#[get("/auth/ui/auth.js")]
async fn auth_js() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(ui::AUTH_JS)
}

#[get("/auth/ui/config")]
async fn auth_ui_config() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(ui::AUTH_UI_CONFIG_JSON)
}

#[post("/auth/login")]
async fn login_stub() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn scope() -> Scope {
    web::scope("")
        .service(health)
        .service(admin_ui)
        .service(auth_ui)
        .service(auth_js)
        .service(auth_ui_config)
        .service(login_stub)
}
