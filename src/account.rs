//! Account management service.
//!
//! Provides higher-level operations on top of [`UserStore`]:
//!
//! - Profile update (display name)
//! - Password change (requires current password)
//! - Password reset (two-step: request token → complete with new password)
//! - Email verification (request token → verify)
//! - Email change (request token with new address → verify)
//! - Account deletion

use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::RngCore;
use time::{Duration as TimeDuration, OffsetDateTime};

use crate::{
    config::AuthConfig,
    error::{AuthError, AuthResult},
    models::{
        ChangePasswordInput, CompletePasswordResetInput, EmailVerificationToken, Event, EventType,
        PasswordResetToken, RequestPasswordResetInput, TenantId, UpdateProfileInput, User, UserId,
    },
    traits::{EmailVerificationStore, EventBus, PasswordResetStore, TelemetryStore, UserStore},
};

/// Service for account lifecycle operations.
#[derive(Clone)]
pub struct AccountService<U, T, E, P, V>
where
    U: UserStore,
    T: TelemetryStore,
    E: EventBus,
    P: PasswordResetStore,
    V: EmailVerificationStore,
{
    _config: AuthConfig,
    users: Arc<U>,
    telemetry: Arc<T>,
    events: Arc<E>,
    password_resets: Arc<P>,
    email_verifications: Arc<V>,
}

impl<U, T, E, P, V> AccountService<U, T, E, P, V>
where
    U: UserStore,
    T: TelemetryStore,
    E: EventBus,
    P: PasswordResetStore,
    V: EmailVerificationStore,
{
    const PASSWORD_RESET_TTL: TimeDuration = TimeDuration::hours(1);
    const EMAIL_VERIFY_TTL: TimeDuration = TimeDuration::hours(24);

    pub fn new(
        config: AuthConfig,
        users: Arc<U>,
        telemetry: Arc<T>,
        events: Arc<E>,
        password_resets: Arc<P>,
        email_verifications: Arc<V>,
    ) -> Self {
        Self {
            _config: config,
            users,
            telemetry,
            events,
            password_resets,
            email_verifications,
        }
    }

    // ── Profile ──────────────────────────────────────────────────────────────

    /// Updates mutable profile fields (currently: display name).
    pub async fn update_profile(&self, input: UpdateProfileInput) -> AuthResult<User> {
        let user = self
            .users
            .update_profile(&input.user_id, input.display_name.as_deref())
            .await?;

        self.emit(
            EventType::ProfileUpdated,
            Some(input.tenant_id),
            Some(input.user_id),
            serde_json::json!({"action": "profile_updated"}),
        )
        .await?;

        Ok(user)
    }

    // ── Password change ───────────────────────────────────────────────────────

    /// Changes the user's password after verifying the current one.
    pub async fn change_password(&self, input: ChangePasswordInput) -> AuthResult<()> {
        let user = self
            .users
            .get_user_by_id(&input.user_id)
            .await?
            .ok_or(AuthError::NotFound)?;

        let hash = user.password_hash.ok_or(AuthError::InvalidCredentials)?;
        let parsed = PasswordHash::new(&hash).map_err(|err| AuthError::Crypto(err.to_string()))?;
        Argon2::default()
            .verify_password(input.current_password.as_bytes(), &parsed)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let new_hash = hash_password(&input.new_password)?;
        self.users
            .set_password_hash(&input.user_id, &new_hash)
            .await?;

        self.emit(
            EventType::PasswordReset,
            Some(input.tenant_id),
            Some(input.user_id),
            serde_json::json!({"action": "password_changed"}),
        )
        .await
    }

    // ── Password reset (two-step) ─────────────────────────────────────────────

    /// Step 1: generates and stores a password-reset token.
    ///
    /// Returns the raw token that must be sent to the user via email.
    /// Returns `Err(AuthError::NotFound)` when no account with that email exists.
    pub async fn request_password_reset(
        &self,
        input: RequestPasswordResetInput,
    ) -> AuthResult<String> {
        let tenant_id = TenantId(input.tenant_id.clone());
        let user = self
            .users
            .get_user_by_email(&tenant_id, &input.email)
            .await?
            .ok_or(AuthError::NotFound)?;

        let token = generate_token();
        self.password_resets
            .create_password_reset(PasswordResetToken {
                token: token.clone(),
                user_id: user.id.clone(),
                tenant_id: tenant_id.clone(),
                expires_at: OffsetDateTime::now_utc() + Self::PASSWORD_RESET_TTL,
            })
            .await?;

        self.emit(
            EventType::PasswordResetRequested,
            Some(tenant_id),
            Some(user.id),
            serde_json::json!({"email": input.email}),
        )
        .await?;

        Ok(token)
    }

    /// Step 2: validates the token and sets the new password.
    pub async fn complete_password_reset(
        &self,
        input: CompletePasswordResetInput,
    ) -> AuthResult<()> {
        let record = self
            .password_resets
            .consume_password_reset(&input.token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if OffsetDateTime::now_utc() > record.expires_at {
            return Err(AuthError::InvalidToken);
        }

        let new_hash = hash_password(&input.new_password)?;
        self.users
            .set_password_hash(&record.user_id, &new_hash)
            .await?;

        self.emit(
            EventType::PasswordReset,
            Some(record.tenant_id),
            Some(record.user_id),
            serde_json::json!({"action": "password_reset"}),
        )
        .await
    }

    // ── Email verification ────────────────────────────────────────────────────

    /// Generates an email verification token for a newly registered user.
    ///
    /// Returns the raw token to be embedded in the verification link.
    pub async fn request_email_verification(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> AuthResult<String> {
        let token = generate_token();
        self.email_verifications
            .create_email_verification(EmailVerificationToken {
                token: token.clone(),
                user_id: user_id.clone(),
                tenant_id: tenant_id.clone(),
                new_email: None,
                expires_at: OffsetDateTime::now_utc() + Self::EMAIL_VERIFY_TTL,
            })
            .await?;

        self.emit(
            EventType::EmailVerificationRequested,
            Some(tenant_id),
            Some(user_id),
            serde_json::json!({}),
        )
        .await?;

        Ok(token)
    }

    /// Verifies the email-verification token and marks the account as verified.
    pub async fn verify_email(&self, token: &str) -> AuthResult<()> {
        let record = self
            .email_verifications
            .consume_email_verification(token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if OffsetDateTime::now_utc() > record.expires_at {
            return Err(AuthError::InvalidToken);
        }

        self.users.set_email_verified(&record.user_id).await?;

        self.emit(
            EventType::EmailVerified,
            Some(record.tenant_id),
            Some(record.user_id),
            serde_json::json!({}),
        )
        .await
    }

    // ── Email change ──────────────────────────────────────────────────────────

    /// Generates an email-change verification token for the new address.
    ///
    /// The new email is stored in the token record.  Call
    /// [`AccountService::confirm_email_change`] when the user clicks the link.
    pub async fn request_email_change(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
        new_email: String,
    ) -> AuthResult<String> {
        let token = generate_token();
        self.email_verifications
            .create_email_verification(EmailVerificationToken {
                token: token.clone(),
                user_id: user_id.clone(),
                tenant_id: tenant_id.clone(),
                new_email: Some(new_email),
                expires_at: OffsetDateTime::now_utc() + Self::EMAIL_VERIFY_TTL,
            })
            .await?;

        self.emit(
            EventType::EmailVerificationRequested,
            Some(tenant_id),
            Some(user_id),
            serde_json::json!({"action": "email_change_requested"}),
        )
        .await?;

        Ok(token)
    }

    /// Confirms an email change by consuming the token and updating the address.
    pub async fn confirm_email_change(&self, token: &str) -> AuthResult<()> {
        let record = self
            .email_verifications
            .consume_email_verification(token)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if OffsetDateTime::now_utc() > record.expires_at {
            return Err(AuthError::InvalidToken);
        }

        let new_email = record.new_email.ok_or_else(|| {
            AuthError::Validation("token is not an email-change token".to_string())
        })?;

        self.users.update_email(&record.user_id, &new_email).await?;

        self.emit(
            EventType::EmailChanged,
            Some(record.tenant_id),
            Some(record.user_id),
            serde_json::json!({"new_email": new_email}),
        )
        .await
    }

    // ── Account deletion ──────────────────────────────────────────────────────

    /// Permanently deletes the user account.
    pub async fn delete_account(&self, user_id: UserId, tenant_id: TenantId) -> AuthResult<()> {
        self.users.delete_user(&user_id).await?;

        self.emit(
            EventType::AccountDeleted,
            Some(tenant_id),
            Some(user_id),
            serde_json::json!({}),
        )
        .await
    }

    // ── internal ─────────────────────────────────────────────────────────────

    async fn emit(
        &self,
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
        self.telemetry.persist_event(event.clone()).await?;
        self.events.publish(event).await
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn hash_password(password: &str) -> AuthResult<String> {
    let mut salt_bytes = [0_u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt = argon2::password_hash::SaltString::encode_b64(&salt_bytes)
        .map_err(|err| AuthError::Crypto(err.to_string()))?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|err| AuthError::Crypto(err.to_string()))
}
