//! API-key (Machine-to-Machine) service.
//!
//! # Flow
//!
//! 1. An authenticated user calls [`ApiKeyManager::issue`].
//!    - A cryptographically random key is generated.
//!    - Its SHA-256 hash (prefixed with the key id) is stored via
//!      [`ApiKeyStore`].
//!    - The plaintext key is returned **once** in [`IssuedApiKey`]; the caller
//!      must relay it to the client.
//! 2. On each M2M request the client sends the key in the
//!    `Authorization: ApiKey <key>` header.
//! 3. Call [`ApiKeyManager::authenticate`] to validate the key and obtain the
//!    [`ApiKey`] metadata (scopes, ip_allowlist, …).

use std::sync::Arc;

use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    error::{AuthError, AuthResult},
    models::{ApiKey, IssueApiKeyInput, IssuedApiKey},
    traits::ApiKeyStore,
};

const KEY_BYTES: usize = 32;

/// Service for issuing and validating API keys.
#[derive(Clone)]
pub struct ApiKeyManager<K: ApiKeyStore> {
    store: Arc<K>,
}

impl<K: ApiKeyStore> ApiKeyManager<K> {
    pub fn new(store: Arc<K>) -> Self {
        Self { store }
    }

    /// Issues a new API key for the given user and returns the plaintext key.
    ///
    /// The plaintext key is **not** stored and cannot be retrieved afterwards.
    pub async fn issue(&self, input: IssueApiKeyInput) -> AuthResult<IssuedApiKey> {
        let id = Uuid::new_v4();
        let raw_key = generate_key();
        let hashed_key = hash_key(&id, &raw_key);

        let key = ApiKey {
            id,
            user_id: input.user_id,
            tenant_id: input.tenant_id,
            label: input.label.clone(),
            scopes: input.scopes.clone(),
            ip_allowlist: input.ip_allowlist,
            hashed_key,
            created_at: OffsetDateTime::now_utc(),
        };

        self.store.create_api_key(key).await?;

        Ok(IssuedApiKey {
            id,
            plaintext_key: format!("{id}.{raw_key}"),
            label: input.label,
            scopes: input.scopes,
            created_at: OffsetDateTime::now_utc(),
        })
    }

    /// Validates a plaintext API key string (format: `<uuid>.<hex-key>`) and
    /// returns the associated [`ApiKey`] metadata.
    pub async fn authenticate(&self, plaintext: &str) -> AuthResult<ApiKey> {
        let (id_str, raw_key) = plaintext
            .split_once('.')
            .ok_or(AuthError::InvalidCredentials)?;

        let id = Uuid::parse_str(id_str).map_err(|_| AuthError::InvalidCredentials)?;
        let key_record = self
            .store
            .get_api_key(&id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let expected_hash = hash_key(&id, raw_key);
        if key_record.hashed_key != expected_hash {
            return Err(AuthError::InvalidCredentials);
        }

        Ok(key_record)
    }

    /// Revokes an API key by id.
    pub async fn revoke(&self, id: &Uuid) -> AuthResult<()> {
        self.store.revoke_api_key(id).await
    }

    /// Lists all API keys for a user.
    pub async fn list_for_user(&self, user_id: &crate::models::UserId) -> AuthResult<Vec<ApiKey>> {
        self.store.list_api_keys_for_user(user_id).await
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn generate_key() -> String {
    use rand::RngCore;
    let mut bytes = [0_u8; KEY_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn hash_key(id: &Uuid, raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(b":");
    hasher.update(raw_key.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_format_roundtrip() {
        let id = Uuid::new_v4();
        let raw = "deadbeef";
        let h1 = hash_key(&id, raw);
        let h2 = hash_key(&id, raw);
        assert_eq!(h1, h2);

        let wrong_id = Uuid::new_v4();
        let h3 = hash_key(&wrong_id, raw);
        assert_ne!(h1, h3);
    }
}
