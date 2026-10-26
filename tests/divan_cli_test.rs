use cccp::cli::{Cli, Commands, TicketAction};
use clap::Parser;

#[test]
fn test_tickets_plural_alias_invokes_divan() {
    let args = Cli::try_parse_from(["cccp", "tickets", "divan", "owner/repo"])
        .expect("Failed to parse tickets plural alias");
    match args.command {
        Some(Commands::Ticket(t)) => match t.action {
            Some(TicketAction::Divan(divan)) => {
                let (repo, prompt) = divan.resolve_repo_and_prompt();
                assert_eq!(repo.as_deref(), Some("owner/repo"));
                assert_eq!(prompt, None);
            }
            _ => panic!("Expected TicketAction::Divan"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }
}

#[test]
fn test_direct_divan_command() {
    let args = Cli::try_parse_from(["cccp", "divan", "owner/repo", "bug in auth"])
        .expect("Failed to parse direct cccp divan");
    match args.command {
        Some(Commands::Divan(divan)) => {
            let (repo, prompt) = divan.resolve_repo_and_prompt();
            assert_eq!(repo.as_deref(), Some("owner/repo"));
            assert_eq!(prompt.as_deref(), Some("bug in auth"));
        }
        _ => panic!("Expected Commands::Divan"),
    }
}

#[test]
fn test_tickets_divan_with_interspersed_options_and_qualified_repo() {
    let args = Cli::try_parse_from([
        "cccp",
        "tickets",
        "divan",
        "--dry-run",
        "-r",
        "lumen/lauda-client",
        "--model",
        "llama3.2",
        "fix",
    ])
    .expect("Failed to parse divan with options and positional prompt");

    match args.command {
        Some(Commands::Ticket(t)) => match t.action {
            Some(TicketAction::Divan(divan)) => {
                assert!(divan.dry_run);
                assert_eq!(divan.model.as_deref(), Some("llama3.2"));
                let (repo, prompt) = divan.resolve_repo_and_prompt();
                assert_eq!(repo.as_deref(), Some("lumen/lauda-client"));
                assert_eq!(prompt.as_deref(), Some("fix"));
            }
            _ => panic!("Expected TicketAction::Divan"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }
}

#[test]
fn test_divan_args_mutation_empty_and_whitespace_resolution() {
    let empty_divan = cccp::cli::DivanArgs::default();
    let (repo, prompt) = empty_divan.resolve_repo_and_prompt();
    assert_eq!(repo, None);
    assert_eq!(prompt, None);

    let whitespace_target = cccp::cli::DivanArgs {
        target: Some("  something with spaces  ".to_string()),
        ..Default::default()
    };
    let (repo, prompt) = whitespace_target.resolve_repo_and_prompt();
    assert_eq!(repo, None);
    assert_eq!(prompt.as_deref(), Some("something with spaces"));

    let single_repo_target = cccp::cli::DivanArgs {
        target: Some("  my-repo  ".to_string()),
        ..Default::default()
    };
    let (repo, prompt) = single_repo_target.resolve_repo_and_prompt();
    assert_eq!(repo.as_deref(), Some("my-repo"));
    assert_eq!(prompt, None);
}
