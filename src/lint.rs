//! Strict Conventional Commits syntactic analyzer and linter.
//! Validates commit structure, types, scopes, subscopes, length limits, and body separation.

use crate::config::Config;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl LintResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn failed(errors: Vec<String>, warnings: Vec<String>) -> Self {
        Self {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// Parse and validate a commit message string
pub fn lint_commit_message(msg: &str, config: &Config, strict: bool) -> LintResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let trimmed_msg = msg.trim();
    if trimmed_msg.is_empty() {
        errors.push("Commit message cannot be empty".to_string());
        return LintResult::failed(errors, warnings);
    }

    let mut lines = msg.lines();
    let header = lines.next().unwrap_or("").trim();

    // 1. Header length rule (<= 72 chars recommended / enforced in strict)
    let header_len = header.chars().count();
    if header_len > 72 {
        let msg = format!("Header line exceeds 72 characters (length: {})", header_len);
        if strict {
            errors.push(msg);
        } else {
            warnings.push(msg);
        }
    }

    // 2. Mandatory blank line separating subject and body
    if let Some(second_line) = lines.next() {
        if !second_line.trim().is_empty() {
            errors.push(
                "A blank line is required between the subject line and commit body".to_string(),
            );
        }
    }

    // 3. Header structure: must contain a colon ':'
    let Some((header_prefix, subject_raw)) = header.split_once(':') else {
        errors.push(
            "Commit message must follow format '<type>(<scope>): <subject>' or '<type>: <subject>'"
                .to_string(),
        );
        return LintResult::failed(errors, warnings);
    };

    // 4. Subject checks
    let subject = subject_raw.trim();
    if subject.is_empty() {
        errors.push("Commit message must have a subject description".to_string());
    } else if subject.ends_with('.') {
        errors.push("Subject line must not end with a period/dot ('.')".to_string());
    }

    // 5. Breaking change indicator '!'
    let clean_prefix = if let Some(stripped) = header_prefix.strip_suffix('!') {
        stripped
    } else {
        header_prefix
    };

    // 6. Check empty parentheses (e.g. `feat():`)
    if clean_prefix.contains("()") {
        errors.push("Scope cannot be empty (found '()')".to_string());
        return LintResult::failed(errors, warnings);
    }

    // 7. Extract type and scope
    let (commit_type, scope_part) = if clean_prefix.contains('(') {
        if let Some((t, rest)) = clean_prefix.split_once('(') {
            if let Some(s) = rest.strip_suffix(')') {
                (t.trim(), s.trim())
            } else {
                errors.push("Unmatched parenthesis in commit scope".to_string());
                return LintResult::failed(errors, warnings);
            }
        } else {
            (clean_prefix.trim(), "")
        }
    } else {
        (clean_prefix.trim(), "")
    };

    // 8. Validate commit type
    if commit_type.is_empty() {
        errors.push("Commit type cannot be empty".to_string());
    } else if config.strict_types {
        let is_valid_type = config.commit_types.iter().any(|t| t == commit_type);
        if !is_valid_type {
            errors.push(format!(
                "Invalid type '{}'. Must be one of: {}",
                commit_type,
                config.commit_types.join(", ")
            ));
        }
    } else if !commit_type
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        errors.push(format!(
            "Invalid type '{}'. Type must consist of alphanumeric characters, hyphens, or underscores",
            commit_type
        ));
    }

    // 9. Validate scopes & subscopes
    if !scope_part.is_empty() {
        let scopes: Vec<&str> = scope_part.split(',').map(|s| s.trim()).collect();

        if scopes.len() > 1 && config.disable_multiple_scopes {
            errors.push("Multiple scopes are disabled by configuration".to_string());
        }

        for scope_item in scopes {
            if scope_item.is_empty() {
                errors.push("Empty scope item found in scope list".to_string());
                continue;
            }

            if scope_item.contains('/') {
                if config.disable_subscopes {
                    errors.push(format!(
                        "Subscopes are disabled by configuration (found '{}')",
                        scope_item
                    ));
                    continue;
                }

                if let Some((scope, subscope)) = scope_item.split_once('/') {
                    let scope = scope.trim();
                    let subscope = subscope.trim();

                    if config.strict_scopes && !config.allow_any_scope {
                        let is_valid = config.commit_scopes.iter().any(|s| s == scope);
                        if !is_valid {
                            errors.push(format!(
                                "Invalid scope '{}'. Must be one of: {}",
                                scope,
                                config.commit_scopes.join(", ")
                            ));
                        }
                    }

                    if config.strict_subscopes && !config.allow_any_subscope {
                        let is_valid = config.commit_subscopes.iter().any(|ss| ss == subscope);
                        if !is_valid {
                            errors.push(format!(
                                "Invalid subscope '{}'. Must be one of: {}",
                                subscope,
                                config.commit_subscopes.join(", ")
                            ));
                        }
                    }
                }
            } else if config.strict_scopes && !config.allow_any_scope {
                let is_valid = config.commit_scopes.iter().any(|s| s == scope_item);
                if !is_valid {
                    errors.push(format!(
                        "Invalid scope '{}'. Must be one of: {}",
                        scope_item,
                        config.commit_scopes.join(", ")
                    ));
                }
            }
        }
    }

    LintResult::failed(errors, warnings)
}

/// Lint a commit message from a file (e.g. .git/COMMIT_EDITMSG)
pub fn lint_commit_file(file: &Path, config: &Config, strict: bool) -> Result<LintResult, String> {
    let content = std::fs::read_to_string(file).map_err(|e| {
        format!(
            "Failed to read commit message file '{}': {}",
            file.display(),
            e
        )
    })?;

    // Filter out git comment lines (starting with #)
    let cleaned_lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect();
    let cleaned_msg = cleaned_lines.join("\n");

    Ok(lint_commit_message(&cleaned_msg, config, strict))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_conventional_commits() {
        let cfg = Config::default();

        let r1 = lint_commit_message("feat: add awesome feature", &cfg, false);
        assert!(r1.is_valid, "Expected valid: {:?}", r1.errors);

        let r2 = lint_commit_message("fix(core): fix nil pointer dereference", &cfg, false);
        assert!(r2.is_valid, "Expected valid: {:?}", r2.errors);

        let r3 = lint_commit_message("feat(api)!: breaking change endpoint", &cfg, false);
        assert!(r3.is_valid, "Expected valid: {:?}", r3.errors);

        let r4 = lint_commit_message(
            "docs(readme): update guide\n\nDetailed explanation",
            &cfg,
            false,
        );
        assert!(r4.is_valid, "Expected valid: {:?}", r4.errors);
    }

    #[test]
    fn test_missing_colon() {
        let cfg = Config::default();
        let res = lint_commit_message("feat add feature without colon", &cfg, false);
        assert!(!res.is_valid);
        assert!(res.errors[0].contains("must follow format"));
    }

    #[test]
    fn test_missing_blank_line_before_body() {
        let cfg = Config::default();
        let res = lint_commit_message("feat: add feature\nNo blank line body here", &cfg, false);
        assert!(!res.is_valid);
        assert!(res
            .errors
            .iter()
            .any(|e| e.contains("blank line is required")));
    }

    #[test]
    fn test_trailing_dot_rejection() {
        let cfg = Config::default();
        let res = lint_commit_message("feat: add feature with trailing dot.", &cfg, false);
        assert!(!res.is_valid);
        assert!(res.errors.iter().any(|e| e.contains("period/dot")));
    }

    #[test]
    fn test_invalid_type() {
        let cfg = Config::default();
        let res = lint_commit_message("badtype: something", &cfg, false);
        assert!(!res.is_valid);
        assert!(res
            .errors
            .iter()
            .any(|e| e.contains("Invalid type 'badtype'")));
    }

    #[test]
    fn test_empty_scope() {
        let cfg = Config::default();
        let res = lint_commit_message("feat(): empty scope", &cfg, false);
        assert!(!res.is_valid);
        assert!(res
            .errors
            .iter()
            .any(|e| e.contains("Scope cannot be empty")));
    }

    #[test]
    fn test_strict_length_limit() {
        let cfg = Config::default();
        let long_header = format!("feat: {}", "a".repeat(80));

        // Normal mode produces a warning
        let res_normal = lint_commit_message(&long_header, &cfg, false);
        assert!(res_normal.is_valid);
        assert!(!res_normal.warnings.is_empty());

        // Strict mode produces an error
        let res_strict = lint_commit_message(&long_header, &cfg, true);
        assert!(!res_strict.is_valid);
        assert!(!res_strict.errors.is_empty());
    }

    #[test]
    fn test_multiple_scopes_disabling() {
        let mut cfg = Config::default();
        cfg.disable_multiple_scopes = true;

        let res = lint_commit_message("feat(ui,api): dual scope", &cfg, false);
        assert!(!res.is_valid);
        assert!(res
            .errors
            .iter()
            .any(|e| e.contains("Multiple scopes are disabled")));
    }

    #[test]
    fn test_subscopes_disabling() {
        let mut cfg = Config::default();
        cfg.disable_subscopes = true;

        let res = lint_commit_message("feat(ui/button): subscoped", &cfg, false);
        assert!(!res.is_valid);
        assert!(res
            .errors
            .iter()
            .any(|e| e.contains("Subscopes are disabled")));
    }
}
