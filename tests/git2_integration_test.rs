use cccp::git::GitRepo;
use std::fs;
use std::process::Command;

#[test]
fn test_git2_engine_open_and_inspect_repo() {
    let repo = GitRepo::open(None).expect("Failed to open repository via git");
    assert!(!repo.head_id.is_empty(), "head_id deve ser preenchido");
    assert_eq!(
        repo.head_short_id.len(),
        7,
        "head_short_id deve ter 7 caracteres"
    );
    assert_eq!(
        repo.head_date.len(),
        8,
        "head_date deve ter formato YYYYMMDD"
    );
    assert!(
        repo.root_dir.is_dir(),
        "root_dir deve ser um diretório existente"
    );
}

#[test]
fn test_git2_remote_url_and_status() {
    let repo_dir = std::path::Path::new(".");
    let remote = cccp::git::get_remote_url(repo_dir);
    assert!(remote.is_some(), "Deveria resolver remote origin");
    let status = cccp::git::inspect_local_status(repo_dir);
    assert!(
        status.is_some(),
        "Deveria inspecionar status local com sucesso"
    );
}

#[test]
fn test_git2_mutation_empty_and_detached_heads() {
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let p = temp_dir.path();

    let init_st = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(p)
        .status()
        .expect("Failed to init git repo");
    assert!(init_st.success());

    Command::new("git")
        .args(["config", "user.name", "Comrade Test"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@soviet.org"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(p)
        .status()
        .unwrap();

    // Inspecionar repositório vazio recém-criado sem commits
    let empty_repo = GitRepo::open(Some(p)).expect("Failed to open empty repo");
    assert!(
        empty_repo.head_id.is_empty(),
        "Repositório vazio deve ter head_id vazio"
    );
    assert!(
        !empty_repo.is_dirty,
        "Repositório vazio não deve ser marcado como dirty"
    );
    assert_eq!(empty_repo.commits_since_tag.len(), 0);

    // Criar commit inicial
    fs::write(p.join("init.txt"), "hello\n").unwrap();
    Command::new("git")
        .args(["add", "init.txt"])
        .current_dir(p)
        .status()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(p)
        .status()
        .unwrap();

    let rev_out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(p)
        .output()
        .unwrap();
    let commit_id = String::from_utf8_lossy(&rev_out.stdout).trim().to_string();

    let loaded = GitRepo::open(Some(p)).expect("Failed to reload repo with commit");
    assert_eq!(loaded.head_id, commit_id);
    assert_eq!(loaded.head_short_id.len(), 7);
    assert_eq!(loaded.commits_since_tag.len(), 1);
}
