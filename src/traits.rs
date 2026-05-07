use async_trait::async_trait;

use crate::{error::AuthResult, models::*};

/// Persistence contract for user lifecycle operations.
#[async_trait]
pub trait UserStore: Send + Sync {
    /// Creates a user record.
    async fn create_user(&self, user: User) -> AuthResult<User>;
    /// Finds a user by tenant and email.
    async fn get_user_by_email(
        &self,
        tenant_id: &TenantId,
        email: &str,
    ) -> AuthResult<Option<User>>;
    /// Finds a user by id.
    async fn get_user_by_id(&self, user_id: &UserId) -> AuthResult<Option<User>>;
    /// Persists account links.
    async fn update_oauth_links(
        &self,
        user_id: &UserId,
        links: Vec<LinkedOAuthAccount>,
    ) -> AuthResult<()>;
}

/// Persistence contract for refresh-token-backed sessions.
#[async_trait]
pub trait SessionStore: Send + Sync {
    /// Saves a newly-issued session.
    async fn create_session(&self, session: Session) -> AuthResult<()>;
    /// Finds a session by refresh token id.
    async fn get_session_by_refresh_token(
        &self,
        refresh_token_id: &uuid::Uuid,
    ) -> AuthResult<Option<Session>>;
    /// Revokes the active session.
    async fn revoke_session(&self, session_id: &uuid::Uuid) -> AuthResult<()>;
}

/// Persistence contract for API key issuance and validation.
#[async_trait]
pub trait ApiKeyStore: Send + Sync {
    /// Stores a generated API key hash.
    async fn create_api_key(&self, key: ApiKey) -> AuthResult<()>;
    /// Looks up an API key by id.
    async fn get_api_key(&self, id: &uuid::Uuid) -> AuthResult<Option<ApiKey>>;
    /// Revokes an API key.
    async fn revoke_api_key(&self, id: &uuid::Uuid) -> AuthResult<()>;
}

/// Persistence contract for role and permission checks in a tenant context.
#[async_trait]
pub trait RolesPermissionsStore: Send + Sync {
    /// Returns true when user has every required permission in the tenant.
    async fn user_has_permissions(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        required: &[String],
    ) -> AuthResult<bool>;
}

/// Persistence contract for pending account-link flows.
#[async_trait]
pub trait PendingLinkStore: Send + Sync {
    /// Creates a pending link token for a user/provider pair.
    async fn create_pending_link(
        &self,
        user_id: &UserId,
        provider: &str,
        provider_user_id: &str,
    ) -> AuthResult<String>;
    /// Consumes and resolves a pending link token.
    async fn consume_pending_link(
        &self,
        token: &str,
    ) -> AuthResult<Option<(UserId, LinkedOAuthAccount)>>;
}

/// Storage contract for auth telemetry persistence.
#[async_trait]
pub trait TelemetryStore: Send + Sync {
    /// Persists an auth domain event.
    async fn persist_event(&self, event: Event) -> AuthResult<()>;
}

/// Fanout contract for distributed SSE notifications.
#[async_trait]
pub trait SseDistributor: Send + Sync {
    /// Publishes an event for connected subscribers.
    async fn publish(&self, event: &Event) -> AuthResult<()>;
}

/// Event bus abstraction for internal pub/sub.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publishes an event to subscribers.
    async fn publish(&self, event: Event) -> AuthResult<()>;
}
