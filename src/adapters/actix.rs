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
        .body(ui::render_admin_html())
}

#[get("/auth/admin/assets/{asset}")]
async fn admin_asset(asset: web::Path<String>) -> HttpResponse {
    match asset.into_inner().as_str() {
        "admin.css" => HttpResponse::Ok()
            .content_type("text/css; charset=utf-8")
            .body(ui::ADMIN_CSS),
        "admin.js" => HttpResponse::Ok()
            .content_type("application/javascript")
            .body(ui::ADMIN_JS),
        _ => HttpResponse::NotFound().finish(),
    }
}

#[get("/auth/ui")]
async fn auth_ui_root() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(ui::AUTH_LOGIN_HTML)
}

#[get("/auth/ui/{tail:.*}")]
async fn auth_ui_tail(tail: web::Path<String>) -> HttpResponse {
    let path = tail.into_inner();
    let path = path.trim_matches('/');
    if let Some((content_type, content)) = ui::auth_ui_asset(path) {
        return HttpResponse::Ok().content_type(content_type).body(content);
    }
    if let Some(page) = ui::auth_ui_page(path) {
        return HttpResponse::Ok().content_type("text/html").body(page);
    }
    HttpResponse::NotFound().finish()
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
        .service(admin_asset)
        .service(auth_ui_root)
        .service(auth_ui_config)
        .service(auth_ui_tail)
        .service(login_stub)
}
