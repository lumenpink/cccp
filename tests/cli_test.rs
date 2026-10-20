//! Functional tests for CCCP Command Line Interface (CLI).
//!
//! Validates command parsing, global flags (-C, -v, -q, --no-color, --lang),
//! subcommands, argument validation, defaults, conversions, and error handling.

use cccp::cli::{
    AgentSubcommands, BumpType, Cli, Commands, DepsSubcommands, ForgeActions, HookSubcommands,
    ProfileCommand, RepoSubcommands, SpoolSubcommands, TicketAction, VersionFormat,
};
use clap::error::ErrorKind;
use clap::Parser;
use std::path::PathBuf;

#[test]
fn test_cli_debug_assert() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}

// =============================================================================
// 1. Global Flags Tests
// =============================================================================

#[test]
fn test_global_flags_defaults() {
    let cli = Cli::try_parse_from(["cccp"]).expect("should parse root cccp");
    assert_eq!(cli.dir, None);
    assert_eq!(cli.verbose, 0);
    assert!(!cli.quiet);
    assert!(!cli.no_color);
    assert_eq!(cli.lang, None);
    assert!(cli.command.is_none());
}

#[test]
fn test_global_flags_short_and_long() {
    // Test -C / --dir
    let cli =
        Cli::try_parse_from(["cccp", "-C", "/custom/path", "status"]).expect("should parse -C");
    assert_eq!(cli.dir, Some(PathBuf::from("/custom/path")));
    assert!(matches!(cli.command, Some(Commands::Status(_))));

    let cli = Cli::try_parse_from(["cccp", "--dir", "/another/path", "status"])
        .expect("should parse --dir");
    assert_eq!(cli.dir, Some(PathBuf::from("/another/path")));

    // Test -v / -vv / -vvv / --verbose
    let cli = Cli::try_parse_from(["cccp", "-v", "status"]).expect("should parse -v");
    assert_eq!(cli.verbose, 1);

    let cli = Cli::try_parse_from(["cccp", "-vv", "status"]).expect("should parse -vv");
    assert_eq!(cli.verbose, 2);

    let cli = Cli::try_parse_from(["cccp", "-vvv", "status"]).expect("should parse -vvv");
    assert_eq!(cli.verbose, 3);

    let cli = Cli::try_parse_from(["cccp", "--verbose", "--verbose", "status"])
        .expect("should parse --verbose multiple times");
    assert_eq!(cli.verbose, 2);

    // Test -q / --quiet
    let cli = Cli::try_parse_from(["cccp", "-q", "status"]).expect("should parse -q");
    assert!(cli.quiet);

    let cli = Cli::try_parse_from(["cccp", "--quiet", "status"]).expect("should parse --quiet");
    assert!(cli.quiet);

    // Test --no-color
    let cli =
        Cli::try_parse_from(["cccp", "--no-color", "status"]).expect("should parse --no-color");
    assert!(cli.no_color);

    // Test --lang
    let cli =
        Cli::try_parse_from(["cccp", "--lang", "pt-BR", "status"]).expect("should parse --lang");
    assert_eq!(cli.lang.as_deref(), Some("pt-BR"));
}

#[test]
fn test_global_flags_positioned_after_subcommand() {
    // In Clap, global flags can be positioned after subcommands as well
    let cli = Cli::try_parse_from([
        "cccp",
        "status",
        "-C",
        "/tmp",
        "-vv",
        "--no-color",
        "--lang",
        "ru-RU",
    ])
    .expect("should parse global flags after subcommand");
    assert_eq!(cli.dir, Some(PathBuf::from("/tmp")));
    assert_eq!(cli.verbose, 2);
    assert!(cli.no_color);
    assert_eq!(cli.lang.as_deref(), Some("ru-RU"));
    assert!(matches!(cli.command, Some(Commands::Status(_))));
}

// =============================================================================
// 2. Primary Subcommand Parsing Tests
// =============================================================================

#[test]
fn test_commit_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "commit",
        "-t",
        "feat",
        "-s",
        "api",
        "-m",
        "implement endpoints",
        "-b",
        "detailed body line 1\nline 2",
        "--breaking",
        "-a",
        "--dry-run",
        "-S",
    ])
    .expect("should parse commit flags");

    match cli.command {
        Some(Commands::Commit(c)) => {
            assert_eq!(c.commit_type.as_deref(), Some("feat"));
            assert_eq!(c.scope.as_deref(), Some("api"));
            assert_eq!(c.message.as_deref(), Some("implement endpoints"));
            assert_eq!(c.body.as_deref(), Some("detailed body line 1\nline 2"));
            assert!(c.breaking);
            assert!(c.all);
            assert!(c.dry_run);
            assert!(c.sign);
            assert!(!c.no_sign);
        }
        _ => panic!("Expected Commands::Commit"),
    }

    let cli_no_sign =
        Cli::try_parse_from(["cccp", "commit", "--no-sign"]).expect("should parse --no-sign");
    match cli_no_sign.command {
        Some(Commands::Commit(c)) => {
            assert!(c.no_sign);
            assert!(!c.sign);
        }
        _ => panic!("Expected Commands::Commit"),
    }
}

#[test]
fn test_lint_command_options() {
    let cli = Cli::try_parse_from(["cccp", "lint", "-m", "fix: resolve crash", "--strict"])
        .expect("should parse lint message and strict");
    match cli.command {
        Some(Commands::Lint(l)) => {
            assert_eq!(l.message.as_deref(), Some("fix: resolve crash"));
            assert!(l.strict);
            assert!(l.file.is_none());
            assert!(l.range.is_none());
        }
        _ => panic!("Expected Commands::Lint"),
    }

    let cli_file = Cli::try_parse_from(["cccp", "lint", "-f", ".git/COMMIT_EDITMSG"])
        .expect("should parse lint file");
    match cli_file.command {
        Some(Commands::Lint(l)) => {
            assert_eq!(l.file, Some(PathBuf::from(".git/COMMIT_EDITMSG")));
            assert!(!l.strict);
        }
        _ => panic!("Expected Commands::Lint"),
    }

    let cli_range = Cli::try_parse_from(["cccp", "lint", "-r", "origin/main..HEAD"])
        .expect("should parse lint range");
    match cli_range.command {
        Some(Commands::Lint(l)) => {
            assert_eq!(l.range.as_deref(), Some("origin/main..HEAD"));
        }
        _ => panic!("Expected Commands::Lint"),
    }
}

#[test]
fn test_version_command_options() {
    // cccp version parses as Commands::Version without flags
    let cli = Cli::try_parse_from(["cccp", "version"]).expect("should parse version default");
    assert!(matches!(cli.command, Some(Commands::Version(_))));
}

#[test]
fn test_gen_version_command_options() {
    // Default format is Clean
    let cli =
        Cli::try_parse_from(["cccp", "gen-version"]).expect("should parse gen-version default");
    match cli.command {
        Some(Commands::GenVersion(v)) => {
            assert_eq!(v.format, VersionFormat::Clean);
            assert!(!v.write);
            assert_eq!(v.bump, None);
        }
        _ => panic!("Expected Commands::GenVersion"),
    }

    // Format dev, write flag, bump major
    let cli = Cli::try_parse_from(["cccp", "gen-version", "-f", "dev", "-w", "-b", "major"])
        .expect("should parse gen-version flags");
    match cli.command {
        Some(Commands::GenVersion(v)) => {
            assert_eq!(v.format, VersionFormat::Dev);
            assert!(v.write);
            assert_eq!(v.bump, Some(BumpType::Major));
        }
        _ => panic!("Expected Commands::GenVersion"),
    }

    // Other formats and bump types
    let cli = Cli::try_parse_from(["cccp", "gen-version", "-f", "json", "-b", "minor"])
        .expect("should parse json and minor");
    match cli.command {
        Some(Commands::GenVersion(v)) => {
            assert_eq!(v.format, VersionFormat::Json);
            assert_eq!(v.bump, Some(BumpType::Minor));
        }
        _ => panic!("Expected Commands::GenVersion"),
    }

    let cli = Cli::try_parse_from(["cccp", "gen-version", "-f", "tag", "-b", "patch"])
        .expect("should parse tag and patch");
    match cli.command {
        Some(Commands::GenVersion(v)) => {
            assert_eq!(v.format, VersionFormat::Tag);
            assert_eq!(v.bump, Some(BumpType::Patch));
        }
        _ => panic!("Expected Commands::GenVersion"),
    }
}

#[test]
fn test_hook_command_options_and_subcommands() {
    // Install flag
    let cli = Cli::try_parse_from([
        "cccp",
        "hook",
        "--install",
        "--force",
        "--hook-dir",
        "/custom/hooks",
    ])
    .expect("should parse hook install");
    match cli.command {
        Some(Commands::Hook(h)) => {
            assert!(h.install);
            assert!(!h.uninstall);
            assert!(h.force);
            assert_eq!(h.hook_dir, Some(PathBuf::from("/custom/hooks")));
            assert!(h.hook.is_none());
        }
        _ => panic!("Expected Commands::Hook"),
    }

    // Uninstall flag
    let cli = Cli::try_parse_from(["cccp", "hook", "-u"]).expect("should parse hook -u");
    match cli.command {
        Some(Commands::Hook(h)) => {
            assert!(h.uninstall);
            assert!(!h.install);
        }
        _ => panic!("Expected Commands::Hook"),
    }

    // Subcommand: commit-msg
    let cli = Cli::try_parse_from([
        "cccp",
        "hook",
        "commit-msg",
        ".git/COMMIT_EDITMSG",
        "message",
        "abc1234",
    ])
    .expect("should parse hook commit-msg");
    match cli.command {
        Some(Commands::Hook(h)) => match h.hook {
            Some(HookSubcommands::CommitMsg { file, source, sha }) => {
                assert_eq!(file, PathBuf::from(".git/COMMIT_EDITMSG"));
                assert_eq!(source.as_deref(), Some("message"));
                assert_eq!(sha.as_deref(), Some("abc1234"));
            }
            _ => panic!("Expected HookSubcommands::CommitMsg"),
        },
        _ => panic!("Expected Commands::Hook"),
    }

    // Subcommand: pre-push
    let cli = Cli::try_parse_from([
        "cccp",
        "hook",
        "pre-push",
        "origin",
        "git@codeberg.org:user/repo.git",
    ])
    .expect("should parse hook pre-push");
    match cli.command {
        Some(Commands::Hook(h)) => match h.hook {
            Some(HookSubcommands::PrePush { remote, url }) => {
                assert_eq!(remote.as_deref(), Some("origin"));
                assert_eq!(url.as_deref(), Some("git@codeberg.org:user/repo.git"));
            }
            _ => panic!("Expected HookSubcommands::PrePush"),
        },
        _ => panic!("Expected Commands::Hook"),
    }

    // Subcommand: post-commit
    let cli = Cli::try_parse_from(["cccp", "hook", "post-commit"])
        .expect("should parse hook post-commit");
    match cli.command {
        Some(Commands::Hook(h)) => {
            assert!(matches!(h.hook, Some(HookSubcommands::PostCommit)));
        }
        _ => panic!("Expected Commands::Hook"),
    }

    // Subcommand: reference-transaction
    let cli = Cli::try_parse_from(["cccp", "hook", "reference-transaction", "committed"])
        .expect("should parse hook reference-transaction");
    match cli.command {
        Some(Commands::Hook(h)) => match h.hook {
            Some(HookSubcommands::ReferenceTransaction { state }) => {
                assert_eq!(state.as_deref(), Some("committed"));
            }
            _ => panic!("Expected HookSubcommands::ReferenceTransaction"),
        },
        _ => panic!("Expected Commands::Hook"),
    }
}

#[test]
fn test_status_command_options() {
    let cli = Cli::try_parse_from(["cccp", "status", "-d", "--json"]).expect("should parse status");
    match cli.command {
        Some(Commands::Status(s)) => {
            assert!(s.detailed);
            assert!(s.json);
        }
        _ => panic!("Expected Commands::Status"),
    }
}

#[test]
fn test_ticket_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "ticket",
        "-r",
        "codeberg",
        "-o",
        "tickets.md",
        "-l",
        "25",
        "-s",
        "closed",
    ])
    .expect("should parse ticket flags");

    match cli.command {
        Some(Commands::Ticket(t)) => {
            assert_eq!(t.remote.as_deref(), Some("codeberg"));
            assert_eq!(t.output, Some(PathBuf::from("tickets.md")));
            assert_eq!(t.limit, 25);
            assert_eq!(t.state, "closed");
        }
        _ => panic!("Expected Commands::Ticket"),
    }
}

#[test]
fn test_ticket_close_subcommand_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "ticket",
        "close",
        "133",
        "134",
        "-m",
        "Resolved canonically via CCCP",
    ])
    .expect("should parse ticket close subcommand");

    match cli.command {
        Some(Commands::Ticket(t)) => match t.action {
            Some(cccp::cli::TicketAction::Close { ids, comment }) => {
                assert_eq!(ids, vec![133, 134]);
                assert_eq!(comment.as_deref(), Some("Resolved canonically via CCCP"));
            }
            _ => panic!("Expected TicketAction::Close"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }
}

#[test]
fn test_merge_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "merge",
        "feat/133-feature",
        "-f",
        "-m",
        "Merge commit message",
    ])
    .expect("should parse merge flags");

    match cli.command {
        Some(Commands::Merge(m)) => {
            assert_eq!(m.branch, "feat/133-feature");
            assert!(m.force);
            assert_eq!(m.message.as_deref(), Some("Merge commit message"));
        }
        _ => panic!("Expected Commands::Merge"),
    }
}

#[test]
fn test_sync_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "sync",
        "-t",
        "-a",
        "-c",
        "-f",
        "--dir",
        "/target/repo",
    ])
    .expect("should parse sync flags");
    match cli.command {
        Some(Commands::Sync(s)) => {
            assert!(s.tickets);
            assert!(s.agents);
            assert!(s.catalog);
            assert!(s.force);
            assert_eq!(s.dir, Some(PathBuf::from("/target/repo")));
        }
        _ => panic!("Expected Commands::Sync"),
    }
}

#[test]
fn test_divan_command_options_and_shortcuts() {
    // Test Ollama shortcut (-o) under cccp ticket divan
    let cli = Cli::try_parse_from([
        "cccp",
        "ticket",
        "divan",
        "-o",
        "-m",
        "deepseek-coder",
        "refactor tests",
    ])
    .expect("should parse ticket divan -o");
    match cli.command {
        Some(Commands::Ticket(t)) => match t.action {
            Some(TicketAction::Divan(d)) => {
                assert!(d.ollama);
                assert!(!d.gemini);
                assert_eq!(d.model.as_deref(), Some("deepseek-coder"));
                let (repo, prompt) = d.resolve_repo_and_prompt();
                assert_eq!(repo, None);
                assert_eq!(prompt.as_deref(), Some("refactor tests"));
            }
            _ => panic!("Expected TicketAction::Divan"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }

    // Test Gemini shortcut (-g) and visible alias (--pg) under ticket divan
    let cli = Cli::try_parse_from([
        "cccp",
        "ticket",
        "divan",
        "--pg",
        "--direct",
        "--dry-run",
        "my-repo",
        "issue report",
    ])
    .expect("should parse ticket divan --pg");
    match cli.command {
        Some(Commands::Ticket(t)) => match t.action {
            Some(TicketAction::Divan(d)) => {
                assert!(d.gemini);
                assert!(d.direct);
                assert!(d.dry_run);
                let (repo, prompt) = d.resolve_repo_and_prompt();
                assert_eq!(repo.as_deref(), Some("my-repo"));
                assert_eq!(prompt.as_deref(), Some("issue report"));
            }
            _ => panic!("Expected TicketAction::Divan"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }

    // Top-level divan is rejected
    assert!(Cli::try_parse_from(["cccp", "divan"]).is_err());
}

#[test]
fn test_forge_command_options() {
    let cli = Cli::try_parse_from(["cccp", "forge", "detect"]).expect("should parse forge detect");
    match cli.command {
        Some(Commands::Forge(f)) => {
            assert!(matches!(f.action, Some(ForgeActions::Detect)));
        }
        _ => panic!("Expected Commands::Forge"),
    }

    let cli = Cli::try_parse_from(["cccp", "forge", "issues", "-s", "all"])
        .expect("should parse forge issues");
    match cli.command {
        Some(Commands::Forge(f)) => match f.action {
            Some(ForgeActions::Issues { state }) => {
                assert_eq!(state, "all");
            }
            _ => panic!("Expected ForgeActions::Issues"),
        },
        _ => panic!("Expected Commands::Forge"),
    }

    let cli = Cli::try_parse_from(["cccp", "forge", "prs", "-s", "closed"])
        .expect("should parse forge prs");
    match cli.command {
        Some(Commands::Forge(f)) => match f.action {
            Some(ForgeActions::Prs { state }) => {
                assert_eq!(state, "closed");
            }
            _ => panic!("Expected ForgeActions::Prs"),
        },
        _ => panic!("Expected Commands::Forge"),
    }
}

#[test]
fn test_soviet_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "soviet",
        "--anthem",
        "--quote",
        "--plan",
        "--order227",
    ])
    .expect("should parse soviet options");
    match cli.command {
        Some(Commands::Soviet(s)) => {
            assert!(s.anthem);
            assert!(s.quote);
            assert!(s.plan);
            assert!(s.order227);
        }
        _ => panic!("Expected Commands::Soviet"),
    }
}

#[test]
fn test_tag_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "tag",
        "v2.0.0",
        "-t",
        "Release 2.0.0",
        "-m",
        "Official major release",
        "--no-v",
        "-s",
    ])
    .expect("should parse tag options");

    match cli.command {
        Some(Commands::Tag(t)) => {
            assert_eq!(t.version.as_deref(), Some("v2.0.0"));
            assert_eq!(t.title.as_deref(), Some("Release 2.0.0"));
            assert_eq!(t.message.as_deref(), Some("Official major release"));
            assert!(t.no_v);
            assert!(t.sign);
        }
        _ => panic!("Expected Commands::Tag"),
    }
}

#[test]
fn test_changelog_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "changelog",
        "-o",
        "HISTORY.md",
        "--dry-run",
        "-r",
        "v1.0.0..HEAD",
    ])
    .expect("should parse changelog options");
    match cli.command {
        Some(Commands::Changelog(c)) => {
            assert_eq!(c.output, PathBuf::from("HISTORY.md"));
            assert!(c.dry_run);
            assert_eq!(c.range.as_deref(), Some("v1.0.0..HEAD"));
        }
        _ => panic!("Expected Commands::Changelog"),
    }
}

#[test]
fn test_push_command_options() {
    let cli = Cli::try_parse_from(["cccp", "push", "origin", "main", "--force"])
        .expect("should parse push options");
    match cli.command {
        Some(Commands::Push(p)) => {
            assert_eq!(p.git_args, vec!["origin", "main", "--force"]);
            assert!(!p.all);
            assert!(!p.dry_run);
        }
        _ => panic!("Expected Commands::Push"),
    }

    let cli_batch =
        Cli::try_parse_from(["cccp", "push", "-a", "-n"]).expect("should parse batch push options");
    match cli_batch.command {
        Some(Commands::Push(p)) => {
            assert!(p.all);
            assert!(p.dry_run);
            assert!(p.git_args.is_empty());
        }
        _ => panic!("Expected Commands::Push"),
    }

    let cli_cat_push = Cli::try_parse_from(["cccp", "repo", "push", "--dry-run"])
        .expect("should parse repo push options");
    match cli_cat_push.command {
        Some(Commands::Repo(c)) => match c.action {
            Some(RepoSubcommands::Push { dry_run }) => {
                assert!(dry_run);
            }
            _ => panic!("Expected RepoSubcommands::Push"),
        },
        _ => panic!("Expected Commands::Repo"),
    }
}

#[test]
fn test_batch_assemble_command_options() {
    let cli_cat_assemble = Cli::try_parse_from(["cccp", "repo", "assemble", "--dry-run"])
        .expect("should parse repo assemble options");
    match cli_cat_assemble.command {
        Some(Commands::Repo(c)) => match c.action {
            Some(RepoSubcommands::Assemble { dry_run }) => {
                assert!(dry_run);
            }
            _ => panic!("Expected RepoSubcommands::Assemble"),
        },
        _ => panic!("Expected Commands::Repo"),
    }

    let cli_profile_assemble = Cli::try_parse_from(["cccp", "profile", "assemble", "--all", "-n"])
        .expect("should parse profile assemble options");
    match cli_profile_assemble.command {
        Some(Commands::Profile(ProfileCommand::Assemble(args))) => {
            assert!(args.all);
            assert!(args.dry_run);
        }
        _ => panic!("Expected Commands::Profile(ProfileCommand::Assemble)"),
    }
}

#[test]
fn test_config_command_options() {
    let cli = Cli::try_parse_from(["cccp", "config", "default_base_version", "1.0.0", "-g"])
        .expect("should parse config set");
    match cli.command {
        Some(Commands::Config(c)) => {
            assert_eq!(c.key.as_deref(), Some("default_base_version"));
            assert_eq!(c.value.as_deref(), Some("1.0.0"));
            assert!(c.is_global);
            assert!(!c.list);
        }
        _ => panic!("Expected Commands::Config"),
    }

    let cli_list = Cli::try_parse_from(["cccp", "config", "-l"]).expect("should parse config list");
    match cli_list.command {
        Some(Commands::Config(c)) => {
            assert!(c.list);
            assert!(c.key.is_none());
        }
        _ => panic!("Expected Commands::Config"),
    }
}

#[test]
fn test_check_command_options() {
    let cli = Cli::try_parse_from(["cccp", "check", "--strict"]).expect("should parse check");
    match cli.command {
        Some(Commands::Check(a)) => {
            assert!(a.strict);
        }
        _ => panic!("Expected Commands::Check"),
    }
    assert!(Cli::try_parse_from(["cccp", "audit"]).is_err());
}

#[test]
fn test_soviet_comrades_flag() {
    let cli = Cli::try_parse_from(["cccp", "soviet", "--comrades"])
        .expect("should parse soviet comrades");
    match cli.command {
        Some(Commands::Soviet(s)) => {
            assert!(s.comrades);
        }
        _ => panic!("Expected Commands::Soviet"),
    }
    assert!(Cli::try_parse_from(["cccp", "comrades"]).is_err());
}

#[test]
fn test_completion_command_options() {
    let cli =
        Cli::try_parse_from(["cccp", "completion", "bash"]).expect("should parse completion bash");
    match cli.command {
        Some(Commands::Completion(c)) => {
            assert_eq!(c.shell, clap_complete::Shell::Bash);
        }
        _ => panic!("Expected Commands::Completion"),
    }
}

#[test]
fn test_repo_command_options() {
    // Repo Add
    let cli = Cli::try_parse_from([
        "cccp",
        "repo",
        "add",
        "codeberg.org/lumenpink/joaju",
        "-n",
        "joaju",
        "-d",
        "Microframework",
    ])
    .expect("should parse repo add");
    match cli.command {
        Some(Commands::Repo(c)) => match c.action {
            Some(RepoSubcommands::Add {
                repo,
                name,
                description,
                aliases,
            }) => {
                assert_eq!(repo, "codeberg.org/lumenpink/joaju");
                assert_eq!(name.as_deref(), Some("joaju"));
                assert_eq!(description.as_deref(), Some("Microframework"));
                assert!(aliases.is_empty());
            }
            _ => panic!("Expected RepoSubcommands::Add"),
        },
        _ => panic!("Expected Commands::Repo"),
    }

    // Repo AddNew
    let cli = Cli::try_parse_from(["cccp", "repo", "add-new", "--from-src", "/home/lumen/src"])
        .expect("should parse repo add-new");
    match cli.command {
        Some(Commands::Repo(c)) => match c.action {
            Some(RepoSubcommands::AddNew { from_src }) => {
                assert_eq!(from_src.as_deref(), Some("/home/lumen/src"));
            }
            _ => panic!("Expected RepoSubcommands::AddNew"),
        },
        _ => panic!("Expected Commands::Repo"),
    }

    // Repo List with filters
    let cli = Cli::try_parse_from([
        "cccp", "repo", "list", "-s", "clean", "-t", "5", "--sort", "tickets", "-r",
    ])
    .expect("should parse repo list");
    match cli.command {
        Some(Commands::Repo(c)) => match c.action {
            Some(RepoSubcommands::List(f)) => {
                assert_eq!(f.status.as_deref(), Some("clean"));
                assert_eq!(f.min_tickets, Some(5));
                assert_eq!(f.sort, "tickets");
                assert!(f.reverse);
            }
            _ => panic!("Expected RepoSubcommands::List"),
        },
        _ => panic!("Expected Commands::Repo"),
    };

    // Top-level 'catalog' is rejected (absorbed into 'repo')
    assert!(Cli::try_parse_from(["cccp", "catalog", "list"]).is_err());
}

#[test]
fn test_editor_command_removed() {
    assert!(Cli::try_parse_from(["cccp", "editor", "foo.txt"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "edit", "bar.txt"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "editor"]).is_err());
}

#[test]
fn test_init_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "repo",
        "init",
        "new-project",
        "--root",
        "/tmp/new-project",
        "--desc",
        "Sample project",
        "--lang",
        "rust,python",
        "--license",
        "Apache-2.0",
        "--profile",
        "profiles/custom.md",
        "--no-git",
        "--no-hooks",
        "--no-ci",
        "--no-sync",
        "-q",
    ])
    .expect("should parse repo init options");

    match cli.command {
        Some(Commands::Repo(r)) => match r.action {
            Some(RepoSubcommands::Init(i)) => {
                assert_eq!(i.name.as_deref(), Some("new-project"));
                assert_eq!(i.root, Some(PathBuf::from("/tmp/new-project")));
                assert_eq!(i.desc.as_deref(), Some("Sample project"));
                assert_eq!(i.lang, "rust,python");
                assert_eq!(i.license, "Apache-2.0");
                assert_eq!(i.profile, "profiles/custom.md");
                assert!(i.no_git);
                assert!(i.no_hooks);
                assert!(i.no_ci);
                assert!(i.no_sync);
                assert!(i.quiet);
            }
            _ => panic!("Expected RepoSubcommands::Init"),
        },
        _ => panic!("Expected Commands::Repo"),
    }

    assert!(Cli::try_parse_from(["cccp", "init"]).is_err());
}

#[test]
fn test_ci_command_removed() {
    assert!(Cli::try_parse_from(["cccp", "ci", "status"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "ci", "run"]).is_err());
    assert!(Cli::try_parse_from(["cccp", "ci", "log"]).is_err());
}

#[test]
fn test_deps_command_options() {
    let cli = Cli::try_parse_from(["cccp", "deps", "install", "--all", "act", "jq"])
        .expect("should parse deps install");
    match cli.command {
        Some(Commands::Deps(d)) => match d.action {
            Some(DepsSubcommands::Install { all, deps }) => {
                assert!(all);
                assert_eq!(deps, vec!["act", "jq"]);
            }
            _ => panic!("Expected DepsSubcommands::Install"),
        },
        _ => panic!("Expected Commands::Deps"),
    }
}

#[test]
fn test_agent_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "agent",
        "-u",
        "http://example.com/template",
        "update",
    ])
    .expect("should parse agent update");
    match cli.command {
        Some(Commands::Agent(a)) => {
            assert_eq!(a.url.as_deref(), Some("http://example.com/template"));
            assert!(matches!(a.action, Some(AgentSubcommands::Update)));
        }
        _ => panic!("Expected Commands::Agent"),
    }

    let cli =
        Cli::try_parse_from(["cccp", "agent", "get", "my_key"]).expect("should parse agent get");
    match cli.command {
        Some(Commands::Agent(a)) => match a.action {
            Some(AgentSubcommands::Get { key }) => {
                assert_eq!(key, "my_key");
            }
            _ => panic!("Expected AgentSubcommands::Get"),
        },
        _ => panic!("Expected Commands::Agent"),
    }

    let cli = Cli::try_parse_from(["cccp", "agent", "unset", "old_key"])
        .expect("should parse agent unset");
    match cli.command {
        Some(Commands::Agent(a)) => match a.action {
            Some(AgentSubcommands::Unset { key }) => {
                assert_eq!(key, "old_key");
            }
            _ => panic!("Expected AgentSubcommands::Unset"),
        },
        _ => panic!("Expected Commands::Agent"),
    }
}

#[test]
fn test_set_av_command_removed() {
    assert!(Cli::try_parse_from(["cccp", "set-av", "project_type", "cli"]).is_err());
}

#[test]
fn test_serve_command_removed() {
    assert!(Cli::try_parse_from(["cccp", "serve", "./site"]).is_err());
}

#[test]
fn test_spool_command_options() {
    let cli = Cli::try_parse_from(["cccp", "spool", "history", "-n", "100"])
        .expect("should parse spool history");
    match cli.command {
        Some(Commands::Spool(s)) => match s.action {
            Some(SpoolSubcommands::History { limit }) => {
                assert_eq!(limit, 100);
            }
            _ => panic!("Expected SpoolSubcommands::History"),
        },
        _ => panic!("Expected Commands::Spool"),
    }

    let cli = Cli::try_parse_from(["cccp", "spool", "status", "--json"])
        .expect("should parse spool status");
    match cli.command {
        Some(Commands::Spool(s)) => match s.action {
            Some(SpoolSubcommands::Status { json }) => {
                assert!(json);
            }
            _ => panic!("Expected SpoolSubcommands::Status"),
        },
        _ => panic!("Expected Commands::Spool"),
    }

    let cli = Cli::try_parse_from(["cccp", "spool", "clean", "-d", "14"])
        .expect("should parse spool clean");
    match cli.command {
        Some(Commands::Spool(s)) => match s.action {
            Some(SpoolSubcommands::Clean { days }) => {
                assert_eq!(days, 14);
            }
            _ => panic!("Expected SpoolSubcommands::Clean"),
        },
        _ => panic!("Expected Commands::Spool"),
    }

    let cli = Cli::try_parse_from(["cccp", "spool", "worker", "--daemon", "--job-id", "job-42"])
        .expect("should parse spool worker");
    match cli.command {
        Some(Commands::Spool(s)) => match s.action {
            Some(SpoolSubcommands::Worker(w)) => {
                assert!(w.daemon);
                assert!(!w.once);
                assert_eq!(w.job_id.as_deref(), Some("job-42"));
            }
            _ => panic!("Expected SpoolSubcommands::Worker"),
        },
        _ => panic!("Expected Commands::Spool"),
    }
}

#[test]
fn test_cockpit_command_options() {
    let cli = Cli::try_parse_from([
        "cccp",
        "cockpit",
        "tickets",
        "-p",
        "9000",
        "--host",
        "0.0.0.0",
        "--no-open",
        "-t",
        "secret-token",
        "--no-diva",
        "--no-status",
        "--no-tickets",
        "-q",
    ])
    .expect("should parse cockpit options");

    match cli.command {
        Some(Commands::Cockpit(c)) => {
            assert_eq!(c.tab.as_deref(), Some("tickets"));
            assert_eq!(c.port, Some(9000));
            assert_eq!(c.host.as_deref(), Some("0.0.0.0"));
            assert!(c.no_open);
            assert_eq!(c.token.as_deref(), Some("secret-token"));
            assert!(c.no_diva);
            assert!(c.no_status);
            assert!(c.no_tickets);
            assert!(c.quiet);
        }
        _ => panic!("Expected Commands::Cockpit"),
    }
}

// =============================================================================
// 3. Validation & Error Handling Tests
// =============================================================================

#[test]
fn test_invalid_subcommand_rejected() {
    let err = Cli::try_parse_from(["cccp", "invalid-command"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
}

#[test]
fn test_unknown_flag_rejected() {
    let err = Cli::try_parse_from(["cccp", "--nonexistent-flag"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::UnknownArgument);

    let err = Cli::try_parse_from(["cccp", "status", "--bogus"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::UnknownArgument);
}

#[test]
fn test_invalid_enum_variant_rejected() {
    // Invalid bump type in gen-version
    let err = Cli::try_parse_from(["cccp", "gen-version", "--bump", "invalid_level"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidValue);

    // Invalid format in gen-version
    let err = Cli::try_parse_from(["cccp", "gen-version", "--format", "unsupported"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidValue);

    // Invalid shell in completion
    let err = Cli::try_parse_from(["cccp", "completion", "nosuchshell"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidValue);
}

#[test]
fn test_missing_required_arguments_rejected() {
    // completion requires <SHELL>
    let err = Cli::try_parse_from(["cccp", "completion"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);

    // agent set requires <KEY> <VALUE>
    let err = Cli::try_parse_from(["cccp", "agent", "set", "only_key"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);

    // spool poll requires <ID>
    let err = Cli::try_parse_from(["cccp", "spool", "poll"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
}

#[test]
fn test_invalid_number_type_rejected() {
    let err = Cli::try_parse_from(["cccp", "cockpit", "-p", "invalid"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::ValueValidation);

    let err = Cli::try_parse_from(["cccp", "ticket", "-l", "not_a_number"]).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::ValueValidation);
}
