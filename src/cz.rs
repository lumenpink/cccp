//! Interactive Conventional Commits wizard (Commitizen / Soviet Assembly Line).
//! Built with `inquire` for modern, responsive terminal prompts.

use crate::cli::CzArgs;
use crate::config::Config;
use crate::cultural::Theme;
use inquire::validator::Validation;
use inquire::{Confirm, Select, Text};

/// Composed Conventional Commit message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposedCommit {
    pub commit_type: String,
    pub scope: Option<String>,
    pub is_breaking: bool,
    pub subject: String,
    pub body: Option<String>,
    pub breaking_description: Option<String>,
    pub issue_references: Option<String>,
    pub formatted_message: String,
}

#[derive(Debug, Clone)]
struct TypeOption {
    name: &'static str,
    description: &'static str,
}

impl std::fmt::Display for TypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<10} — {}", self.name, self.description)
    }
}

/// Run the interactive wizard or assemble commit message from CLI flags
pub fn compose_commit(
    args: &CzArgs,
    config: &Config,
    theme: &Theme,
) -> Result<ComposedCommit, String> {
    // If message is supplied via CLI flags, assemble non-interactively
    if let Some(ref m) = args.message {
        let commit_type = match &args.commit_type {
            Some(t) => t.clone(),
            None => {
                return Err(
                    "Commit type must be specified with -t/--type when using -m/--message non-interactively"
                        .to_string(),
                );
            }
        };
        validate_subject_string(m)?;
        let subject = m.trim().to_string();
        let scope = args.scope.as_ref().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let body = args.body.as_ref().and_then(|b| {
            let trimmed = b.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let is_breaking = args.breaking;
        let breaking_desc: Option<String> = None;
        let issues: Option<String> = None;

        let formatted_message = format_conventional_commit(
            &commit_type,
            scope.as_deref(),
            is_breaking,
            &subject,
            body.as_deref(),
            breaking_desc.as_deref(),
            issues.as_deref(),
        );

        println!("\n{}", theme.bold("★ Proposed Commit Message ★"));
        println!("{}\n", theme.cyan(&formatted_message));

        return Ok(ComposedCommit {
            commit_type,
            scope,
            is_breaking,
            subject,
            body,
            breaking_description: breaking_desc,
            issue_references: issues,
            formatted_message,
        });
    }

    let commit_type = match &args.commit_type {
        Some(t) => t.clone(),
        None => prompt_commit_type(config)?,
    };

    let scope = match &args.scope {
        Some(s) => {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        None => prompt_scope(config)?,
    };

    let is_breaking = if args.breaking {
        true
    } else {
        prompt_is_breaking()?
    };

    let subject = match &args.message {
        Some(m) => {
            validate_subject_string(m)?;
            m.trim().to_string()
        }
        None => prompt_subject()?,
    };

    let body = match &args.body {
        Some(b) => {
            let trimmed = b.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        None => prompt_body()?,
    };

    let breaking_desc = if is_breaking {
        prompt_breaking_description()?
    } else {
        None
    };

    let issues = prompt_issues()?;

    // Format full Conventional Commit message
    let formatted_message = format_conventional_commit(
        &commit_type,
        scope.as_deref(),
        is_breaking,
        &subject,
        body.as_deref(),
        breaking_desc.as_deref(),
        issues.as_deref(),
    );

    // Display formatted preview
    println!("\n{}", theme.bold("★ Proposed Commit Message ★"));
    println!("{}\n", theme.cyan(&formatted_message));

    Ok(ComposedCommit {
        commit_type,
        scope,
        is_breaking,
        subject,
        body,
        breaking_description: breaking_desc,
        issue_references: issues,
        formatted_message,
    })
}

fn prompt_commit_type(_config: &Config) -> Result<String, String> {
    let type_options = vec![
        TypeOption {
            name: "feat",
            description: "A new feature for the collective",
        },
        TypeOption {
            name: "fix",
            description: "A bug fix (correcting imperialist sabotage)",
        },
        TypeOption {
            name: "docs",
            description: "Documentation updates or State archives",
        },
        TypeOption {
            name: "style",
            description: "Code style/formatting changes (no logic changes)",
        },
        TypeOption {
            name: "refactor",
            description: "Code refactoring without changing functionality",
        },
        TypeOption {
            name: "perf",
            description: "Performance optimization (Stakhanovite efficiency)",
        },
        TypeOption {
            name: "test",
            description: "Adding or correcting tests",
        },
        TypeOption {
            name: "build",
            description: "Changes affecting build system or dependencies",
        },
        TypeOption {
            name: "ci",
            description: "Continuous integration / automated factory pipelines",
        },
        TypeOption {
            name: "chore",
            description: "Routine maintenance tasks and housekeeping",
        },
        TypeOption {
            name: "revert",
            description: "Reverting a previous state decree",
        },
    ];

    let prompt_text = crate::t!("commit.type_prompt");
    let choice = Select::new(&prompt_text, type_options)
        .prompt()
        .map_err(|e| format!("Prompt aborted: {}", e))?;

    Ok(choice.name.to_string())
}

fn prompt_scope(config: &Config) -> Result<Option<String>, String> {
    let help_msg = format!("e.g. {}", config.commit_scopes.join(", "));
    let prompt_text = crate::t!("commit.scope_prompt");
    let scope_input = Text::new(&prompt_text)
        .with_help_message(&help_msg)
        .prompt()
        .map_err(|e| format!("Prompt aborted: {}", e))?;

    let trimmed = scope_input.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn prompt_is_breaking() -> Result<bool, String> {
    let prompt_text = crate::t!("commit.breaking_prompt");
    Confirm::new(&prompt_text)
        .with_default(false)
        .prompt()
        .map_err(|e| format!("Prompt aborted: {}", e))
}

fn prompt_subject() -> Result<String, String> {
    let prompt_text = crate::t!("commit.summary_prompt");
    let empty_err = crate::t!("commit.empty_summary");
    Text::new(&prompt_text)
        .with_validator(move |input: &str| {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                Ok(Validation::Invalid(empty_err.clone().into()))
            } else if trimmed.ends_with('.') {
                Ok(Validation::Invalid(
                    "Subject must not end with a period/dot ('.')".into(),
                ))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()
        .map_err(|e| format!("Prompt aborted: {}", e))
        .map(|s| s.trim().to_string())
}

fn prompt_body() -> Result<Option<String>, String> {
    let prompt_text = crate::t!("commit.body_prompt");
    let body_input = Text::new(&prompt_text)
        .prompt()
        .map_err(|e| format!("Prompt aborted: {}", e))?;

    let trimmed = body_input.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn prompt_breaking_description() -> Result<Option<String>, String> {
    let prompt_text = crate::t!("commit.breaking_desc_prompt");
    let desc = Text::new(&prompt_text)
        .prompt()
        .map_err(|e| format!("Prompt aborted: {}", e))?;

    let trimmed = desc.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn prompt_issues() -> Result<Option<String>, String> {
    let prompt_text = crate::t!("commit.issues_prompt");
    let issues_input = Text::new(&prompt_text)
        .prompt()
        .map_err(|e| format!("Prompt aborted: {}", e))?;

    let trimmed = issues_input.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

/// Format the full commit message according to Conventional Commits specification
pub fn format_conventional_commit(
    commit_type: &str,
    scope: Option<&str>,
    is_breaking: bool,
    subject: &str,
    body: Option<&str>,
    breaking_desc: Option<&str>,
    issues: Option<&str>,
) -> String {
    let mut header = commit_type.to_string();

    if let Some(s) = scope {
        let clean_scope = s.trim();
        if !clean_scope.is_empty() {
            header.push('(');
            header.push_str(clean_scope);
            header.push(')');
        }
    }

    if is_breaking {
        header.push('!');
    }

    header.push_str(": ");
    header.push_str(subject.trim());

    let mut full_msg = header;

    if let Some(b) = body {
        let clean_body = b.trim();
        if !clean_body.is_empty() {
            full_msg.push_str("\n\n");
            full_msg.push_str(clean_body);
        }
    }

    if let Some(brk) = breaking_desc {
        let clean_brk = brk.trim();
        if !clean_brk.is_empty() {
            full_msg.push_str("\n\nBREAKING CHANGE: ");
            full_msg.push_str(clean_brk);
        }
    }

    if let Some(iss) = issues {
        let clean_iss = iss.trim();
        if !clean_iss.is_empty() {
            full_msg.push_str("\n\n");
            if clean_iss.to_ascii_lowercase().starts_with("close")
                || clean_iss.to_ascii_lowercase().starts_with("fix")
                || clean_iss.to_ascii_lowercase().starts_with("resolve")
            {
                full_msg.push_str(clean_iss);
            } else {
                full_msg.push_str(&format!("Closes {}", clean_iss));
            }
        }
    }

    full_msg
}

fn validate_subject_string(s: &str) -> Result<(), String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("Subject description cannot be empty".to_string());
    }
    if trimmed.ends_with('.') {
        return Err("Subject line must not end with a period/dot ('.')".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_conventional_commit_simple() {
        let msg = format_conventional_commit(
            "feat",
            Some("cli"),
            false,
            "add interactive wizard",
            None,
            None,
            None,
        );
        assert_eq!(msg, "feat(cli): add interactive wizard");
    }

    #[test]
    fn test_format_conventional_commit_breaking_and_body() {
        let msg = format_conventional_commit(
            "fix",
            Some("api"),
            true,
            "change authentication protocol",
            Some("Migrate to JWT tokens for enhanced security"),
            Some("Old sessions are invalidated immediately"),
            Some("#42"),
        );

        let expected = "fix(api)!: change authentication protocol\n\n\
                        Migrate to JWT tokens for enhanced security\n\n\
                        BREAKING CHANGE: Old sessions are invalidated immediately\n\n\
                        Closes #42";
        assert_eq!(msg, expected);
    }

    #[test]
    fn test_format_without_scope() {
        let msg =
            format_conventional_commit("chore", None, false, "bump dependencies", None, None, None);
        assert_eq!(msg, "chore: bump dependencies");
    }

    #[test]
    fn test_subject_validation() {
        assert!(validate_subject_string("valid subject").is_ok());
        assert!(validate_subject_string("invalid subject with dot.").is_err());
        assert!(validate_subject_string("   ").is_err());
    }
}
