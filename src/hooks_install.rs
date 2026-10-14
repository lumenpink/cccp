//! Git hooks installer and minimal stub generator for CCCP.
//! Installs POSIX-compliant `/bin/sh` hook stubs with PATH discovery,
//! installation directory fallbacks, automatic backup, and clean uninstallation.

use crate::cli::InstallArgs;
use crate::cultural::Theme;
use crate::git::GitRepo;
use std::fs;
use std::path::{Path, PathBuf};

pub const HOOK_NAMES: [&str; 6] = [
    "commit-msg",
    "pre-commit",
    "pre-push",
    "post-commit",
    "reference-transaction",
    "post-merge",
];

/// Determines the hooks directory for the repository
pub fn resolve_hooks_dir(repo: &GitRepo, cli_hook_dir: Option<&Path>) -> PathBuf {
    if let Some(d) = cli_hook_dir {
        return d.to_path_buf();
    }

    // Check if core.hooksPath is configured in git
    if let Ok(output) = std::process::Command::new("git")
        .args(["config", "core.hooksPath"])
        .current_dir(&repo.root_dir)
        .output()
    {
        if output.status.success() {
            let custom_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !custom_path.is_empty() {
                let p = PathBuf::from(custom_path);
                if p.is_absolute() {
                    return p;
                } else {
                    return repo.root_dir.join(p);
                }
            }
        }
    }

    repo.root_dir.join(".git").join("hooks")
}

pub const CURRENT_HOOK_TEMPLATE_VERSION: u32 = 1;

/// Computes a deterministic 16-hex hash (fnv-1a 64-bit) of hook content
pub fn compute_hook_checksum(content: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in content.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

/// Checks if an existing file is a CCCP-managed hook
pub fn is_cccp_hook(path: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        content.contains("cccp-hook-version:") || content.contains("cccp hook")
    } else {
        false
    }
}

/// Generate canonical POSIX shell stub for a hook
pub fn generate_hook_stub(hook_name: &str, repo_root: &Path, version: &str) -> String {
    let root_str = repo_root.display();
    let body = format!(
        r#"# 1. Prioritize cccp in PATH
if command -v cccp >/dev/null 2>&1; then
    exec cccp hook {hook_name} "$@"
fi

# 2. Search common user and system installation directories
for candidate in \
    "$HOME/.local/bin/cccp" \
    "$HOME/.cargo/bin/cccp" \
    /usr/local/bin/cccp \
    /usr/bin/cccp \
    /opt/cccp/bin/cccp \
    "{root_str}/target/release/cccp" \
    "{root_str}/cccp"
do
    if [ -x "$candidate" ]; then
        exec "$candidate" hook {hook_name} "$@"
    fi
done

echo "★ GOSPLAN ALERT: COMRADE! ★" >&2
echo "CRITICAL ERROR: 'cccp' binary was not found in \$PATH or standard installation directories (~/.local/bin, ~/.cargo/bin, /usr/local/bin, /usr/bin)." >&2
echo "Sabotage of the Five-Year Plan is inadmissible! Install cccp or configure your PATH." >&2
echo "«НИ ШАГУ НАЗАД!» — Order No. 227" >&2
exit 1
"#
    );
    let checksum = compute_hook_checksum(&body);
    let template_ver = CURRENT_HOOK_TEMPLATE_VERSION;

    format!(
        "#!/bin/sh\n# cccp-hook-version: {version}\n# cccp-hook-template-version: {template_ver}\n# cccp-hook-checksum: {checksum}\n# CCCP - Conventional Commits Compliance Program (Soviet Git Hook)\n\n{body}"
    )
}

#[cfg(unix)]
pub fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(path)?;
    let mut perms = metadata.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
pub fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Install or update CCCP hooks in the repository
pub fn install_hooks(
    hooks_dir: &Path,
    repo_root: &Path,
    force: bool,
    theme: &Theme,
) -> Result<(), String> {
    fs::create_dir_all(hooks_dir).map_err(|e| {
        format!(
            "Failed to create hooks directory '{}': {}",
            hooks_dir.display(),
            e
        )
    })?;

    let version = env!("CARGO_PKG_VERSION");
    println!(
        "{} Installing CCCP Soviet Git Hooks in '{}' (v{})...",
        theme.red_bold("★"),
        hooks_dir.display(),
        version
    );

    for hook_name in &HOOK_NAMES {
        let hook_path = hooks_dir.join(hook_name);
        let backup_path = hooks_dir.join(format!("{}.bak", hook_name));

        if hook_path.exists() {
            if is_cccp_hook(&hook_path) {
                // Already a CCCP hook, silently overwrite/upgrade
                if force {
                    println!(
                        "  {} Updating CCCP hook '{}' (forced)",
                        theme.cyan("•"),
                        hook_name
                    );
                } else {
                    println!("  {} Updating CCCP hook '{}'", theme.cyan("•"), hook_name);
                }
            } else {
                // Non-CCCP custom/foreign hook
                println!(
                    "  {} Non-CCCP hook detected at '{}'. Backing up to '{}.bak'",
                    theme.yellow("!"),
                    hook_name,
                    hook_name
                );
                fs::rename(&hook_path, &backup_path).map_err(|e| {
                    format!(
                        "Failed to backup existing hook '{}' to '{}.bak': {}",
                        hook_path.display(),
                        hook_name,
                        e
                    )
                })?;
            }
        } else {
            println!("  {} Creating hook '{}'", theme.green("+"), hook_name);
        }

        let stub_content = generate_hook_stub(hook_name, repo_root, version);
        fs::write(&hook_path, stub_content).map_err(|e| {
            format!(
                "Failed to write hook stub to '{}': {}",
                hook_path.display(),
                e
            )
        })?;

        make_executable(&hook_path).map_err(|e| {
            format!(
                "Failed to set executable permissions on '{}': {}",
                hook_path.display(),
                e
            )
        })?;
    }

    println!(
        "{} All 4 Gosplan security hooks successfully installed and synchronized.",
        theme.green("✓")
    );
    Ok(())
}

/// Uninstall CCCP hooks and restore backups if available
pub fn uninstall_hooks(hooks_dir: &Path, theme: &Theme) -> Result<(), String> {
    if !hooks_dir.exists() {
        println!(
            "{} Hooks directory '{}' does not exist. Nothing to uninstall.",
            theme.yellow("Notice:"),
            hooks_dir.display()
        );
        return Ok(());
    }

    println!(
        "{} Uninstalling CCCP hooks from '{}'...",
        theme.red_bold("★"),
        hooks_dir.display()
    );

    for hook_name in &HOOK_NAMES {
        let hook_path = hooks_dir.join(hook_name);
        let backup_path = hooks_dir.join(format!("{}.bak", hook_name));

        if hook_path.exists() {
            if is_cccp_hook(&hook_path) {
                fs::remove_file(&hook_path).map_err(|e| {
                    format!("Failed to remove hook '{}': {}", hook_path.display(), e)
                })?;
                println!("  {} Removed CCCP hook '{}'", theme.red("✗"), hook_name);

                // Restore backup if it exists
                if backup_path.exists() {
                    fs::rename(&backup_path, &hook_path).map_err(|e| {
                        format!(
                            "Failed to restore backup '{}.bak' to '{}': {}",
                            hook_name,
                            hook_path.display(),
                            e
                        )
                    })?;
                    let _ = make_executable(&hook_path);
                    println!(
                        "  {} Restored pre-CCCP backup '{}.bak' -> '{}'",
                        theme.green("✓"),
                        hook_name,
                        hook_name
                    );
                }
            } else {
                println!(
                    "  {} Preserving non-CCCP hook '{}'",
                    theme.dim("•"),
                    hook_name
                );
            }
        } else if backup_path.exists() {
            // Hook was missing but backup was present
            fs::rename(&backup_path, &hook_path).map_err(|e| {
                format!(
                    "Failed to restore backup '{}.bak' to '{}': {}",
                    hook_name,
                    hook_path.display(),
                    e
                )
            })?;
            let _ = make_executable(&hook_path);
            println!(
                "  {} Restored backup '{}.bak' -> '{}'",
                theme.green("✓"),
                hook_name,
                hook_name
            );
        }
    }

    println!("{} CCCP hooks uninstalled successfully.", theme.green("✓"));
    Ok(())
}

/// Main entry point for the `cccp install` command
pub fn handle_install(args: &InstallArgs, repo: &GitRepo, theme: &Theme) -> Result<(), String> {
    let hooks_dir = resolve_hooks_dir(repo, args.hook_dir.as_deref());

    if args.uninstall {
        uninstall_hooks(&hooks_dir, theme)
    } else {
        install_hooks(&hooks_dir, &repo.root_dir, args.force, theme)
    }
}
