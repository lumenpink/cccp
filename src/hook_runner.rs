//! Fast Git hook runner for commit-msg, pre-push, post-commit, and reference-transaction.
//! Executes in < 3ms with Soviet Gosplan Quality Control diagnostics.

use crate::config::Config;
use crate::cultural::Theme;
use crate::git::GitRepo;
use crate::lint::lint_commit_file;
use std::io::{self, BufRead, IsTerminal};
use std::path::Path;
use std::process::Command;

/// Format blocked message when direct git commit is executed
pub fn pre_commit_blocked_message(theme: &Theme) -> String {
    format!(
        "\n{}\n{}\nToda evolução de código deve ser coordenada pelo Maestro CCCP:\n\n  • Fases 3 a 8 : {}\n  • Fase 9      : {}\n\n",
        theme.red_bold("★ [CCCP MAESTRO] Acesso Direto Bloqueado ★"),
        theme.yellow("Execuções manuais diretas de 'git commit' são proibidas pela norma GOST v2.0 (Kollontai 2.0)."),
        theme.cyan("cccp commit \"<resumo>\""),
        theme.cyan("cccp close \"<resumo>\" -b \"<laudo>\"")
    )
}

/// Format blocked message when direct git push is executed
pub fn pre_push_blocked_message(theme: &Theme) -> String {
    format!(
        "\n{}\n{}\nUtilize o comando do Maestro CCCP:\n\n  • Envio Seguro : {}\n\n",
        theme.red_bold("★ [CCCP MAESTRO] Envio Direto Bloqueado ★"),
        theme.yellow("Execuções manuais diretas de 'git push' são proibidas pela norma GOST v2.0 (Kollontai 2.0)."),
        theme.cyan("cccp push")
    )
}

/// Format guidance message when commit-msg rejects Conventional Commit format
pub fn commit_msg_rejected_guidance(theme: &Theme) -> String {
    format!(
        "\n{}\n  {}\n  {}\n  {}\n\n",
        theme.bold("Gosplan Directive:"),
        theme.cyan("Format required: <type>(<scope>): <subject>"),
        theme.dim("Example: feat(core): implement Stakhanovite efficiency algorithm"),
        theme.dim("Valid types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert")
    )
}

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

        eprint!("{}", commit_msg_rejected_guidance(theme));

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
    let is_authorized = std::env::var("CCCP_MAESTRO_ACTIVE").as_deref() == Ok("1")
        || std::env::var("CCCP_BYPASS_LOCKS").as_deref() == Ok("1")
        || std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("FORGEJO_ACTIONS").is_ok();

    if !is_authorized {
        eprint!("{}", pre_push_blocked_message(theme));
        return Ok(1);
    }

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

    // The pre-push hook is a lightweight, instantaneous semantic verification gate.
    // Heavy containerized CI (act) is strictly decoupled from regular git push and
    // executed exclusively as a Pre-Tag Gate during `cccp tag`.
    // Act is only run during pre-push if explicitly forced via CCCP_ACT_FORCE=1.
    if std::env::var("CCCP_ACT_FORCE").as_deref() == Ok("1") {
        let branch = Command::new("git")
            .args(["symbolic-ref", "--short", "HEAD"])
            .current_dir(&repo.root_dir)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "HEAD".to_string());

        return match crate::act::act_ci_gate(
            &repo.root_dir,
            remote.unwrap_or("origin"),
            &branch,
            theme,
            verbose,
        ) {
            Ok(true) => {
                if verbose > 0 {
                    println!("{} pre-push CI gate passed successfully.", theme.green("✓"));
                }
                Ok(0)
            }
            Ok(false) => Ok(1),
            Err(e) => {
                eprintln!("{}: {}", theme.red_bold("Error"), e);
                Ok(1)
            }
        };
    }

    if verbose > 0 {
        println!(
            "{} pre-push semantic checks passed successfully.",
            theme.green("✓")
        );
    }
    Ok(0)
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
                let id_raw = after[..end_pos].trim();
                let token = id_raw
                    .split(|c: char| c.is_whitespace() || c == '-')
                    .next()
                    .unwrap_or(id_raw);
                if let Ok(id) = token.parse::<u64>() {
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
pub fn run_pre_commit(repo: &GitRepo, theme: &Theme, verbose: u8) -> Result<i32, String> {
    let is_authorized = std::env::var("CCCP_MAESTRO_ACTIVE").as_deref() == Ok("1")
        || std::env::var("CCCP_BYPASS_LOCKS").as_deref() == Ok("1")
        || std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("FORGEJO_ACTIONS").is_ok();

    if !is_authorized {
        eprint!("{}", pre_commit_blocked_message(theme));
        return Ok(1);
    }

    if std::env::var("HOOK_ACTIVE").as_deref() == Ok("1") {
        return Ok(0);
    }

    let version_file = repo.root_dir.join("VERSION");
    if !version_file.is_file() {
        return Ok(0);
    }

    let config = Config::load_hierarchical(Some(&repo.root_dir));
    let no_v = config.no_v;
    let default_base = &config.default_base_version;

    let predicted = crate::semver_calc::calculate_version(repo, default_base, no_v, None)?;

    // For the upcoming commit in pre-commit, the commit count is commits_since_tag + 1
    let next_commits_count = repo.commits_since_tag.len() + 1;
    let pre_str = format!("dev.{}", next_commits_count);
    let build_str = format!("{}.{}", repo.head_date, repo.head_short_id);

    let mut dev_ver = match semver::Version::parse(&predicted.target_version) {
        Ok(v) => v,
        Err(e) => {
            if verbose > 0 {
                eprintln!(
                    "{}: Failed to parse target version: {}",
                    theme.yellow("Warning"),
                    e
                );
            }
            return Ok(0);
        }
    };

    dev_ver.pre = match semver::Prerelease::new(&pre_str) {
        Ok(p) => p,
        Err(_) => semver::Prerelease::EMPTY,
    };
    dev_ver.build = match semver::BuildMetadata::new(&build_str) {
        Ok(b) => b,
        Err(_) => semver::BuildMetadata::EMPTY,
    };

    let new_version = dev_ver.to_string();

    let current_content = std::fs::read_to_string(&version_file)
        .unwrap_or_default()
        .trim()
        .to_string();

    if current_content != new_version {
        if let Err(e) = std::fs::write(&version_file, format!("{}\n", new_version)) {
            eprintln!(
                "{}: Failed to write VERSION file: {}",
                theme.yellow("Warning"),
                e
            );
            return Ok(0);
        }

        let status = Command::new("git")
            .args(["add", "VERSION"])
            .current_dir(&repo.root_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                if verbose > 0 {
                    println!(
                        "{} Pre-commit: VERSION synchronized to '{}'.",
                        theme.green("✓"),
                        new_version
                    );
                }
            }
            _ => {
                if verbose > 0 {
                    eprintln!(
                        "{}: Failed to stage VERSION file during pre-commit.",
                        theme.yellow("Warning")
                    );
                }
            }
        }
    }

    Ok(0)
}

/// Reconcile VERSION file after a merge if the calculated version diverges from disk
pub fn reconcile_merge_version(repo: &GitRepo, theme: &Theme) -> Result<Option<String>, String> {
    let version_file = repo.root_dir.join("VERSION");
    if !version_file.is_file() {
        return Ok(None);
    }

    // Open repository freshly at current state
    let fresh_repo = GitRepo::open(Some(&repo.root_dir))?;

    // Only true merge commits (>= 2 parents from divergent branches) need VERSION reconciliation.
    // Linear fast-forward merges do not diverge and should not create spurious chore commits.
    if fresh_repo.head_parent_count <= 1 {
        return Ok(None);
    }

    // If the latest commit is already a reconciliation commit, do not chain another
    if fresh_repo
        .commits_since_tag
        .first()
        .map(|c| c.summary.starts_with("chore(version):"))
        .unwrap_or(false)
    {
        return Ok(None);
    }

    let config = Config::load_hierarchical(Some(&repo.root_dir));
    let no_v = config.no_v;
    let default_base = &config.default_base_version;

    let predicted = crate::semver_calc::calculate_version(&fresh_repo, default_base, no_v, None)?;

    let current_content = std::fs::read_to_string(&version_file)
        .unwrap_or_default()
        .trim()
        .to_string();

    // In sequential fast-forward merges, VERSION already matches the calculated state of HEAD
    if predicted.is_release && current_content == predicted.target_version {
        return Ok(None);
    }
    if !predicted.is_release && current_content == predicted.dev_version {
        return Ok(None);
    }

    // Version diverged! Calculate target version for the reconciliation commit
    let target_version = if predicted.is_release {
        predicted.target_version.clone()
    } else {
        let next_count = fresh_repo.commits_since_tag.len() + 1;
        let pre_str = format!("dev.{}", next_count);
        let build_str = format!("{}.{}", fresh_repo.head_date, fresh_repo.head_short_id);

        let mut dev_ver = match semver::Version::parse(&predicted.target_version) {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };
        dev_ver.pre = semver::Prerelease::new(&pre_str).unwrap_or(semver::Prerelease::EMPTY);
        dev_ver.build =
            semver::BuildMetadata::new(&build_str).unwrap_or(semver::BuildMetadata::EMPTY);
        dev_ver.to_string()
    };

    if current_content == target_version {
        return Ok(None);
    }

    // Write reconciled version
    std::fs::write(&version_file, format!("{}\n", target_version))
        .map_err(|e| format!("Failed to write VERSION file: {}", e))?;

    let add_status = Command::new("git")
        .args(["add", "VERSION"])
        .current_dir(&repo.root_dir)
        .status()
        .map_err(|e| format!("Failed to stage VERSION: {}", e))?;

    if !add_status.success() {
        return Err("git add VERSION failed".to_string());
    }

    let commit_msg = format!(
        "chore(version): sincronizar VERSION para {}",
        target_version
    );
    let mut commit_cmd = Command::new("git");
    commit_cmd
        .args(["commit", "-m", &commit_msg])
        .env("CCCP_MAESTRO_ACTIVE", "1")
        .current_dir(&repo.root_dir);
    let commit_status = commit_cmd
        .status()
        .map_err(|e| format!("Failed to commit reconciled VERSION: {}", e))?;

    if !commit_status.success() {
        return Err("git commit for VERSION reconciliation failed".to_string());
    }

    println!(
        "{} Reconciliado VERSION pós-merge para '{}'.",
        theme.cyan("★ [GOSPLAN]"),
        target_version
    );

    Ok(Some(target_version))
}
