//! CCCP GOST Standards Compliance & Quality Audit Engine
//! Implements agnostic repository inspection executing declarative rules via Lunokhod.

use crate::cultural::Theme;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

#[derive(Debug, Clone)]
pub struct AuditCheck {
    pub code: &'static str,
    pub title: &'static str,
    pub detail: String,
    pub status: CheckStatus,
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub gost_version: String,
    pub profile: String,
    pub languages: String,
    pub checks: Vec<AuditCheck>,
    pub passed: usize,
    pub warnings: usize,
    pub failures: usize,
    pub percentage: usize,
    pub is_success: bool,
}

/// Autodetecta a linguagem/stack predominante inspecionando os manifestos do repositório
pub fn autodetect_languages(root: &Path) -> String {
    if root.join("Cargo.toml").is_file() {
        "rust".to_string()
    } else if root.join("package.json").is_file() {
        "javascript".to_string()
    } else if root.join("composer.json").is_file() {
        "php".to_string()
    } else if root.join("go.mod").is_file() {
        "go".to_string()
    } else if root.join("pyproject.toml").is_file() || root.join("requirements.txt").is_file() {
        "python".to_string()
    } else if root.join("DESCRIPTION").is_file() {
        "r".to_string()
    } else {
        "none".to_string()
    }
}

/// Extrai metadados do frontmatter YAML de `.gost/GOST.md`
fn extract_gost_frontmatter(content: &str) -> (Option<String>, Option<String>, Option<String>) {
    let mut standard = None;
    let mut version = None;
    let mut languages = None;
    let mut in_yaml = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_yaml = !in_yaml;
            if !in_yaml && standard.is_some() && version.is_some() && languages.is_some() {
                break;
            }
            continue;
        }

        // Se encontrar tags HTML (ex: comentários <!-- ... --> de separação), não é bloco YAML
        if in_yaml && trimmed.starts_with('<') {
            in_yaml = false;
            continue;
        }

        if in_yaml {
            if let Some(rest) = trimmed.strip_prefix("standard:") {
                let clean = rest
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if standard.is_none() && !clean.is_empty() {
                    standard = Some(clean);
                }
            } else if let Some(rest) = trimmed.strip_prefix("version:") {
                let clean = rest
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if version.is_none() && !clean.is_empty() {
                    version = Some(clean);
                }
            } else if let Some(rest) = trimmed.strip_prefix("languages:") {
                let clean = rest
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches('[')
                    .trim_matches(']')
                    .replace(['"', '\''], "")
                    .trim()
                    .to_string();
                if languages.is_none() && !clean.is_empty() {
                    languages = Some(clean);
                }
            }
        }
    }

    (version, standard, languages)
}

/// Parse GOST specification and metadata from `.gost/` (with fallback to legacy GOSTVER)
pub fn read_gostver(root: &Path) -> (String, String, String) {
    let mut ver = String::new();
    let mut prof = String::new();
    let mut langs = String::new();

    // 1. Prioridade GOST v3: extrair do frontmatter de .gost/GOST.md
    let gost_md = root.join(".gost/GOST.md");
    if let Ok(content) = fs::read_to_string(&gost_md) {
        let (extracted_ver, extracted_prof, extracted_langs) = extract_gost_frontmatter(&content);
        if let Some(v) = extracted_ver {
            ver = v;
        }
        if let Some(p) = extracted_prof {
            prof = p;
        }
        if let Some(l) = extracted_langs {
            langs = l;
        }
    }

    // 2. Fallback transiente: checar arquivo legado GOSTVER na raiz
    let gostver_file = root.join("GOSTVER");
    if gostver_file.is_file() {
        if let Ok(content) = fs::read_to_string(&gostver_file) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((raw_key, raw_val)) = trimmed.split_once('=') {
                    let key = raw_key.trim();
                    let val = raw_val.trim().trim_matches('"').trim_matches('\'').trim();
                    if key == "GOST_VERSION" && ver.is_empty() {
                        ver = val.to_string();
                    } else if key == "GOST_PROFILE_URL" && prof.is_empty() {
                        prof = val.to_string();
                    } else if key == "GOST_LANGUAGES" && langs.is_empty() {
                        langs = val.to_string();
                    }
                }
            }
        }
    }

    // 3. Fallback de perfil: checar .rules/profile.toml se presente
    if (ver.is_empty() || prof.is_empty()) && root.join(".rules/profile.toml").is_file() {
        if let Ok(profile_cfg) = crate::profile::read_profile_config(root) {
            if ver.is_empty() {
                ver = profile_cfg.version;
            }
            if prof.is_empty() {
                prof = profile_cfg.url;
            }
        }
    }

    // 4. Autodetecção de linguagem caso não declarada explicitamente
    if langs.is_empty() || langs.eq_ignore_ascii_case("none") {
        langs = autodetect_languages(root);
    }

    (
        if ver.is_empty() {
            "Undeclared".to_string()
        } else {
            ver
        },
        if prof.trim().is_empty() || prof.trim().eq_ignore_ascii_case("none") {
            "None".to_string()
        } else {
            prof
        },
        if langs.is_empty() || langs.eq_ignore_ascii_case("none") {
            "None".to_string()
        } else {
            langs
        },
    )
}

/// Extracts declared APP_TYPE from the GOSTVER file in a repository directory,
/// defaulting to "binary" if Cargo.toml exists, or "manifesto" otherwise.
pub fn read_app_type(root: &Path) -> String {
    let gostver_file = root.join("GOSTVER");
    if let Ok(content) = fs::read_to_string(gostver_file) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("APP_TYPE=") {
                let val = rest
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim()
                    .to_lowercase();
                if !val.is_empty() {
                    return val;
                }
            }
        }
    }
    if root.join("Cargo.toml").is_file() {
        "binary".to_string()
    } else {
        "manifesto".to_string()
    }
}

/// Executes full GOST compliance check on the specified root directory
pub fn run_gost_audit(root: &Path, strict: bool, theme: &Theme) -> AuditReport {
    run_gost_audit_with_options(root, strict, None, None, theme)
}

/// Executes GOST compliance check filtering Lua rules by scope
pub fn run_gost_audit_with_scope(
    root: &Path,
    strict: bool,
    scope: Option<&str>,
    theme: &Theme,
) -> AuditReport {
    run_gost_audit_with_options(root, strict, scope, None, theme)
}

/// Executes GOST compliance check filtering Lua rules by scope and locale language
pub fn run_gost_audit_with_options(
    root: &Path,
    strict: bool,
    scope: Option<&str>,
    lang: Option<&str>,
    theme: &Theme,
) -> AuditReport {
    run_gost_audit_full(root, strict, false, scope, lang, theme)
}

/// Executes GOST compliance check with scope, language and failures-only filtering
pub fn run_gost_audit_full(
    root: &Path,
    strict: bool,
    failures_only: bool,
    scope: Option<&str>,
    lang: Option<&str>,
    theme: &Theme,
) -> AuditReport {
    let (gost_ver, gost_prof, gost_langs) = read_gostver(root);
    let mut checks = Vec::new();

    // Check if declarative rules catalog exists (.rules/RULES.toml, .rules/rules.toml, .gost/rules.toml, rules.toml or specs/rules.toml).
    let has_rules = root.join(".rules/RULES.toml").is_file()
        || root.join(".rules/rules.toml").is_file()
        || root.join(".gost/rules.toml").is_file()
        || root.join("rules.toml").is_file()
        || root.join("specs/rules.toml").is_file();

    // If rules are absent but profile/gostver is present, attempt automatic assembly
    if !has_rules
        && (root.join(".rules/profile.toml").is_file()
            || root.join("GOSTVER").is_file()
            || root.join("profile.md").is_file())
    {
        let _ = crate::profile::assemble_rules_specs(root);
    }

    let disabled_rules = crate::lua_rules::read_disabled_rules(root);
    let target_scope = scope.unwrap_or("project");
    let chosen_lang = lang.unwrap_or("en");
    let lua_checks = crate::lua_rules::evaluate_lua_rules_with_lang(
        root,
        &disabled_rules,
        chosen_lang,
        Some(target_scope),
    );

    if lua_checks.is_empty() {
        checks.push(AuditCheck {
            code: "NO-RULES-FOUND",
            title: "Declarative Rules Catalog",
            detail:
                "No declarative rules catalog found (.rules/RULES.toml, .rules/rules.toml, specs/rules.toml)"
                    .to_string(),
            status: CheckStatus::Fail,
        });
    } else {
        checks.extend(lua_checks);
    }

    let passed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Pass)
        .count();
    let warnings = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();
    let failures = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let total = passed + warnings + failures;
    let percentage = (passed * 100).checked_div(total).unwrap_or(100);
    let is_success = failures == 0 && (!strict || warnings == 0);

    let report = AuditReport {
        gost_version: gost_ver,
        profile: gost_prof,
        languages: gost_langs,
        checks,
        passed,
        warnings,
        failures,
        percentage,
        is_success,
    };

    display_audit_report_with_options(&report, strict, failures_only, theme);
    report
}

/// Formats the Normative Conformance Audit Report into an English string
pub fn render_audit_report(report: &AuditReport, strict: bool, theme: &Theme) -> String {
    render_audit_report_with_options(report, strict, false, theme)
}

/// Formats the Normative Conformance Audit Report into an English string with optional failure filtering
pub fn render_audit_report_with_options(
    report: &AuditReport,
    strict: bool,
    failures_only: bool,
    theme: &Theme,
) -> String {
    if failures_only && report.failures == 0 && (!strict || report.warnings == 0) {
        return format!(
            "\n{} {}\n\n",
            theme.green("✔"),
            theme.bold(&format!(
                "All {} checks passed (100% compliance)",
                report.checks.len()
            ))
        );
    }

    let mut out = String::new();
    out.push('\n');
    out.push_str(&theme.bold("☭ [RULES] NORMATIVE CONFORMANCE AUDIT"));
    if failures_only {
        out.push_str(&format!(" {}", theme.yellow("(FAILURES ONLY)")));
    }
    out.push('\n');
    out.push_str(&format!(
        "Standard: {} | Profile: {} | Languages: {}\n",
        report.gost_version, report.profile, report.languages
    ));
    out.push_str(
        &theme.dim("────────────────────────────────────────────────────────────────────────────"),
    );
    out.push('\n');

    for check in &report.checks {
        if failures_only && check.status != CheckStatus::Fail && check.status != CheckStatus::Warn {
            continue;
        }
        let status_badge = match check.status {
            CheckStatus::Pass => theme.green("[PASS]"),
            CheckStatus::Warn => theme.yellow("[WARN]"),
            CheckStatus::Fail => theme.red_bold("[FAIL]"),
            CheckStatus::Skip => theme.cyan("[SKIP]"),
        };
        out.push_str(&format!(
            "  {} {:<25} : {}\n",
            status_badge, check.code, check.detail
        ));
    }

    out.push_str(
        &theme.dim("────────────────────────────────────────────────────────────────────────────"),
    );
    out.push('\n');
    out.push_str(&format!(
        "Total checks: {} | Conforming: {} | Warnings: {} | Failures: {}\n",
        report.checks.len(),
        theme.green(&report.passed.to_string()),
        theme.yellow(&report.warnings.to_string()),
        if report.failures > 0 {
            theme.red_bold(&report.failures.to_string())
        } else {
            theme.green("0")
        }
    ));
    out.push_str(&format!(
        "Compliance Index: {}\n\n",
        theme.bold(&format!("{}%", report.percentage))
    ));

    if report.failures > 0 {
        out.push_str(
            &theme
                .red_bold("✘ Critical non-conformances detected. Fix items marked with [FAIL].\n"),
        );
    } else if strict && report.warnings > 0 {
        out.push_str(
            &theme.red_bold("✘ Strict mode enabled: warnings treated as blocking failures.\n"),
        );
    } else {
        out.push_str(&theme.green("✔ Project conforms with normative technical standards.\n"));
    }

    out
}

/// Renders the Normative Conformance Audit Report to standard output
pub fn display_audit_report(report: &AuditReport, strict: bool, theme: &Theme) {
    display_audit_report_with_options(report, strict, false, theme);
}

/// Renders the Normative Conformance Audit Report to standard output with failure filtering options
pub fn display_audit_report_with_options(
    report: &AuditReport,
    strict: bool,
    failures_only: bool,
    theme: &Theme,
) {
    print!(
        "{}",
        render_audit_report_with_options(report, strict, failures_only, theme)
    );
}
