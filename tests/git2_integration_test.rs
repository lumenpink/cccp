use cccp::git::GitRepo;

#[test]
fn test_git2_engine_open_and_inspect_repo() {
    let repo = GitRepo::open(None).expect("Failed to open repository via git2");
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
    let repo = git2::Repository::init(temp_dir.path()).expect("Failed to init git repo");

    // Inspecionar repositório vazio recém-criado sem commits
    let empty_repo = GitRepo::open(Some(temp_dir.path())).expect("Failed to open empty repo");
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
    let sig = git2::Signature::now("Comrade Test", "test@soviet.org").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    let commit_id = repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            "chore: initial commit",
            &tree,
            &[],
        )
        .unwrap();

    let loaded = GitRepo::open(Some(temp_dir.path())).expect("Failed to reload repo with commit");
    assert_eq!(loaded.head_id, commit_id.to_string());
    assert_eq!(loaded.head_short_id.len(), 7);
    assert_eq!(loaded.commits_since_tag.len(), 1);
}
