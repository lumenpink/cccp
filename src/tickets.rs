//! Multi-forge issue synchronization engine for `.antigravity/tickets.md`.
//! Compiles upstream tickets into clean Markdown with strict 10 KB safety ceiling.

use crate::cli::TicketsArgs;
use crate::config::Config;
use crate::cultural::Theme;
use crate::forges::{
    detect_forge_from_repo, parse_remote_url, ForgeClient, ForgeIssue, RepoForgeInfo,
};
use crate::git::GitRepo;
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_MAX_BYTES: usize = 10240; // 10 KB safety ceiling

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncResult {
    pub count: usize,
    pub bytes_written: usize,
    pub max_bytes: usize,
    pub stopped_early: bool,
    pub output_path: PathBuf,
}

/// Formats a single issue into the standard Antigravity Markdown ticket format
pub fn format_ticket_markdown(issue: &ForgeIssue) -> String {
    let labels_str = if issue.labels.is_empty() {
        "none".to_string()
    } else {
        issue.labels.join(", ")
    };

    let desc = if issue.body.trim().is_empty() {
        "_No description provided._"
    } else {
        issue.body.trim()
    };

    format!(
        "## [#{}] {}\n- **Status**: `{}`\n- **Labels**: {}\n- **Link**: {}\n\n### Description\n{}\n\n---\n",
        issue.number, issue.title, issue.state, labels_str, issue.html_url, desc
    )
}

/// Generates the complete Markdown document from a list of issues while strictly respecting max_bytes
pub fn generate_tickets_markdown(
    repo_name: &str,
    issues: &[ForgeIssue],
    max_bytes: usize,
) -> (String, usize, bool) {
    let timestamp = format_utc_now();
    let header = format!(
        "# Active Tickets - {}\n> Synced at: {}\n> Ceiling: {} bytes (strictly uncut tickets)\n\n",
        repo_name, timestamp, max_bytes
    );

    let mut content = header;
    let mut current_size = content.len();
    let mut count = 0;
    let mut stopped_early = false;

    for issue in issues {
        let block = format_ticket_markdown(issue);
        let block_size = block.len();

        if current_size + block_size > max_bytes {
            stopped_early = true;
            break;
        }

        content.push_str(&block);
        current_size += block_size;
        count += 1;
    }

    (content, count, stopped_early)
}

/// Computes the formatted UTC timestamp in YYYY-MM-DD HH:MM:SS UTC without external dependencies
pub fn format_utc_now() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, h, m, s
    )
}

pub fn days_to_ymd(days_since_epoch: u64) -> (u32, u32, u32) {
    let z = days_since_epoch + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m as u32, d as u32)
}

/// Resolves target output path for tickets.md, ensuring external repositories are not written into cwd
pub fn resolve_tickets_output_path(
    args: &TicketsArgs,
    repo: &GitRepo,
    forge_info: &RepoForgeInfo,
    config: &Config,
) -> Result<PathBuf, String> {
    if let Some(ref o) = args.output {
        return if o.is_absolute() {
            Ok(o.clone())
        } else {
            Ok(repo.root_dir.join(o))
        };
    }

    let current_forge = detect_forge_from_repo(repo).ok();
    let is_same_repo = current_forge
        .as_ref()
        .map(|cf| {
            cf.repo.eq_ignore_ascii_case(&forge_info.repo)
                && cf.owner.eq_ignore_ascii_case(&forge_info.owner)
        })
        .unwrap_or(args.remote.is_none());

    if is_same_repo {
        Ok(repo.root_dir.join(&config.tickets_file))
    } else {
        // Target is a different repository: find local clone
        let catalog = crate::catalog::load_catalog(Some(config), None);
        let slug = format!("{}/{}", forge_info.owner, forge_info.repo);
        let local_target_dir = catalog
            .iter()
            .find(|e| {
                e.slug.eq_ignore_ascii_case(&slug)
                    || e.name.eq_ignore_ascii_case(&forge_info.repo)
            })
            .and_then(crate::catalog::find_local_repo_path)
            .or_else(|| {
                std::env::var("HOME").ok().and_then(|h| {
                    let p = PathBuf::from(h).join("src").join(&forge_info.repo);
                    if p.join(".git").exists() {
                        Some(p)
                    } else {
                        None
                    }
                })
            });

        if let Some(target_dir) = local_target_dir {
            Ok(target_dir.join(&config.tickets_file))
        } else {
            Err(format!(
                "Target repository '{}/{}' has no local clone found in catalog or ~/src. Refusing to overwrite current repository cache. Use -o/--output to specify a destination file.",
                forge_info.owner, forge_info.repo
            ))
        }
    }
}

/// Synchronizes issues from the configured forge into `.antigravity/tickets.md`
pub fn sync_tickets(
    args: &TicketsArgs,
    repo: &GitRepo,
    config: &Config,
    _theme: &Theme,
) -> Result<SyncResult, String> {
    let forge_info = if let Some(ref r) = args.remote {
        parse_remote_url(r).ok_or_else(|| format!("Invalid remote URL '{}'", r))?
    } else {
        detect_forge_from_repo(repo)?
    };

    println!(
        "Fetching open issues from {} ({}/{})...",
        forge_info.provider, forge_info.owner, forge_info.repo
    );

    let client = ForgeClient::new();
    let issues = client.fetch_issues(&forge_info, &args.state, args.limit)?;

    let max_bytes = DEFAULT_MAX_BYTES;
    let (md_content, count, stopped_early) =
        generate_tickets_markdown(&forge_info.repo, &issues, max_bytes);

    let output_path = resolve_tickets_output_path(args, repo, &forge_info, config)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
    }

    fs::write(&output_path, &md_content).map_err(|e| {
        format!(
            "Failed to write tickets file '{}': {}",
            output_path.display(),
            e
        )
    })?;

    let bytes_written = md_content.len();

    println!(
        "Done! Synced {} complete ticket(s) into {} ({} / {} bytes).",
        count,
        output_path.display(),
        bytes_written,
        max_bytes
    );

    if stopped_early {
        println!(
            "Notice: Sync stopped early at {} bytes to avoid severing the next issue mid-text.",
            bytes_written
        );
    }

    Ok(SyncResult {
        count,
        bytes_written,
        max_bytes,
        stopped_early,
        output_path,
    })
}

/// Close one or more issues on the remote forge associated with the repository
pub fn close_tickets(
    repo: &GitRepo,
    ids: &[u64],
    comment: Option<&str>,
    theme: &Theme,
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }

    let forge_info = detect_forge_from_repo(repo)?;
    let client = ForgeClient::new();

    for &id in ids {
        println!(
            "{} Closing ticket #{} on {} ({}/{})...",
            theme.cyan("★"),
            id,
            forge_info.provider,
            forge_info.owner,
            forge_info.repo
        );

        match client.close_issue(&forge_info, id, comment) {
            Ok(()) => {
                println!("  {} Ticket #{} closed successfully.", theme.green("✓"), id);
            }
            Err(e) => {
                eprintln!("  {} Failed to close ticket #{}: {}", theme.red("✗"), id, e);
                return Err(e);
            }
        }
    }

    Ok(())
}
