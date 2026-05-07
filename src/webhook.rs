use hmac::{Hmac, Mac};
use sha2::Sha256;
use wasmtime::{Engine, Instance, Module, Store};

use crate::error::{AuthError, AuthResult};

type HmacSha256 = Hmac<Sha256>;

pub fn sign_webhook(secret: &str, payload: &[u8]) -> AuthResult<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|err| AuthError::Crypto(err.to_string()))?;
    mac.update(payload);
    let result = mac.finalize().into_bytes();
    Ok(hex::encode(result))
}

pub fn verify_webhook(secret: &str, payload: &[u8], expected_hex: &str) -> AuthResult<bool> {
    let expected = hex::decode(expected_hex).map_err(|err| AuthError::Crypto(err.to_string()))?;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|err| AuthError::Crypto(err.to_string()))?;
    mac.update(payload);
    Ok(mac.verify_slice(&expected).is_ok())
}

pub fn run_inbound_action_wasm(module_bytes: &[u8]) -> AuthResult<()> {
    let engine = Engine::default();
    let module =
        Module::new(&engine, module_bytes).map_err(|err| AuthError::Sandbox(err.to_string()))?;
    let mut store = Store::new(&engine, ());
    Instance::new(&mut store, &module, &[]).map_err(|err| AuthError::Sandbox(err.to_string()))?;
    Ok(())
}
