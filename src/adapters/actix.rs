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
        .service(login_stub)
}
