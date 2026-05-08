use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthApiCompatibilityNotes {
    pub contract_target: String,
    pub cookie_conventions: String,
    pub bearer_conventions: String,
    pub known_deviations: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "Created"),
        (status = 400, description = "Invalid input")
    )
)]
pub fn signup_openapi() {}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub fn login_openapi() {}

pub fn compatibility_notes() -> AuthApiCompatibilityNotes {
    AuthApiCompatibilityNotes {
        contract_target: "ng-awesome-node-auth + awesome-node-auth-flutter".to_string(),
        cookie_conventions: "HTTP-only refresh cookie + bearer access token".to_string(),
        bearer_conventions: "Authorization: Bearer <access_token>".to_string(),
        known_deviations: vec![
            "OAuth provider-specific payload shape remains TODO in this initial crate skeleton".to_string(),
            "Inbound webhook action decorators are scaffolded via Wasmtime runner, action DSL parity is pending".to_string(),
            "Built-in UI runtime parity is partial: /auth/ui, /auth/ui/auth.js, and /auth/ui/config are available, but full headless/runtime parity is pending".to_string(),
            "Dynamic template-store parity is partial: built-in localized templates are bundled, but full mail/UI i18n store management parity is pending".to_string(),
        ],
    }
}
