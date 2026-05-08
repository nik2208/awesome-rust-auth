pub const AUTH_JS: &str = include_str!("assets/auth.js");
pub const AUTH_BASE_CSS: &str = include_str!("assets/base.css");
pub const AUTH_UI_I18N_KEYS_JSON: &str = include_str!("assets/ui-i18n-keys.json");
pub const AUTH_LOGIN_HTML: &str = include_str!("assets/login.html");
pub const AUTH_REGISTER_HTML: &str = include_str!("assets/register.html");
pub const AUTH_FORGOT_PASSWORD_HTML: &str = include_str!("assets/forgot-password.html");
pub const AUTH_RESET_PASSWORD_HTML: &str = include_str!("assets/reset-password.html");
pub const AUTH_MAGIC_LINK_HTML: &str = include_str!("assets/magic-link.html");
pub const AUTH_2FA_HTML: &str = include_str!("assets/2fa.html");
pub const AUTH_VERIFY_EMAIL_HTML: &str = include_str!("assets/verify-email.html");
pub const AUTH_ACCOUNT_CONFLICT_HTML: &str = include_str!("assets/account-conflict.html");
pub const AUTH_LINK_VERIFY_HTML: &str = include_str!("assets/link-verify.html");

pub const ADMIN_CSS: &str = include_str!("assets/admin.css");
pub const ADMIN_JS: &str = include_str!("assets/admin.js");
pub const ADMIN_HTML_TEMPLATE: &str = include_str!("assets/admin.html");

pub const AUTH_UI_CONFIG_JSON: &str = r##"{
  "apiPrefix": "/auth",
  "features": {
    "register": false,
    "magicLink": false,
    "sms": false,
    "google": false,
    "github": false,
    "forgotPassword": false,
    "verifyEmail": false,
    "twoFactor": false
  },
  "ui": {
    "primaryColor": "#4a90d9",
    "secondaryColor": "#6c757d",
    "siteName": "Awesome Node Auth"
  },
  "translations": {},
  "lang": "en",
  "headless": false
}"##;

pub const ADMIN_CONFIG_JSON: &str = r#"{
  "base": "/auth/admin",
  "featSessions": false,
  "featRoles": false,
  "featTenants": false,
  "featMetadata": false,
  "feat2faPolicy": false,
  "featControl": false,
  "featLinkedAccounts": false,
  "featApiKeys": true,
  "featWebhooks": true,
  "featTemplates": true,
  "featUpload": false,
  "uploadBaseUrl": "/auth/admin/assets/uploads",
  "sessionBased": false,
  "authApiPrefix": "/auth",
  "cookiePrefix": null
}"#;

pub fn render_admin_html() -> String {
    ADMIN_HTML_TEMPLATE
        .replace("__ADMIN_BASE_URL__", "/auth/admin")
        .replace("__ADMIN_CONFIG__", ADMIN_CONFIG_JSON)
}

pub fn auth_ui_asset(path: &str) -> Option<(&'static str, &'static str)> {
    match path {
        "auth.js" => Some(("application/javascript", AUTH_JS)),
        "base.css" => Some(("text/css; charset=utf-8", AUTH_BASE_CSS)),
        "auth.css" => Some(("text/css; charset=utf-8", AUTH_BASE_CSS)),
        "ui-i18n-keys.json" => Some(("application/json", AUTH_UI_I18N_KEYS_JSON)),
        _ => None,
    }
}

pub fn auth_ui_page(path: &str) -> Option<&'static str> {
    match path.trim_matches('/') {
        "" | "login" => Some(AUTH_LOGIN_HTML),
        "register" => Some(AUTH_REGISTER_HTML),
        "forgot-password" => Some(AUTH_FORGOT_PASSWORD_HTML),
        "reset-password" => Some(AUTH_RESET_PASSWORD_HTML),
        "magic-link" => Some(AUTH_MAGIC_LINK_HTML),
        "2fa" => Some(AUTH_2FA_HTML),
        "verify-email" => Some(AUTH_VERIFY_EMAIL_HTML),
        "account-conflict" => Some(AUTH_ACCOUNT_CONFLICT_HTML),
        "link-verify" => Some(AUTH_LINK_VERIFY_HTML),
        _ => None,
    }
}
