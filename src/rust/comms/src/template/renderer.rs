use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Datelike;
use minijinja::value::Value as MjValue;
use minijinja::{AutoEscape, Environment, UndefinedBehavior};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::error::CommsError;
use crate::template::{TemplateLoader, TemplateSource};

/// Application-level branding values that feed the partial templates.
/// Identical for every recipient; bound at renderer construction time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrandingContext {
    pub company_name: String,
    pub company_address: String,
    pub company_url: String,
    pub logo_url: String,
}

/// Application-level context — everything that's the same for every recipient.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppContext {
    pub app_name: String,
    pub server_url: String,
    pub branding: BrandingContext,
}

/// Per-message render context. The renderer composes this with the cached
/// app-level context (including pre-rendered partials) to produce the final
/// render.
pub struct TemplateContext {
    /// Per-message values supplied by the consumer (e.g. `first_name`,
    /// `reset_link`, `expires_at`). Keys must not collide with app-level
    /// reserved names; collisions are rejected at render time so the consumer
    /// can't shadow `{{ logo }}` or similar.
    pub vars: Map<String, Value>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self { vars: Map::new() }
    }

    pub fn insert(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }
}

impl Default for TemplateContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a successful render. `subject` is always populated (templates
/// require one); `html` is populated when the template carries an html
/// section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedTemplate {
    pub subject: Option<String>,
    pub text: String,
    pub html: Option<String>,
}

/// Set of partial files supplied to the renderer at construction. All six
/// strings are raw template source — the renderer parses them as `minijinja`
/// templates and pre-renders against `BrandingContext` once.
pub struct BrandingPartialSources {
    pub logo_html: String,
    pub logo_text: String,
    pub header_html: String,
    pub header_text: String,
    pub footer_html: String,
    pub footer_text: String,
}

/// Names of partial tokens injected as application-level context at render
/// time. Per-message contexts are checked against this set so a caller can't
/// shadow them.
const PARTIAL_TOKENS: &[&str] = &["logo", "header", "footer"];

/// Names of branding-config tokens injected as application-level context at
/// render time. Per-message contexts are checked against this set too.
const BRANDING_TOKENS: &[&str] = &[
    "app_name",
    "server_url",
    "company_name",
    "company_address",
    "company_url",
    "copyright_year",
    "logo_url",
];

/// Renders templates against per-message and application-level context.
///
/// Branding partials (`logo`, `header`, `footer`) are pre-rendered once at
/// construction and injected as field-aware tokens — `{{ logo }}` resolves to
/// the `.html` partial when rendering an HTML field and the `.text` partial
/// when rendering a `subject` or `text` field. Pre-rendered HTML partials are
/// marked as safe so they aren't HTML-escaped a second time during per-message
/// render.
pub struct TemplateRenderer {
    env: Environment<'static>,
    sources: BTreeMap<String, TemplateSource>,
    app_text: BTreeMap<String, MjValue>,
    app_html: BTreeMap<String, MjValue>,
}

impl std::fmt::Debug for TemplateRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TemplateRenderer")
            .field("templates", &self.sources.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

impl TemplateRenderer {
    /// Build a renderer over the given templates and partials.
    ///
    /// All templates from `templates` are loaded, parsed, and registered up
    /// front so validation errors surface at construction rather than at the
    /// first send. All six branding partials are pre-rendered against
    /// `branding` and cached.
    pub fn new(
        app: AppContext,
        templates: Arc<dyn TemplateLoader>,
        partials: BrandingPartialSources,
    ) -> Result<Self, CommsError> {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        // Templates are registered as "<name>/<field>" — turn on HTML
        // autoescape only for the html field; subject and text fields
        // produce plain text where escaping would be wrong.
        env.set_auto_escape_callback(|name| {
            if name.ends_with("/html") {
                AutoEscape::Html
            } else {
                AutoEscape::None
            }
        });
        let mut sources: BTreeMap<String, TemplateSource> = BTreeMap::new();

        let names = templates.names();
        for name in &names {
            let toml_text = templates.load(name)?;
            let src = TemplateSource::parse(name, &toml_text)?;
            register_template_fields(&mut env, name, &src)?;
            sources.insert(name.clone(), src);
        }

        let copyright_year: i32 = chrono::Utc::now().year();
        let branding_ctx = build_branding_context(&app, copyright_year);

        let logo_html = render_partial("partials/logo.html", &partials.logo_html, &branding_ctx)?;
        let logo_text = render_partial("partials/logo.text", &partials.logo_text, &branding_ctx)?;
        let header_html = render_partial("partials/header.html", &partials.header_html, &branding_ctx)?;
        let header_text = render_partial("partials/header.text", &partials.header_text, &branding_ctx)?;
        let footer_html = render_partial("partials/footer.html", &partials.footer_html, &branding_ctx)?;
        let footer_text = render_partial("partials/footer.text", &partials.footer_text, &branding_ctx)?;

        let mut app_common: BTreeMap<String, MjValue> = BTreeMap::new();
        app_common.insert("app_name".into(), MjValue::from(app.app_name.clone()));
        app_common.insert("server_url".into(), MjValue::from(app.server_url.clone()));
        app_common.insert(
            "company_name".into(),
            MjValue::from(app.branding.company_name.clone()),
        );
        app_common.insert(
            "company_address".into(),
            MjValue::from(app.branding.company_address.clone()),
        );
        app_common.insert(
            "company_url".into(),
            MjValue::from(app.branding.company_url.clone()),
        );
        app_common.insert("copyright_year".into(), MjValue::from(copyright_year));
        app_common.insert(
            "logo_url".into(),
            MjValue::from(app.branding.logo_url.clone()),
        );

        let mut app_text = app_common.clone();
        app_text.insert("logo".into(), MjValue::from(logo_text));
        app_text.insert("header".into(), MjValue::from(header_text));
        app_text.insert("footer".into(), MjValue::from(footer_text));

        let mut app_html = app_common;
        // Pre-rendered HTML partials are marked safe — they're already valid
        // HTML, and the per-message HTML field would otherwise escape them.
        app_html.insert("logo".into(), MjValue::from_safe_string(logo_html));
        app_html.insert("header".into(), MjValue::from_safe_string(header_html));
        app_html.insert("footer".into(), MjValue::from_safe_string(footer_html));

        Ok(Self {
            env,
            sources,
            app_text,
            app_html,
        })
    }

    /// Render the named template against the supplied per-message context.
    pub fn render(
        &self,
        template_name: &str,
        ctx: &TemplateContext,
    ) -> Result<RenderedTemplate, CommsError> {
        for key in ctx.vars.keys() {
            if PARTIAL_TOKENS.contains(&key.as_str())
                || BRANDING_TOKENS.contains(&key.as_str())
            {
                return Err(CommsError::TemplateRender(format!(
                    "per-message context cannot shadow application-level token '{key}'"
                )));
            }
        }

        let src = self
            .sources
            .get(template_name)
            .ok_or_else(|| CommsError::TemplateNotFound(template_name.to_owned()))?;

        let subject = self.render_field(template_name, "subject", &ctx.vars, false)?;
        let text = self.render_field(template_name, "text", &ctx.vars, false)?;
        let html = if src.html.is_some() {
            Some(self.render_field(template_name, "html", &ctx.vars, true)?)
        } else {
            None
        };
        Ok(RenderedTemplate {
            subject: Some(subject),
            text,
            html,
        })
    }

    fn render_field(
        &self,
        template_name: &str,
        field: &str,
        per_message: &Map<String, Value>,
        html_mode: bool,
    ) -> Result<String, CommsError> {
        let registered = format!("{template_name}/{field}");
        let tmpl = self
            .env
            .get_template(&registered)
            .map_err(|e| CommsError::TemplateRender(format!("{registered}: {e}")))?;

        let mut ctx: BTreeMap<String, MjValue> = if html_mode {
            self.app_html.clone()
        } else {
            self.app_text.clone()
        };
        for (k, v) in per_message {
            ctx.insert(k.clone(), MjValue::from_serialize(v));
        }

        // `from_iter` (vs `from_serialize`) preserves any safe-string markers
        // on the values — losing them would cause double-escape on pre-rendered
        // HTML partials in the html field.
        let ctx_value: MjValue = ctx.into_iter().collect();
        tmpl.render(ctx_value)
            .map_err(|e| CommsError::TemplateRender(format!("{registered}: {e}")))
    }
}

fn build_branding_context(app: &AppContext, copyright_year: i32) -> MjValue {
    [
        ("app_name".to_owned(), MjValue::from(app.app_name.clone())),
        ("server_url".to_owned(), MjValue::from(app.server_url.clone())),
        (
            "company_name".to_owned(),
            MjValue::from(app.branding.company_name.clone()),
        ),
        (
            "company_address".to_owned(),
            MjValue::from(app.branding.company_address.clone()),
        ),
        (
            "company_url".to_owned(),
            MjValue::from(app.branding.company_url.clone()),
        ),
        ("copyright_year".to_owned(), MjValue::from(copyright_year)),
        (
            "logo_url".to_owned(),
            MjValue::from(app.branding.logo_url.clone()),
        ),
    ]
    .into_iter()
    .collect()
}

fn register_template_fields(
    env: &mut Environment<'static>,
    name: &str,
    src: &TemplateSource,
) -> Result<(), CommsError> {
    if let Some(subject) = &src.subject {
        env.add_template_owned(format!("{name}/subject"), subject.clone())
            .map_err(|e| CommsError::Config(format!("template '{name}/subject': {e}")))?;
    }
    env.add_template_owned(format!("{name}/text"), src.text.clone())
        .map_err(|e| CommsError::Config(format!("template '{name}/text': {e}")))?;
    if let Some(html) = &src.html {
        env.add_template_owned(format!("{name}/html"), html.clone())
            .map_err(|e| CommsError::Config(format!("template '{name}/html': {e}")))?;
    }
    Ok(())
}

fn render_partial(name: &str, source: &str, ctx: &MjValue) -> Result<String, CommsError> {
    // Partials run in a fresh environment so they only see app-level branding
    // tokens; any reference to a per-message var is rejected as undefined.
    let mut partial_env = Environment::new();
    partial_env.set_undefined_behavior(UndefinedBehavior::Strict);
    partial_env
        .add_template_owned(name.to_owned(), source.to_owned())
        .map_err(|e| CommsError::Config(format!("partial '{name}': {e}")))?;
    let tmpl = partial_env
        .get_template(name)
        .map_err(|e| CommsError::Config(format!("partial '{name}': {e}")))?;
    tmpl.render(ctx)
        .map_err(|e| CommsError::Config(format!("partial '{name}': {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::embedded_loader::EmbeddedTemplateLoader;
    use pretty_assertions::assert_eq;

    fn sample_app_context() -> AppContext {
        AppContext {
            app_name: "Maze".into(),
            server_url: "https://example.com".into(),
            branding: BrandingContext {
                company_name: "Maze, Inc.".into(),
                company_address: "123 Example St".into(),
                company_url: "https://example.com".into(),
                logo_url: "https://example.com/logo.png".into(),
            },
        }
    }

    fn sample_partials() -> BrandingPartialSources {
        BrandingPartialSources {
            logo_html: r#"<img src="{{ logo_url }}" alt="{{ company_name }}">"#.into(),
            logo_text: "{{ company_name }}".into(),
            header_html: r#"<h1>{{ company_name }}</h1>"#.into(),
            header_text: "== {{ company_name }} ==".into(),
            footer_html: r#"<p>&copy; {{ copyright_year }} {{ company_name }} &middot; {{ company_address }}</p>"#.into(),
            footer_text: "(c) {{ copyright_year }} {{ company_name }} - {{ company_address }}".into(),
        }
    }

    fn build_renderer(template_name: &str, template_toml: &str) -> TemplateRenderer {
        let templates: Arc<dyn TemplateLoader> = Arc::new(
            EmbeddedTemplateLoader::from_pairs(&[(template_name, template_toml)]),
        );
        TemplateRenderer::new(sample_app_context(), templates, sample_partials())
            .expect("renderer construction")
    }

    #[test]
    fn renders_template_subject_text_and_html() {
        let toml = r#"
            subject = "Hi {{ first_name }} from {{ app_name }}"
            text = "Hello {{ first_name }}, click {{ reset_link }}"
            html = "<p>Hello {{ first_name }}, <a href=\"{{ reset_link }}\">click</a></p>"
        "#;
        let r = build_renderer("greet", toml);
        let ctx = TemplateContext::new()
            .insert("first_name", "Alice")
            .insert("reset_link", "https://example.com/r/abc");
        let out = r.render("greet", &ctx).expect("render");
        assert_eq!(out.subject.as_deref(), Some("Hi Alice from Maze"));
        assert_eq!(out.text, "Hello Alice, click https://example.com/r/abc");
        // The html field auto-escapes per-message vars (defense in depth);
        // minijinja follows the OWASP recommendation and escapes `/` to
        // `&#x2f;` as well as the standard HTML metacharacters. Browsers
        // decode the entity transparently when navigating the link.
        assert_eq!(
            out.html.as_deref(),
            Some(
                "<p>Hello Alice, <a href=\"https:&#x2f;&#x2f;example.com&#x2f;r&#x2f;abc\">click</a></p>"
            )
        );
    }

    #[test]
    fn renders_template_without_html_section() {
        let toml = r#"
            subject = "{{ app_name }} update"
            text = "{{ app_name }}: visit {{ reset_link }}"
        "#;
        let r = build_renderer("ping", toml);
        let ctx = TemplateContext::new().insert("reset_link", "https://example.com/r/abc");
        let out = r.render("ping", &ctx).expect("render");
        assert_eq!(out.subject.as_deref(), Some("Maze update"));
        assert_eq!(out.text, "Maze: visit https://example.com/r/abc");
        assert_eq!(out.html, None);
    }

    #[test]
    fn logo_token_is_field_aware() {
        let toml = r#"
            subject = "{{ logo }} subject"
            text = "{{ logo }} body"
            html = "{{ logo }}"
        "#;
        let r = build_renderer("brand", toml);
        let out = r.render("brand", &TemplateContext::new()).expect("render");
        // Subject and text get the rendered text-mode partial.
        assert_eq!(out.subject.as_deref(), Some("Maze, Inc. subject"));
        assert_eq!(out.text, "Maze, Inc. body");
        // HTML field gets the rendered HTML-mode partial. The partial itself
        // ran with autoescape on (defense in depth — even branding config
        // values are escaped to prevent accidental HTML breakage if they
        // contain `&`, `<`, etc.). The pre-rendered output is then marked
        // safe so the outer template doesn't double-escape it.
        let html = out.html.expect("html");
        assert!(html.starts_with("<img src=\""), "{html}");
        assert!(html.contains("alt=\"Maze, Inc.\""), "{html}");
        // The URL appears in the rendered output, with `/` OWASP-escaped to
        // `&#x2f;` by the partial render. Browsers decode this transparently
        // when following the link.
        assert!(
            html.contains("&#x2f;&#x2f;example.com&#x2f;logo.png"),
            "{html}"
        );
    }

    #[test]
    fn footer_and_header_tokens_are_field_aware() {
        let toml = r#"
            subject = "ignore"
            text = "{{ header }}\n{{ footer }}"
            html = "<header>{{ header }}</header><footer>{{ footer }}</footer>"
        "#;
        let r = build_renderer("brand", toml);
        let out = r.render("brand", &TemplateContext::new()).expect("render");
        let text = out.text;
        assert!(text.contains("== Maze, Inc. =="), "{text}");
        assert!(text.contains("(c)"), "{text}");

        let html = out.html.expect("html");
        assert!(html.contains("<h1>Maze, Inc.</h1>"), "{html}");
        assert!(html.contains("&copy;"), "{html}");
    }

    #[test]
    fn html_per_message_vars_are_escaped() {
        // Defense-in-depth: per-message vars going into an html field must be
        // auto-escaped to prevent injection if the consumer ever passes
        // unsanitised content.
        let toml = r#"
            subject = "x"
            text = "x"
            html = "<p>{{ note }}</p>"
        "#;
        let r = build_renderer("escape", toml);
        let ctx = TemplateContext::new().insert("note", "<script>alert(1)</script>");
        let out = r.render("escape", &ctx).expect("render");
        let html = out.html.expect("html");
        assert!(html.contains("&lt;script&gt;"), "{html}");
        assert!(!html.contains("<script>"), "{html}");
    }

    #[test]
    fn company_name_substitutes_in_per_message_template() {
        let toml = r#"
            subject = "From {{ company_name }}"
            text = "Sent by {{ company_name }}"
        "#;
        let r = build_renderer("brand", toml);
        let out = r.render("brand", &TemplateContext::new()).expect("render");
        assert_eq!(out.subject.as_deref(), Some("From Maze, Inc."));
        assert_eq!(out.text, "Sent by Maze, Inc.");
    }

    #[test]
    fn company_url_substitutes_in_per_message_template() {
        let toml = r#"
            subject = "x"
            text = "Visit {{ company_url }}"
            html = "<a href=\"{{ company_url }}\">{{ company_name }}</a>"
        "#;
        let r = build_renderer("brand", toml);
        let out = r.render("brand", &TemplateContext::new()).expect("render");
        assert_eq!(out.text, "Visit https://example.com");
        // html field auto-escapes per-OWASP, so the URL appears with `/`
        // entities. Browsers decode them transparently when navigating.
        let html = out.html.expect("html");
        assert!(html.contains("&#x2f;&#x2f;example.com"), "{html}");
        assert!(html.contains("Maze, Inc."), "{html}");
    }

    #[test]
    fn per_message_context_cannot_shadow_app_token() {
        let toml = r#"
            subject = "x"
            text = "x"
        "#;
        let r = build_renderer("greet", toml);
        let ctx = TemplateContext::new().insert("logo", "evil");
        let err = r.render("greet", &ctx).expect_err("must reject shadow");
        assert!(err.to_string().contains("logo"), "{err}");

        let ctx = TemplateContext::new().insert("company_name", "evil");
        let err = r.render("greet", &ctx).expect_err("must reject shadow");
        assert!(err.to_string().contains("company_name"), "{err}");
    }

    #[test]
    fn partial_referencing_per_message_token_fails_at_construction() {
        let templates: Arc<dyn TemplateLoader> = Arc::new(EmbeddedTemplateLoader::new());
        let mut partials = sample_partials();
        // first_name is per-message, not app-level — strict undefined behaviour
        // makes the partial fail at startup with a clear error rather than
        // silently rendering empty later.
        partials.footer_text = "(c) {{ first_name }}".into();

        let err = TemplateRenderer::new(sample_app_context(), templates, partials)
            .expect_err("must fail");
        let s = err.to_string();
        assert!(s.contains("partial"), "{s}");
        // Strict undefined-behaviour: minijinja reports "undefined value" with
        // the partial name attached. The variable name itself isn't always in
        // the message, so we just assert the failure mode is recognisable.
        assert!(s.contains("undefined"), "{s}");
        assert!(s.contains("partials/footer.text"), "{s}");
    }

    #[test]
    fn unknown_template_returns_not_found() {
        let r = build_renderer("greet", "subject = \"x\"\ntext = \"hi\"");
        let err = r
            .render("absent", &TemplateContext::new())
            .expect_err("must miss");
        assert!(matches!(err, CommsError::TemplateNotFound(n) if n == "absent"));
    }

    #[test]
    fn malformed_template_fails_at_construction() {
        let templates: Arc<dyn TemplateLoader> = Arc::new(EmbeddedTemplateLoader::from_pairs(&[(
            "broken",
            "subject = \"hi {{ unterminated\"\ntext = \"x\"",
        )]));
        let err = TemplateRenderer::new(sample_app_context(), templates, sample_partials())
            .expect_err("must fail");
        assert!(err.to_string().contains("broken/subject"), "{err}");
    }
}
