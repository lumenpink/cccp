//! Fast Git hook runner for commit-msg, pre-push, post-commit, and reference-transaction.
//! Executes in < 3ms with Soviet Gosplan Quality Control diagnostics.

use crate::config::Config;
use crate::cultural::Theme;
use crate::git::GitRepo;
use crate::lint::lint_commit_file;
use std::io::{self, BufRead, IsTerminal};
use std::path::Path;
use std::process::Command;

/// Handler for the `commit-msg` git hook
pub fn run_commit_msg(
    file: &Path,
    config: &Config,
    theme: &Theme,
    verbose: u8,
) -> Result<i32, String> {
    if !file.exists() {
        return Err(format!(
            "Commit message file '{}' does not exist",
            file.display()
        ));
    }

    // Run strict linting
    let result = lint_commit_file(file, config, true)?;

    if result.is_valid {
        if verbose > 0 {
            println!(
                "{} Commit message compliant with Conventional Commits 1.0.0",
                theme.green("✓")
            );
        }
        for warning in result.warnings {
            eprintln!("  {} {}", theme.yellow("Warning:"), warning);
        }
        Ok(0)
    } else {
        eprintln!(
            "\n{}\n{}",
            theme.red_bold("★ GOSPLAN QUALITY CONTROL: COMMIT MESSAGE REJECTED ★"),
            theme.gold_bold("«НИ ШАГУ НАЗАД!» — Order No. 227 (Not one step back from standards)")
        );
        eprintln!(
            "The commit message in '{}' violates Conventional Commits 1.0.0:\n",
            file.display()
        );

        for error in result.errors {
            eprintln!("  {} {}", theme.red("✗"), error);
        }
        for warning in result.warnings {
            eprintln!("  {} {}", theme.yellow("Warning:"), warning);
        }

        eprintln!(
            "\n{}\n  {}\n  {}",
            theme.bold("Gosplan Directive:"),
            theme.cyan("Format required: <type>(<scope>): <subject>"),
            theme.dim("Example: feat(core): implement Stakhanovite efficiency algorithm")
        );
        eprintln!(
            "  {}\n",
            theme.dim("Valid types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert")
        );

        Ok(1)
    }
}

/// Handler for the `pre-push` git hook
pub fn run_pre_push(
    remote: Option<&str>,
    _url: Option<&str>,
    stdin_override: Option<&str>,
    repo: &GitRepo,
    _config: &Config,
    theme: &Theme,
    verbose: u8,
) -> Result<i32, String> {
    let lines: Vec<String> = if let Some(content) = stdin_override {
        content.lines().map(|s| s.to_string()).collect()
    } else if !io::stdin().is_terminal() {
        let stdin = io::stdin();
        stdin.lock().lines().map_while(Result::ok).collect()
    } else {
        Vec::new()
    };

    if lines.is_empty() {
        if verbose > 0 {
            println!(
                "{} pre-push: No ref updates received on stdin.",
                theme.dim("•")
            );
        }
        return Ok(0);
    }

    let mut has_error = false;

    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let local_ref = parts[0];
        let local_sha = parts[1];
        let remote_ref = parts[2];

        // Ignore ref deletions (e.g. git push origin --delete branch)
        if local_sha == "0000000000000000000000000000000000000000" || local_ref == "(delete)" {
            continue;
        }

        // Check if pushing a release tag (refs/tags/*)
        if let Some(tag_name) = remote_ref.strip_prefix("refs/tags/") {
            let clean_tag = tag_name.strip_prefix('v').unwrap_or(tag_name);

            // Only validate SemVer release tags (starting with digit)
            if clean_tag.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                // Inspect VERSION file inside local_sha
                let ver_output = Command::new("git")
                    .args(["show", &format!("{}:VERSION", local_sha)])
                    .current_dir(&repo.root_dir)
                    .output();

                let ver_in_commit = match ver_output {
                    Ok(out) if out.status.success() => {
                        String::from_utf8_lossy(&out.stdout).trim().to_string()
                    }
                    _ => String::new(),
                };

                if ver_in_commit.is_empty() {
                    eprintln!(
                        "\n{}\n{}\nTag '{}' points to a commit without a VERSION file.\nTarget commit: {}\n\n{}\nRun 'cccp tag' to generate the release commit and synchronized CHANGELOG.\n",
                        theme.red_bold("★ CCCP GOSPLAN QUALITY CONTROL: PUSH REJECTED ★"),
                        theme.yellow("Error: Missing VERSION File"),
                        tag_name,
                        local_sha,
                        theme.cyan("Gosplan Directive: Releases must contain an official VERSION file.")
                    );
                    has_error = true;
                } else if ver_in_commit.contains("-dev") || ver_in_commit.contains('+') {
                    eprintln!(
                        "\n{}\n{}\nTag '{}' points to a development version: '{}'.\nTarget commit: {}\n\n{}\nRun 'cccp tag' to generate clean release commits and synchronize CHANGELOG.\n",
                        theme.red_bold("★ CCCP GOSPLAN QUALITY CONTROL: PUSH REJECTED ★"),
                        theme.yellow("Error: Development Version in Release Tag"),
                        tag_name,
                        ver_in_commit,
                        local_sha,
                        theme.cyan("Gosplan Directive: Pre-release/development versions cannot be pushed as release tags.")
                    );
                    has_error = true;
                } else if ver_in_commit != clean_tag && ver_in_commit != tag_name {
                    eprintln!(
                        "\n{}\n{}\nTag '{}' does not match VERSION file in target commit ('{}').\nTarget commit: {}\n\n{}\nRun 'cccp tag' to generate clean release commits and synchronize CHANGELOG.\n",
                        theme.red_bold("★ CCCP GOSPLAN QUALITY CONTROL: PUSH REJECTED ★"),
                        theme.yellow("Error: Tag and VERSION Mismatch"),
                        tag_name,
                        ver_in_commit,
                        local_sha,
                        theme.cyan("Gosplan Directive: Tag name and VERSION file must be strictly synchronized.")
                    );
                    has_error = true;
                } else if verbose > 0 {
                    println!(
                        "{} Tag '{}' verified against VERSION '{}'.",
                        theme.green("✓"),
                        tag_name,
                        ver_in_commit
                    );
                }
            }
        }
    }

    if has_error {
        return Ok(1);
    }

    // Run Act local CI gate if applicable
    let branch = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(&repo.root_dir)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "HEAD".to_string());

    match crate::act::act_ci_gate(
        &repo.root_dir,
        remote.unwrap_or("origin"),
        &branch,
        theme,
        verbose,
    ) {
        Ok(true) => {
            if verbose > 0 {
                println!("{} pre-push checks passed successfully.", theme.green("✓"));
            }
            Ok(0)
        }
        Ok(false) => Ok(1),
        Err(e) => {
            eprintln!("{}: {}", theme.red_bold("Error"), e);
            Ok(1)
        }
    }
}

/// Handler for the `post-commit` git hook
pub fn run_post_commit(theme: &Theme, verbose: u8) -> Result<i32, String> {
    if std::env::var("HOOK_ACTIVE").as_deref() == Ok("1") {
        return Ok(0);
    }

    if verbose > 0 {
        println!("{} CCCP post-commit hook completed.", theme.green("✓"));
    }
    Ok(0)
}

/// Handler for the `reference-transaction` git hook
pub fn run_reference_transaction(
    state: Option<&str>,
    theme: &Theme,
    verbose: u8,
) -> Result<i32, String> {
    if verbose > 0 {
        println!(
            "{} reference-transaction state: {}",
            theme.dim("•"),
            state.unwrap_or("unknown")
        );
    }
    Ok(0)
}

/// Extract ticket IDs from commit messages containing the canonical `(closes #<id>)` tag
pub fn extract_closes_ids(text: &str) -> Vec<u64> {
    let mut ids = Vec::new();
    // Match canonical pattern: (closes #<id>)
    for line in text.lines() {
        let mut remainder = line;
        while let Some(start_pos) = remainder.find("(closes #") {
            let after = &remainder[start_pos + "(closes #".len()..];
            if let Some(end_pos) = after.find(')') {
                let id_str = &after[..end_pos].trim();
                if let Ok(id) = id_str.parse::<u64>() {
                    if !ids.contains(&id) {
                        ids.push(id);
                    }
                }
                remainder = &after[end_pos + 1..];
            } else {
                break;
            }
        }
    }
    ids
}

/// Handler for the `post-merge` git hook
pub fn run_post_merge(
    is_squash: Option<&str>,
    repo: &GitRepo,
    theme: &Theme,
    verbose: u8,
) -> Result<i32, String> {
    if std::env::var("HOOK_ACTIVE").as_deref() == Ok("1") {
        return Ok(0);
    }

    if verbose > 0 {
        println!(
            "{} CCCP post-merge hook triggered (is_squash: {})...",
            theme.green("✓"),
            is_squash.unwrap_or("0")
        );
    }

    // Inspect commits integrated by this merge (ORIG_HEAD..HEAD)
    let output = Command::new("git")
        .args([
            "log",
            "ORIG_HEAD..HEAD",
            "--pretty=format:%B---COMMIT-END---",
        ])
        .current_dir(&repo.root_dir)
        .output();

    let commit_logs = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => {
            // Fallback: check HEAD commit if ORIG_HEAD was not set (e.g. fresh merge)
            Command::new("git")
                .args(["log", "-1", "--pretty=format:%B"])
                .current_dir(&repo.root_dir)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default()
        }
    };

    let ticket_ids = extract_closes_ids(&commit_logs);
    if ticket_ids.is_empty() {
        if verbose > 0 {
            println!("  No canonical '(closes #<id>)' tags found in merged commits.");
        }
        return Ok(0);
    }

    println!(
        "{} Detected {} ticket(s) to close in merged commits: {:?}",
        theme.cyan("★ [GOSPLAN]"),
        ticket_ids.len(),
        ticket_ids
    );

    let comment = "Fechado automaticamente pelo merge no CCCP / GOST DD-TDD Fase 9.";
    if let Err(e) = crate::tickets::close_tickets(repo, &ticket_ids, Some(comment), theme) {
        eprintln!(
            "  {} Notice: Could not close tickets automatically: {}",
            theme.yellow("!"),
            e
        );
    }

    Ok(0)
}

/// Handler for the `pre-commit` git hook
pub fn run_pre_commit(
    _repo: &GitRepo,
    _theme: &Theme,
    _verbose: u8,
) -> Result<i32, String> {
    Ok(0)
}

/// Reconcile VERSION file after a merge if the calculated version diverges from disk
pub fn reconcile_merge_version(
    _repo: &GitRepo,
    _theme: &Theme,
) -> Result<Option<String>, String> {
    Ok(None)
}

