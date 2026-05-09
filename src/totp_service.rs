//! TOTP (Time-based One-Time Password) helpers.
//!
//! Wraps `totp-rs` to provide setup (secret generation + QR URI) and
//! verification.  The secret is stored encrypted at rest in the user record
//! via [`UserStore::set_totp`].
//!
//! # Flow
//!
//! 1. Call [`setup_totp`] → return [`TotpSetup`] to the client.
//! 2. Client scans the QR-code URI with an authenticator app and sends back
//!    the first 6-digit code.
//! 3. Call [`verify_totp_code`] to confirm the code before persisting the
//!    secret.
//! 4. Persist the secret via `UserStore::set_totp(user_id, Some(&secret))`.
//! 5. On every login with MFA, call [`verify_totp_code`] before issuing
//!    tokens.

use totp_rs::{Algorithm, Secret, TOTP};

use crate::{
    error::{AuthError, AuthResult},
    models::TotpSetup,
};

const TOTP_DIGITS: usize = 6;
const TOTP_STEP: u64 = 30;

/// Generates a fresh TOTP secret and the corresponding `otpauth://` URI.
///
/// `account_name` is displayed in the authenticator app (typically the user's
/// email address). `issuer` is the application name shown in the app.
pub fn setup_totp(account_name: &str, issuer: &str) -> AuthResult<TotpSetup> {
    let secret = Secret::generate_secret();
    let secret_b32 = secret.to_encoded().to_string();

    let totp = TOTP::new(
        Algorithm::SHA1,
        TOTP_DIGITS,
        1,
        TOTP_STEP,
        secret
            .to_bytes()
            .map_err(|err| AuthError::Crypto(format!("totp secret: {err}")))?,
        Some(issuer.to_string()),
        account_name.to_string(),
    )
    .map_err(|err| AuthError::Crypto(format!("totp init: {err}")))?;

    let uri = totp.get_url();

    Ok(TotpSetup {
        secret: secret_b32,
        uri,
    })
}

/// Returns `true` when `code` is a valid TOTP code for `base32_secret` at the
/// current time (±1 step window for clock skew).
pub fn verify_totp_code(base32_secret: &str, code: &str) -> AuthResult<bool> {
    let secret = Secret::Encoded(base32_secret.to_string());
    let totp = TOTP::new(
        Algorithm::SHA1,
        TOTP_DIGITS,
        1,
        TOTP_STEP,
        secret
            .to_bytes()
            .map_err(|err| AuthError::Crypto(format!("totp secret: {err}")))?,
        None,
        String::new(),
    )
    .map_err(|err| AuthError::Crypto(format!("totp init: {err}")))?;

    Ok(totp.check_current(code).unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_produces_uri() {
        let setup = setup_totp("user@example.com", "MyApp").unwrap();
        assert!(setup.uri.starts_with("otpauth://totp/"));
        assert!(!setup.secret.is_empty());
    }

    #[test]
    fn verify_with_generated_code() {
        let setup = setup_totp("user@example.com", "MyApp").unwrap();
        let secret = Secret::Encoded(setup.secret.clone());
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            TOTP_DIGITS,
            1,
            TOTP_STEP,
            secret.to_bytes().unwrap(),
            None,
            String::new(),
        )
        .unwrap();
        let code = totp.generate_current().unwrap();
        assert!(verify_totp_code(&setup.secret, &code).unwrap());
    }

    #[test]
    fn wrong_code_is_rejected() {
        let setup = setup_totp("user@example.com", "MyApp").unwrap();
        assert!(!verify_totp_code(&setup.secret, "000000").unwrap_or(true));
    }
}
