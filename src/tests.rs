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
}

#[test]
fn english_template_renders() {
    let engine = MailTemplateEngine::with_builtin_locales().expect("engine should initialize");
    let rendered = engine
        .render("en", "welcome", &serde_json::json!({"name": "Niko"}))
        .expect("template should render");
    assert!(rendered.contains("Niko"));
}
