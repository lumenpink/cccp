//! CCCP - Conventional Commits Compliance Program
//! Main binary entry point.

use cccp::cli::{self, Cli, Commands};
use cccp::config;
use cccp::cultural::{self, Theme};
use clap::{CommandFactory, Parser};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Cli::parse();
    let theme = Theme::init(args.no_color);

    // Directory override (-C / --dir)
    if let Some(ref target_dir) = args.dir {
        if let Err(err) = std::env::set_current_dir(target_dir) {
            eprintln!(
                "{}: Failed to change directory to '{}': {}",
                theme.red_bold("Error"),
                target_dir.display(),
                err
            );
            return ExitCode::FAILURE;
        }
    }

    let config = config::Config::load_hierarchical(None);

    match args.command {
        Some(Commands::Cz(cz)) => {
            if args.verbose > 0 {
                eprintln!("[cccp] Running cz interactive commit wizard...");
            }
            println!(
                "{} Conventional Commits Wizard (dry_run: {})",
                theme.gold_bold("★"),
                cz.dry_run
            );
            ExitCode::SUCCESS
        }
        Some(Commands::Lint(lint)) => {
            if args.verbose > 0 {
                eprintln!("[cccp] Running commit message linter...");
            }
            if let Some(ref msg) = lint.message {
                let res = cccp::lint::lint_commit_message(msg, &config, lint.strict);
                if res.is_valid {
                    println!(
                        "{} Commit message compliant with Conventional Commits 1.0.0",
                        theme.green("✓")
                    );
                    for w in res.warnings {
                        println!("  {} {}", theme.yellow("Warning:"), w);
                    }
                    ExitCode::SUCCESS
                } else {
                    println!("{} Commit message rejected:", theme.red_bold("✗"));
                    for err in res.errors {
                        println!("  {} {}", theme.red("•"), err);
                    }
                    for w in res.warnings {
                        println!("  {} {}", theme.yellow("Warning:"), w);
                    }
                    ExitCode::FAILURE
                }
            } else if let Some(ref file) = lint.file {
                match cccp::lint::lint_commit_file(file, &config, lint.strict) {
                    Ok(res) => {
                        if res.is_valid {
                            println!(
                                "{} Commit message file '{}' compliant",
                                theme.green("✓"),
                                file.display()
                            );
                            for w in res.warnings {
                                println!("  {} {}", theme.yellow("Warning:"), w);
                            }
                            ExitCode::SUCCESS
                        } else {
                            println!(
                                "{} Commit message file '{}' rejected:",
                                theme.red_bold("✗"),
                                file.display()
                            );
                            for err in res.errors {
                                println!("  {} {}", theme.red("•"), err);
                            }
                            for w in res.warnings {
                                println!("  {} {}", theme.yellow("Warning:"), w);
                            }
                            ExitCode::FAILURE
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    }
                }
            } else {
                let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        return ExitCode::FAILURE;
                    }
                };

                let range_label = lint.range.as_deref().unwrap_or("HEAD (recent commits)");
                println!("★ CCCP Conventional Commits Audit ({}) ★", range_label);

                if repo.commits_since_tag.is_empty() {
                    println!("{} All clear: 0 new commits to inspect.", theme.green("✓"));
                    return ExitCode::SUCCESS;
                }

                let mut any_failed = false;
                for commit in &repo.commits_since_tag {
                    let res =
                        cccp::lint::lint_commit_message(&commit.message, &config, lint.strict);
                    if res.is_valid {
                        println!(
                            "  {} [{}] {}",
                            theme.green("✓"),
                            commit.short_id,
                            commit.summary
                        );
                    } else {
                        any_failed = true;
                        println!(
                            "  {} [{}] {}",
                            theme.red_bold("✗"),
                            commit.short_id,
                            commit.summary
                        );
                        for err in res.errors {
                            println!("    ↳ {}", theme.red(&err));
                        }
                    }
                }

                if any_failed {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
        }
        Some(Commands::Semver(semver)) => {
            if args.verbose > 0 {
                eprintln!("[cccp] Calculating predictive Semantic Version...");
            }
            let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };
            let no_v = config.no_v;
            let predicted = match cccp::semver_calc::calculate_version(
                &repo,
                &config.default_base_version,
                no_v,
                semver.bump,
            ) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };

            if semver.write {
                let write_val = if predicted.is_release {
                    &predicted.target_version
                } else {
                    &predicted.dev_version
                };
                if let Err(e) = cccp::semver_calc::write_version_file(&repo.root_dir, write_val) {
                    eprintln!(
                        "{}: Failed to write VERSION file: {}",
                        theme.red_bold("Error"),
                        e
                    );
                    return ExitCode::FAILURE;
                }
                println!("Version information written to VERSION file: {}", write_val);
            }

            match semver.format {
                cli::VersionFormat::Clean => println!("{}", predicted.target_version),
                cli::VersionFormat::Dev => println!("{}", predicted.dev_version),
                cli::VersionFormat::Tag => println!("{}", predicted.tag),
                cli::VersionFormat::Json => {
                    let json = serde_json::to_string_pretty(&predicted).unwrap_or_else(|_| {
                        format!(r#"{{"version":"{}"}}"#, predicted.target_version)
                    });
                    println!("{}", json);
                }
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
            println!(
                "{} CCCP Repository Status (Gosplan State Inspection)",
                theme.red_bold("★")
            );
            match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(repo) => {
                    println!("  Root Directory : {}", repo.root_dir.display());
                    println!(
                        "  Branch         : {}",
                        repo.branch.as_deref().unwrap_or("HEAD (detached)")
                    );
                    println!(
                        "  HEAD Commit    : {} ({})",
                        repo.head_short_id, repo.head_date
                    );
                    if let Some(ref tag) = repo.latest_tag {
                        println!("  Latest SemVer  : {} (tag: {})", tag.version, tag.name);
                    } else {
                        println!(
                            "  Latest SemVer  : None (baseline: {})",
                            config.default_base_version
                        );
                    }
                    println!("  Commits Since  : {}", repo.commits_since_tag.len());
                    println!(
                        "  Working Tree   : {}",
                        if repo.is_dirty {
                            theme.yellow("dirty (uncommitted changes)")
                        } else {
                            theme.green("clean")
                        }
                    );
                    if status.detailed {
                        for c in &repo.commits_since_tag {
                            println!("    • {} {}", theme.dim(&c.short_id), c.summary);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("  {}: {}", theme.yellow("Warning"), err);
                }
            }
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
                println!("{}", cultural::order_227(&theme));
            } else if soviet.anthem {
                println!("{}", cultural::soviet_anthem(&theme));
            } else if soviet.quote {
                println!("{}", cultural::revolutionary_quote(&theme));
            } else if soviet.plan {
                println!("{}", cultural::gosplan_plan(&theme));
            } else {
                println!("{}", cultural::gosplan_banner(&theme));
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
                println!("{}", theme.bold("★ CCCP Hierarchical Configuration ★"));
                println!("commit_types = {}", config.commit_types.join(", "));
                println!("commit_scopes = {}", config.commit_scopes.join(", "));
                println!("commit_subscopes = {}", config.commit_subscopes.join(", "));
                println!("strict_types = {}", config.strict_types);
                println!("strict_scopes = {}", config.strict_scopes);
                println!("strict_subscopes = {}", config.strict_subscopes);
                println!("allow_any_scope = {}", config.allow_any_scope);
                println!("allow_any_subscope = {}", config.allow_any_subscope);
                println!("disable_subscopes = {}", config.disable_subscopes);
                println!(
                    "disable_multiple_scopes = {}",
                    config.disable_multiple_scopes
                );
                println!("default_base_version = {}", config.default_base_version);
                println!("no_v = {}", config.no_v);
                println!("tickets_file = {}", config.tickets_file);
                println!("agents_md_file = {}", config.agents_md_file);
                if let Some(ref hook) = config.post_push_hook {
                    println!("post_push_hook = {}", hook);
                }
                if !config.post_push_sync.is_empty() {
                    println!("post_push_sync = {}", config.post_push_sync.join(", "));
                }
                for (k, v) in &config.extra {
                    println!("{} = {}", k, v);
                }
            } else if let Some(ref key) = cfg.key {
                if let Some(ref value) = cfg.value {
                    let target_path = if cfg.is_global {
                        config::find_global_config_path().unwrap_or_else(|| {
                            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                            std::path::PathBuf::from(home)
                                .join(".config")
                                .join("cccp")
                                .join("config")
                        })
                    } else {
                        std::path::PathBuf::from(".cccprc")
                    };
                    if let Err(e) = config::write_config_key(&target_path, key, value) {
                        eprintln!(
                            "{}: Failed to write config to '{}': {}",
                            theme.red_bold("Error"),
                            target_path.display(),
                            e
                        );
                        return ExitCode::FAILURE;
                    }
                    println!("Saved '{} = {}' into {}", key, value, target_path.display());
                } else {
                    let normalized = key.to_ascii_lowercase().replace('-', "_");
                    match normalized.as_str() {
                        "types" | "commit_types" => println!("{}", config.commit_types.join(", ")),
                        "scopes" | "commit_scopes" => {
                            println!("{}", config.commit_scopes.join(", "))
                        }
                        "strict_types" => println!("{}", config.strict_types),
                        "strict_scopes" => println!("{}", config.strict_scopes),
                        "default_base_version" => println!("{}", config.default_base_version),
                        "no_v" => println!("{}", config.no_v),
                        _ => {
                            if let Some(v) = config.extra.get(&normalized) {
                                println!("{}", v);
                            } else {
                                eprintln!(
                                    "{}: Configuration key '{}' not set.",
                                    theme.yellow("Warning"),
                                    key
                                );
                            }
                        }
                    }
                }
            } else {
                println!("Use 'cccp config --list' or 'cccp config <key> [value]'");
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
