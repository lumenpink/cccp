use cccp::cli::{BumpType, Cli, Commands, RepoSubcommands, TicketAction};
use clap::Parser;

#[test]
fn test_check_canonical_command() {
    let parsed = Cli::parse_from(["cccp", "check", "--strict"]);
    match parsed.command {
        Some(Commands::Check(args)) => {
            assert!(args.strict);
        }
        _ => panic!("Expected Commands::Check"),
    }

    // Verify obsolete 'audit' alias is rejected
    assert!(Cli::try_parse_from(["cccp", "audit"]).is_err());
}

#[test]
fn test_repo_domain_subcommands() {
    // repo init
    let parsed_init = Cli::parse_from(["cccp", "repo", "init", "--profile", "lumen"]);
    match parsed_init.command {
        Some(Commands::Repo(repo_args)) => match repo_args.action {
            Some(RepoSubcommands::Init(init_args)) => {
                assert_eq!(init_args.profile, "lumen");
            }
            _ => panic!("Expected RepoSubcommands::Init"),
        },
        _ => panic!("Expected Commands::Repo"),
    }

    // repo list
    let parsed_list = Cli::parse_from(["cccp", "repo", "list", "--dirty"]);
    match parsed_list.command {
        Some(Commands::Repo(repo_args)) => match repo_args.action {
            Some(RepoSubcommands::List(filter)) => {
                assert!(filter.dirty);
            }
            _ => panic!("Expected RepoSubcommands::List"),
        },
        _ => panic!("Expected Commands::Repo"),
    }
}

#[test]
fn test_ticket_domain_new_and_divan_subcommands() {
    // ticket new direct via CLI flags
    let parsed_new = Cli::parse_from([
        "cccp",
        "ticket",
        "new",
        "--title",
        "feat(core): direct ticket creation",
        "--body",
        "Ticket created directly from CLI flags without opening editor",
    ]);
    match parsed_new.command {
        Some(Commands::Ticket(ticket_args)) => match ticket_args.action {
            Some(TicketAction::New { title, body, .. }) => {
                assert_eq!(title.as_deref(), Some("feat(core): direct ticket creation"));
                assert!(body.unwrap().contains("without opening editor"));
            }
            _ => panic!("Expected TicketAction::New"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }

    // ticket divan (AI triage)
    let parsed_divan = Cli::parse_from(["cccp", "ticket", "divan", "triage something"]);
    match parsed_divan.command {
        Some(Commands::Ticket(ticket_args)) => match ticket_args.action {
            Some(TicketAction::Divan(divan_args)) => {
                let (_, prompt) = divan_args.resolve_repo_and_prompt();
                assert_eq!(prompt.as_deref(), Some("triage something"));
            }
            _ => panic!("Expected TicketAction::Divan"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }
}

#[test]
fn test_version_and_gen_version_separation() {
    // cccp version displays current installed versions
    let parsed_ver = Cli::parse_from(["cccp", "version"]);
    assert!(matches!(parsed_ver.command, Some(Commands::Version(_))));

    // cccp gen-version computes next SemVer
    let parsed_gen = Cli::parse_from(["cccp", "gen-version", "--bump", "minor"]);
    match parsed_gen.command {
        Some(Commands::GenVersion(args)) => {
            assert_eq!(args.bump, Some(BumpType::Minor));
        }
        _ => panic!("Expected Commands::GenVersion"),
    }
}

#[test]
fn test_removed_root_commands_are_rejected() {
    // editor, ci, serve, set-av, check-update, comrades must not exist at root
    assert!(Cli::try_parse_from(["cccp", "editor"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "ci"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "serve"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "set-av", "VAR=val"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "check-update"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "comrades"]).is_err());
}
