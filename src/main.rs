//! CCCP - Conventional Commits Compliance Program
//! Main binary entry point.

use cccp::cli::{self, Cli, Commands};
use cccp::config;
use cccp::cultural::{self, Theme};
use clap::{CommandFactory, Parser};
use std::io::{IsTerminal, Write};
use std::process::{Command, ExitCode};

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
    cccp::i18n::init(args.lang.as_deref(), &config);

    if let Ok(current_dir) = std::env::current_dir() {
        let _ = cccp::update::check_hook_version(&current_dir);
    }
    let _ = cccp::update::check_auto_update(&config);

    match args.command {
        Some(Commands::Commit(cz)) => {
            if args.verbose > 0 {
                eprintln!("[cccp] Running interactive commit wizard...");
            }
            let composed = match cccp::cz::compose_commit(&cz, &config, &theme) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), err);
                    return ExitCode::FAILURE;
                }
            };

            let commit_opts = cccp::commit_exec::CommitOptions {
                all: cz.all,
                sign: cz.sign,
                no_sign: cz.no_sign,
                dry_run: cz.dry_run,
                dir: args.dir.clone(),
            };

            match cccp::commit_exec::execute_commit(
                &composed.formatted_message,
                &commit_opts,
                &theme,
            ) {
                Ok(0) => ExitCode::SUCCESS,
                Ok(code) => ExitCode::from(code as u8),
                Err(err) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), err);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Lint(lint)) => {
            if args.verbose > 0 {
                eprintln!("[cccp] Running commit message linter...");
            }
            if let Some(ref msg) = lint.message {
                let res = cccp::lint::lint_commit_message(msg, &config, lint.strict);
                if res.is_valid {
                    println!(
                        "{} Commit message compliant with Conventional Commits 1.0.0 (Party Line Compliant)",
                        theme.green("✓")
                    );
                    for w in res.warnings {
                        println!("  {} {}", theme.yellow("Warning:"), w);
                    }
                    ExitCode::SUCCESS
                } else {
                    println!(
                        "{} Commit message rejected (Ideological Deviation):",
                        theme.red_bold("✗")
                    );
                    for err in res.errors {
                        println!("  {} {}", theme.red("•"), err);
                    }
                    for w in res.warnings {
                        println!("  {} {}", theme.yellow("Warning:"), w);
                    }
                    println!(
                        "{}",
                        theme.red_bold(
                            "Directive: Order No. 227 - Not One Step Back from Clean Commits!"
                        )
                    );
                    ExitCode::FAILURE
                }
            } else if let Some(ref file) = lint.file {
                match cccp::lint::lint_commit_file(file, &config, lint.strict) {
                    Ok(res) => {
                        if res.is_valid {
                            println!(
                                "{} Commit message file '{}' compliant with Conventional Commits 1.0.0 (Party Line Compliant)",
                                theme.green("✓"),
                                file.display()
                            );
                            for w in res.warnings {
                                println!("  {} {}", theme.yellow("Warning:"), w);
                            }
                            ExitCode::SUCCESS
                        } else {
                            println!(
                                "{} Commit message file '{}' rejected (Ideological Deviation):",
                                theme.red_bold("✗"),
                                file.display()
                            );
                            for err in res.errors {
                                println!("  {} {}", theme.red("•"), err);
                            }
                            for w in res.warnings {
                                println!("  {} {}", theme.yellow("Warning:"), w);
                            }
                            println!(
                                "{}",
                                theme.red_bold(
                                    "Directive: Order No. 227 - Not One Step Back from Clean Commits!"
                                )
                            );
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
                println!(
                    "{}",
                    theme.bold("========================================================")
                );
                println!(
                    "{}",
                    theme.bold(" ★ CCCP Party Line Compliance (Commit Linting) ★")
                );
                println!(
                    "{}",
                    theme.bold("========================================================")
                );
                println!("Inspected Range: {}", range_label);

                if repo.commits_since_tag.is_empty() {
                    println!("Result: 0 commits found in range to inspect.");
                    println!("{} Status: All clear for the State Plan.", theme.green("✓"));
                    println!(
                        "{}",
                        theme.bold("========================================================")
                    );
                    return ExitCode::SUCCESS;
                }

                println!(
                    "{}",
                    theme.dim("────────────────────────────────────────────────────────")
                );

                let mut passed_count = 0;
                let mut failed_count = 0;

                for commit in &repo.commits_since_tag {
                    let res =
                        cccp::lint::lint_commit_message(&commit.message, &config, lint.strict);
                    if res.is_valid {
                        passed_count += 1;
                        println!(
                            "  {} [{}] {}",
                            theme.green("✓"),
                            commit.short_id,
                            commit.summary
                        );
                    } else {
                        failed_count += 1;
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

                println!(
                    "{}",
                    theme.bold("========================================================")
                );
                println!(
                    "Inspection Summary: {} inspected, {} compliant, {} ideological deviations.",
                    repo.commits_since_tag.len(),
                    theme.green(&passed_count.to_string()),
                    if failed_count > 0 {
                        theme.red_bold(&failed_count.to_string())
                    } else {
                        theme.green("0")
                    }
                );

                if failed_count > 0 {
                    println!(
                        "{}",
                        theme.red_bold(
                            "Directive: Order No. 227 - Not One Step Back from Clean Commits!"
                        )
                    );
                    println!(
                        "{}",
                        theme.red(
                            "FAILED: Non-compliant commits detected. Correct history before merge."
                        )
                    );
                    println!(
                        "{}",
                        theme.bold("========================================================")
                    );
                    ExitCode::FAILURE
                } else {
                    println!(
                        "{}",
                        theme.green("PASSED: Full ideological compliance verified. Glory to the collective!")
                    );
                    println!(
                        "{}",
                        theme.bold("========================================================")
                    );
                    ExitCode::SUCCESS
                }
            }
        }
        Some(Commands::Version(semver)) => {
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
        Some(Commands::Hook(hook)) => {
            if hook.install || hook.uninstall {
                let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        return ExitCode::FAILURE;
                    }
                };
                let install_opts = cli::InstallArgs {
                    force: hook.force,
                    hook_dir: hook.hook_dir.clone(),
                    uninstall: hook.uninstall,
                };
                if let Err(e) = cccp::hooks_install::handle_install(&install_opts, &repo, &theme) {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            } else if let Some(ref sub) = hook.hook {
                match sub {
                    cli::HookSubcommands::CommitMsg {
                        file,
                        source: _,
                        sha: _,
                    } => {
                        match cccp::hook_runner::run_commit_msg(file, &config, &theme, args.verbose)
                        {
                            Ok(0) => ExitCode::SUCCESS,
                            Ok(code) => ExitCode::from(code as u8),
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                ExitCode::FAILURE
                            }
                        }
                    }
                    cli::HookSubcommands::PrePush { remote, url } => {
                        let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                            Ok(r) => r,
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                return ExitCode::FAILURE;
                            }
                        };
                        match cccp::hook_runner::run_pre_push(
                            remote.as_deref(),
                            url.as_deref(),
                            None,
                            &repo,
                            &config,
                            &theme,
                            args.verbose,
                        ) {
                            Ok(0) => ExitCode::SUCCESS,
                            Ok(code) => ExitCode::from(code as u8),
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                ExitCode::FAILURE
                            }
                        }
                    }
                    cli::HookSubcommands::PostCommit => {
                        match cccp::hook_runner::run_post_commit(&theme, args.verbose) {
                            Ok(0) => ExitCode::SUCCESS,
                            Ok(code) => ExitCode::from(code as u8),
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                ExitCode::FAILURE
                            }
                        }
                    }
                    cli::HookSubcommands::ReferenceTransaction { state } => {
                        match cccp::hook_runner::run_reference_transaction(
                            state.as_deref(),
                            &theme,
                            args.verbose,
                        ) {
                            Ok(0) => ExitCode::SUCCESS,
                            Ok(code) => ExitCode::from(code as u8),
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                ExitCode::FAILURE
                            }
                        }
                    }
                    cli::HookSubcommands::PostMerge { is_squash } => {
                        let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                            Ok(r) => r,
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                return ExitCode::FAILURE;
                            }
                        };
                        match cccp::hook_runner::run_post_merge(
                            is_squash.as_deref(),
                            &repo,
                            &theme,
                            args.verbose,
                        ) {
                            Ok(0) => ExitCode::SUCCESS,
                            Ok(code) => ExitCode::from(code as u8),
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                ExitCode::FAILURE
                            }
                        }
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
                let install_opts = cli::InstallArgs {
                    force: hook.force,
                    hook_dir: hook.hook_dir.clone(),
                    uninstall: false,
                };
                if let Err(e) = cccp::hooks_install::handle_install(&install_opts, &repo, &theme) {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
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
        Some(Commands::Ticket(tickets)) => {
            let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };

            if let Some(cli::TicketAction::Close {
                ref ids,
                ref comment,
            }) = tickets.action
            {
                if let Err(e) = cccp::tickets::close_tickets(&repo, ids, comment.as_deref(), &theme)
                {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            } else if let Err(e) = cccp::tickets::sync_tickets(&tickets, &repo, &config, &theme) {
                eprintln!("{}: {}", theme.red_bold("Error"), e);
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Some(Commands::Sync(sync_args)) => {
            let sync_all = !sync_args.tickets && !sync_args.agents && !sync_args.catalog;
            let start_path = sync_args.dir.as_deref().or(args.dir.as_deref());
            let repo = cccp::git::GitRepo::open(start_path).ok();
            let repo_root = repo.as_ref().map(|r| r.root_dir.as_path()).or(start_path);

            let mut success = true;

            // 1. Sync tickets
            if sync_all || sync_args.tickets {
                if let Some(ref r) = repo {
                    println!("{} Synchronizing forge tickets...", theme.cyan("★"));
                    let tickets_opts = cli::TicketArgs {
                        remote: None,
                        output: None,
                        limit: 50,
                        state: "open".to_string(),
                        action: None,
                    };
                    if let Err(e) = cccp::tickets::sync_tickets(&tickets_opts, r, &config, &theme) {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        success = false;
                    }
                } else {
                    eprintln!(
                        "{}: Cannot sync tickets outside a git repository.",
                        theme.yellow("Warning")
                    );
                }
            }

            // 2. Sync GOST cascading profile and specs (.gost/GOST.md)
            if let Some(root) = repo_root {
                if root.join("GOSTVER").is_file() {
                    println!(
                        "{} Assembling cascading GOST specification (.gost/GOST.md)...",
                        theme.cyan("★")
                    );
                    if let Err(e) = cccp::profile::assemble_gost_specs(root) {
                        eprintln!(
                            "{}: Failed to assemble GOST specs: {}",
                            theme.yellow("Warning"),
                            e
                        );
                    }
                }
            }

            // 3. Sync agents (AGENTS.md)
            if sync_all || sync_args.agents {
                println!("{} Synchronizing AGENTS.md directives...", theme.cyan("★"));
                if let Err(e) = cccp::agents::cmd_agent_update(None, None, repo_root, &theme) {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    success = false;
                }
            }

            // 3. Sync catalog (~/src)
            if sync_all || sync_args.catalog {
                println!("{} Synchronizing repository catalog...", theme.cyan("★"));
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let src_dir = std::path::Path::new(&home).join("src");
                match cccp::catalog::scan_and_add_from_src(&src_dir, Some(&config)) {
                    Ok(res) => {
                        println!(
                            "  Catalog updated: {} added, {} existing.",
                            res.added.len(),
                            res.skipped_existing.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        success = false;
                    }
                }
            }

            if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some(Commands::Divan(diva)) => {
            let start_path = args.dir.as_deref().map(std::path::Path::new);

            // 1. Determine target repo and prompt from arguments
            let (target_repo_arg, mut prompt_arg) = diva.resolve_repo_and_prompt();

            // 2. Resolve target forge and repository
            let git_repo = cccp::git::GitRepo::open(args.dir.as_deref()).ok();
            let mut detected_forge = if let Some(ref target) = target_repo_arg {
                match cccp::catalog::resolve_target_repo(target, Some(&config), start_path) {
                    Some(info) => Some(info),
                    None => {
                        eprintln!(
                            "{}: Target repository '{}' not found in catalog or workspaces.",
                            theme.red_bold("Error"),
                            target
                        );
                        return ExitCode::FAILURE;
                    }
                }
            } else if let Some(ref r) = git_repo {
                cccp::forges::detect_forge_from_repo(r).ok()
            } else {
                None
            };

            // 3. If no forge detected yet, pick from catalog interactively
            if detected_forge.is_none() {
                let catalog_entries = cccp::catalog::load_catalog(Some(&config), start_path);
                if !catalog_entries.is_empty() && std::io::stdin().is_terminal() {
                    println!(
                        "{} No git repository detected in current directory.",
                        theme.yellow("★ [GOSPLAN]")
                    );
                    println!("Select target repository from your catalog:");
                    match inquire::Select::new("Target repository:", catalog_entries).prompt() {
                        Ok(selected) => {
                            detected_forge = Some(selected.to_forge_info());
                        }
                        Err(_) => {
                            eprintln!("{}: Operation cancelled.", theme.yellow("Notice"));
                            return ExitCode::SUCCESS;
                        }
                    }
                }
            }

            // 4. Fail-fast forge token check
            if !diva.dry_run {
                if let Some(ref f) = detected_forge {
                    if cccp::forges::resolve_token_for_host(&f.host, f.provider, Some(&config))
                        .is_none()
                    {
                        let domain_var = cccp::forges::host_to_token_env_var(&f.host);
                        eprintln!(
                            "{}: No authentication token found for forge host '{}' ({}/{}).",
                            theme.red_bold("Fail Fast"),
                            f.host,
                            f.owner,
                            f.repo
                        );
                        eprintln!(
                            "  Please configure {} in ~/.env or .cccprc.",
                            theme.yellow(&domain_var)
                        );
                        return ExitCode::FAILURE;
                    }
                }
            }

            // 5. Fail-fast AI provider and connectivity check
            let mut client = cccp::ai::AiClient::from_config(&config);
            if diva.ollama {
                client = client.with_provider(cccp::ai::AiProvider::Ollama);
                if diva.model.is_none() {
                    client = client.with_model(&config.ollama_model);
                }
            } else if diva.gemini {
                client = client.with_provider(cccp::ai::AiProvider::Gemini);
                if diva.model.is_none() {
                    client = client.with_model(&config.gemini_model);
                }
            } else if let Some(ref p_str) = diva.provider {
                match p_str.parse::<cccp::ai::AiProvider>() {
                    Ok(p) => {
                        client = client.with_provider(p);
                        if diva.model.is_none() {
                            let default_model = match p {
                                cccp::ai::AiProvider::Ollama => &config.ollama_model,
                                cccp::ai::AiProvider::Gemini => &config.gemini_model,
                            };
                            client = client.with_model(default_model);
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        return ExitCode::FAILURE;
                    }
                }
            }
            if let Some(ref m) = diva.model {
                client = client.with_model(m);
            }
            if let Some(ref h) = diva.host {
                client = client.with_host(h);
            }
            if let Some(t) = diva.temperature {
                client = client.with_temperature(t);
            }

            if !diva.direct {
                if let Err(e) = client.check_access() {
                    eprintln!("{}: {}", theme.red_bold("Fail Fast"), e);
                    return ExitCode::FAILURE;
                }

                if client.provider == cccp::ai::AiProvider::Ollama {
                    match client.resolve_ollama_model(diva.model.is_some()) {
                        Ok(resolved) => {
                            if resolved != client.model && diva.model.is_none() {
                                println!(
                                    "{} Default model '{}' not found. Using installed model '{}'...",
                                    theme.gold_bold("★ [GOSPLAN]"),
                                    client.model,
                                    resolved
                                );
                            }
                            client = client.with_model(resolved);
                        }
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Fail Fast"), e);
                            return ExitCode::FAILURE;
                        }
                    }
                }
            }

            // 6. Resolve prompt (interactive if empty)
            let prompt = if let Some(p) = prompt_arg.take() {
                p
            } else if std::io::stdin().is_terminal() {
                let target_label = if let Some(ref f) = detected_forge {
                    format!("{} ({}/{})", f.provider, f.owner, f.repo)
                } else {
                    "standalone".to_string()
                };
                let initial_prompt = format!(
                    "# The Divan — Target: {}\n# Describe the issue, bug, or feature request below.\n# Lines starting with '#' will be ignored.\n# Press Ctrl+S (or F2) to submit to AI triage, or Ctrl+Q (Esc) to cancel.\n\n",
                    target_label
                );
                match cccp::editor::edit_text("Divan - Triage Prompt", &initial_prompt) {
                    Ok(Some(edited)) => {
                        let filtered: Vec<&str> = edited
                            .lines()
                            .filter(|line| !line.trim().starts_with('#'))
                            .collect();
                        let prompt_text = filtered.join("\n").trim().to_string();
                        if prompt_text.is_empty() {
                            eprintln!(
                                "{}: No description provided. Exiting.",
                                theme.yellow("Notice")
                            );
                            return ExitCode::SUCCESS;
                        }
                        prompt_text
                    }
                    _ => {
                        eprintln!(
                            "{}: No description provided. Exiting.",
                            theme.yellow("Notice")
                        );
                        return ExitCode::SUCCESS;
                    }
                }
            } else {
                let mut line = String::new();
                std::io::stdin().read_line(&mut line).unwrap_or_default();
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    eprintln!(
                        "{}: Please provide a description or problem report for triage.",
                        theme.red_bold("Error")
                    );
                    eprintln!("Usage: cccp diva [OPTIONS] [REPO] [REPORT]");
                    return ExitCode::FAILURE;
                }
                trimmed
            };

            let repo_ctx = cccp::ai::build_enriched_context_with_provider(
                git_repo.as_ref(),
                detected_forge.as_ref(),
                &config,
                Some(client.provider),
            );

            let mut tickets = if diva.direct {
                println!(
                    "{}",
                    theme.yellow(
                        "★ [GOSPLAN] Direct mode enabled (--direct): skipping AI triage stage."
                    )
                );
                vec![cccp::ai::DecomposedTicket {
                    title: prompt.clone(),
                    rationale: Some("Direct mode - created without AI triage".to_string()),
                    labels: vec!["triage".to_string()],
                    body: format!("## Description\n{}\n", prompt),
                }]
            } else {
                println!(
                    "{} Consulting {} ({}) for ticket decomposition...",
                    theme.yellow("★ [GOSPLAN]"),
                    theme.cyan(&client.provider.to_string()),
                    theme.yellow(&client.model)
                );
                match client.triage(&prompt, Some(&repo_ctx)) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        return ExitCode::FAILURE;
                    }
                }
            };

            if tickets.is_empty() {
                println!(
                    "{} No tickets were identified by the AI model.",
                    theme.yellow("★ [GOSPLAN]")
                );
                return ExitCode::SUCCESS;
            }

            println!();
            println!(
                "{}",
                theme.bold(&format!(
                    "★ [GOSPLAN] The Divan identified {} ticket(s) for review ★",
                    tickets.len()
                ))
            );

            let forge_client = cccp::forges::ForgeClient::new();
            let mut created_count = 0;
            let mut idx = 0;

            while idx < tickets.len() {
                let dest_str = detected_forge
                    .as_ref()
                    .map(|f| format!("{} ({}/{})", f.provider, f.owner, f.repo));
                print!(
                    "{}",
                    cccp::ai::render_ticket_card(
                        &tickets[idx],
                        idx,
                        tickets.len(),
                        dest_str.as_deref(),
                        &theme
                    )
                );

                let publish_label = if let Some(ref f) = detected_forge {
                    format!("Publish to {} ({}/{})", f.provider, f.owner, f.repo)
                } else {
                    "Accept".to_string()
                };

                print!(
                    "  [y] {}  [n] Skip  [t] Edit title  [e] Edit body  [r] Refine with AI  [v] View body  [q] Quit → ",
                    publish_label
                );
                let _ = std::io::stdout().flush();

                let key = cccp::ai::read_single_key().unwrap_or('n');
                match key {
                    'y' => {
                        if diva.dry_run {
                            println!(
                                "\n  {}",
                                theme.dim("[DRY-RUN] Simulation active. Ticket was not published.")
                            );
                            idx += 1;
                            continue;
                        }

                        if let Some(ref forge_info) = detected_forge {
                            println!(
                                "\n  -> Publishing ticket to {} ({}/{})...",
                                forge_info.provider, forge_info.owner, forge_info.repo
                            );
                            match forge_client.create_issue(
                                forge_info,
                                &tickets[idx].title,
                                &tickets[idx].body,
                                &tickets[idx].labels,
                            ) {
                                Ok(res) => {
                                    println!(
                                        "  {} Ticket created: {}",
                                        theme.green("✓"),
                                        res.html_url
                                    );
                                    let branch = cccp::forges::suggest_branch_name(
                                        res.number,
                                        &tickets[idx].title,
                                    );
                                    println!("     Suggested branch: {}", theme.cyan(&branch));
                                    created_count += 1;
                                    idx += 1;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "\n  {}: {}",
                                        theme.red_bold("Error publishing ticket"),
                                        e
                                    );
                                    eprintln!(
                                        "  You may edit title/body [t/e], retry [y], or skip [n]."
                                    );
                                }
                            }
                        } else {
                            println!(
                                "\n  {}: No git remote or forge detected to publish the ticket.",
                                theme.yellow("Warning")
                            );
                            idx += 1;
                        }
                    }
                    'n' => {
                        println!("\n  Skipped.");
                        idx += 1;
                    }
                    't' => {
                        println!("\n  Launching text editor for title...");
                        match cccp::ai::open_text_editor(&tickets[idx].title, "diva-title") {
                            Ok(new_title) => {
                                let trimmed = new_title.trim();
                                if !trimmed.is_empty() {
                                    tickets[idx].title = trimmed.to_string();
                                    println!("  {} Title updated.", theme.green("✓"));
                                }
                            }
                            Err(e) => {
                                eprintln!("  {}: {}", theme.red_bold("Editor error"), e);
                                if let Ok(inline) = inquire::Text::new("Edit title:")
                                    .with_initial_value(&tickets[idx].title)
                                    .prompt()
                                {
                                    let trimmed = inline.trim();
                                    if !trimmed.is_empty() {
                                        tickets[idx].title = trimmed.to_string();
                                        println!("  {} Title updated.", theme.green("✓"));
                                    }
                                }
                            }
                        }
                    }
                    'e' | 'b' => {
                        println!("\n  Launching text editor for ticket body...");
                        match cccp::ai::open_text_editor(&tickets[idx].body, "diva-body") {
                            Ok(new_body) => {
                                let trimmed = new_body.trim();
                                if !trimmed.is_empty() {
                                    tickets[idx].body = new_body;
                                    println!("  {} Body updated.", theme.green("✓"));
                                }
                            }
                            Err(e) => {
                                eprintln!("  {}: {}", theme.red_bold("Editor error"), e);
                            }
                        }
                    }
                    'v' | 'f' => {
                        println!("\n--- FULL TICKET BODY ---");
                        println!("{}", tickets[idx].body);
                        println!("------------------------");
                    }
                    'r' => {
                        println!();
                        print!("  Enter refinement instructions for AI: ");
                        let _ = std::io::stdout().flush();
                        let mut ref_input = String::new();
                        if std::io::stdin().read_line(&mut ref_input).is_ok()
                            && !ref_input.trim().is_empty()
                        {
                            let refine_prompt = format!(
                                "Original user report:\n{}\n\nCurrent proposed ticket:\nTitle: {}\nBody:\n{}\n\nRefinement request:\n{}",
                                prompt,
                                tickets[idx].title,
                                tickets[idx].body,
                                ref_input.trim()
                            );
                            println!(
                                "  {} Refining tickets with {} ({})...",
                                theme.yellow("★ [GOSPLAN]"),
                                client.provider,
                                client.model
                            );
                            match client.triage(&refine_prompt, Some(&repo_ctx)) {
                                Ok(new_tickets) if !new_tickets.is_empty() => {
                                    println!(
                                        "  {} Tickets successfully refined! ({} ticket(s))",
                                        theme.green("✓"),
                                        new_tickets.len()
                                    );
                                    tickets = new_tickets;
                                    idx = 0;
                                    continue;
                                }
                                Ok(_) => eprintln!(
                                    "  {}: AI returned empty ticket list. Keeping current tickets.",
                                    theme.yellow("Warning")
                                ),
                                Err(e) => eprintln!(
                                    "  {}: AI refinement failed: {}. Keeping current tickets.",
                                    theme.red_bold("Error"),
                                    e
                                ),
                            }
                        }
                    }
                    'q' => {
                        println!("\n  Triage session cancelled by user.");
                        return ExitCode::SUCCESS;
                    }
                    _ => continue,
                }
            }

            if created_count > 0 {
                println!();
                let target_git_repo = if let Some(ref f) = detected_forge {
                    let is_current = git_repo
                        .as_ref()
                        .and_then(|r| cccp::forges::detect_forge_from_repo(r).ok())
                        .map(|cf| {
                            cf.repo.eq_ignore_ascii_case(&f.repo)
                                && cf.owner.eq_ignore_ascii_case(&f.owner)
                        })
                        .unwrap_or(false);

                    if is_current {
                        git_repo.clone()
                    } else {
                        let catalog = cccp::catalog::load_catalog(Some(&config), start_path);
                        let slug = format!("{}/{}", f.owner, f.repo);
                        let local_path = catalog
                            .iter()
                            .find(|e| {
                                e.slug.eq_ignore_ascii_case(&slug)
                                    || e.name.eq_ignore_ascii_case(&f.repo)
                            })
                            .and_then(cccp::catalog::find_local_repo_path)
                            .or_else(|| {
                                std::env::var("HOME").ok().and_then(|h| {
                                    let p = std::path::PathBuf::from(h).join("src").join(&f.repo);
                                    if p.join(".git").exists() {
                                        Some(p)
                                    } else {
                                        None
                                    }
                                })
                            });

                        local_path.and_then(|p| cccp::git::GitRepo::open(Some(&p)).ok())
                    }
                } else {
                    git_repo.clone()
                };

                if let Some(ref repo) = target_git_repo {
                    println!(
                        "{} Syncing .antigravity/tickets.md in {}...",
                        theme.gold_bold("★ [GOSPLAN]"),
                        repo.root_dir.display()
                    );
                    let tickets_args = cccp::cli::TicketsArgs {
                        state: "open".to_string(),
                        limit: 30,
                        remote: detected_forge
                            .as_ref()
                            .map(|f| format!("https://{}/{}/{}", f.host, f.owner, f.repo)),
                        output: None,
                        action: None,
                    };
                    let _ = cccp::tickets::sync_tickets(&tickets_args, repo, &config, &theme);
                } else if let Some(ref f) = detected_forge {
                    println!(
                        "{}: Target repository '{}/{}' has no local clone; skipping local .antigravity/tickets.md update.",
                        theme.yellow("Notice"),
                        f.owner,
                        f.repo
                    );
                }
                println!("Session concluded. {} ticket(s) published.", created_count);
            }

            ExitCode::SUCCESS
        }
        Some(Commands::Forge(forges)) => {
            let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };
            let info = match cccp::forges::detect_forge_from_repo(&repo) {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };

            match forges.action {
                Some(cli::ForgeActions::Detect) => {
                    println!(
                        "{} Forge detected: {} (host: {}, owner: {}, repo: {})",
                        theme.green("✓"),
                        theme.gold_bold(&info.provider.to_string()),
                        info.host,
                        info.owner,
                        info.repo
                    );
                }
                Some(cli::ForgeActions::Issues { state }) => {
                    let client = cccp::forges::ForgeClient::new();
                    match client.fetch_issues(&info, &state, 30) {
                        Ok(issues) => {
                            cccp::forges::display_issues(&issues, &theme);
                        }
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            return ExitCode::FAILURE;
                        }
                    }
                }
                Some(cli::ForgeActions::Prs { state }) => {
                    let client = cccp::forges::ForgeClient::new();
                    match client.fetch_pull_requests(&info, &state, 30) {
                        Ok(prs) => {
                            cccp::forges::display_prs(&prs, &theme);
                        }
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            return ExitCode::FAILURE;
                        }
                    }
                }
                None => {
                    println!(
                        "{} Forge: {} ({}/{}) at {}",
                        theme.gold_bold("★"),
                        info.provider,
                        info.owner,
                        info.repo,
                        info.host
                    );
                    println!(
                        "Use 'cccp forges --help' to see available actions (detect, issues, prs)."
                    );
                }
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
            if push.all {
                let entries = cccp::catalog::load_catalog(Some(&config), None);
                let report = cccp::catalog::batch_push_catalog(&entries, push.dry_run, &theme);
                println!(
                    "{}",
                    cccp::catalog::format_batch_push_report(&report, &theme)
                );
                if !report.failed.is_empty() {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            } else {
                let mut cmd = std::process::Command::new("git");
                cmd.arg("push");
                if push.dry_run {
                    cmd.arg("--dry-run");
                }
                if !push.git_args.is_empty() {
                    cmd.args(&push.git_args);
                }
                match cmd.status() {
                    Ok(status) if status.success() => ExitCode::SUCCESS,
                    Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
                    Err(e) => {
                        eprintln!("Error executing git push: {}", e);
                        ExitCode::FAILURE
                    }
                }
            }
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
                println!("ai_provider = {}", config.ai_provider);
                println!("ollama_model = {}", config.ollama_model);
                println!("ollama_host = {}", config.ollama_host);
                println!("ollama_temperature = {}", config.ollama_temperature);
                println!("gemini_model = {}", config.gemini_model);
                println!("gemini_temperature = {}", config.gemini_temperature);
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
                        "ai_provider" | "provider" | "llm_provider" => {
                            println!("{}", config.ai_provider)
                        }
                        "ollama_model" => println!("{}", config.ollama_model),
                        "gemini_model" => println!("{}", config.gemini_model),
                        "ollama_host" | "ollama_url" => println!("{}", config.ollama_host),
                        "ollama_temperature" => println!("{}", config.ollama_temperature),
                        "gemini_temperature" => println!("{}", config.gemini_temperature),
                        "ai_temperature" | "temperature" => {
                            println!("{}", config.ollama_temperature)
                        }
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
        Some(Commands::Audit(gost)) => {
            let start_dir = args
                .dir
                .as_deref()
                .unwrap_or_else(|| std::path::Path::new("."));
            let report = cccp::audit::run_gost_audit(start_dir, gost.strict, &theme);
            if report.is_success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Some(Commands::Comrades(c)) => {
            if c.format == "json" {
                println!(
                    "[{{\"name\":\"Lumen Pink\",\"email\":\"hi@lumen.pink\",\"role\":\"Hero of Socialist Code & Project Architect\"}}]"
                );
            } else {
                println!();
                println!(
                    "{}",
                    theme.bold(
                        "★ CCCP Honorary Stakhanovite Brigade (Comrades of the Revolution) ★"
                    )
                );
                println!(
                    "{}",
                    theme.dim("────────────────────────────────────────────────────────────────────────────")
                );
                println!(
                    "  • {} <hi@lumen.pink> — {}",
                    theme.gold_bold("Lumen Pink"),
                    theme.cyan("Hero of Socialist Code & Project Architect")
                );
                println!(
                    "{}",
                    theme.dim("────────────────────────────────────────────────────────────────────────────")
                );
                println!("Glory to the workers who maintain clean commits and strict SemVer!");
                println!();
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Completion(comp)) => {
            let mut cmd = Cli::command();
            clap_complete::generate(comp.shell, &mut cmd, "cccp", &mut std::io::stdout());
            ExitCode::SUCCESS
        }
        Some(Commands::Catalog(catalog_args)) => match catalog_args.action {
            Some(cli::CatalogSubcommands::Add {
                repo,
                name,
                description,
            }) => {
                match cccp::catalog::add_remote_repo(
                    &repo,
                    name.as_deref(),
                    description.as_deref(),
                    Some(&config),
                ) {
                    Ok((entry, path, is_updated)) => {
                        let action_str = if is_updated { "UPDATED" } else { "REGISTERED" };
                        println!(
                            "{} Comrade! Repository {} in State Catalog ({})",
                            theme.gold_bold("★"),
                            action_str,
                            path.display()
                        );
                        println!("  • Name:   {}", theme.bold(&entry.name));
                        println!("  • Forge:  {}", entry.forge);
                        println!("  • Slug:   {}", theme.cyan(&entry.slug));
                        if !entry.description.is_empty() {
                            println!("  • Desc:   {}", entry.description);
                        }
                        if !entry.topics.is_empty() {
                            println!("  • Topics: {}", entry.topics.join(", "));
                        }
                        if !entry.license.is_empty() {
                            println!("  • License: {}", entry.license);
                        }
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::CatalogSubcommands::AddNew { from_src }) => {
                let raw_path = from_src.unwrap_or_else(|| "~/src".to_string());
                let target_dir = cccp::catalog::expand_tilde(&raw_path);
                match cccp::catalog::scan_and_add_from_src(&target_dir, Some(&config)) {
                    Ok(res) => {
                        println!(
                            "{} Comrade! Ingested repositories from '{}' into State Catalog ({})",
                            theme.gold_bold("★"),
                            target_dir.display(),
                            res.catalog_path.display()
                        );
                        println!(
                            "  • Total directories scanned: {}",
                            theme.bold(&res.total_scanned.to_string())
                        );
                        println!(
                            "  • Newly registered:          {}",
                            theme.green(&res.added.len().to_string())
                        );
                        if !res.skipped_existing.is_empty() {
                            println!(
                                "  • Already registered:        {}",
                                theme.yellow(&res.skipped_existing.len().to_string())
                            );
                        }
                        if !res.skipped_not_git_or_remote.is_empty() {
                            println!(
                                "  • Skipped (no git/remote):   {}",
                                theme.dim(&res.skipped_not_git_or_remote.len().to_string())
                            );
                        }

                        if !res.added.is_empty() {
                            println!("\n{} Newly Registered Repositories:", theme.gold_bold("★"));
                            for item in &res.added {
                                println!(
                                    "    • {} [{}] ({})",
                                    theme.bold(&item.name),
                                    theme.cyan(&item.forge),
                                    item.slug
                                );
                            }
                        }
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            }
            None => {
                let entries = cccp::catalog::load_catalog(Some(&config), None);
                if entries.is_empty() {
                    println!("{} State Catalog is currently empty.", theme.yellow("★"));
                } else {
                    let total_count = entries.len();
                    let mut summaries = cccp::catalog::summarize_catalog(&entries);
                    cccp::catalog::filter_and_sort_summaries(&mut summaries, &catalog_args.filter);

                    if catalog_args.filter.status.is_some()
                        || catalog_args.filter.min_tickets.is_some()
                    {
                        println!(
                            "{} Gosplan State Catalog ({} of {} repositories matching filter):\n",
                            theme.gold_bold("★"),
                            summaries.len(),
                            total_count
                        );
                    } else {
                        println!(
                            "{} Gosplan State Catalog ({} repositories registered):\n",
                            theme.gold_bold("★"),
                            summaries.len()
                        );
                    }
                    let table = cccp::catalog::format_catalog_table(&summaries, &theme);
                    println!("{}", table);
                }
                ExitCode::SUCCESS
            }
            Some(cli::CatalogSubcommands::List(list_filter)) => {
                let entries = cccp::catalog::load_catalog(Some(&config), None);
                if entries.is_empty() {
                    println!("{} State Catalog is currently empty.", theme.yellow("★"));
                } else {
                    let merged_filter = catalog_args.filter.merge(&list_filter);
                    let total_count = entries.len();
                    let mut summaries = cccp::catalog::summarize_catalog(&entries);
                    cccp::catalog::filter_and_sort_summaries(&mut summaries, &merged_filter);

                    if merged_filter.status.is_some() || merged_filter.min_tickets.is_some() {
                        println!(
                            "{} Gosplan State Catalog ({} of {} repositories matching filter):\n",
                            theme.gold_bold("★"),
                            summaries.len(),
                            total_count
                        );
                    } else {
                        println!(
                            "{} Gosplan State Catalog ({} repositories registered):\n",
                            theme.gold_bold("★"),
                            summaries.len()
                        );
                    }
                    let table = cccp::catalog::format_catalog_table(&summaries, &theme);
                    println!("{}", table);
                }
                ExitCode::SUCCESS
            }
            Some(cli::CatalogSubcommands::New(newrepo_args)) => {
                match cccp::newrepo::cmd_newrepo(newrepo_args.into(), &theme) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::CatalogSubcommands::Push { dry_run }) => {
                let entries = cccp::catalog::load_catalog(Some(&config), None);
                let report = cccp::catalog::batch_push_catalog(&entries, dry_run, &theme);
                println!(
                    "{}",
                    cccp::catalog::format_batch_push_report(&report, &theme)
                );
                if !report.failed.is_empty() {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
            Some(cli::CatalogSubcommands::Assemble { dry_run }) => {
                let entries = cccp::catalog::load_catalog(Some(&config), None);
                let report = cccp::catalog::batch_assemble_gost(&entries, dry_run, &theme);
                println!(
                    "{}",
                    cccp::catalog::format_batch_assemble_report(&report, &theme)
                );
                if !report.failed.is_empty() {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
        },
        Some(Commands::Ci(act_args)) => {
            let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };
            match act_args.action {
                Some(cli::ActSubcommands::Status) | None => {
                    cccp::act::run_act_status(&repo.root_dir);
                    ExitCode::SUCCESS
                }
                Some(cli::ActSubcommands::Run { job }) => {
                    match cccp::act::run_act_run(
                        &repo.root_dir,
                        job.as_deref(),
                        &theme,
                        args.verbose,
                    ) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Some(cli::ActSubcommands::Log { follow }) => {
                    match cccp::act::run_act_log(&repo.root_dir, follow, &theme) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
            }
        }
        Some(Commands::Deps(deps_args)) => match deps_args.action {
            Some(cli::DepsSubcommands::List) => {
                cccp::deps::cmd_deps_list(&theme);
                ExitCode::SUCCESS
            }
            Some(cli::DepsSubcommands::Check { deps }) => {
                match cccp::deps::cmd_deps_check(&deps, &theme) {
                    Ok(true) => ExitCode::SUCCESS,
                    Ok(false) => ExitCode::FAILURE,
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::DepsSubcommands::Install { all, deps }) => {
                match cccp::deps::cmd_deps_install(&deps, all, &theme) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    }
                }
            }
            None => match cccp::deps::cmd_deps_check(&deps_args.deps, &theme) {
                Ok(true) => ExitCode::SUCCESS,
                Ok(false) => ExitCode::FAILURE,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                }
            },
        },
        Some(Commands::Agent(agent_args)) => {
            let start_path = args.dir.as_deref().map(std::path::Path::new);
            let repo = cccp::git::GitRepo::open(args.dir.as_deref()).ok();
            let repo_root = repo.as_ref().map(|r| r.root_dir.as_path()).or(start_path);

            match agent_args.action {
                Some(cli::AgentSubcommands::Update) | None => {
                    match cccp::agents::cmd_agent_update(
                        agent_args.url.as_deref(),
                        agent_args.file.as_deref(),
                        repo_root,
                        &theme,
                    ) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Some(cli::AgentSubcommands::Check) => {
                    match cccp::agents::cmd_agent_check(
                        agent_args.url.as_deref(),
                        agent_args.file.as_deref(),
                        repo_root,
                        &theme,
                    ) {
                        Ok(true) => ExitCode::SUCCESS,
                        Ok(false) => ExitCode::FAILURE,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Some(cli::AgentSubcommands::Set { key, value }) => {
                    match cccp::agents::cmd_agent_set(&key, &value, repo_root) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Some(cli::AgentSubcommands::Get { key }) => {
                    match cccp::agents::cmd_agent_get(&key, repo_root) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Some(cli::AgentSubcommands::Unset { key }) => {
                    match cccp::agents::cmd_agent_unset(&key, repo_root) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Some(cli::AgentSubcommands::Fill) => {
                    match cccp::agents::cmd_agent_fill(
                        agent_args.url.as_deref(),
                        agent_args.file.as_deref(),
                        repo_root,
                        &theme,
                    ) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
            }
        }
        Some(Commands::SetAv(set_av_args)) => {
            let start_path = args.dir.as_deref().map(std::path::Path::new);
            let repo = cccp::git::GitRepo::open(args.dir.as_deref()).ok();
            let repo_root = repo.as_ref().map(|r| r.root_dir.as_path()).or(start_path);

            match cccp::agents::cmd_agent_set(&set_av_args.key, &set_av_args.value, repo_root) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Init(newrepo_args)) => {
            match cccp::newrepo::cmd_newrepo(newrepo_args.into(), &theme) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Serve(serve_args)) => {
            match cccp::serve::cmd_serve(serve_args.into(), &theme) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Spool(spool_args)) => match spool_args.action {
            Some(cli::SpoolSubcommands::Send(send_args)) => {
                let prompt = if send_args.prompt.is_empty() {
                    use std::io::{IsTerminal, Read};
                    if !std::io::stdin().is_terminal() {
                        let mut s = String::new();
                        let _ = std::io::stdin().read_to_string(&mut s);
                        s
                    } else {
                        String::new()
                    }
                } else {
                    send_args.prompt.join(" ")
                };
                match cccp::matraca::cmd_matraca_send(
                    send_args.client_id.as_deref(),
                    send_args.workspace.as_deref(),
                    &prompt,
                    None,
                ) {
                    Ok(resp) => {
                        println!("{}", resp);
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::SpoolSubcommands::Poll { id }) => {
                match cccp::matraca::cmd_matraca_poll(&id, None) {
                    Ok(resp) => {
                        println!("{}", resp);
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::SpoolSubcommands::Worker(worker_args)) => {
                match cccp::matraca::cmd_matraca_worker(
                    worker_args.daemon,
                    worker_args.job_id.as_deref(),
                    None,
                ) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::SpoolSubcommands::History { limit }) => {
                match cccp::matraca::cmd_matraca_history(limit, None) {
                    Ok(resp) => {
                        println!("{}", resp);
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::SpoolSubcommands::Status { json }) => {
                match cccp::matraca::cmd_matraca_status(json, None) {
                    Ok(resp) => {
                        print!("{}", resp);
                        if !resp.ends_with('\n') {
                            println!();
                        }
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::SpoolSubcommands::Clean { days }) => {
                match cccp::matraca::cmd_matraca_clean(days, None) {
                    Ok(resp) => {
                        println!("{}", resp);
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            }
            None => {
                if !spool_args.prompt.is_empty() {
                    let prompt = spool_args.prompt.join(" ");
                    match cccp::matraca::cmd_matraca_send(None, None, &prompt, None) {
                        Ok(resp) => {
                            println!("{}", resp);
                            ExitCode::SUCCESS
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                            ExitCode::FAILURE
                        }
                    }
                } else {
                    println!("cccp spool - Mobile & Offline-Tolerant Queue & Spool for Antigravity (agy)");
                    println!("=========================================================================\n");
                    println!("Usage:");
                    println!(
                        "  cccp spool send [--client-id <id>] [--workspace <dir>] <prompt...>"
                    );
                    println!("  cccp spool poll <job_id|client_id>");
                    println!("  cccp spool worker [--daemon] [--once] [--job-id <id>]");
                    println!("  cccp spool history [--limit <N>]");
                    println!("  cccp spool status [--json]");
                    println!("  cccp spool clean [--days <N>]\n");
                    println!("Note:");
                    println!("  To open the interactive cockpit in browser: cccp cockpit");
                    ExitCode::SUCCESS
                }
            }
        },
        Some(Commands::Cockpit(cockpit_args)) => {
            match cccp::web::cmd_cockpit(cockpit_args.into()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), err);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Update(update_args)) => match cccp::update::cmd_update(&update_args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{}: {}", theme.red_bold("Error"), err);
                ExitCode::FAILURE
            }
        },
        Some(Commands::CheckUpdate(check_args)) => {
            match cccp::update::cmd_check_update(check_args.channel.as_deref()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), err);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Editor(editor_args)) => {
            match cccp::editor::edit_file_or_scratch(editor_args.file.as_deref()) {
                Ok(_) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("{}: {}", theme.red_bold("Editor Error"), err);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Merge(merge_args)) => {
            let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };

            // Inspect commits that the target branch would introduce
            let branch_rev = format!("HEAD..{}", merge_args.branch);
            let log_output = Command::new("git")
                .args(["log", &branch_rev, "--pretty=format:%B---COMMIT-END---"])
                .current_dir(&repo.root_dir)
                .output();

            let commits_text = match log_output {
                Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
                _ => {
                    eprintln!(
                        "{}: Branch '{}' does not exist or has no commit log relative to HEAD.",
                        theme.red_bold("Merge Lock Rejected"),
                        merge_args.branch
                    );
                    return ExitCode::FAILURE;
                }
            };

            let closing_ids = cccp::hook_runner::extract_closes_ids(&commits_text);
            if closing_ids.is_empty() && !merge_args.force {
                eprintln!(
                    "\n{}\n{}\nBranch '{}' does not contain any commit with canonical closing tag: '(closes #<id>)'.\n\n{}\n{}\n",
                    theme.red_bold("★ GOSPLAN QUALITY CONTROL: MERGE LOCK REJECTED ★"),
                    theme.gold_bold("«НИ ШАГУ НАЗАД!» — Order No. 227 (Strict DD-TDD Phase 9 Compliance)"),
                    merge_args.branch,
                    theme.cyan("Gosplan Directive: Every feature/fix branch MUST contain '(closes #<id>)' in its final commit before merge to main."),
                    theme.dim("If this merge is intentional and administrative, rerun with --force.")
                );
                return ExitCode::FAILURE;
            }

            println!(
                "{} Integrating branch '{}' (verified ticket closing tags: {:?})...",
                theme.cyan("★ [GOSPLAN]"),
                merge_args.branch,
                closing_ids
            );

            let mut git_merge = Command::new("git");
            git_merge.arg("merge");
            if let Some(ref msg) = merge_args.message {
                git_merge.args(["-m", msg]);
            }
            git_merge.arg(&merge_args.branch);
            git_merge.current_dir(&repo.root_dir);

            match git_merge.status() {
                Ok(st) if st.success() => {
                    println!(
                        "{} Branch '{}' merged successfully into current branch.",
                        theme.green("✓"),
                        merge_args.branch
                    );
                    // Automatically close tickets if not already closed by post-merge hook
                    if !closing_ids.is_empty() {
                        let comment =
                            "Fechado automaticamente pelo merge no CCCP / GOST DD-TDD Fase 9.";
                        let _ = cccp::tickets::close_tickets(
                            &repo,
                            &closing_ids,
                            Some(comment),
                            &theme,
                        );
                    }
                    ExitCode::SUCCESS
                }
                Ok(st) => {
                    eprintln!(
                        "{}: git merge exited with code {}",
                        theme.red_bold("Error"),
                        st.code().unwrap_or(1)
                    );
                    ExitCode::FAILURE
                }
                Err(e) => {
                    eprintln!(
                        "{}: Failed to invoke git merge: {}",
                        theme.red_bold("Error"),
                        e
                    );
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Profile(cmd)) => {
            let root_dir = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r.root_dir,
                Err(_) => match std::env::current_dir() {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        return ExitCode::FAILURE;
                    }
                },
            };
            match cccp::profile::handle_profile_command(&root_dir, Some(cmd), &theme) {
                Ok(_) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                }
            }
        }
        None => {
            let mut cmd = Cli::command();
            let _ = cmd.print_help();
            println!();
            ExitCode::SUCCESS
        }
    }
}
