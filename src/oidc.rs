use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::{
    config::AuthConfig,
    error::{AuthError, AuthResult},
    models::{TenantId, UserId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub response_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    pub alg: String,
    pub k: String,
    pub kid: String,
    pub r#use: String,
}

/// Standard OIDC ID-token claims (subset used by this crate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdTokenClaims {
    /// Issuer.
    pub iss: String,
    /// Subject (user id).
    pub sub: String,
    /// Audience.
    pub aud: String,
    /// Expiration time (Unix timestamp).
    pub exp: i64,
    /// Issued at (Unix timestamp).
    pub iat: i64,
    /// Tenant identifier (custom claim).
    pub tid: String,
    /// User email, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Whether the email has been verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
}

pub fn discovery(config: &AuthConfig, base_url: &str) -> OidcDiscovery {
    OidcDiscovery {
        issuer: config.issuer.clone(),
        authorization_endpoint: format!("{base_url}/oidc/authorize"),
        token_endpoint: format!("{base_url}/oidc/token"),
        userinfo_endpoint: format!("{base_url}/oidc/userinfo"),
        jwks_uri: format!("{base_url}/oidc/jwks"),
        response_types_supported: vec!["code".to_string()],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec!["HS256".to_string()],
    }
}

pub fn jwks(config: &AuthConfig) -> JwkSet {
    let key_fingerprint = Sha256::digest(config.jwt_secret.as_bytes());
    JwkSet {
        keys: vec![Jwk {
            kty: "oct".to_string(),
            alg: "HS256".to_string(),
            k: URL_SAFE_NO_PAD.encode(key_fingerprint),
            kid: "main".to_string(),
            r#use: "sig".to_string(),
        }],
    }
}

/// Issues a signed OIDC ID token for the given user.
///
/// The token is signed with the same HS256 key used for access tokens and
/// contains the standard `iss`, `sub`, `aud`, `exp`, `iat` claims plus the
/// custom `tid` (tenant) claim and optional email/email_verified claims.
pub fn issue_id_token(
    config: &AuthConfig,
    user_id: &UserId,
    tenant_id: &TenantId,
    email: Option<&str>,
    email_verified: Option<bool>,
    ttl_secs: i64,
) -> AuthResult<String> {
    let now = OffsetDateTime::now_utc();
    let claims = IdTokenClaims {
        iss: config.issuer.clone(),
        sub: user_id.0.to_string(),
        aud: config.audience.clone(),
        exp: (now + time::Duration::seconds(ttl_secs)).unix_timestamp(),
        iat: now.unix_timestamp(),
        tid: tenant_id.0.clone(),
        email: email.map(str::to_string),
        email_verified,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|err| AuthError::Crypto(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthConfig,
        models::{TenantId, UserId},
    };
    use uuid::Uuid;

    fn config() -> AuthConfig {
        AuthConfig::builder()
            .jwt_secret("a-secret-that-is-long-enough")
            .build()
            .unwrap()
    }

    #[test]
    fn discovery_endpoints_use_base_url() {
        let cfg = config();
        let disc = discovery(&cfg, "https://auth.example.com");
        assert_eq!(disc.token_endpoint, "https://auth.example.com/oidc/token");
        assert_eq!(disc.jwks_uri, "https://auth.example.com/oidc/jwks");
    }

    #[test]
    fn id_token_is_issued_successfully() {
        let cfg = config();
        let uid = UserId(Uuid::new_v4());
        let tid = TenantId("tenant1".to_string());
        let token =
            issue_id_token(&cfg, &uid, &tid, Some("u@example.com"), Some(true), 3600).unwrap();
        assert!(!token.is_empty());
    }
}
