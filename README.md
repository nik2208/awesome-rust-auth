# awesome-rust-auth

`awesome-rust-auth` is a framework-agnostic Rust authentication crate inspired by [awesome-node-auth](https://www.awesomenodeauth.com).

## Parity Snapshot vs `awesome-node-auth`

| Capability | Status in `awesome-rust-auth` | Notes |
|---|---|---|
| Auth strategies (email/password, magic link, SMS OTP, TOTP 2FA, OAuth linking) | ➖ Partial | Email/password is implemented in core service; OAuth linking types/contracts exist; magic-link, SMS OTP, and TOTP flows are not wired yet. |
| Token management (cookie/bearer, access/refresh rotation, secure cookies) | ➖ Partial | HS256 access/refresh issuance plus refresh rotation/revocation exist in `AuthService`; cookie transport, CSRF, and client-specific HTTP behavior are still missing. |
| Identity Provider (IdP) mode (OIDC discovery, authorization, token, userinfo, JWKS) | ➖ Scaffolded | Discovery and JWKS helpers are present, but full OIDC authorization/token/userinfo endpoints are not implemented yet. |
| Stateful sessions | ➖ Partial | `SessionStore` plus session creation/revocation/rotation exist in core logic, but full device/session management APIs are not present yet. |
| Dynamic email templates + UI i18n fallback | ➖ Partial | Handlebars rendering with built-in `en`/`it` templates exists, but the template set and runtime UI i18n parity are still minimal. |
| CSRF protection | ❌ Not implemented | No browser CSRF middleware or double-submit cookie flow is currently shipped. |
| Account management | ➖ Partial | Signup and login are implemented; profile update, email change, password reset/change, verification, and account deletion are still missing. |
| Account linking | ➖ Scaffolded | Linked-account models and `PendingLinkStore` contract exist, but the end-to-end link/unlink flows are not implemented yet. |
| RBAC | ➖ Scaffolded | `RolesPermissionsStore` is defined, but there is no higher-level enforcement or token enrichment flow yet. |
| Multi-tenancy | ➖ Partial | Tenant-aware IDs/models and store contracts are present, but tenant management APIs and end-to-end flows are not implemented. |
| Admin panel | ➖ Scaffolded | `/auth/admin` serves an embedded placeholder page; full admin SPA and management APIs are not implemented yet. |
| Built-in UI + auth runtime (`auth.js`) | ➖ Scaffolded | `/auth/ui` and `/auth/ui/auth.js` are served, but they are placeholder assets rather than full parity UI/runtime. |
| Client libraries compatibility (Angular + Flutter) | ➖ In progress | Compatibility is a design target, but REST surface, cookie conventions, and bearer strategy parity are not complete yet. |
| Event-driven tooling (event bus, SSE, inbound/outbound webhooks, telemetry) | ➖ Partial | In-memory event bus, telemetry trait, SSE distributor trait, and webhook helpers exist; full HTTP endpoints and distributed integrations are still pending. |
| API keys (M2M) | ➖ Scaffolded | API-key models and store contracts exist, but issuance, auth middleware, and audit/API flows are not implemented yet. |
| OpenAPI / Swagger docs | ➖ Partial | `utoipa` is wired for initial login/signup docs, but the full auth/admin/tools surface is not documented yet. |
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
