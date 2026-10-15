use cccp::cli::TicketsArgs;
use cccp::config::Config;
use cccp::forges::{ForgeIssue, RepoForgeInfo};
use cccp::git::GitRepo;
use cccp::tickets::{
    format_ticket_markdown, format_utc_now, generate_tickets_markdown, resolve_tickets_output_path,
    DEFAULT_MAX_BYTES,
};
use std::fs;
use tempfile::tempdir;

fn sample_issue(number: u64, title: &str, body: &str, labels: Vec<&str>) -> ForgeIssue {
    ForgeIssue {
        number,
        title: title.to_string(),
        state: "open".to_string(),
        author: "lumenpink".to_string(),
        labels: labels.into_iter().map(String::from).collect(),
        body: body.to_string(),
        html_url: format!("https://codeberg.org/lumenpink/cccp/issues/{}", number),
        created_at: "2026-09-30T10:00:00Z".to_string(),
        comments: Vec::new(),
    }
}

#[test]
fn test_format_ticket_markdown_complete() {
    let issue = sample_issue(
        84,
        "feat(tickets): markdown sync",
        "Full issue description with details",
        vec!["rust", "enhancement"],
    );

    let md = format_ticket_markdown(&issue);
    assert!(md.contains("## [#84] feat(tickets): markdown sync\n"));
    assert!(md.contains("- **Status**: `open`\n"));
    assert!(md.contains("- **Labels**: rust, enhancement\n"));
    assert!(md.contains("- **Link**: https://codeberg.org/lumenpink/cccp/issues/84\n"));
    assert!(md.contains("### Description\nFull issue description with details\n\n---\n"));
}

#[test]
fn test_format_ticket_markdown_defaults() {
    let issue = sample_issue(1, "Empty issue", "   ", vec![]);
    let md = format_ticket_markdown(&issue);
    assert!(md.contains("- **Labels**: none\n"));
    assert!(md.contains("### Description\n_No description provided._\n"));
}

#[test]
fn test_generate_tickets_markdown_ceiling_enforcement() {
    let issue1 = sample_issue(1, "Task One", "Description 1", vec!["core"]);
    let issue2 = sample_issue(2, "Task Two", "Description 2", vec!["cli"]);
    let issue3 = sample_issue(3, "Task Three", "Description 3", vec!["test"]);

    let issues = vec![issue1, issue2, issue3];

    // High ceiling: all tickets included
    let (md_all, count_all, stopped_all) =
        generate_tickets_markdown("cccp", &issues, DEFAULT_MAX_BYTES);
    assert_eq!(count_all, 3);
    assert!(!stopped_all);
    assert!(md_all.contains("## [#1] Task One"));
    assert!(md_all.contains("## [#2] Task Two"));
    assert!(md_all.contains("## [#3] Task Three"));

    // Tight ceiling: only 1 ticket can fit
    let small_ceiling = 350;
    let (md_small, count_small, stopped_small) =
        generate_tickets_markdown("cccp", &issues, small_ceiling);
    assert_eq!(count_small, 1);
    assert!(stopped_small);
    assert!(md_small.contains("## [#1] Task One"));
    assert!(!md_small.contains("## [#2] Task Two"));
    assert!(md_small.len() <= small_ceiling);

    // Verify tickets are never truncated in half
    assert!(md_small.ends_with("---\n"));
}

#[test]
fn test_format_utc_now_structure() {
    let ts = format_utc_now();
    assert!(ts.ends_with(" UTC"));
    assert_eq!(ts.len(), 23); // "YYYY-MM-DD HH:MM:SS UTC"
    assert_eq!(&ts[4..5], "-");
    assert_eq!(&ts[7..8], "-");
    assert_eq!(&ts[10..11], " ");
    assert_eq!(&ts[13..14], ":");
    assert_eq!(&ts[16..17], ":");
}

#[test]
fn test_atomic_file_write_and_read() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".antigravity").join("tickets.md");

    let issues = vec![sample_issue(
        42,
        "Answer to everything",
        "Don't panic",
        vec!["guide"],
    )];
    let (content, count, _) = generate_tickets_markdown("guide", &issues, DEFAULT_MAX_BYTES);
    assert_eq!(count, 1);

    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, &content).unwrap();

    let read_back = fs::read_to_string(&file_path).unwrap();
    assert_eq!(read_back, content);
}

#[test]
fn test_resolve_tickets_output_path_different_repo_rejects_cwd() {
    let temp = tempdir().unwrap();
    let current_repo_dir = temp.path().join("current-repo");
    fs::create_dir_all(&current_repo_dir).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&current_repo_dir)
        .status()
        .unwrap();

    let repo = GitRepo::open(Some(&current_repo_dir)).unwrap();
    let config = Config::default();

    // 1. Same repo / no remote override -> writes to current repo
    let args_same = TicketsArgs::default();
    let forge_same = RepoForgeInfo::new(
        cccp::forges::ForgeType::Codeberg,
        "codeberg.org".to_string(),
        "lumenpink".to_string(),
        "current-repo".to_string(),
    );
    let path_same = resolve_tickets_output_path(&args_same, &repo, &forge_same, &config).unwrap();
    assert_eq!(path_same, current_repo_dir.join(".antigravity/tickets.md"));

    // 2. Different repo with no local clone -> must NOT write to current_repo_dir
    let args_diff = TicketsArgs {
        remote: Some("https://codeberg.org/lumenpink/other-repo".to_string()),
        ..Default::default()
    };
    let forge_diff = RepoForgeInfo::new(
        cccp::forges::ForgeType::Codeberg,
        "codeberg.org".to_string(),
        "lumenpink".to_string(),
        "other-repo".to_string(),
    );
    let res = resolve_tickets_output_path(&args_diff, &repo, &forge_diff, &config);
    assert!(
        res.is_err(),
        "Expected error when other-repo has no local clone, but got {:?}",
        res
    );
}

#[test]
fn test_resolve_tickets_output_path_explicit_output_override() {
    let temp = tempdir().unwrap();
    let current_repo_dir = temp.path().join("current-repo");
    fs::create_dir_all(&current_repo_dir).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&current_repo_dir)
        .status()
        .unwrap();

    let repo = GitRepo::open(Some(&current_repo_dir)).unwrap();
    let config = Config::default();

    let explicit_file = temp.path().join("custom").join("my-tickets.md");
    let args = TicketsArgs {
        output: Some(explicit_file.clone()),
        ..Default::default()
    };
    let forge = RepoForgeInfo::new(
        cccp::forges::ForgeType::Codeberg,
        "codeberg.org".to_string(),
        "lumenpink".to_string(),
        "any-repo".to_string(),
    );

    let resolved = resolve_tickets_output_path(&args, &repo, &forge, &config).unwrap();
    assert_eq!(resolved, explicit_file);

    // Relative output path resolves relative to repo root
    let args_rel = TicketsArgs {
        output: Some(std::path::PathBuf::from("docs/issues.md")),
        ..Default::default()
    };
    let resolved_rel = resolve_tickets_output_path(&args_rel, &repo, &forge, &config).unwrap();
    assert_eq!(resolved_rel, current_repo_dir.join("docs/issues.md"));
}
