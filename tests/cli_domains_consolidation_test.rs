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
fn test_ci_canonical_command() {
    let parsed = Cli::parse_from(["cccp", "ci", "status"]);
    assert!(matches!(parsed.command, Some(Commands::Ci(_))));

    // cccp ci subcommands
    let parsed_pub = Cli::parse_from(["cccp", "ci", "publish-artifacts", "--dry-run"]);
    assert!(matches!(parsed_pub.command, Some(Commands::Ci(_))));

    let parsed_deploy = Cli::parse_from(["cccp", "ci", "deploy-site", "--dry-run"]);
    assert!(matches!(parsed_deploy.command, Some(Commands::Ci(_))));

    // cccp act alias
    let parsed_act = Cli::parse_from(["cccp", "act", "status"]);
    assert!(matches!(parsed_act.command, Some(Commands::Ci(_))));

    // Invalid subcommand is rejected
    assert!(Cli::try_parse_from(["cccp", "ci", "nonexistent-subcommand"]).is_err());
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
    // editor, serve, set-av, check-update, comrades must not exist at root
    assert!(Cli::try_parse_from(["cccp", "editor"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "serve"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "set-av", "VAR=val"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "check-update"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "comrades"]).is_err());
}

#[test]
fn test_mutation_root_help_slim_and_canonical() {
    use clap::CommandFactory;
    let help = Cli::command().render_help().to_string();

    // Canonical commands must be present
    assert!(help.contains("ticket"));
    assert!(help.contains("repo"));
    assert!(help.contains("sync"));
    assert!(help.contains("check"));
    assert!(help.contains("version"));
    assert!(help.contains("gen-version"));
    assert!(help.contains("ci"));

    // Removed commands must not appear in root commands list
    let lines: Vec<&str> = help.lines().collect();
    let command_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.starts_with("  ") && !line.starts_with("    "))
        .copied()
        .collect();

    for cmd in &[
        "editor",
        "serve",
        "comrades",
        "set-av",
        "check-update",
        "audit",
        "catalog",
    ] {
        assert!(
            !command_lines
                .iter()
                .any(|l| l.trim_start().starts_with(cmd)),
            "Removed command '{}' must not appear in help root subcommands list",
            cmd
        );
    }
}

#[test]
fn test_mutation_version_flags_strictness() {
    // cccp version rejects SemVer bump and write flags (UnknownArgument)
    let err = Cli::try_parse_from(["cccp", "version", "--bump", "patch"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);

    let err = Cli::try_parse_from(["cccp", "version", "-b", "major"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);

    let err = Cli::try_parse_from(["cccp", "version", "-w"]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);

    // cccp version supports format output (e.g. json or text)
    let ver_json = Cli::try_parse_from(["cccp", "version", "-f", "json"]).unwrap();
    match ver_json.command {
        Some(Commands::Version(args)) => {
            assert_eq!(args.format, "json");
        }
        _ => panic!("Expected Version"),
    }

    // cccp gen-version accepts bump and format
    let ok = Cli::try_parse_from(["cccp", "gen-version", "-b", "patch", "-f", "json"]).unwrap();
    match ok.command {
        Some(Commands::GenVersion(args)) => {
            assert_eq!(args.bump, Some(BumpType::Patch));
            assert_eq!(args.format, cccp::cli::VersionFormat::Json);
        }
        _ => panic!("Expected GenVersion"),
    }
}

#[test]
fn test_mutation_repo_default_and_subcommands() {
    // cccp repo without subcommands has action None (defaults to list)
    let parsed_root_repo = Cli::try_parse_from(["cccp", "repo"]).unwrap();
    match parsed_root_repo.command {
        Some(Commands::Repo(r)) => {
            assert!(r.action.is_none());
        }
        _ => panic!("Expected Commands::Repo"),
    }

    // cccp repo sync-tickets parses correctly
    let parsed_sync_t = Cli::try_parse_from(["cccp", "repo", "sync-tickets", "repo1"]).unwrap();
    match parsed_sync_t.command {
        Some(Commands::Repo(r)) => match r.action {
            Some(RepoSubcommands::UpdateTickets { repos, all }) => {
                assert_eq!(repos, vec!["repo1"]);
                assert!(!all);
            }
            _ => panic!("Expected UpdateTickets"),
        },
        _ => panic!("Expected Commands::Repo"),
    }
}

#[test]
fn test_mutation_update_check_flag() {
    let parsed = Cli::try_parse_from(["cccp", "update", "--check"]).unwrap();
    match parsed.command {
        Some(Commands::Update(up)) => {
            assert!(up.check);
        }
        _ => panic!("Expected Commands::Update"),
    }

    let parsed_no_check = Cli::try_parse_from(["cccp", "update"]).unwrap();
    match parsed_no_check.command {
        Some(Commands::Update(up)) => {
            assert!(!up.check);
        }
        _ => panic!("Expected Commands::Update"),
    }
}
