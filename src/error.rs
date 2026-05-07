use thiserror::Error;

pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("resource not found")]
    NotFound,
    #[error("token is invalid")]
    InvalidToken,
    #[error("token has been revoked")]
    RevokedToken,
    #[error("tenant mismatch")]
    TenantMismatch,
    #[error("config error: {0}")]
    Config(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("sandbox error: {0}")]
    Sandbox(String),
}
