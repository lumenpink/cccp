use cccp::cli::CommitArgs;
use cccp::ddtdd::is_chore_commit;

#[test]
fn test_is_chore_commit_detects_various_chore_patterns() {
    let args1 = CommitArgs {
        summary: Some("atualizar docs e configs".to_string()),
        commit_type: Some("chore".to_string()),
        scope: None,
        phase: None,
        message: None,
        body: None,
        breaking: false,
        all: false,
        dry_run: false,
        sign: false,
        no_sign: false,
        force: false,
    };
    assert!(is_chore_commit(&args1), "Tipo explícito 'chore' deve ser reconhecido");

    let mut args2 = args1.clone();
    args2.commit_type = None;
    args2.summary = Some("chore: atualizar gitignore".to_string());
    assert!(is_chore_commit(&args2), "Prefixo 'chore:' no summary deve ser reconhecido");

    let mut args3 = args1.clone();
    args3.commit_type = None;
    args3.summary = Some("chore(ci): otimizar steps".to_string());
    assert!(is_chore_commit(&args3), "Prefixo 'chore(scope):' deve ser reconhecido");

    let mut args4 = args1.clone();
    args4.commit_type = None;
    args4.summary = None;
    args4.message = Some("chore: mensagem direta".to_string());
    assert!(is_chore_commit(&args4), "Prefixo 'chore:' no message deve ser reconhecido");
}

#[test]
fn test_is_chore_commit_rejects_non_chore() {
    let args = CommitArgs {
        summary: Some("feat(auth): login de usuario".to_string()),
        commit_type: Some("feat".to_string()),
        scope: Some("auth".to_string()),
        phase: None,
        message: None,
        body: None,
        breaking: false,
        all: false,
        dry_run: false,
        sign: false,
        no_sign: false,
        force: false,
    };
    assert!(!is_chore_commit(&args), "Commits feat não são chore");
}
