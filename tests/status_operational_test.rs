use cccp::git::{GitRepo, StatusDetails};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_status_details_staged_unstaged_untracked() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    // 1. Inicializa repo git
    let _ = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(root)
        .output();

    // Commit inicial com modified.txt
    fs::write(root.join("modified.txt"), "v1").unwrap();
    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output();

    // 2. Modifica modified.txt (unstaged)
    fs::write(root.join("modified.txt"), "v2").unwrap();

    // 3. Cria staged.txt e adiciona ao índice (staged)
    fs::write(root.join("staged.txt"), "staged content").unwrap();
    let _ = Command::new("git")
        .args(["add", "staged.txt"])
        .current_dir(root)
        .output();

    // 4. Cria untracked.txt (untracked)
    fs::write(root.join("untracked.txt"), "untracked").unwrap();

    let repo = GitRepo::open(Some(root)).expect("open repo");
    let details: StatusDetails = repo.get_status_details();

    assert!(
        details.staged.iter().any(|f| f.contains("staged.txt")),
        "staged.txt deve constar em staged"
    );
    assert!(
        details.unstaged.iter().any(|f| f.contains("modified.txt")),
        "modified.txt deve constar em unstaged"
    );
    assert!(
        details
            .untracked
            .iter()
            .any(|f| f.contains("untracked.txt")),
        "untracked.txt deve constar em untracked"
    );
}

#[test]
fn test_status_details_clean_repository() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let _ = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(root)
        .output();

    fs::write(root.join("clean.txt"), "content").unwrap();
    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root)
        .output();

    let repo = GitRepo::open(Some(root)).expect("open clean repo");
    let details = repo.get_status_details();

    assert!(details.staged.is_empty(), "Repo limpo não deve ter staged");
    assert!(
        details.unstaged.is_empty(),
        "Repo limpo não deve ter unstaged"
    );
    assert!(
        details.untracked.is_empty(),
        "Repo limpo não deve ter untracked"
    );
}
