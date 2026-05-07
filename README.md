# awesome-rust-auth

`awesome-rust-auth` is a framework-agnostic Rust authentication crate inspired by [awesome-node-auth](https://www.awesomenodeauth.com).

## Features

- Access/refresh JWT pair with rotation + revocation primitives
- Email/password auth (`argon2`) with validator-based request validation
- OAuth account-link model scaffolding
- Tenant-aware domain model + RBAC store contract
- Session, API key, telemetry, SSE, event-bus, and webhook traits
- Embedded Admin UI (`/auth/admin`), Auth UI (`/auth/ui`), and `auth.js` (`/auth/ui/auth.js`)
- Mail templating with Handlebars i18n templates (`en`, `it` built in)
- OIDC discovery + JWKS helpers for IDP mode
- OpenAPI generation via `utoipa`
- Adapter modules behind feature flags for Axum, Actix-web, and Warp

## Installation

```toml
[dependencies]
awesome-rust-auth = { version = "0.1.0", features = ["axum"] }
```

## Configuration

```rust
use awesome_rust_auth::AuthConfig;

let config = AuthConfig::builder()
    .issuer("https://auth.example.com")
    .audience("my-app")
    .jwt_secret("replace-with-a-long-secret")
    .enable_idp_mode(true)
    .build()?;
# Ok::<(), awesome_rust_auth::AuthError>(())
```

## Adapter usage

### Axum

```rust
#[cfg(feature = "axum")]
{
    let app = awesome_rust_auth::adapters::axum::router();
    let _ = app;
}
```

### Actix-web

```rust
#[cfg(feature = "actix")]
{
    let scope = awesome_rust_auth::adapters::actix::scope();
    let _ = scope;
}
```

### Warp

```rust
#[cfg(feature = "warp")]
{
    let routes = awesome_rust_auth::adapters::warp::routes();
    let _ = routes;
}
```

## API compatibility target

This crate targets the same REST shape and token conventions used by:

- `ng-awesome-node-auth`
- `awesome-node-auth-flutter`

Current explicit deviations (also available via `compatibility_notes()`):

1. OAuth provider-specific request/response payload details are scaffolded but not fully parity-mapped yet.
2. Inbound webhook dynamic action decorator DSL parity is pending; secure Wasmtime execution is provided.

## Examples

- `examples/axum-postgres`
- `examples/actix-mongodb`
- `examples/warp-sqlite`

## Crate design principles

- Database-agnostic persistence via traits in `src/traits.rs`
- Framework-agnostic core in `src/service.rs`
- No `unsafe`
- Public API designed for semver-stable evolution from `0.1.0`
