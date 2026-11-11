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

/// Parse GOSTVER file if present
pub fn read_gostver(root: &Path) -> (String, String, String) {
    let gostver_file = root.join("GOSTVER");
    let mut ver = String::new();
    let mut prof = String::new();
    let mut langs = String::new();

    if let Ok(content) = fs::read_to_string(gostver_file) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("GOST_VERSION=") {
                ver = rest.trim_matches('"').trim_matches('\'').to_string();
            } else if let Some(rest) = trimmed.strip_prefix("GOST_PROFILE_URL=") {
                prof = rest.trim_matches('"').trim_matches('\'').to_string();
            } else if let Some(rest) = trimmed.strip_prefix("GOST_LANGUAGES=") {
                langs = rest.trim_matches('"').trim_matches('\'').to_string();
            }
        }
    }

    (
        if ver.is_empty() {
            "Não declarado".to_string()
        } else {
            ver
        },
        if prof.is_empty() {
            "Nenhum".to_string()
        } else {
            prof
        },
        if langs.is_empty() {
            "Nenhuma".to_string()
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
    run_gost_audit_with_scope(root, strict, None, theme)
}

/// Executes GOST compliance check filtering Lua rules by scope
pub fn run_gost_audit_with_scope(
    root: &Path,
    strict: bool,
    scope: Option<&str>,
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
    if !has_rules && (root.join(".rules/profile.toml").is_file() || root.join("GOSTVER").is_file() || root.join("profile.md").is_file()) {
        let _ = crate::profile::assemble_rules_specs(root);
    }

    let disabled_rules = crate::lua_rules::read_disabled_rules(root);
    let target_scope = scope.unwrap_or("project");
    let lua_checks =
        crate::lua_rules::evaluate_lua_rules_for_scope(root, &disabled_rules, Some(target_scope));

    if lua_checks.is_empty() {
        checks.push(AuditCheck {
            code: "NO-RULES-FOUND",
            title: "Catálogo de Regras Declarativas",
            detail:
                "Nenhum catálogo de regras declarativas encontrado (.gost/rules.toml, rules.toml)"
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

    display_audit_report(&report, strict, theme);
    report
}

/// Renders the Gosplan Quality Audit Report to standard output
pub fn display_audit_report(report: &AuditReport, strict: bool, theme: &Theme) {
    println!();
    println!(
        "{}",
        theme.bold("☭ [ГОСТ / GOSPLAN] COMISSÃO DE CONTROLE DE QUALIDADE GOST")
    );
    println!(
        "Norma: {} | Perfil: {} | Linguagens: {}",
        report.gost_version, report.profile, report.languages
    );
    println!(
        "{}",
        theme.dim("────────────────────────────────────────────────────────────────────────────")
    );

    for check in &report.checks {
        let status_badge = match check.status {
            CheckStatus::Pass => theme.green("[PASS]"),
            CheckStatus::Warn => theme.yellow("[WARN]"),
            CheckStatus::Fail => theme.red_bold("[FAIL]"),
            CheckStatus::Skip => theme.cyan("[SKIP]"),
        };
        println!("  {} {:<25} : {}", status_badge, check.code, check.detail);
    }

    println!(
        "{}",
        theme.dim("────────────────────────────────────────────────────────────────────────────")
    );
    println!(
        "Total de verificações: {} | Conformes: {} | Avisos: {} | Falhas: {}",
        report.checks.len(),
        theme.green(&report.passed.to_string()),
        theme.yellow(&report.warnings.to_string()),
        if report.failures > 0 {
            theme.red_bold(&report.failures.to_string())
        } else {
            theme.green("0")
        }
    );
    println!(
        "Índice de Conformidade Gosplan: {}",
        theme.bold(&format!("{}%", report.percentage))
    );
    println!();

    if report.failures > 0 {
        eprintln!(
            "{}",
            theme.red_bold(
                "✘ Inconformidades críticas detectadas. Corrija os itens marcados com [FAIL]."
            )
        );
    } else if strict && report.warnings > 0 {
        eprintln!(
            "{}",
            theme.red_bold("✘ Modo estrito ativado: avisos tratados como falhas bloqueantes.")
        );
    } else {
        println!(
            "{}",
            theme.green("✔ Projeto aprovado pela Comissão de Normas Técnicas GOST.")
        );
    }
}
