pub const ADMIN_UI_HTML: &str = include_str!("assets/admin.html");
pub const AUTH_UI_HTML: &str = include_str!("assets/auth.html");
pub const AUTH_JS: &str = include_str!("assets/auth.js");
pub const AUTH_UI_CONFIG_JSON: &str = r#"{
  "apiPrefix": "/auth",
  "loginPath": "/auth/login",
  "refreshPath": "/auth/refresh",
  "logoutPath": "/auth/logout",
  "mePath": "/auth/me",
  "registerPath": "/auth/register",
  "builtInLocales": ["en", "it"]
}"#;
