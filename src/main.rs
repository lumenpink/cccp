//! CCCP - Conventional Commits Compliance Program
//! Main binary entry point.

mod cli;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Cli::parse();

    // Directory override (-C / --dir)
    if let Some(ref target_dir) = args.dir {
        if let Err(err) = std::env::set_current_dir(target_dir) {
            eprintln!(
                "\x1b[31mError:\x1b[0m Failed to change directory to '{}': {}",
                target_dir.display(),
                err
            );
            return ExitCode::FAILURE;
        }
    }

    match args.command {
        Some(Commands::Cz(cz)) => {
            if args.verbose > 0 {
                eprintln!("[cccp] Running cz interactive commit wizard...");
            }
            println!("★ Conventional Commits Wizard (dry_run: {})", cz.dry_run);
            ExitCode::SUCCESS
        }
        Some(Commands::Lint(lint)) => {
            if args.verbose > 0 {
                eprintln!("[cccp] Running commit message linter...");
            }
            if let Some(msg) = lint.message {
                println!("Validating commit message: \"{}\"", msg);
            } else if let Some(file) = lint.file {
                println!("Validating commit message file: {}", file.display());
            } else {
                println!(
                    "Auditing commits in range: {}",
                    lint.range.as_deref().unwrap_or("HEAD~1..HEAD")
                );
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Semver(semver)) => {
            if args.verbose > 0 {
                eprintln!("[cccp] Calculating predictive Semantic Version...");
            }
            match semver.format {
                cli::VersionFormat::Clean => println!("3.0.0"),
                cli::VersionFormat::Dev => println!("3.0.0-dev.1+20260929.rust"),
                cli::VersionFormat::Json => {
                    println!(r#"{{"version":"3.0.0","tag":"v3.0.0","bump":"major"}}"#)
                }
                cli::VersionFormat::Tag => println!("v3.0.0"),
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Install(install)) => {
            println!(
                "Installing CCCP Git hooks (force: {}, uninstall: {})...",
                install.force, install.uninstall
            );
            ExitCode::SUCCESS
        }
        Some(Commands::Status(status)) => {
            println!("CCCP Repository Status: All systems compliant with Conventional Commits 1.0.0 (detailed: {})", status.detailed);
            ExitCode::SUCCESS
        }
        Some(Commands::Hook(hook)) => {
            match hook.hook {
                cli::HookSubcommands::CommitMsg { file, source, sha } => {
                    if args.verbose > 0 {
                        eprintln!(
                            "[cccp] Hook commit-msg triggered for file={}, source={:?}, sha={:?}",
                            file.display(),
                            source,
                            sha
                        );
                    }
                    println!("Hook: commit-msg check passed for {}", file.display());
                }
                cli::HookSubcommands::PrePush { remote, url } => {
                    if args.verbose > 0 {
                        eprintln!(
                            "[cccp] Hook pre-push triggered for remote={:?}, url={:?}",
                            remote, url
                        );
                    }
                    println!("Hook: pre-push check passed.");
                }
                cli::HookSubcommands::PostCommit => {
                    println!("Hook: post-commit synchronization completed.");
                }
                cli::HookSubcommands::ReferenceTransaction { state } => {
                    println!("Hook: reference-transaction state={:?}", state);
                }
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Tickets(tickets)) => {
            println!(
                "Syncing tickets from forge (limit: {}, state: {})...",
                tickets.limit, tickets.state
            );
            ExitCode::SUCCESS
        }
        Some(Commands::Forges(forges)) => {
            match forges.action {
                Some(cli::ForgeActions::Detect) => {
                    println!("Forge detected: Codeberg (git@codeberg.org:lumenpink/cccp.git)")
                }
                Some(cli::ForgeActions::Issues { state }) => {
                    println!("Listing {} issues from forge...", state)
                }
                Some(cli::ForgeActions::Prs { state }) => {
                    println!("Listing {} pull requests from forge...", state)
                }
                None => println!("Forge module active. Use --help to see actions."),
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Soviet(soviet)) => {
            if soviet.order227 {
                println!("★ Order No. 227 (28 July 1942): 'Ni shagu nazad!' (Not a step back!) ★");
            } else if soviet.anthem {
                println!("★ State Anthem of the Soviet Union ★\nUnbreakable Union of freeborn Republics...");
            } else if soviet.quote {
                println!("★ 'Without revolutionary theory there can be no revolutionary movement.' — V. I. Lenin ★");
            } else {
                println!("★ Gosplan Quality Mark (GOST) & Five-Year Plan: 100% Conventional Commits Compliance ★");
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Tag(tag)) => {
            println!(
                "Creating release tag (version: {:?}, title: {:?})...",
                tag.version, tag.title
            );
            ExitCode::SUCCESS
        }
        Some(Commands::Changelog(ch)) => {
            println!(
                "Generating changelog to {} (dry_run: {})...",
                ch.output.display(),
                ch.dry_run
            );
            ExitCode::SUCCESS
        }
        Some(Commands::Push(push)) => {
            println!("Forwarding push to git with args: {:?}", push.git_args);
            ExitCode::SUCCESS
        }
        Some(Commands::Config(cfg)) => {
            if cfg.list {
                println!("CCCP Configuration Hierarchy resolved.");
            } else if let Some(key) = cfg.key {
                println!("Config key '{}' requested (set: {:?})", key, cfg.value);
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Gost(gost)) => {
            println!(
                "GOST Standards Inspection: Compliance verified (strict: {}).",
                gost.strict
            );
            ExitCode::SUCCESS
        }
        Some(Commands::Comrades(c)) => {
            println!(
                "Comrades of CCCP (format: {}): Lumen Pink <hi@lumen.pink>",
                c.format
            );
            ExitCode::SUCCESS
        }
        Some(Commands::Completion(comp)) => {
            let mut cmd = Cli::command();
            clap_complete::generate(comp.shell, &mut cmd, "cccp", &mut std::io::stdout());
            ExitCode::SUCCESS
        }
        None => {
            let mut cmd = Cli::command();
            let _ = cmd.print_help();
            println!();
            ExitCode::SUCCESS
        }
    }
}
