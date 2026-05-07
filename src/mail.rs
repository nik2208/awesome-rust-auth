use std::collections::HashMap;

use handlebars::Handlebars;

use crate::error::{AuthError, AuthResult};

pub const EN_WELCOME_TEMPLATE: &str = include_str!("templates/en/welcome.hbs");
pub const IT_WELCOME_TEMPLATE: &str = include_str!("templates/it/welcome.hbs");

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
        templates.insert("en".to_string(), en);

        let mut it = Handlebars::new();
        it.register_template_string("welcome", IT_WELCOME_TEMPLATE)
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
