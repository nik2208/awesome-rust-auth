//! SMS / email OTP (One-Time Password) authentication flow.
//!
//! # Flow
//!
//! 1. User requests an OTP via [`OtpService::request`].
//!    - A random 6-digit code is generated.
//!    - Its SHA-256 hash is stored via [`OtpStore`].
//!    - The caller is responsible for delivering the plaintext code to the user
//!      (SMS gateway, transactional email, etc.).
//! 2. User submits the code via [`OtpService::verify`].
//!    - The service retrieves and consumes (deletes) the stored hash.
//!    - The code is verified by re-hashing and comparing.
//!    - On success an access/refresh token pair is issued.

use std::sync::Arc;

use rand::Rng;
use sha2::{Digest, Sha256};
use time::{Duration as TimeDuration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    config::AuthConfig,
    error::{AuthError, AuthResult},
    models::{
        AccessToken, Event, EventType, OtpRecord, RefreshToken, RequestOtpInput, Session, TenantId,
        UserId, VerifyOtpInput,
    },
    service::AuthService,
    traits::{EventBus, OtpStore, SessionStore, TelemetryStore, UserStore},
};

/// Service that drives the OTP authentication flow.
#[derive(Clone)]
pub struct OtpService<U, S, T, E, O>
where
    U: UserStore,
    S: SessionStore,
    T: TelemetryStore,
    E: EventBus,
    O: OtpStore,
{
    config: AuthConfig,
    users: Arc<U>,
    sessions: Arc<S>,
    telemetry: Arc<T>,
    events: Arc<E>,
    otps: Arc<O>,
}

impl<U, S, T, E, O> OtpService<U, S, T, E, O>
where
    U: UserStore,
    S: SessionStore,
    T: TelemetryStore,
    E: EventBus,
    O: OtpStore,
{
    const OTP_TTL: TimeDuration = TimeDuration::minutes(10);

    pub fn new(
        config: AuthConfig,
        users: Arc<U>,
        sessions: Arc<S>,
        telemetry: Arc<T>,
        events: Arc<E>,
        otps: Arc<O>,
    ) -> Self {
        Self {
            config,
            users,
            sessions,
            telemetry,
            events,
            otps,
        }
    }

    /// Generates a 6-digit OTP for the given email address and stores its hash.
    ///
    /// Returns the plaintext code so the caller can deliver it to the user.
    /// Returns `Err(AuthError::NotFound)` when no account with that email
    /// exists in the tenant.
    pub async fn request(&self, input: RequestOtpInput) -> AuthResult<String> {
        let tenant_id = TenantId(input.tenant_id.clone());
        let user = self
            .users
            .get_user_by_email(&tenant_id, &input.email)
            .await?
            .ok_or(AuthError::NotFound)?;

        let code = generate_otp();
        let code_hash = hash_code(&code);

        self.otps
            .create_otp(OtpRecord {
                user_id: user.id.clone(),
                tenant_id: tenant_id.clone(),
                code_hash,
                expires_at: OffsetDateTime::now_utc() + Self::OTP_TTL,
            })
            .await?;

        emit_event(
            &*self.telemetry,
            &*self.events,
            EventType::OtpRequested,
            Some(tenant_id),
            Some(user.id),
            serde_json::json!({"email": input.email}),
        )
        .await?;

        Ok(code)
    }

    /// Verifies an OTP code and, on success, issues an access/refresh token pair.
    pub async fn verify(&self, input: VerifyOtpInput) -> AuthResult<(AccessToken, RefreshToken)> {
        let tenant_id = TenantId(input.tenant_id.clone());
        let user = self
            .users
            .get_user_by_email(&tenant_id, &input.email)
            .await?
            .ok_or(AuthError::NotFound)?;

        let record = self
            .otps
            .consume_otp(&user.id)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if OffsetDateTime::now_utc() > record.expires_at {
            return Err(AuthError::InvalidToken);
        }

        if hash_code(&input.code) != record.code_hash {
            return Err(AuthError::InvalidCredentials);
        }

        let base = AuthService::<U, S, T, E>::bare(&self.config);
        let session = Session {
            id: Uuid::new_v4(),
            user_id: user.id.clone(),
            tenant_id: user.tenant_id.clone(),
            refresh_token_id: Uuid::new_v4(),
            expires_at: OffsetDateTime::now_utc()
                + TimeDuration::seconds(self.config.refresh_token_ttl.as_secs() as i64),
            revoked_at: None,
        };
        self.sessions.create_session(session.clone()).await?;

        let access = base.mint_access_token(&user.id, &user.tenant_id, &session.id)?;
        let refresh =
            base.mint_refresh_token(&user.id, &user.tenant_id, &session.refresh_token_id)?;

        emit_event(
            &*self.telemetry,
            &*self.events,
            EventType::Login,
            Some(user.tenant_id),
            Some(user.id),
            serde_json::json!({"method": "otp", "session_id": session.id}),
        )
        .await?;

        Ok((access, refresh))
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn generate_otp() -> String {
    let code: u32 = rand::thread_rng().gen_range(0..1_000_000);
    format!("{code:06}")
}

fn hash_code(code: &str) -> String {
    let digest = Sha256::digest(code.as_bytes());
    hex::encode(digest)
}

async fn emit_event(
    telemetry: &impl TelemetryStore,
    events: &impl EventBus,
    event_type: EventType,
    tenant_id: Option<TenantId>,
    user_id: Option<UserId>,
    metadata: serde_json::Value,
) -> AuthResult<()> {
    let event = Event {
        event_type,
        tenant_id,
        user_id,
        metadata,
        happened_at: OffsetDateTime::now_utc(),
    };
    telemetry.persist_event(event.clone()).await?;
    events.publish(event).await
}
