use axum::{Json, Router, routing::{get, post}};

use crate::ui;

async fn admin_ui() -> &'static str {
    ui::ADMIN_UI_HTML
}

async fn auth_ui() -> &'static str {
    ui::AUTH_UI_HTML
}

async fn auth_js() -> &'static str {
    ui::AUTH_JS
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
        .route("/auth/ui", get(auth_ui))
        .route("/auth/ui/auth.js", get(auth_js))
        .route("/auth/ui/config", get(auth_ui_config))
        .route("/auth/login", post(health))
}
