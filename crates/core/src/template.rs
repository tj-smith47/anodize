// Template rendering powered by Tera.
// Supports both Go-style `{{ .Field }}` and Tera-style `{{ Field }}`.
// Go-style templates are preprocessed (leading dots stripped) before Tera renders them.
// Tera gives us: if/else/endif, for loops, pipes (| lower, | upper, | replace),
// | default, | trim, | title, and many more built-in filters.

use std::collections::HashMap;
use std::sync::LazyLock;
use anyhow::{Context as _, Result};
use regex::Regex;

/// Regex to find Go-style dot-prefixed references inside `{{ }}` blocks.
/// Matches `{{ .Field }}`, `{{.Field}}`, `{{ .Env.VAR }}`, and also expressions
/// like `{{ .Field | filter }}`. We only strip the dot from the variable name.
static GO_DOT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{(\s*)\.(\w+)").unwrap()
});

pub struct TemplateVars {
    vars: HashMap<String, String>,
    env: HashMap<String, String>,
}

impl TemplateVars {
    pub fn new() -> Self {
        Self { vars: HashMap::new(), env: HashMap::new() }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.vars.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    pub fn set_env(&mut self, key: &str, value: &str) {
        self.env.insert(key.to_string(), value.to_string());
    }
}

impl Default for TemplateVars {
    fn default() -> Self {
        Self::new()
    }
}

/// Preprocess a template: convert Go-style `{{ .Field }}` to Tera-style `{{ Field }}`.
/// Handles both `{{ .Field }}` and `{{.Field}}` (no spaces).
/// Also handles chained access like `{{ .Env.VAR }}` → `{{ Env.VAR }}`.
fn preprocess(template: &str) -> String {
    // Replace `{{<optional whitespace>.<word>` with `{{<optional whitespace><word>`
    // This strips the leading dot while preserving whitespace and the rest of the expression.
    GO_DOT_RE.replace_all(template, "{{${1}${2}").to_string()
}

/// Build a `tera::Context` from `TemplateVars`.
/// - Regular vars are inserted at the top level: `ProjectName`, `Version`, etc.
/// - Env vars are nested under an `Env` key as a HashMap, so `{{ Env.GITHUB_TOKEN }}` works.
fn build_tera_context(vars: &TemplateVars) -> tera::Context {
    let mut ctx = tera::Context::new();
    for (k, v) in &vars.vars {
        ctx.insert(k.as_str(), v);
    }
    ctx.insert("Env", &vars.env);
    ctx
}

/// Render a template string with the given variables.
///
/// Supports both Go-style (`{{ .Field }}`) and native Tera-style (`{{ Field }}`).
/// Go-style references are preprocessed into Tera-style before rendering.
///
/// Because this uses Tera under the hood, all Tera features are available:
/// conditionals (`{% if %}` / `{% else %}` / `{% endif %}`), loops (`{% for %}`),
/// filters (`| lower`, `| upper`, `| default`, `| trim`, `| title`, `| replace`, etc.).
pub fn render(template: &str, vars: &TemplateVars) -> Result<String> {
    let preprocessed = preprocess(template);
    let ctx = build_tera_context(vars);
    tera::Tera::one_off(&preprocessed, &ctx, false)
        .with_context(|| format!("failed to render template: {}", template))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vars() -> TemplateVars {
        let mut vars = TemplateVars::new();
        vars.set("ProjectName", "cfgd");
        vars.set("Version", "1.2.3");
        vars.set("Tag", "v1.2.3");
        vars.set("Os", "linux");
        vars.set("Arch", "amd64");
        vars.set("ShortCommit", "abc1234");
        vars.set("Major", "1");
        vars.set("Minor", "2");
        vars.set("Patch", "3");
        vars.set_env("GITHUB_TOKEN", "tok123");
        vars
    }

    #[test]
    fn test_simple_substitution() {
        let vars = test_vars();
        let result = render("{{ .ProjectName }}-{{ .Version }}", &vars).unwrap();
        assert_eq!(result, "cfgd-1.2.3");
    }

    #[test]
    fn test_env_access() {
        let vars = test_vars();
        let result = render("{{ .Env.GITHUB_TOKEN }}", &vars).unwrap();
        assert_eq!(result, "tok123");
    }

    #[test]
    fn test_no_spaces() {
        let vars = test_vars();
        let result = render("{{.ProjectName}}-{{.Version}}", &vars).unwrap();
        assert_eq!(result, "cfgd-1.2.3");
    }

    #[test]
    fn test_missing_var() {
        let vars = test_vars();
        let result = render("{{ .Missing }}", &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_name_template() {
        let vars = test_vars();
        let result = render("{{ .ProjectName }}-{{ .Version }}-{{ .Os }}-{{ .Arch }}", &vars).unwrap();
        assert_eq!(result, "cfgd-1.2.3-linux-amd64");
    }

    #[test]
    fn test_literal_text_preserved() {
        let vars = test_vars();
        let result = render("prefix-{{ .Tag }}-suffix.tar.gz", &vars).unwrap();
        assert_eq!(result, "prefix-v1.2.3-suffix.tar.gz");
    }

    // Tera-style tests (no leading dot)

    #[test]
    fn test_tera_simple_substitution() {
        let vars = test_vars();
        let result = render("{{ ProjectName }}-{{ Version }}", &vars).unwrap();
        assert_eq!(result, "cfgd-1.2.3");
    }

    #[test]
    fn test_tera_env_access() {
        let vars = test_vars();
        let result = render("{{ Env.GITHUB_TOKEN }}", &vars).unwrap();
        assert_eq!(result, "tok123");
    }

    #[test]
    fn test_tera_archive_name() {
        let vars = test_vars();
        let result = render("{{ ProjectName }}-{{ Version }}-{{ Os }}-{{ Arch }}", &vars).unwrap();
        assert_eq!(result, "cfgd-1.2.3-linux-amd64");
    }

    #[test]
    fn test_tera_missing_var() {
        let vars = test_vars();
        let result = render("{{ Missing }}", &vars);
        assert!(result.is_err());
    }
}
