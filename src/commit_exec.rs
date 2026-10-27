//! Commit dispatcher via Git CLI with support for GPG/SSH signing and interactive pinentry.

use crate::cultural::Theme;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Default)]
pub struct CommitOptions {
    pub all: bool,
    pub sign: bool,
    pub no_sign: bool,
    pub dry_run: bool,
    pub dir: Option<PathBuf>,
}

/// Checks if there are any staged changes in the Git index
pub fn has_staged_changes(dir: Option<&Path>) -> Result<bool, String> {
    let mut cmd = Command::new("git");
    cmd.args(["diff", "--cached", "--quiet"]);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    let status = cmd
        .status()
        .map_err(|e| format!("Failed to execute 'git diff --cached': {}", e))?;

    // git diff --cached --quiet exits with 1 if there are differences (staged changes),
    // and 0 if there are no differences (empty stage).
    Ok(status.code() == Some(1))
}

/// Dispatches the commit execution to the `git commit` CLI command.
///
/// Uses a secure temporary file (`git commit -F <tempfile>`) to prevent shell escape/injection
/// and preserves the TTY for interactive signing agents (e.g. GPG pinentry, SSH-agent).
pub fn execute_commit(message: &str, opts: &CommitOptions, theme: &Theme) -> Result<i32, String> {
    if opts.dry_run {
        println!(
            "{} Dry run enabled: commit was not executed.",
            theme.gold_bold("★")
        );
        return Ok(0);
    }

    // Verify staged changes unless -a/--all is enabled
    if !opts.all {
        let has_staged = has_staged_changes(opts.dir.as_deref())?;
        if !has_staged {
            eprintln!("{}: No changes staged for commit.", theme.yellow("Warning"));
            eprintln!(
                "Stage your changes first with '{}' or pass '{}' to commit all tracked modifications.",
                theme.cyan("git add <files>"),
                theme.cyan("-a / --all")
            );
            return Ok(1);
        }
    }

    // Create a temporary file with the commit message
    let mut temp = NamedTempFile::new()
        .map_err(|e| format!("Failed to create temporary commit file: {}", e))?;
    temp.write_all(message.as_bytes())
        .map_err(|e| format!("Failed to write commit message to temporary file: {}", e))?;
    if !message.ends_with('\n') {
        temp.write_all(b"\n")
            .map_err(|e| format!("Failed to append newline to commit message: {}", e))?;
    }
    temp.flush()
        .map_err(|e| format!("Failed to flush temporary commit file: {}", e))?;

    let temp_path = temp.path().to_path_buf();

    let mut cmd = Command::new("git");
    cmd.arg("commit");
    cmd.env("CCCP_MAESTRO_ACTIVE", "1");
    cmd.arg("-F");
    cmd.arg(&temp_path);

    if opts.all {
        cmd.arg("-a");
    }

    if opts.sign {
        cmd.arg("-S");
    } else if opts.no_sign {
        cmd.arg("--no-gpg-sign");
    }

    if let Some(ref d) = opts.dir {
        cmd.current_dir(d);
    }

    // Inherit TTY stdio so GPG/SSH pinentry and git hooks work interactively
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to execute 'git commit': {}", e))?;

    let exit_code = status.code().unwrap_or(1);
    if exit_code == 0 {
        println!("{} State decree recorded successfully.", theme.green("✓"));
    }

    Ok(exit_code)
}
