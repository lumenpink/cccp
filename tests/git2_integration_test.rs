use cccp::git::GitRepo;

#[test]
fn test_git2_engine_open_and_inspect_repo() {
    let repo = GitRepo::open(None).expect("Failed to open repository via git2");
    assert!(!repo.head_id.is_empty(), "head_id deve ser preenchido");
    assert_eq!(repo.head_short_id.len(), 7, "head_short_id deve ter 7 caracteres");
    assert_eq!(repo.head_date.len(), 8, "head_date deve ter formato YYYYMMDD");
    assert!(repo.root_dir.is_dir(), "root_dir deve ser um diretório existente");
}

#[test]
fn test_git2_remote_url_and_status() {
    let repo_dir = std::path::Path::new(".");
    let remote = cccp::git::get_remote_url(repo_dir);
    assert!(remote.is_some(), "Deveria resolver remote origin");
    let status = cccp::git::inspect_local_status(repo_dir);
    assert!(status.is_some(), "Deveria inspecionar status local com sucesso");
}
