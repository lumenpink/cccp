//! Integration and contract tests for CCCP Epic and Inbox Governance (Ticket #74, GOST-25)

use cccp::cultural::Theme;
use cccp::git::GitRepo;
use cccp::governance::{
    cmd_epic_finish, cmd_epic_list, cmd_epic_start, cmd_inbox_accept, cmd_inbox_list,
    cmd_inbox_review,
};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn setup_temp_repo() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let p = dir.path();

    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Gosplan Auditor"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "auditor@gosplan.su"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();

    fs::write(p.join("README.md"), "# Sovereign Core\n").unwrap();
    fs::write(p.join("VERSION"), "0.1.0\n").unwrap();
    fs::write(p.join("GOSTVER"), "GOST_VERSION=\"2.0.0\"\n").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial repository setup"])
        .current_dir(p)
        .status()
        .unwrap();

    dir
}

#[test]
fn test_epic_lifecycle_start_list_finish() {
    let temp = setup_temp_repo();
    let p = temp.path();
    let theme = Theme::with_color(false);

    let repo = GitRepo::open(Some(p)).expect("open repo");

    // 1. Start epic
    let res_start = cmd_epic_start(&repo, "rewrite-engine", &theme);
    assert!(res_start.is_ok(), "epic start must succeed: {:?}", res_start.err());

    let repo_after_start = GitRepo::open(Some(p)).unwrap();
    assert_eq!(
        repo_after_start.branch.as_deref(),
        Some("epic/rewrite-engine"),
        "Active branch must be epic/rewrite-engine"
    );

    // 2. List epics
    let epics = cmd_epic_list(&repo_after_start, &theme).expect("list epics");
    assert!(
        epics.contains(&"epic/rewrite-engine".to_string()),
        "Epic list must contain epic/rewrite-engine"
    );

    // Commit change on epic
    fs::write(p.join("feature.txt"), "epic work").unwrap();
    Command::new("git").args(["add", "feature.txt"]).current_dir(p).status().unwrap();
    Command::new("git").args(["commit", "-m", "feat(epic): work on rewrite"]).current_dir(p).status().unwrap();

    // 3. Finish epic
    let res_finish = cmd_epic_finish(&repo_after_start, "rewrite-engine", &theme);
    assert!(res_finish.is_ok(), "epic finish must succeed: {:?}", res_finish.err());

    let repo_final = GitRepo::open(Some(p)).unwrap();
    assert_eq!(repo_final.branch.as_deref(), Some("main"));
    assert!(p.join("feature.txt").exists());
}

#[test]
fn test_inbox_quarantine_list_review_accept() {
    let temp = setup_temp_repo();
    let p = temp.path();
    let theme = Theme::with_color(false);

    // Set up mock mirrors
    Command::new("git").args(["remote", "add", "origin", "git@gambi.2lp.in:lumen/test.git"]).current_dir(p).status().unwrap();
    Command::new("git").args(["remote", "add", "codeberg", "https://codeberg.org/lumenpink/test.git"]).current_dir(p).status().unwrap();
    Command::new("git").args(["remote", "add", "github", "https://github.com/lumenpink/test.git"]).current_dir(p).status().unwrap();

    // Create an external contribution in quarantine branch inbox/pr-42
    Command::new("git").args(["checkout", "-b", "inbox/pr-42"]).current_dir(p).status().unwrap();
    fs::write(p.join("external_patch.txt"), "community patch").unwrap();
    Command::new("git").args(["add", "external_patch.txt"]).current_dir(p).status().unwrap();
    Command::new("git").args(["commit", "-m", "fix: community external contribution"]).current_dir(p).status().unwrap();

    // Switch back to main
    Command::new("git").args(["checkout", "main"]).current_dir(p).status().unwrap();

    let repo = GitRepo::open(Some(p)).unwrap();

    // 1. List inbox
    let inbox_branches = cmd_inbox_list(&repo, &theme).expect("list inbox");
    assert!(inbox_branches.contains(&"inbox/pr-42".to_string()));

    // 2. Review inbox item
    let review_info = cmd_inbox_review(&repo, "pr-42", &theme).expect("review inbox");
    assert!(review_info.contains("community external contribution"));

    // 3. Accept inbox item into main
    let res_accept = cmd_inbox_accept(&repo, "pr-42", &theme);
    assert!(res_accept.is_ok(), "accept inbox must succeed: {:?}", res_accept.err());

    assert!(p.join("external_patch.txt").is_file());

    // GOST-25 Remotes Check: codeberg and github must have pushurl = no_push
    let codeberg_pushurl = Command::new("git")
        .args(["config", "remote.codeberg.pushurl"])
        .current_dir(p)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    assert_eq!(codeberg_pushurl, "no_push", "GOST-25 requires codeberg mirror pushurl = no_push");

    let github_pushurl = Command::new("git")
        .args(["config", "remote.github.pushurl"])
        .current_dir(p)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    assert_eq!(github_pushurl, "no_push", "GOST-25 requires github mirror pushurl = no_push");
}
