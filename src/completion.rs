//! Dynamic autocompletion and shell installation module.
//! Supports extracting catalog repository candidates dynamically
//! and installing shell completion snippets idempotently.

use std::fs;
use std::path::{Path, PathBuf};

/// Marker comments for idempotent injection into shell RC files
pub const BEGIN_MARKER: &str = "# >>> CCCP SHELL COMPLETION >>>";
pub const END_MARKER: &str = "# <<< CCCP SHELL COMPLETION <<<";

/// Extract dynamic completion candidates from Gosplan catalog (slugs, names, aliases)
pub fn get_catalog_candidates(
    config: Option<&crate::config::Config>,
    start_dir: Option<&Path>,
) -> Vec<String> {
    let entries = crate::catalog::load_catalog(config, start_dir);
    let mut candidates = Vec::new();
    for entry in entries {
        if !entry.slug.is_empty() && !candidates.contains(&entry.slug) {
            candidates.push(entry.slug.clone());
        }
        if !entry.name.is_empty() && !candidates.contains(&entry.name) {
            candidates.push(entry.name.clone());
        }
        for alias in entry.aliases {
            let clean = alias.trim().to_string();
            if !clean.is_empty() && !candidates.contains(&clean) {
                candidates.push(clean);
            }
        }
    }
    candidates
}

/// Detects the target RC file for a given shell if no explicit path is provided
pub fn detect_rc_path(shell: clap_complete::Shell) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let home_path = PathBuf::from(home);
    match shell {
        clap_complete::Shell::Bash => Some(home_path.join(".bashrc")),
        clap_complete::Shell::Zsh => Some(home_path.join(".zshrc")),
        clap_complete::Shell::Fish => {
            Some(home_path.join(".config").join("fish").join("config.fish"))
        }
        _ => None,
    }
}

/// Generates the shell configuration snippet for completion
pub fn generate_completion_snippet(shell: clap_complete::Shell) -> String {
    let shell_name = match shell {
        clap_complete::Shell::Bash => "bash",
        clap_complete::Shell::Zsh => "zsh",
        clap_complete::Shell::Fish => "fish",
        clap_complete::Shell::PowerShell => "powershell",
        clap_complete::Shell::Elvish => "elvish",
        _ => "bash",
    };

    format!(
        "{}\nsource <(cccp completion {})\n{}",
        BEGIN_MARKER, shell_name, END_MARKER
    )
}

/// Idempotently installs or updates the CCCP completion block in a shell RC file
pub fn install_completion_into_file(
    rc_path: &Path,
    shell: clap_complete::Shell,
    dry_run: bool,
) -> Result<String, String> {
    let snippet = generate_completion_snippet(shell);

    let existing = if rc_path.exists() {
        fs::read_to_string(rc_path)
            .map_err(|e| format!("Falha ao ler arquivo {}: {}", rc_path.display(), e))?
    } else {
        String::new()
    };

    let new_content = if existing.contains(BEGIN_MARKER) && existing.contains(END_MARKER) {
        // Replace existing block
        let start = existing.find(BEGIN_MARKER).unwrap();
        let end_idx = existing.find(END_MARKER).unwrap() + END_MARKER.len();
        let prefix = existing[..start].trim_end();
        let suffix = existing[end_idx..].trim_start();
        if prefix.is_empty() {
            if suffix.is_empty() {
                format!("{}\n", snippet)
            } else {
                format!("{}\n\n{}", snippet, suffix)
            }
        } else if suffix.is_empty() {
            format!("{}\n\n{}\n", prefix, snippet)
        } else {
            format!("{}\n\n{}\n\n{}", prefix, snippet, suffix)
        }
    } else {
        // Append to end of file
        let mut content = existing;
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&snippet);
        content.push('\n');
        content
    };

    if !dry_run {
        if let Some(parent) = rc_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(rc_path, &new_content)
            .map_err(|e| format!("Falha ao escrever em {}: {}", rc_path.display(), e))?;
    }

    Ok(new_content)
}
