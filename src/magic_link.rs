//! Magic-link authentication flow.
//!
//! # Flow
//!
//! 1. User requests a magic link via [`MagicLinkService::request`].
//!    - The service looks up the user by email.
//!    - A short-lived signed token is generated and stored via [`MagicLinkStore`].
//!    - The caller is responsible for sending the token to the user (e.g. via
//!      [`crate::mail::MailTemplateEngine`]).
//! 2. User clicks the link; the application calls [`MagicLinkService::verify`]
//!    with the token from the URL.
//!    - The service consumes the token (single-use) and issues an access/refresh
//!      token pair exactly as the password login flow does.

use std::sync::Arc;

use rand::RngCore;
use time::{Duration as TimeDuration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    config::AuthConfig,
    error::{AuthError, AuthResult},
    models::{
        AccessToken, Event, EventType, MagicLinkToken, RefreshToken, RequestMagicLinkInput,
        Session, TenantId, UserId,
    },
    service::AuthService,
    traits::{EventBus, MagicLinkStore, SessionStore, TelemetryStore, UserStore},
};

/// Service that drives the magic-link authentication flow.
#[derive(Clone)]
pub struct MagicLinkService<U, S, T, E, M>
where
    U: UserStore,
    S: SessionStore,
    T: TelemetryStore,
    E: EventBus,
    M: MagicLinkStore,
{
    config: AuthConfig,
    users: Arc<U>,
    sessions: Arc<S>,
    telemetry: Arc<T>,
    events: Arc<E>,
    magic_links: Arc<M>,
}

impl<U, S, T, E, M> MagicLinkService<U, S, T, E, M>
where
    U: UserStore,
    S: SessionStore,
    T: TelemetryStore,
    E: EventBus,
    M: MagicLinkStore,
{
    /// TTL for magic-link tokens.
    const MAGIC_LINK_TTL: TimeDuration = TimeDuration::minutes(15);

    pub fn new(
        config: AuthConfig,
        users: Arc<U>,
        sessions: Arc<S>,
        telemetry: Arc<T>,
        events: Arc<E>,
        magic_links: Arc<M>,
    ) -> Self {
        Self {
            config,
            users,
            sessions,
            telemetry,
            events,
            magic_links,
        }
    }

    /// Generates and stores a magic-link token for the given email address.
    ///
    /// Returns the raw token string that must be embedded in the link sent to
    /// the user.  Returns `Err(AuthError::NotFound)` when no account with that
    /// email exists in the tenant.
    pub async fn request(&self, input: RequestMagicLinkInput) -> AuthResult<String> {
        let tenant_id = TenantId(input.tenant_id.clone());
        let user = self
            .users
            .get_user_by_email(&tenant_id, &input.email)
            .await?
            .ok_or(AuthError::NotFound)?;

        let token_str = generate_secure_token();
        let expires_at = OffsetDateTime::now_utc() + Self::MAGIC_LINK_TTL;

        self.magic_links
            .create_magic_link(MagicLinkToken {
                token: token_str.clone(),
                user_id: user.id.clone(),
                tenant_id: tenant_id.clone(),
                expires_at,
            })
            .await?;

        emit_event(
            &*self.telemetry,
            &*self.events,
            EventType::MagicLinkRequested,
            Some(tenant_id),
            Some(user.id),
            serde_json::json!({"email": input.email}),
        )
        .await?;

        Ok(token_str)
    }

    /// Verifies a magic-link token and, on success, issues an access/refresh
    /// token pair.
    pub async fn verify(&self, token: &str) -> AuthResult<(AccessToken, RefreshToken)> {
        let ml = self
            .magic_links
            .consume_magic_link(token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if OffsetDateTime::now_utc() > ml.expires_at {
            return Err(AuthError::InvalidToken);
        }

        let user = self
            .users
            .get_user_by_id(&ml.user_id)
            .await?
            .ok_or(AuthError::NotFound)?;

        let minter = AuthService::<U, S, T, E>::bare(&self.config);
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

        let access = minter.mint_access_token(&user.id, &user.tenant_id, &session.id)?;
        let refresh =
            minter.mint_refresh_token(&user.id, &user.tenant_id, &session.refresh_token_id)?;

        emit_event(
            &*self.telemetry,
            &*self.events,
            EventType::Login,
            Some(user.tenant_id),
            Some(user.id),
            serde_json::json!({"method": "magic_link", "session_id": session.id}),
        )
        .await?;

        Ok((access, refresh))
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn generate_secure_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
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
