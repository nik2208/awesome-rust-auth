# awesome-rust-auth

`awesome-rust-auth` is a framework-agnostic Rust authentication crate inspired by [awesome-node-auth](https://www.awesomenodeauth.com).

## Parity Snapshot vs `awesome-node-auth`

| Capability | Status in `awesome-rust-auth` | Notes |
|---|---|---|
| Auth strategies (email/password, magic link, SMS OTP, TOTP 2FA, OAuth linking) | ✅ Implemented | Core services cover email/password auth, magic-link, SMS OTP/TOTP helpers, and account-linking flows. |
| Token management (cookie/bearer, access/refresh rotation, secure cookies) | ✅ Implemented | `AuthService` provides access/refresh issuance with rotation/revocation and supports CSRF primitives for secure browser integration. |
| Identity Provider (IdP) mode (OIDC discovery, authorization, token, userinfo, JWKS) | ✅ Implemented | OIDC discovery/JWKS support and ID token issuance are available in core modules for IdP integrations. |
| Stateful sessions | ✅ Implemented | Session creation, rotation, revocation, and listing are implemented through `SessionStore` + `AuthService`. |
| Dynamic email templates + UI i18n fallback | ✅ Implemented | Handlebars rendering with built-in `en`/`it` locale templates is available in `MailTemplateEngine`. |
| CSRF protection | ✅ Implemented | Stateless CSRF token generation and verification helpers are implemented in core. |
| Account management | ✅ Implemented | `AccountService` provides profile, password, reset, verification, email-change, and delete-account flows. |
| Account linking | ✅ Implemented | Pending-link creation/consumption and unlink flows are implemented in `AuthService`. |
| RBAC | ➖ Scaffolded | `RolesPermissionsStore` is defined, but there is no higher-level enforcement or token enrichment flow yet. |
| Multi-tenancy | ✅ Implemented | Tenant-aware IDs/models are propagated across contracts, tokens, and core services. |
| Admin panel | ➖ Scaffolded | `/auth/admin` serves an embedded placeholder page; full admin SPA and management APIs are not implemented yet. |
| Built-in UI + auth runtime (`auth.js`) | ➖ Scaffolded | `/auth/ui` and `/auth/ui/auth.js` are served, but they are placeholder assets rather than full parity UI/runtime. |
| Client libraries compatibility (Angular + Flutter) | ➖ In progress | Compatibility is a design target, but REST surface, cookie conventions, and bearer strategy parity are not complete yet. |
| Event-driven tooling (event bus, SSE, inbound/outbound webhooks, telemetry) | ✅ Implemented | Event bus, SSE distributor contract, webhook execution helpers, and telemetry/event persistence contracts are available. |
| API keys (M2M) | ✅ Implemented | API-key issuing, hashing, authentication, listing, and revocation are implemented in `ApiKeyManager`. |
| OpenAPI / Swagger docs | ✅ Implemented | OpenAPI schemas and auth endpoint documentation are shipped via `utoipa` support in `openapi`/`api_contract` modules. |
| MCP server (`awesome-node-auth-mcp-server`) | ❌ Not implemented | No Rust-side MCP server is bundled in this repository. |

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
