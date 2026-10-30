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
