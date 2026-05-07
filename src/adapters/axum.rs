use axum::{
    Router,
    routing::{get, post},
};

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

async fn health() -> &'static str {
    "ok"
}

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/admin", get(admin_ui))
        .route("/auth/ui", get(auth_ui))
        .route("/auth/ui/auth.js", get(auth_js))
        .route("/auth/login", post(health))
}
