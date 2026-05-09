use crate::{AuthConfig, api_contract::compatibility_notes, mail::MailTemplateEngine};

#[test]
fn config_builder_rejects_short_secret() {
    let result = AuthConfig::builder().jwt_secret("short").build();
    assert!(result.is_err());
}

#[test]
fn compatibility_notes_include_deviations() {
    let notes = compatibility_notes();
    assert!(!notes.known_deviations.is_empty());
    assert!(
        notes.known_deviations.iter().any(|note| note
            .contains("OAuth provider-specific payload shape follows the Rust service contract")),
        "compatibility notes should mention OAuth provider payload deviation"
    );
    assert!(
        notes
            .known_deviations
            .iter()
            .any(|note| note.contains("Inbound webhook action decorators are executed through the built-in Wasmtime sandbox model")),
        "compatibility notes should mention webhook action decorator deviation"
    );
}

#[test]
fn english_template_renders() {
    let engine = MailTemplateEngine::with_builtin_locales().expect("engine should initialize");
    let rendered = engine
        .render(
            "en",
            "welcome",
            &serde_json::json!({"loginUrl": "https://example.com/login", "tempPassword": "temp123"}),
        )
        .expect("template should render");
    assert!(rendered.contains("https://example.com/login"));
    assert!(rendered.contains("temp123"));
}

#[test]
fn builtin_password_reset_templates_render_in_both_locales() {
    let engine = MailTemplateEngine::with_builtin_locales().expect("engine should initialize");
    let en = engine
        .render(
            "en",
            "password_reset",
            &serde_json::json!({"name": "Niko", "link": "https://example.com/reset"}),
        )
        .expect("english password_reset should render");
    assert!(en.contains("https://example.com/reset"));

    let it = engine
        .render(
            "it",
            "password_reset",
            &serde_json::json!({"name": "Niko", "link": "https://example.com/reset"}),
        )
        .expect("italian password_reset should render");
    assert!(it.contains("https://example.com/reset"));
}

#[test]
fn builtin_email_changed_and_invitation_templates_render() {
    let engine = MailTemplateEngine::with_builtin_locales().expect("engine should initialize");

    let changed = engine
        .render(
            "en",
            "email-changed",
            &serde_json::json!({"newEmail": "new@example.com"}),
        )
        .expect("english email-changed should render");
    assert!(changed.contains("new@example.com"));

    let invitation = engine
        .render(
            "it",
            "invitation",
            &serde_json::json!({"link": "https://example.com/invite"}),
        )
        .expect("italian invitation should render");
    assert!(invitation.contains("https://example.com/invite"));
}

#[test]
fn kebab_case_mail_template_aliases_render() {
    let engine = MailTemplateEngine::with_builtin_locales().expect("engine should initialize");
    let rendered = engine
        .render(
            "en",
            "verify-email",
            &serde_json::json!({"link": "https://example.com/verify"}),
        )
        .expect("kebab-case verify-email should render");
    assert!(rendered.contains("https://example.com/verify"));
}
