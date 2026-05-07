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
    pub totp_secret: Option<String>,
    pub is_email_verified: bool,
    pub display_name: Option<String>,
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

/// Plaintext API key returned once at issuance time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedApiKey {
    pub id: Uuid,
    /// Full plaintext key – shown **once** and must be stored by the caller.
    pub plaintext_key: String,
    pub label: String,
    pub scopes: Vec<String>,
    pub created_at: OffsetDateTime,
}

/// TOTP setup data returned when a user enables TOTP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetup {
    /// Base32-encoded TOTP secret.
    pub secret: String,
    /// `otpauth://` URI suitable for QR-code rendering.
    pub uri: String,
}

/// Magic-link token record stored in the persistence layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLinkToken {
    pub token: String,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub expires_at: OffsetDateTime,
}

/// One-time SMS / email OTP record stored in the persistence layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpRecord {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    /// Argon2 / SHA-256 hash of the numeric code.
    pub code_hash: String,
    pub expires_at: OffsetDateTime,
}

/// Password-reset token stored in the persistence layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub token: String,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub expires_at: OffsetDateTime,
}

/// Email-verification / email-change token stored in the persistence layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    pub token: String,
    pub user_id: UserId,
    pub tenant_id: TenantId,
    /// `Some(email)` for email-change requests, `None` for initial verification.
    pub new_email: Option<String>,
    pub expires_at: OffsetDateTime,
}

/// Stateless CSRF token (value + HMAC signature).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub token: String,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EventType {
    Login,
    Signup,
    Failure,
    Refresh,
    Logout,
    MagicLinkRequested,
    OtpRequested,
    MfaEnabled,
    MfaDisabled,
    ProfileUpdated,
    PasswordResetRequested,
    PasswordReset,
    EmailVerificationRequested,
    EmailVerified,
    EmailChanged,
    AccountDeleted,
    AccountLinked,
    AccountUnlinked,
    ApiKeyCreated,
    ApiKeyRevoked,
    PermissionDenied,
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

/// Input for updating a user's profile fields.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct UpdateProfileInput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    #[validate(length(min = 1, max = 100))]
    pub display_name: Option<String>,
}

/// Input for changing the authenticated user's password.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct ChangePasswordInput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    #[validate(length(min = 1))]
    pub current_password: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

/// Input for requesting a password-reset email.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct RequestPasswordResetInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub tenant_id: String,
}

/// Input for completing a password reset using the one-time token.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct CompletePasswordResetInput {
    #[validate(length(min = 1))]
    pub token: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

/// Input for requesting a magic link.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct RequestMagicLinkInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub tenant_id: String,
}

/// Input for requesting an SMS / email OTP.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct RequestOtpInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub tenant_id: String,
}

/// Input for verifying an OTP code.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct VerifyOtpInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub tenant_id: String,
    #[validate(length(min = 1))]
    pub code: String,
}

/// Input for issuing a new API key.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct IssueApiKeyInput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    #[validate(length(min = 1, max = 100))]
    pub label: String,
    pub scopes: Vec<String>,
    pub ip_allowlist: Vec<String>,
}

/// Input for initiating an OAuth account link.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct InitiateLinkInput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    #[validate(length(min = 1))]
    pub provider: String,
    #[validate(length(min = 1))]
    pub provider_user_id: String,
}

/// Input for completing an OAuth account link.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct CompleteLinkInput {
    #[validate(length(min = 1))]
    pub pending_token: String,
}

/// Input for unlinking an OAuth account.
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct UnlinkAccountInput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    #[validate(length(min = 1))]
    pub provider: String,
}
