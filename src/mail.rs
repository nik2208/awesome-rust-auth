use std::collections::HashMap;

use handlebars::Handlebars;

use crate::error::{AuthError, AuthResult};

pub const EN_WELCOME_TEMPLATE: &str = include_str!("templates/en/welcome.hbs");
pub const IT_WELCOME_TEMPLATE: &str = include_str!("templates/it/welcome.hbs");
pub const EN_PASSWORD_RESET_TEMPLATE: &str = include_str!("templates/en/password_reset.hbs");
pub const IT_PASSWORD_RESET_TEMPLATE: &str = include_str!("templates/it/password_reset.hbs");
pub const EN_MAGIC_LINK_TEMPLATE: &str = include_str!("templates/en/magic_link.hbs");
pub const IT_MAGIC_LINK_TEMPLATE: &str = include_str!("templates/it/magic_link.hbs");
pub const EN_VERIFY_EMAIL_TEMPLATE: &str = include_str!("templates/en/verify_email.hbs");
pub const IT_VERIFY_EMAIL_TEMPLATE: &str = include_str!("templates/it/verify_email.hbs");

#[derive(Debug, Clone)]
pub struct MailTemplateEngine {
    templates: HashMap<String, Handlebars<'static>>,
}

impl MailTemplateEngine {
    pub fn with_builtin_locales() -> AuthResult<Self> {
        let mut templates = HashMap::new();

        let mut en = Handlebars::new();
        en.register_template_string("welcome", EN_WELCOME_TEMPLATE)
            .map_err(|err| AuthError::Config(err.to_string()))?;
        en.register_template_string("password_reset", EN_PASSWORD_RESET_TEMPLATE)
            .map_err(|err| AuthError::Config(err.to_string()))?;
        en.register_template_string("magic_link", EN_MAGIC_LINK_TEMPLATE)
            .map_err(|err| AuthError::Config(err.to_string()))?;
        en.register_template_string("verify_email", EN_VERIFY_EMAIL_TEMPLATE)
            .map_err(|err| AuthError::Config(err.to_string()))?;
        templates.insert("en".to_string(), en);

        let mut it = Handlebars::new();
        it.register_template_string("welcome", IT_WELCOME_TEMPLATE)
            .map_err(|err| AuthError::Config(err.to_string()))?;
        it.register_template_string("password_reset", IT_PASSWORD_RESET_TEMPLATE)
            .map_err(|err| AuthError::Config(err.to_string()))?;
        it.register_template_string("magic_link", IT_MAGIC_LINK_TEMPLATE)
            .map_err(|err| AuthError::Config(err.to_string()))?;
        it.register_template_string("verify_email", IT_VERIFY_EMAIL_TEMPLATE)
            .map_err(|err| AuthError::Config(err.to_string()))?;
        templates.insert("it".to_string(), it);

        Ok(Self { templates })
    }

    pub fn register_locale_template(
        &mut self,
        locale: &str,
        template_name: &str,
        content: &str,
    ) -> AuthResult<()> {
        let entry = self.templates.entry(locale.to_string()).or_default();
        entry
            .register_template_string(template_name, content)
            .map_err(|err| AuthError::Config(err.to_string()))
    }

    pub fn render(
        &self,
        locale: &str,
        template_name: &str,
        context: &serde_json::Value,
    ) -> AuthResult<String> {
        let engine = self
            .templates
            .get(locale)
            .or_else(|| self.templates.get("en"))
            .ok_or_else(|| AuthError::Config("no templates registered".to_string()))?;

        engine
            .render(template_name, context)
            .map_err(|err| AuthError::Config(err.to_string()))
    }
}
