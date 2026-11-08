//! Lua Rules Engine for GOST Compliance
//! Executes declarative rules defined in `rules.toml` with the `gost.*` runtime.

use crate::audit::{AuditCheck, CheckStatus};
use mlua::{Lua, LuaSerdeExt, MultiValue, Value};
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

pub fn load_rules_from_repo(root: &Path) -> HashMap<String, RuleDefinition> {
    let mut all_rules = HashMap::new();
    let candidates = [
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
    let file_names = [format!("{}.toml", lang), "pt-br.toml".to_string(), "en.toml".to_string()];

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

fn list_files_recursive(dir: &Path, base: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel) = path.strip_prefix(base) {
                    files.push(rel.to_string_lossy().to_string());
                }
            } else if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name != ".git" && name != "target" && name != "node_modules" {
                    files.extend(list_files_recursive(&path, base));
                }
            }
        }
    }
    files
}

pub fn create_lua_environment(root: &Path) -> mlua::Result<Lua> {
    let lua = Lua::new();
    let root_path = root.to_path_buf();

    let gost = lua.create_table()?;

    // 1. gost.file_exists(path)
    let rp = root_path.clone();
    let file_exists = lua.create_function(move |_, path: String| {
        let p = rp.join(&path);
        Ok(p.exists())
    })?;
    gost.set("file_exists", file_exists)?;

    // 2. gost.is_dir(path)
    let rp = root_path.clone();
    let is_dir = lua.create_function(move |_, path: String| {
        let p = rp.join(&path);
        Ok(p.is_dir())
    })?;
    gost.set("is_dir", is_dir)?;

    // 3. gost.read_file(path)
    let rp = root_path.clone();
    let read_file = lua.create_function(move |_, path: String| {
        let p = rp.join(&path);
        let content = fs::read_to_string(p).ok();
        Ok(content)
    })?;
    gost.set("read_file", read_file)?;

    // 4. gost.count_lines(path)
    let rp = root_path.clone();
    let count_lines = lua.create_function(move |_, path: String| {
        let p = rp.join(&path);
        let count = match fs::read_to_string(p) {
            Ok(s) => s.lines().count(),
            Err(_) => 0,
        };
        Ok(count)
    })?;
    gost.set("count_lines", count_lines)?;

    // 5. gost.list_files(dir)
    let rp = root_path.clone();
    let list_files = lua.create_function(move |lua, dir_str: Option<String>| {
        let target_dir = match dir_str {
            Some(d) if !d.is_empty() => rp.join(d),
            _ => rp.clone(),
        };
        let files = list_files_recursive(&target_dir, &rp);
        let table = lua.create_table()?;
        for (i, f) in files.into_iter().enumerate() {
            table.set(i + 1, f)?;
        }
        Ok(table)
    })?;
    gost.set("list_files", list_files)?;

    // 6. gost.git_branch()
    let rp = root_path.clone();
    let git_branch = lua.create_function(move |_, ()| {
        if let Ok(repo) = git2::Repository::open(&rp) {
            if let Ok(head) = repo.head() {
                if let Some(name) = head.shorthand() {
                    return Ok(Some(name.to_string()));
                }
            }
        }
        Ok(None)
    })?;
    gost.set("git_branch", git_branch)?;

    // 7. gost.git_commits(limit)
    let rp = root_path.clone();
    let git_commits = lua.create_function(move |lua, limit: Option<usize>| {
        let max_commits = limit.unwrap_or(10);
        let list = lua.create_table()?;

        if let Ok(repo) = git2::Repository::open(&rp) {
            if let Ok(mut revwalk) = repo.revwalk() {
                if revwalk.push_head().is_ok() {
                    let mut idx = 1;
                    for oid_res in revwalk {
                        if idx > max_commits {
                            break;
                        }
                        if let Ok(oid) = oid_res {
                            if let Ok(commit) = repo.find_commit(oid) {
                                let commit_table = lua.create_table()?;
                                commit_table.set("hash", commit.id().to_string())?;
                                let msg = commit.message().unwrap_or_default();
                                let mut lines = msg.lines();
                                let header = lines.next().unwrap_or_default().to_string();
                                let body = lines.collect::<Vec<_>>().join("\n");
                                commit_table.set("header", header)?;
                                commit_table.set("body", body)?;
                                commit_table.set("author", commit.author().name().unwrap_or_default().to_string())?;
                                list.set(idx, commit_table)?;
                                idx += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(list)
    })?;
    gost.set("git_commits", git_commits)?;

    // 8. gost.parse_toml(content)
    let parse_toml = lua.create_function(|lua, content: String| {
        if let Ok(val) = toml::from_str::<toml::Value>(&content) {
            return lua.to_value(&val);
        }
        Ok(Value::Nil)
    })?;
    gost.set("parse_toml", parse_toml)?;

    // 9. gost.parse_json(content)
    let parse_json = lua.create_function(|lua, content: String| {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            return lua.to_value(&val);
        }
        Ok(Value::Nil)
    })?;
    gost.set("parse_json", parse_json)?;

    lua.globals().set("gost", gost)?;
    Ok(lua)
}

fn interpolate_message(template: &str, params: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, val) in params {
        result = result.replace(&format!("{{{}}}", key), val);
    }
    result
}

pub fn evaluate_lua_rules(root: &Path, disabled_rules: &[String]) -> Vec<AuditCheck> {
    let mut checks = Vec::new();
    let rules = load_rules_from_repo(root);
    if rules.is_empty() {
        return checks;
    }

    let locales = load_locales_from_repo(root, "pt-br");
    let lua = match create_lua_environment(root) {
        Ok(l) => l,
        Err(e) => {
            checks.push(AuditCheck {
                code: "LUA-ENV-ERROR",
                title: "Falha na Inicialização do Interpretador Lua",
                detail: e.to_string(),
                status: CheckStatus::Fail,
            });
            return checks;
        }
    };

    // Sort rule IDs for deterministic output
    let mut sorted_keys: Vec<_> = rules.keys().cloned().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        let rule = &rules[&key];

        // Check if rule is disabled
        if disabled_rules.contains(&key) {
            checks.push(AuditCheck {
                code: Box::leak(key.into_boxed_str()),
                title: Box::leak(rule.name.clone().into_boxed_str()),
                detail: format!("Regra dispensada por configuração ({})", rule.description),
                status: CheckStatus::Pass, // Marked as skip/pass in report
            });
            continue;
        }

        if rule.script.trim().is_empty() {
            continue;
        }

        let eval_result: mlua::Result<MultiValue> = lua.load(&rule.script).eval();
        match eval_result {
            Ok(values) => {
                let mut iter = values.into_iter();
                let status_val = iter.next().unwrap_or(Value::Boolean(true));
                let is_pass = match status_val {
                    Value::Boolean(b) => b,
                    Value::Nil => false,
                    _ => true,
                };

                if is_pass {
                    checks.push(AuditCheck {
                        code: Box::leak(key.into_boxed_str()),
                        title: Box::leak(rule.name.clone().into_boxed_str()),
                        detail: rule.description.clone(),
                        status: CheckStatus::Pass,
                    });
                } else {
                    let err_key = match iter.next() {
                        Some(Value::String(s)) => s.to_str().map(|b| b.to_string()).unwrap_or_default(),
                        _ => "ERR_FAILED".to_string(),
                    };

                    let mut params = HashMap::new();
                    if let Some(Value::Table(tbl)) = iter.next() {
                        for pair in tbl.pairs::<String, Value>() {
                            if let Ok((k, v)) = pair {
                                let v_str = match v {
                                    Value::String(s) => s.to_str().map(|b| b.to_string()).unwrap_or_default(),
                                    Value::Integer(i) => i.to_string(),
                                    Value::Number(n) => n.to_string(),
                                    Value::Boolean(b) => b.to_string(),
                                    _ => String::new(),
                                };
                                params.insert(k, v_str);
                            }
                        }
                    }

                    // Resolve message from locales -> rule.messages -> fallback
                    let template = locales
                        .get(&key)
                        .and_then(|le| le.messages.get(&err_key))
                        .or_else(|| rule.messages.get(&err_key))
                        .cloned()
                        .unwrap_or_else(|| {
                            if !rule.description.is_empty() {
                                format!("{}: {}", err_key, rule.description)
                            } else {
                                err_key.clone()
                            }
                        });

                    let resolved_detail = interpolate_message(&template, &params);

                    checks.push(AuditCheck {
                        code: Box::leak(key.into_boxed_str()),
                        title: Box::leak(rule.name.clone().into_boxed_str()),
                        detail: resolved_detail,
                        status: CheckStatus::Fail,
                    });
                }
            }
            Err(e) => {
                checks.push(AuditCheck {
                    code: Box::leak(key.into_boxed_str()),
                    title: Box::leak(rule.name.clone().into_boxed_str()),
                    detail: format!("Erro de execução no script Lua: {}", e),
                    status: CheckStatus::Fail,
                });
            }
        }
    }

    checks
}
