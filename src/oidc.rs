use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::AuthConfig;

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
