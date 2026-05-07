use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::RngCore;
use time::{Duration as TimeDuration, OffsetDateTime};
use uuid::Uuid;
use validator::Validate;

use crate::{
    config::AuthConfig,
    error::{AuthError, AuthResult},
    models::{
        AccessToken, Event, EventType, LoginInput, RefreshToken, Session, SignupInput, TenantId,
        User, UserId,
    },
    traits::{EventBus, SessionStore, TelemetryStore, UserStore},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: String,
    tid: String,
    sid: String,
    exp: i64,
    iat: i64,
    iss: String,
    aud: String,
}

#[derive(Clone)]
pub struct AuthService<U, S, T, E>
where
    U: UserStore,
    S: SessionStore,
    T: TelemetryStore,
    E: EventBus,
{
    config: AuthConfig,
    users: Arc<U>,
    sessions: Arc<S>,
    telemetry: Arc<T>,
    events: Arc<E>,
}

impl<U, S, T, E> AuthService<U, S, T, E>
where
    U: UserStore,
    S: SessionStore,
    T: TelemetryStore,
    E: EventBus,
{
    pub fn new(
        config: AuthConfig,
        users: Arc<U>,
        sessions: Arc<S>,
        telemetry: Arc<T>,
        events: Arc<E>,
    ) -> Self {
        Self {
            config,
            users,
            sessions,
            telemetry,
            events,
        }
    }

    pub async fn signup(&self, input: SignupInput) -> AuthResult<User> {
        input
            .validate()
            .map_err(|err| AuthError::Validation(err.to_string()))?;

        let tenant_id = TenantId(input.tenant_id.clone());
        if self
            .users
            .get_user_by_email(&tenant_id, &input.email)
            .await?
            .is_some()
        {
            return Err(AuthError::Validation("email already exists".to_string()));
        }

        let salt = {
            let mut bytes = [0_u8; 16];
            rand::thread_rng().fill_bytes(&mut bytes);
            bytes
        };
        let salt = argon2::password_hash::SaltString::encode_b64(&salt)
            .map_err(|err| AuthError::Crypto(err.to_string()))?;
        let password_hash = Argon2::default()
            .hash_password(input.password.as_bytes(), &salt)
            .map_err(|err| AuthError::Crypto(err.to_string()))?
            .to_string();

        let user = User {
            id: UserId(Uuid::new_v4()),
            tenant_id,
            email: input.email,
            password_hash: Some(password_hash),
            oauth_accounts: Vec::new(),
            is_mfa_enabled: false,
            created_at: OffsetDateTime::now_utc(),
        };

        let created = self.users.create_user(user).await?;
        self.emit_event(
            EventType::Signup,
            Some(created.tenant_id.clone()),
            Some(created.id.clone()),
            serde_json::json!({"email": created.email}),
        )
        .await?;
        Ok(created)
    }

    pub async fn login(&self, input: LoginInput) -> AuthResult<(AccessToken, RefreshToken)> {
        input
            .validate()
            .map_err(|err| AuthError::Validation(err.to_string()))?;

        let tenant_id = TenantId(input.tenant_id.clone());
        let user = self
            .users
            .get_user_by_email(&tenant_id, &input.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;
        let hash = user
            .password_hash
            .clone()
            .ok_or(AuthError::InvalidCredentials)?;

        let parsed = PasswordHash::new(&hash).map_err(|err| AuthError::Crypto(err.to_string()))?;
        Argon2::default()
            .verify_password(input.password.as_bytes(), &parsed)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let now = OffsetDateTime::now_utc();
        let session = Session {
            id: Uuid::new_v4(),
            user_id: user.id.clone(),
            tenant_id: user.tenant_id.clone(),
            refresh_token_id: Uuid::new_v4(),
            expires_at: now + TimeDuration::seconds(self.config.refresh_token_ttl.as_secs() as i64),
            revoked_at: None,
        };
        self.sessions.create_session(session.clone()).await?;

        let access = self.issue_token(
            &user.id,
            &user.tenant_id,
            &session.id,
            self.config.access_token_ttl.as_secs() as i64,
        )?;
        let refresh = self.issue_refresh_token(
            &user.id,
            &user.tenant_id,
            &session.refresh_token_id,
            self.config.refresh_token_ttl.as_secs() as i64,
        )?;

        self.emit_event(
            EventType::Login,
            Some(user.tenant_id),
            Some(user.id),
            serde_json::json!({"session_id": session.id}),
        )
        .await?;
        Ok((access, refresh))
    }

    pub async fn rotate_refresh_token(
        &self,
        token: &str,
    ) -> AuthResult<(AccessToken, RefreshToken)> {
        let claims = self.decode_claims(token)?;
        let refresh_id = Uuid::parse_str(&claims.sid).map_err(|_| AuthError::InvalidToken)?;
        let existing = self
            .sessions
            .get_session_by_refresh_token(&refresh_id)
            .await?
            .ok_or(AuthError::InvalidToken)?;
        if existing.revoked_at.is_some() {
            return Err(AuthError::RevokedToken);
        }

        self.sessions.revoke_session(&existing.id).await?;

        let new_session = Session {
            id: Uuid::new_v4(),
            user_id: existing.user_id.clone(),
            tenant_id: existing.tenant_id.clone(),
            refresh_token_id: Uuid::new_v4(),
            expires_at: OffsetDateTime::now_utc()
                + TimeDuration::seconds(self.config.refresh_token_ttl.as_secs() as i64),
            revoked_at: None,
        };
        self.sessions.create_session(new_session.clone()).await?;

        let access = self.issue_token(
            &new_session.user_id,
            &new_session.tenant_id,
            &new_session.id,
            self.config.access_token_ttl.as_secs() as i64,
        )?;
        let refresh = self.issue_refresh_token(
            &new_session.user_id,
            &new_session.tenant_id,
            &new_session.refresh_token_id,
            self.config.refresh_token_ttl.as_secs() as i64,
        )?;

        self.emit_event(
            EventType::Refresh,
            Some(new_session.tenant_id),
            Some(new_session.user_id),
            serde_json::json!({"old_session_id": existing.id, "new_session_id": new_session.id}),
        )
        .await?;
        Ok((access, refresh))
    }

    pub async fn revoke_refresh_token(&self, refresh_token_id: Uuid) -> AuthResult<()> {
        let session = self
            .sessions
            .get_session_by_refresh_token(&refresh_token_id)
            .await?
            .ok_or(AuthError::NotFound)?;
        self.sessions.revoke_session(&session.id).await?;
        self.emit_event(
            EventType::Logout,
            Some(session.tenant_id),
            Some(session.user_id),
            serde_json::json!({"session_id": session.id}),
        )
        .await
    }

    fn issue_token(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        session_id: &Uuid,
        ttl_secs: i64,
    ) -> AuthResult<AccessToken> {
        let now = OffsetDateTime::now_utc();
        let exp = now + TimeDuration::seconds(ttl_secs);
        let claims = Claims {
            sub: user_id.0.to_string(),
            tid: tenant_id.0.clone(),
            sid: session_id.to_string(),
            exp: exp.unix_timestamp(),
            iat: now.unix_timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|err| AuthError::Crypto(err.to_string()))?;

        Ok(AccessToken {
            token,
            expires_at: exp,
        })
    }

    fn issue_refresh_token(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        refresh_token_id: &Uuid,
        ttl_secs: i64,
    ) -> AuthResult<RefreshToken> {
        let now = OffsetDateTime::now_utc();
        let exp = now + TimeDuration::seconds(ttl_secs);
        let claims = Claims {
            sub: user_id.0.to_string(),
            tid: tenant_id.0.clone(),
            sid: refresh_token_id.to_string(),
            exp: exp.unix_timestamp(),
            iat: now.unix_timestamp(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|err| AuthError::Crypto(err.to_string()))?;

        Ok(RefreshToken {
            token,
            token_id: *refresh_token_id,
            expires_at: exp,
        })
    }

    fn decode_claims(&self, token: &str) -> AuthResult<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(std::slice::from_ref(&self.config.audience));
        validation.set_issuer(std::slice::from_ref(&self.config.issuer));
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|_| AuthError::InvalidToken)
    }

    async fn emit_event(
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
