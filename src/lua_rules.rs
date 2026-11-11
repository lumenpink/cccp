//! Lua Rules Engine for GOST Compliance
//! Executes declarative rules defined in `rules.toml` with the `gost.*` runtime.

use crate::audit::{AuditCheck, CheckStatus};
use mlua::Lua;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RulesManifest {
    #[serde(default)]
    pub rules: HashMap<String, RuleDefinition>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RuleDefinition {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub docs_url: Option<String>,
    #[serde(default)]
    pub script: String,
    #[serde(default)]
    pub messages: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LocaleManifest {
    #[serde(default)]
    pub rules: HashMap<String, LocaleRuleEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LocaleRuleEntry {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub messages: HashMap<String, String>,
}

pub fn read_disabled_rules(root: &Path) -> Vec<String> {
    let mut disabled = Vec::new();
    let config_path = root.join(".gost").join("config");
    if let Ok(content) = fs::read_to_string(config_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("DISABLED_RULES=") {
                let clean = rest.trim_matches('"').trim_matches('\'');
                for item in clean.split(',') {
                    let rule = item.trim();
                    if !rule.is_empty() {
                        disabled.push(rule.to_string());
                    }
                }
            }
        }
    }
    disabled
}

pub fn disable_rules_in_config(root: &Path, rules_to_disable: &[String]) -> std::io::Result<()> {
    let gost_dir = root.join(".gost");
    let _ = fs::create_dir_all(&gost_dir);
    let config_path = gost_dir.join("config");

    let mut current_disabled = read_disabled_rules(root);
    for r in rules_to_disable {
        let trimmed = r.trim().to_string();
        if !trimmed.is_empty() && !current_disabled.contains(&trimmed) {
            current_disabled.push(trimmed);
        }
    }

    write_disabled_rules(&config_path, &current_disabled)
}

pub fn enable_rules_in_config(root: &Path, rules_to_enable: &[String]) -> std::io::Result<()> {
    let gost_dir = root.join(".gost");
    let _ = fs::create_dir_all(&gost_dir);
    let config_path = gost_dir.join("config");

    let mut current_disabled = read_disabled_rules(root);
    current_disabled.retain(|r| !rules_to_enable.iter().any(|to_en| to_en.trim() == r));

    write_disabled_rules(&config_path, &current_disabled)
}

fn write_disabled_rules(config_path: &Path, disabled: &[String]) -> std::io::Result<()> {
    let mut lines = Vec::new();
    let mut found = false;

    if config_path.is_file() {
        if let Ok(content) = fs::read_to_string(config_path) {
            for line in content.lines() {
                if line.trim().starts_with("DISABLED_RULES=") {
                    lines.push(format!("DISABLED_RULES=\"{}\"", disabled.join(", ")));
                    found = true;
                } else {
                    lines.push(line.to_string());
                }
            }
        }
    }

    if !found {
        lines.push(format!("DISABLED_RULES=\"{}\"", disabled.join(", ")));
    }

    fs::write(config_path, lines.join("\n") + "\n")
}

pub fn load_rules_from_repo(root: &Path) -> HashMap<String, RuleDefinition> {
    let mut all_rules = HashMap::new();
    let candidates = [
        root.join(".rules").join("RULES.toml"),
        root.join(".rules").join("rules.toml"),
        root.join(".gost").join("rules.toml"),
        root.join("rules.toml"),
        root.join("specs").join("rules.toml"),
    ];

    for candidate in &candidates {
        if candidate.is_file() {
            if let Ok(content) = fs::read_to_string(candidate) {
                if let Ok(manifest) = toml::from_str::<RulesManifest>(&content) {
                    for (k, v) in manifest.rules {
                        all_rules.entry(k).or_insert(v);
                    }
                }
            }
        }
    }

    all_rules
}

pub fn load_locales_from_repo(root: &Path, lang: &str) -> HashMap<String, LocaleRuleEntry> {
    let mut all_locales = HashMap::new();
    let file_names = [
        format!("{}.toml", lang),
        "pt-br.toml".to_string(),
        "en.toml".to_string(),
    ];

    for fname in &file_names {
        let candidates = [
            root.join("locales").join(fname),
            root.join(".gost").join("locales").join(fname),
            root.join("specs").join("locales").join(fname),
        ];

        for candidate in &candidates {
            if candidate.is_file() {
                if let Ok(content) = fs::read_to_string(candidate) {
                    if let Ok(manifest) = toml::from_str::<LocaleManifest>(&content) {
                        for (k, v) in manifest.rules {
                            all_locales.entry(k).or_insert(v);
                        }
                    }
                }
            }
        }
    }

    all_locales
}

pub fn create_lua_environment(root: &Path) -> mlua::Result<Lua> {
    lunokhod::create_lua_environment(root)
}

pub fn evaluate_lua_rules_for_scope(
    root: &Path,
    disabled_rules: &[String],
    target_scope: Option<&str>,
) -> Vec<AuditCheck> {
    evaluate_lua_rules_with_lang(root, disabled_rules, "en", target_scope)
}

pub fn evaluate_lua_rules_with_lang(
    root: &Path,
    disabled_rules: &[String],
    lang: &str,
    target_scope: Option<&str>,
) -> Vec<AuditCheck> {
    let effective_lang = if lang.trim().is_empty() {
        "en"
    } else {
        lang.trim()
    };
    let lk_checks = lunokhod::evaluate_lua_rules_for_scope(
        root,
        disabled_rules,
        effective_lang,
        target_scope,
    );
    lk_checks
        .into_iter()
        .map(|c| {
            let status = match c.status {
                lunokhod::CheckStatus::Pass => CheckStatus::Pass,
                lunokhod::CheckStatus::Fail => CheckStatus::Fail,
                lunokhod::CheckStatus::Warn => CheckStatus::Warn,
                lunokhod::CheckStatus::Skip => CheckStatus::Skip,
            };
            AuditCheck {
                code: Box::leak(c.code.into_boxed_str()),
                title: Box::leak(c.title.into_boxed_str()),
                detail: c.detail,
                status,
            }
        })
        .collect()
}

pub fn evaluate_lua_rules(root: &Path, disabled_rules: &[String]) -> Vec<AuditCheck> {
    evaluate_lua_rules_for_scope(root, disabled_rules, Some("project"))
}
