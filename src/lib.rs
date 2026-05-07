#![deny(unsafe_code)]

pub mod api_contract;
pub mod config;
pub mod error;
pub mod event_bus;
pub mod mail;
pub mod models;
pub mod oidc;
pub mod openapi;
pub mod service;
pub mod traits;
pub mod ui;
pub mod webhook;

pub use api_contract::{AuthApiCompatibilityNotes, LoginRequest, LoginResponse, SignupRequest};
pub use config::{AuthConfig, AuthConfigBuilder};
pub use error::{AuthError, AuthResult};
pub use models::{
    AccessToken, ApiKey, Event, EventType, RefreshToken, Session, TenantId, User, UserId,
};
pub use service::AuthService;

pub mod adapters {
    #[cfg(feature = "axum")]
    pub mod axum;

    #[cfg(feature = "actix")]
    pub mod actix;

    #[cfg(feature = "warp")]
    pub mod warp;
}

#[cfg(test)]
mod tests;
