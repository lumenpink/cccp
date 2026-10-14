use cccp::release::{generate_changelog, execute_release_tag, ReleaseTagOptions};
use tempfile::tempdir;
use std::process::Command;
use std::fs;

#[test]
fn test_generate_changelog_groups_conventional_commits() {
    let commits = vec![
        "feat(core): nova funcionalidade de busca",
        "fix(ui): corrigir alinhamento de botoes",
        "docs(readme): atualizar guia de instalacao",
        "chore(deps): atualizar dependencias",
        "feat(api)!: breaking change na rota de usuarios",
    ];

    let changelog = generate_changelog("0.4.0", "2026-10-14", &commits);

    assert!(changelog.contains("## [0.4.0] - 2026-10-14"));
    assert!(changelog.contains("### Features"));
    assert!(changelog.contains("- **core:** nova funcionalidade de busca"));
    assert!(changelog.contains("- **api:** breaking change na rota de usuarios"));
    assert!(changelog.contains("### Bug Fixes"));
    assert!(changelog.contains("- **ui:** corrigir alinhamento de botoes"));
    assert!(changelog.contains("### Documentation"));
    assert!(changelog.contains("- **readme:** atualizar guia de instalacao"));
}

#[test]
fn test_execute_release_tag_end_to_end() {
    let dir = tempdir().expect("tempdir");
    let repo_dir = dir.path();

    // Inicia um repo git temporário hermético
    Command::new("git").args(["init", "-b", "main"]).current_dir(repo_dir).output().expect("git init");
    Command::new("git").args(["config", "user.name", "Gosplan Test"]).current_dir(repo_dir).output().expect("config name");
    Command::new("git").args(["config", "user.email", "test@gosplan.su"]).current_dir(repo_dir).output().expect("config email");

    // Cria commit inicial
    fs::write(repo_dir.join("VERSION"), "0.3.0-dev.10\n").expect("write version");
    Command::new("git").args(["add", "VERSION"]).current_dir(repo_dir).output().expect("add");
    Command::new("git").args(["commit", "-m", "feat: initial commit"]).current_dir(repo_dir).output().expect("commit");

    let options = ReleaseTagOptions {
        version: Some("0.4.0".to_string()),
        title: Some("Calendario e releases estaveis".to_string()),
        message: None,
        no_v: false,
        sign: false,
    };

    let result = execute_release_tag(repo_dir, &options);
    assert!(result.is_ok(), "Execução de release tag deve suceder: {:?}", result.err());

    // 1. VERSION deve conter versão limpa (sem -dev)
    let version_content = fs::read_to_string(repo_dir.join("VERSION")).expect("read VERSION");
    assert_eq!(version_content.trim(), "0.4.0");

    // 2. CHANGELOG.md deve existir e conter a versão
    let changelog_content = fs::read_to_string(repo_dir.join("CHANGELOG.md")).expect("read CHANGELOG");
    assert!(changelog_content.contains("## [0.4.0]"));

    // 3. A tag Git v0.4.0 deve existir
    let tag_output = Command::new("git").args(["tag", "-l", "v0.4.0"]).current_dir(repo_dir).output().expect("git tag -l");
    let tag_name = String::from_utf8_lossy(&tag_output.stdout).trim().to_string();
    assert_eq!(tag_name, "v0.4.0");
}
