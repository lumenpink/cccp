use cccp::cli::{Cli, Commands};
use clap::Parser;

#[test]
fn test_commit_args_parses_short_force_flag() {
    let cli = Cli::try_parse_from(["cccp", "commit", "-m", "teste forçado", "-f"])
        .expect("Deve realizar o parse de -f com sucesso");
    match cli.command {
        Some(Commands::Commit(args)) => {
            assert!(args.force, "Flag -f deve definir force como true");
        }
        _ => panic!("Esperado comando Commit"),
    }
}

#[test]
fn test_commit_args_parses_long_force_flag() {
    let cli = Cli::try_parse_from(["cccp", "commit", "-m", "teste forçado", "--force"])
        .expect("Deve realizar o parse de --force com sucesso");
    match cli.command {
        Some(Commands::Commit(args)) => {
            assert!(args.force, "Flag --force deve definir force como true");
        }
        _ => panic!("Esperado comando Commit"),
    }
}

#[test]
fn test_commit_options_force_defaults_to_false() {
    let opts = cccp::commit_exec::CommitOptions::default();
    assert!(!opts.force, "CommitOptions::force padrão deve ser false");
}

#[test]
fn test_commit_args_parses_combined_flags_with_force() {
    let cli = Cli::try_parse_from(["cccp", "commit", "-a", "-f", "-p", "5", "resumo rápido"])
        .expect("Deve realizar o parse combinado com sucesso");
    match cli.command {
        Some(Commands::Commit(args)) => {
            assert!(args.all);
            assert!(args.force);
            assert_eq!(args.phase.as_deref(), Some("5"));
            assert_eq!(args.summary.as_deref(), Some("resumo rápido"));
        }
        _ => panic!("Esperado comando Commit"),
    }
}
