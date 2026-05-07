use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub email: String,
    pub password_hash: Option<String>,
    pub oauth_accounts: Vec<LinkedOAuthAccount>,
    pub is_mfa_enabled: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedOAuthAccount {
    pub provider: String,
    pub provider_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub refresh_token_id: Uuid,
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    pub token: String,
    pub token_id: Uuid,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub label: String,
    pub scopes: Vec<String>,
    pub ip_allowlist: Vec<String>,
    pub hashed_key: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EventType {
    Login,
    Signup,
    Failure,
    Refresh,
    Logout,
    ApiKeyCreated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub tenant_id: Option<TenantId>,
    pub user_id: Option<UserId>,
    pub metadata: serde_json::Value,
    pub happened_at: OffsetDateTime,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct SignupInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 1))]
    pub tenant_id: String,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    #[validate(length(min = 1))]
    pub tenant_id: String,
}
