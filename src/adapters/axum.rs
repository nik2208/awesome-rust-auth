use axum::{
    Json, Router,
    extract::Path,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};

use crate::ui;

async fn admin_ui() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        ui::render_admin_html(),
    )
}

async fn admin_asset(Path(asset): Path<String>) -> impl IntoResponse {
    match asset.as_str() {
        "admin.css" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
            ui::ADMIN_CSS,
        )
            .into_response(),
        "admin.js" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript")],
            ui::ADMIN_JS,
        )
            .into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn auth_ui_root() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        ui::AUTH_LOGIN_HTML,
    )
}

async fn auth_ui_tail(Path(tail): Path<String>) -> impl IntoResponse {
    if let Some((content_type, content)) = ui::auth_ui_asset(tail.trim_matches('/')) {
        return ([(header::CONTENT_TYPE, content_type)], content).into_response();
    }
    if let Some(page) = ui::auth_ui_page(tail.trim_matches('/')) {
        return ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], page).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}

async fn auth_ui_config() -> Json<serde_json::Value> {
    Json(
        serde_json::from_str(ui::AUTH_UI_CONFIG_JSON)
            .expect("AUTH_UI_CONFIG_JSON should be valid JSON"),
    )
}

async fn health() -> &'static str {
    "ok"
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/admin", get(admin_ui))
        .route("/auth/admin/assets/:asset", get(admin_asset))
        .route("/auth/ui", get(auth_ui_root))
        .route("/auth/ui/", get(auth_ui_root))
        .route("/auth/ui/config", get(auth_ui_config))
        .route("/auth/ui/*tail", get(auth_ui_tail))
        .route("/auth/login", post(health))
}
