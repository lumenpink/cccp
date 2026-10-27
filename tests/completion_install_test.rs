use cccp::cli::{Cli, Commands, CompletionAction};
use cccp::config::Config;
use clap::Parser;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_parsing_completion_install_and_candidates() {
    // 1. Completion install with --bash flag
    let cli = Cli::try_parse_from(["cccp", "completion", "install", "--bash"])
        .expect("should parse cccp completion install --bash");
    match cli.command {
        Some(Commands::Completion(comp)) => match comp.action {
            Some(CompletionAction::Install(inst)) => {
                assert!(inst.bash);
                assert!(!inst.zsh);
                assert!(!inst.dry_run);
            }
            _ => panic!("Expected CompletionAction::Install"),
        },
        _ => panic!("Expected Commands::Completion"),
    }

    // 2. Completion install with --dry-run and path
    let cli = Cli::try_parse_from([
        "cccp",
        "completion",
        "install",
        "--zsh",
        "--dry-run",
        "-p",
        "/tmp/custom_zshrc",
    ])
    .expect("should parse cccp completion install --zsh --dry-run -p ...");
    match cli.command {
        Some(Commands::Completion(comp)) => match comp.action {
            Some(CompletionAction::Install(inst)) => {
                assert!(inst.zsh);
                assert!(inst.dry_run);
                assert_eq!(
                    inst.path.as_deref().map(|p| p.to_str().unwrap()),
                    Some("/tmp/custom_zshrc")
                );
            }
            _ => panic!("Expected CompletionAction::Install"),
        },
        _ => panic!("Expected Commands::Completion"),
    }

    // 3. Completion candidates flag
    let cli = Cli::try_parse_from(["cccp", "completion", "--candidates", "repos"])
        .expect("should parse cccp completion --candidates repos");
    match cli.command {
        Some(Commands::Completion(comp)) => {
            assert_eq!(comp.candidates.as_deref(), Some("repos"));
        }
        _ => panic!("Expected Commands::Completion"),
    }
}

#[test]
fn test_catalog_candidates_generation() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let catalog_json = r#"[
        {
            "name": "lauda",
            "forge": "https://gambi.2lp.in",
            "slug": "lumen/lauda",
            "aliases": ["player", "audio"],
            "description": "Music player"
        },
        {
            "name": "cccp",
            "forge": "https://gambi.2lp.in",
            "slug": "lumen/cccp",
            "aliases": ["maestro"],
            "description": "CCCP CLI"
        }
    ]"#;
    let catalog_path = root.join("catalog.json");
    fs::write(&catalog_path, catalog_json).unwrap();

    let config = Config {
        catalog_file: Some(catalog_path.to_string_lossy().to_string()),
        ..Default::default()
    };

    let candidates = cccp::completion::get_catalog_candidates(Some(&config), Some(root));
    assert!(candidates.contains(&"lumen/lauda".to_string()));
    assert!(candidates.contains(&"lauda".to_string()));
    assert!(candidates.contains(&"player".to_string()));
    assert!(candidates.contains(&"audio".to_string()));
    assert!(candidates.contains(&"lumen/cccp".to_string()));
    assert!(candidates.contains(&"cccp".to_string()));
    assert!(candidates.contains(&"maestro".to_string()));
}

#[test]
fn test_install_completion_into_file_and_idempotency() {
    let tmp = tempdir().unwrap();
    let bashrc = tmp.path().join(".bashrc");

    let initial = "# My custom bashrc\nexport EDITOR=nvim\n";
    fs::write(&bashrc, initial).unwrap();

    // 1. Dry run does not write to disk
    let dry_res = cccp::completion::install_completion_into_file(
        &bashrc,
        clap_complete::Shell::Bash,
        true,
    );
    assert!(dry_res.is_ok());
    assert_eq!(fs::read_to_string(&bashrc).unwrap(), initial);

    // 2. Real installation
    let install_res = cccp::completion::install_completion_into_file(
        &bashrc,
        clap_complete::Shell::Bash,
        false,
    );
    assert!(install_res.is_ok());
    let installed_content = fs::read_to_string(&bashrc).unwrap();
    assert!(installed_content.contains("# My custom bashrc"));
    assert!(installed_content.contains("# >>> CCCP SHELL COMPLETION >>>"));
    assert!(installed_content.contains("source <(cccp completion bash)"));
    assert!(installed_content.contains("# <<< CCCP SHELL COMPLETION <<<"));

    // 3. Idempotent installation: running again does NOT duplicate the block
    let second_install = cccp::completion::install_completion_into_file(
        &bashrc,
        clap_complete::Shell::Bash,
        false,
    );
    assert!(second_install.is_ok());
    let content_after_second = fs::read_to_string(&bashrc).unwrap();
    let count_begin = content_after_second
        .matches("# >>> CCCP SHELL COMPLETION >>>")
        .count();
    let count_end = content_after_second
        .matches("# <<< CCCP SHELL COMPLETION <<<")
        .count();
    assert_eq!(count_begin, 1, "Should have exactly 1 begin block marker");
    assert_eq!(count_end, 1, "Should have exactly 1 end block marker");
}
