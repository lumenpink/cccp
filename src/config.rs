//! Hierarchical Gosplan Configuration Manager for CCCP.
//! Cascading resolution: Defaults < Global ($XDG/config) < Local (.cccprc) < Environment (CCCP_*) < CLI flags.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Resolved CCCP Configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// Allowed commit types (e.g. feat, fix, chore, docs, ...)
    pub commit_types: Vec<String>,
    /// Allowed commit scopes
    pub commit_scopes: Vec<String>,
    /// Allowed commit subscopes
    pub commit_subscopes: Vec<String>,
    /// Strict enforcement of commit types (reject unknown types)
    pub strict_types: bool,
    /// Strict enforcement of commit scopes
    pub strict_scopes: bool,
    /// Strict enforcement of commit subscopes
    pub strict_subscopes: bool,
    /// Allow any scope string even if not in commit_scopes
    pub allow_any_scope: bool,
    /// Allow any subscope string even if not in commit_subscopes
    pub allow_any_subscope: bool,
    /// Disable subscopes entirely
    pub disable_subscopes: bool,
    /// Disable multiple scopes (e.g. `feat(api,db): ...`)
    pub disable_multiple_scopes: bool,
    /// Default base version if repository has no release tags
    pub default_base_version: String,
    /// Do not prefix release tags with 'v'
    pub no_v: bool,
    /// Optional command or script to execute after push
    pub post_push_hook: Option<String>,
    /// Post push sync targets (e.g. ["tickets", "agents"])
    pub post_push_sync: Vec<String>,
    /// Destination path for tickets markdown synchronization
    pub tickets_file: String,
    /// Template URL for AGENTS.md
    pub agents_md_url: String,
    /// Output file path for AGENTS.md
    pub agents_md_file: String,
    /// Arbitrary configuration variables (e.g. agent_var_*, type_desc_*, custom directives)
    pub extra: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            commit_types: vec![
                "feat".into(),
                "fix".into(),
                "perf".into(),
                "refactor".into(),
                "revert".into(),
                "chore".into(),
                "build".into(),
                "ci".into(),
                "docs".into(),
                "ops".into(),
                "style".into(),
                "test".into(),
                "merge".into(),
            ],
            commit_scopes: vec![
                "ui".into(),
                "docs".into(),
                "api".into(),
                "docker".into(),
                "db".into(),
                "updater".into(),
                "micropub".into(),
                "indieauth".into(),
                "activitypub".into(),
                "microsub".into(),
                "twtxt".into(),
                "webmention".into(),
                "theme".into(),
                "feeds".into(),
                "cli".into(),
                "core".into(),
                "config".into(),
                "auth".into(),
                "test".into(),
                "build".into(),
            ],
            commit_subscopes: vec![
                "components".into(),
                "pages".into(),
                "services".into(),
                "utils".into(),
                "auth".into(),
                "models".into(),
                "views".into(),
                "controllers".into(),
                "handlers".into(),
            ],
            strict_types: true,
            strict_scopes: false,
            strict_subscopes: false,
            allow_any_scope: true,
            allow_any_subscope: true,
            disable_subscopes: false,
            disable_multiple_scopes: false,
            default_base_version: "0.0.1".into(),
            no_v: false,
            post_push_hook: None,
            post_push_sync: Vec::new(),
            tickets_file: ".antigravity/tickets.md".into(),
            agents_md_url:
                "https://codeberg.org/lumenpink/cccp/raw/branch/main/src/base/AGENTS_template.md"
                    .into(),
            agents_md_file: ".antigravity/AGENTS.md".into(),
            extra: HashMap::new(),
        }
    }
}

impl Config {
    /// Load hierarchical configuration cascading:
    /// Defaults -> Global config -> Local repo config (.cccprc) -> Environment variables
    pub fn load_hierarchical(repo_dir: Option<&Path>) -> Self {
        let mut config = Self::default();

        // 1. Global config file ($XDG_CONFIG_HOME/cccp/config or ~/.config/cccp/config or ~/.cccprc)
        if let Some(global_path) = find_global_config_path() {
            if let Ok(content) = fs::read_to_string(&global_path) {
                config.merge_from_str(&content);
            }
        }

        // 2. Local config file (<git-root>/.cccprc or current-dir/.cccprc)
        if let Some(local_path) = find_local_config_path(repo_dir) {
            if let Ok(content) = fs::read_to_string(&local_path) {
                config.merge_from_str(&content);
            }
        }

        // 3. Environment variables (CCCP_* or canonical env flags)
        config.merge_from_env();

        config
    }

    /// Merge settings from a configuration file content (either TOML or Key-Value lines)
    pub fn merge_from_str(&mut self, content: &str) {
        // First try parsing key-value pairs (which works for classic .cccprc and simple key = value)
        let kv = parse_key_value_lines(content);
        for (key, val) in kv {
            self.set_key_value(&key, &val);
        }
    }

    /// Set an individual key-value pair, routing to known fields or extra map
    pub fn set_key_value(&mut self, key: &str, val: &str) {
        let normalized = key.to_ascii_lowercase().replace('-', "_");
        match normalized.as_str() {
            "types" | "commit_types" => {
                self.commit_types = parse_string_list(val);
            }
            "scopes" | "commit_scopes" => {
                self.commit_scopes = parse_string_list(val);
            }
            "subscopes" | "commit_subscopes" => {
                self.commit_subscopes = parse_string_list(val);
            }
            "strict_types" => {
                self.strict_types = parse_bool(val, true);
            }
            "strict_scopes" => {
                self.strict_scopes = parse_bool(val, false);
            }
            "strict_subscopes" => {
                self.strict_subscopes = parse_bool(val, false);
            }
            "allow_any_scope" => {
                self.allow_any_scope = parse_bool(val, true);
                if !self.allow_any_scope {
                    self.strict_scopes = true;
                }
            }
            "allow_any_subscope" => {
                self.allow_any_subscope = parse_bool(val, true);
                if !self.allow_any_subscope {
                    self.strict_subscopes = true;
                }
            }
            "disable_subscopes" => {
                self.disable_subscopes = parse_bool(val, false);
            }
            "disable_multiple_scopes" => {
                self.disable_multiple_scopes = parse_bool(val, false);
            }
            "default_base_version" => {
                self.default_base_version = val.trim().to_string();
            }
            "no_v" => {
                self.no_v = parse_bool(val, false);
            }
            "post_push_hook" => {
                let s = val.trim();
                self.post_push_hook = if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                };
            }
            "post_push_sync" => {
                self.post_push_sync = parse_string_list(val);
            }
            "tickets_file" => {
                self.tickets_file = val.trim().to_string();
            }
            "agents_md_url" | "agent_md_url" => {
                self.agents_md_url = val.trim().to_string();
            }
            "agents_md_file" | "agent_md_file" => {
                self.agents_md_file = val.trim().to_string();
            }
            _ => {
                self.extra.insert(normalized, val.trim().to_string());
            }
        }
    }

    /// Read environment variables and apply overrides
    pub fn merge_from_env(&mut self) {
        if let Ok(v) = env::var("CCCP_TYPES").or_else(|_| env::var("COMMIT_TYPES")) {
            self.commit_types = parse_string_list(&v);
        }
        if let Ok(v) = env::var("CCCP_SCOPES").or_else(|_| env::var("COMMIT_SCOPES")) {
            self.commit_scopes = parse_string_list(&v);
        }
        if let Ok(v) = env::var("CCCP_SUBSCOPES").or_else(|_| env::var("COMMIT_SUBSCOPES")) {
            self.commit_subscopes = parse_string_list(&v);
        }
        if let Ok(v) = env::var("STRICT_TYPES").or_else(|_| env::var("CCCP_STRICT_TYPES")) {
            self.strict_types = parse_bool(&v, self.strict_types);
        }
        if let Ok(v) = env::var("STRICT_SCOPES").or_else(|_| env::var("CCCP_STRICT_SCOPES")) {
            self.strict_scopes = parse_bool(&v, self.strict_scopes);
        }
        if let Ok(v) = env::var("STRICT_SUBSCOPES").or_else(|_| env::var("CCCP_STRICT_SUBSCOPES")) {
            self.strict_subscopes = parse_bool(&v, self.strict_subscopes);
        }
        if let Ok(v) = env::var("ALLOW_ANY_SCOPE").or_else(|_| env::var("CCCP_ALLOW_ANY_SCOPE")) {
            self.allow_any_scope = parse_bool(&v, self.allow_any_scope);
        }
        if let Ok(v) =
            env::var("ALLOW_ANY_SUBSCOPE").or_else(|_| env::var("CCCP_ALLOW_ANY_SUBSCOPE"))
        {
            self.allow_any_subscope = parse_bool(&v, self.allow_any_subscope);
        }
        if let Ok(v) = env::var("DISABLE_SUBSCOPES").or_else(|_| env::var("CCCP_DISABLE_SUBSCOPES"))
        {
            self.disable_subscopes = parse_bool(&v, self.disable_subscopes);
        }
        if let Ok(v) =
            env::var("DEFAULT_BASE_VERSION").or_else(|_| env::var("CCCP_DEFAULT_BASE_VERSION"))
        {
            self.default_base_version = v.trim().to_string();
        }
        if let Ok(v) = env::var("NO_V").or_else(|_| env::var("CCCP_NO_V")) {
            self.no_v = parse_bool(&v, self.no_v);
        }
        if let Ok(v) = env::var("POST_PUSH_HOOK").or_else(|_| env::var("CCCP_POST_PUSH_HOOK")) {
            self.post_push_hook = Some(v.trim().to_string());
        }
        if let Ok(v) = env::var("POST_PUSH_SYNC").or_else(|_| env::var("CCCP_POST_PUSH_SYNC")) {
            self.post_push_sync = parse_string_list(&v);
        }
        if let Ok(v) = env::var("TICKETS_FILE").or_else(|_| env::var("CCCP_TICKETS_FILE")) {
            self.tickets_file = v.trim().to_string();
        }
    }
}

/// Helper: find path to global configuration file
pub fn find_global_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg).join("cccp").join("config");
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(home) = env::var("HOME") {
        let p = PathBuf::from(&home)
            .join(".config")
            .join("cccp")
            .join("config");
        if p.is_file() {
            return Some(p);
        }
        let rc = PathBuf::from(&home).join(".cccprc");
        if rc.is_file() {
            return Some(rc);
        }
    }
    None
}

/// Helper: find path to local .cccprc by scanning starting directory upwards
pub fn find_local_config_path(start_dir: Option<&Path>) -> Option<PathBuf> {
    let mut current = match start_dir {
        Some(p) => p.to_path_buf(),
        None => env::current_dir().ok()?,
    };

    loop {
        let candidate = current.join(".cccprc");
        if candidate.is_file() {
            return Some(candidate);
        }
        if current.join(".git").exists() {
            // Reached git root without finding .cccprc
            return None;
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Parse lines with key = value, comments (# or ;), export prefix, and optional quotes
pub fn parse_key_value_lines(content: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        let clean = trimmed.strip_prefix("export ").unwrap_or(trimmed).trim();
        if let Some((k, v)) = clean.split_once('=') {
            let key = k.trim().to_string();
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            result.push((key, val));
        }
    }
    result
}

/// Write or update a key=value pair in a configuration file
pub fn write_config_key(file: &Path, key: &str, value: &str) -> std::io::Result<()> {
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut lines = Vec::new();
    let mut updated = false;
    let normalized = key.to_ascii_lowercase().replace('-', "_");

    if file.exists() {
        let content = fs::read_to_string(file)?;
        for line in content.lines() {
            let trimmed = line.trim();
            let clean = trimmed.strip_prefix("export ").unwrap_or(trimmed).trim();
            if let Some((k, _)) = clean.split_once('=') {
                if k.trim().to_ascii_lowercase().replace('-', "_") == normalized {
                    lines.push(format!("{} = {}", normalized, value));
                    updated = true;
                    continue;
                }
            }
            lines.push(line.to_string());
        }
    }

    if !updated {
        lines.push(format!("{} = {}", normalized, value));
    }

    fs::write(file, lines.join("\n") + "\n")
}

fn parse_string_list(val: &str) -> Vec<String> {
    val.split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_bool(val: &str, default: bool) -> bool {
    let s = val.trim().to_ascii_lowercase();
    match s.as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let cfg = Config::default();
        assert!(cfg.strict_types);
        assert!(!cfg.strict_scopes);
        assert_eq!(cfg.default_base_version, "0.0.1");
        assert!(cfg.commit_types.contains(&"feat".to_string()));
        assert!(cfg.commit_types.contains(&"fix".to_string()));
    }

    #[test]
    fn test_parse_key_value_lines() {
        let raw = r#"
            # Comment line
            types = feat, fix, docs
            strict_types = 0
            export allow_any_scope = "false"
            default_base_version = '1.0.0'
            custom_directive = Maintain high Soviet standards
        "#;
        let mut cfg = Config::default();
        cfg.merge_from_str(raw);

        assert_eq!(cfg.commit_types, vec!["feat", "fix", "docs"]);
        assert!(!cfg.strict_types);
        assert!(!cfg.allow_any_scope);
        assert!(cfg.strict_scopes);
        assert_eq!(cfg.default_base_version, "1.0.0");
        assert_eq!(
            cfg.extra.get("custom_directive"),
            Some(&"Maintain high Soviet standards".to_string())
        );
    }

    #[test]
    fn test_local_overrides_global() {
        let mut cfg = Config::default();

        let global_content = "strict_types = 0\ndefault_base_version = 0.5.0\n";
        cfg.merge_from_str(global_content);
        assert!(!cfg.strict_types);
        assert_eq!(cfg.default_base_version, "0.5.0");

        let local_content = "default_base_version = 2.0.0\n";
        cfg.merge_from_str(local_content);
        // local overrides global default_base_version, but strict_types remains from global
        assert_eq!(cfg.default_base_version, "2.0.0");
        assert!(!cfg.strict_types);
    }
}
