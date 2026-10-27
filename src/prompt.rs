//! Lightweight shell prompt status helper (Bash / Zsh / Starship).
//! Returns fast status of the current repository:
//! - active branch
//! - dirty status
//! - DD-TDD cycle phase & active ticket
//! - open tickets count (from local cache .antigravity/tickets.md)
//! Latency is optimized for sub-15ms terminal prompt rendering.

use crate::cli::PromptArgs;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PromptInfo {
    pub branch: String,
    pub dirty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    pub open_tickets: usize,
    pub clean: bool,
}

pub fn gather_prompt_info(dir: Option<&Path>) -> Option<PromptInfo> {
    let target = match dir {
        Some(d) => d.to_path_buf(),
        None => std::env::current_dir().ok()?,
    };

    // Fast check: if neither target/.git nor any parent has .git
    let git_repo = crate::git::GitRepo::open(Some(&target)).ok()?;

    let branch = git_repo
        .branch
        .clone()
        .unwrap_or_else(|| "HEAD".to_string());
    let dirty = git_repo.is_dirty;

    let (ticket, phase) = if let Some((t_id, _, _)) = crate::ddtdd::infer_ticket_from_branch(&branch)
    {
        let log_messages = get_recent_commit_messages(&git_repo.root_dir, 10);
        let msg_slices: Vec<&str> = log_messages.iter().map(|s| s.as_str()).collect();
        let phase = crate::ddtdd::infer_next_phase(&msg_slices);
        (Some(t_id), Some(phase.to_string()))
    } else {
        (None, None)
    };

    let open_tickets = count_local_open_tickets(&git_repo.root_dir);

    Some(PromptInfo {
        branch,
        dirty,
        ticket,
        phase,
        open_tickets,
        clean: !dirty,
    })
}

fn get_recent_commit_messages(dir: &Path, count: usize) -> Vec<String> {
    let count_arg = format!("-n{}", count);
    let output = Command::new("git")
        .args(["log", &count_arg, "--pretty=format:%s"])
        .current_dir(dir)
        .output();
    if let Ok(out) = output {
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        Vec::new()
    }
}

fn count_local_open_tickets(root_dir: &Path) -> usize {
    let tickets_path = root_dir.join(".antigravity/tickets.md");
    if let Ok(content) = fs::read_to_string(tickets_path) {
        content.lines().filter(|l| l.contains("[open]")).count()
    } else {
        0
    }
}

pub fn generate_prompt(dir: Option<&Path>, args: &PromptArgs) -> Option<String> {
    let info = gather_prompt_info(dir)?;

    if args.json {
        return serde_json::to_string(&info).ok();
    }

    let wrap_escape = |code: &str| -> String {
        if args.no_color {
            String::new()
        } else if args.bash {
            format!("\\[{}\\]", code)
        } else if args.zsh {
            format!("%{{{}}}", code)
        } else {
            code.to_string()
        }
    };

    let reset = wrap_escape("\x1b[0m");
    let branch_color = if info.dirty {
        wrap_escape("\x1b[31;1m") // red bold
    } else {
        wrap_escape("\x1b[32;1m") // green bold
    };
    let ddtdd_color = wrap_escape("\x1b[33;1m"); // yellow bold
    let ticket_color = wrap_escape("\x1b[36m"); // cyan

    let dirty_mark = if info.dirty { "*" } else { "" };

    let mut parts = Vec::new();

    // Branch
    parts.push(format!("{}{}{}{}", branch_color, info.branch, dirty_mark, reset));

    // DD-TDD Phase
    if let (Some(t), Some(p)) = (info.ticket, &info.phase) {
        parts.push(format!("{}[#{} {}]{}", ddtdd_color, t, p, reset));
    }

    // Extended status
    if args.status && info.open_tickets > 0 {
        parts.push(format!("{}⚑{}{}", ticket_color, info.open_tickets, reset));
    }

    Some(parts.join(" "))
}
