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

    match args.command {
        Some(Commands::Start(start_args)) => {
            cccp::ddtdd::handle_start(&start_args, args.dir.as_deref(), &theme)
        }
        Some(Commands::Close(close_args)) => {
            cccp::ddtdd::handle_close(&close_args, args.dir.as_deref(), &theme)
        }
        Some(Commands::Mutate(mutate_args)) => {
            cccp::hygiene::handle_mutate(&mutate_args, args.dir.as_deref(), &theme)
        }
        Some(Commands::Hygiene(hygiene_args)) => {
            cccp::hygiene::handle_hygiene(&hygiene_args, args.dir.as_deref(), &theme)
        }
        Some(Commands::Fix(fix_args)) => {
            let h_args = cccp::cli::HygieneArgs {
                fix: true,
                lang: fix_args.lang,
            };
            cccp::hygiene::handle_hygiene(&h_args, args.dir.as_deref(), &theme)
        }
        Some(Commands::Commit(cz)) => {
            if cz.summary.is_some() || cz.message.is_some() {
                if let Some(code) = cccp::ddtdd::handle_commit(&cz, args.dir.as_deref(), &theme) {
                    return code;
                }
            }
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
        Some(Commands::Version(ver_args)) => {
            let cccp_version = env!("CCCP_CANONICAL_VERSION");
            let repo_dir = args.dir.as_deref().unwrap_or(std::path::Path::new("."));
            let gost_version_file = repo_dir.join("GOSTVER");
            let gost_str = if gost_version_file.is_file() {
                cccp::catalog::extract_gost_version(repo_dir)
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                "none".to_string()
            };
            let repo_version_file = repo_dir.join("VERSION");
            let proj_version = if repo_version_file.is_file() {
                std::fs::read_to_string(&repo_version_file)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            };

            if ver_args.format == "json" {
                println!(
                    "{}",
                    serde_json::json!({
                        "cccp": cccp_version,
                        "gost": gost_str,
                        "project": proj_version,
                    })
                );
            } else {
                println!(
                    "{} CCCP v{}",
                    theme.gold_bold("★"),
                    theme.bold(cccp_version)
                );
                println!("  • GOST Specification: {}", theme.cyan(&gost_str));
                if let Some(pv) = proj_version {
                    println!("  • Project Version:    {}", theme.green(&pv));
                }
            }
            ExitCode::SUCCESS
        }
        Some(Commands::GenVersion(semver)) => {
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
                    cli::HookSubcommands::PreCommit => {
                        let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                            Ok(r) => r,
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                return ExitCode::FAILURE;
                            }
                        };
                        match cccp::hook_runner::run_pre_commit(&repo, &theme, args.verbose) {
                            Ok(0) => ExitCode::SUCCESS,
                            Ok(code) => ExitCode::from(code as u8),
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Error"), e);
                                ExitCode::FAILURE
                            }
                        }
                    }
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
                    if let Some(ref br) = repo.branch {
                        if let Some((tid, kind, slug)) = cccp::ddtdd::infer_ticket_from_branch(br) {
                            let msgs: Vec<&str> = repo
                                .commits_since_tag
                                .iter()
                                .map(|c| c.summary.as_str())
                                .collect();
                            let phase = cccp::ddtdd::infer_next_phase(&msgs);
                            println!(
                                "\n  {} {}",
                                theme.gold_bold("★ DD-TDD Phase Tracker:"),
                                theme.cyan(&format!("#{} ({}/{})", tid, kind, slug))
                            );
                            println!(
                                "    • Fase Atual : {} ({})",
                                theme.green(&phase.to_string()),
                                phase.description()
                            );
                            let next_cmd = match phase {
                                cccp::ddtdd::DdtddPhase::Ph3 => {
                                    "cccp commit \"especificar doc\" (ph3)"
                                }
                                cccp::ddtdd::DdtddPhase::Ph4 => {
                                    "cccp commit \"adicionar teste vermelho\" (ph4)"
                                }
                                cccp::ddtdd::DdtddPhase::Ph5 => {
                                    "cccp commit \"implementar funcionalidade\" (ph5)"
                                }
                                cccp::ddtdd::DdtddPhase::Ph6 => {
                                    "cccp commit \"refatorar código\" (ph6)"
                                }
                                cccp::ddtdd::DdtddPhase::Ph7 => {
                                    "cccp commit \"eliminar mutantes\" (ph7)"
                                }
                                cccp::ddtdd::DdtddPhase::Ph8 => {
                                    "cccp commit \"portão de qualidade\" (ph8)"
                                }
                                cccp::ddtdd::DdtddPhase::Ph9 => {
                                    "cccp close \"resumo do fechamento\""
                                }
                            };
                            println!("    • Próximo    : {}", theme.yellow(next_cmd));
                        }
                    }
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

            match tickets.action {
                Some(cli::TicketAction::Close {
                    ref ids,
                    ref comment,
                }) => {
                    if let Err(e) =
                        cccp::tickets::close_tickets(&repo, ids, comment.as_deref(), &theme)
                    {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Some(cli::TicketAction::Comment { id, ref message }) => {
                    if let Err(e) =
                        cccp::tickets::comment_ticket(&repo, id, message.as_deref(), &theme)
                    {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Some(cli::TicketAction::New {
                    ref title,
                    ref body,
                    ref labels,
                    ref remote,
                }) => {
                    if let Err(e) = cccp::tickets::create_ticket_cli(
                        &repo,
                        title.as_deref(),
                        body.as_deref(),
                        labels,
                        remote.as_deref(),
                        &config,
                        &theme,
                    ) {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Some(cli::TicketAction::Divan(ref divan_args)) => {
                    run_divan_cmd(divan_args, args.dir.as_deref(), &config, &theme)
                }
                Some(cli::TicketAction::Sync {
                    ref remote,
                    ref output,
                    limit,
                    ref state,
                }) => {
                    let sync_opts = cli::TicketArgs {
                        remote: remote.clone(),
                        output: output.clone(),
                        limit,
                        state: state.clone(),
                        action: None,
                    };
                    if let Err(e) = cccp::tickets::sync_tickets(&sync_opts, &repo, &config, &theme)
                    {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Some(cli::TicketAction::List) | None => {
                    if let Err(e) = cccp::tickets::list_local_tickets(&repo, &theme) {
                        eprintln!("{}: {}", theme.red_bold("Error"), e);
                        ExitCode::FAILURE
                    } else {
                        ExitCode::SUCCESS
                    }
                }
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

            // 5. Sync catalog ticket counts
            if sync_args.catalog_tickets {
                println!(
                    "{} Synchronizing catalog ticket counts across all repositories...",
                    theme.cyan("★")
                );
                match cccp::catalog::update_catalog_tickets(&[], true, Some(&config)) {
                    Ok(report) => {
                        println!(
                            "  Catalog ticket counts updated for {} repository(ies).",
                            report.updated.len()
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
            if soviet.comrades {
                print_comrades(&theme, "table");
            } else if soviet.order227 {
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
            let start_path = args.dir.as_deref().map(std::path::Path::new);
            let repo = cccp::git::GitRepo::open(args.dir.as_deref()).ok();
            let repo_root = repo
                .as_ref()
                .map(|r| r.root_dir.as_path())
                .or(start_path)
                .unwrap_or(std::path::Path::new("."));

            let options = cccp::release::ReleaseTagOptions {
                version: tag.version.clone(),
                title: tag.title.clone(),
                message: tag.message.clone(),
                no_v: tag.no_v,
                sign: tag.sign,
            };

            match cccp::release::execute_release_tag(repo_root, &options) {
                Ok(tag_name) => {
                    println!(
                        "\n{}\n{} Release tag '{}' criada com sucesso e sincronizada com VERSION e CHANGELOG.md.\n\n{}\n  git push origin main && git push origin {}\n",
                        theme.gold_bold("★ GOSPLAN RELEASE CONCLUÍDA COM SUCESSO ★"),
                        theme.green("✓"),
                        tag_name,
                        theme.cyan("Execute o comando para publicar as alterações no remote:"),
                        tag_name
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Erro ao criar release tag"), e);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Changelog(ch)) => {
            let start_path = args.dir.as_deref().map(std::path::Path::new);
            let repo = cccp::git::GitRepo::open(args.dir.as_deref()).ok();
            let repo_root = repo
                .as_ref()
                .map(|r| r.root_dir.as_path())
                .or(start_path)
                .unwrap_or(std::path::Path::new("."));

            let options = cccp::release::ChangelogOptions {
                output: ch.output.clone(),
                dry_run: ch.dry_run,
                range: ch.range.clone(),
            };

            match cccp::release::cmd_changelog(repo_root, &options, &theme) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Erro ao gerar CHANGELOG"), e);
                    ExitCode::FAILURE
                }
            }
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
                cccp::push::handle_push(&push, args.dir.as_deref(), &theme)
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
        Some(Commands::Check(gost)) => {
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
        Some(Commands::Completion(ref comp)) => {
            if let Some(ref query) = comp.candidates {
                if query == "repos" {
                    let candidates = cccp::completion::get_catalog_candidates(
                        Some(&config),
                        args.dir.as_deref(),
                    );
                    for c in candidates {
                        println!("{}", c);
                    }
                }
                return ExitCode::SUCCESS;
            }

            if let Some(ref action) = comp.action {
                match action {
                    cli::CompletionAction::Install(ref inst) => {
                        let shell = if inst.zsh {
                            clap_complete::Shell::Zsh
                        } else {
                            inst.shell.unwrap_or(clap_complete::Shell::Bash)
                        };
                        let target_path = inst
                            .path
                            .clone()
                            .or_else(|| cccp::completion::detect_rc_path(shell));
                        let Some(rc_path) = target_path else {
                            eprintln!(
                                "{}: Não foi possível determinar o arquivo de configuração para o shell.",
                                theme.red_bold("Erro")
                            );
                            return ExitCode::FAILURE;
                        };
                        match cccp::completion::install_completion_into_file(
                            &rc_path,
                            shell,
                            inst.dry_run,
                        ) {
                            Ok(_) => {
                                if inst.dry_run {
                                    println!(
                                        "{} [DRY-RUN] Script de autocompletion seria injetado em: {}",
                                        theme.yellow("★ [GOSPLAN]"),
                                        rc_path.display()
                                    );
                                } else {
                                    println!(
                                        "{} Autocompletion instalado com sucesso em: {}",
                                        theme.green("✓"),
                                        rc_path.display()
                                    );
                                    println!(
                                        "  Execute 'source {}' ou abra uma nova sessão de terminal.",
                                        rc_path.display()
                                    );
                                }
                                return ExitCode::SUCCESS;
                            }
                            Err(e) => {
                                eprintln!("{}: {}", theme.red_bold("Erro"), e);
                                return ExitCode::FAILURE;
                            }
                        }
                    }
                }
            }

            if let Some(shell) = comp.shell {
                let mut cmd = Cli::command();
                clap_complete::generate(shell, &mut cmd, "cccp", &mut std::io::stdout());
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "{}: Especifique o shell (ex.: cccp completion bash) ou use 'cccp completion install'.",
                    theme.red_bold("Erro")
                );
                ExitCode::FAILURE
            }
        }
        Some(Commands::Prompt(ref prompt_args)) => {
            if let Some(output) = cccp::prompt::generate_prompt(args.dir.as_deref(), prompt_args) {
                println!("{}", output);
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Repo(catalog_args)) => match catalog_args.action {
            Some(cli::CatalogSubcommands::Add {
                repo,
                name,
                description,
                aliases,
            }) => {
                match cccp::catalog::add_remote_repo(
                    &repo,
                    name.as_deref(),
                    description.as_deref(),
                    &aliases,
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
                        if !entry.aliases.is_empty() {
                            println!("  • Aliases: {}", theme.yellow(&entry.aliases.join(", ")));
                        }
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
            Some(cli::CatalogSubcommands::Alias(alias_args)) => match alias_args.action {
                Some(cli::CatalogAliasSubcommands::Add { repo, aliases }) => {
                    match cccp::catalog::add_repo_aliases(&repo, &aliases, Some(&config)) {
                        Ok(entry) => {
                            println!(
                                "{} Comrade! Added aliases to '{}' ({}):",
                                theme.gold_bold("★"),
                                theme.bold(&entry.name),
                                theme.cyan(&entry.slug)
                            );
                            println!(
                                "  • Current Aliases: {}",
                                theme.yellow(&entry.aliases.join(", "))
                            );
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Some(cli::CatalogAliasSubcommands::Remove { repo, aliases }) => {
                    match cccp::catalog::remove_repo_aliases(&repo, &aliases, Some(&config)) {
                        Ok(entry) => {
                            println!(
                                "{} Comrade! Removed aliases from '{}' ({}):",
                                theme.gold_bold("★"),
                                theme.bold(&entry.name),
                                theme.cyan(&entry.slug)
                            );
                            if entry.aliases.is_empty() {
                                println!("  • Current Aliases: {}", theme.dim("(none)"));
                            } else {
                                println!(
                                    "  • Current Aliases: {}",
                                    theme.yellow(&entry.aliases.join(", "))
                                );
                            }
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Some(cli::CatalogAliasSubcommands::List { repo }) => {
                    let catalog = cccp::catalog::load_catalog(Some(&config), None);
                    if let Some(ref r) = repo {
                        let clean_r = r.trim();
                        if let Some(entry) = catalog.iter().find(|e| {
                            e.slug.eq_ignore_ascii_case(clean_r)
                                || e.name.eq_ignore_ascii_case(clean_r)
                                || e.aliases.iter().any(|a| a.eq_ignore_ascii_case(clean_r))
                        }) {
                            println!(
                                "{} Repository '{}' ({}):",
                                theme.gold_bold("★"),
                                theme.bold(&entry.name),
                                theme.cyan(&entry.slug)
                            );
                            if entry.aliases.is_empty() {
                                println!("  • Aliases: {}", theme.dim("(none)"));
                            } else {
                                println!(
                                    "  • Aliases: {}",
                                    theme.yellow(&entry.aliases.join(", "))
                                );
                            }
                            ExitCode::SUCCESS
                        } else {
                            eprintln!(
                                "{}: Repositório '{}' não encontrado no catálogo.",
                                theme.red_bold("Error"),
                                clean_r
                            );
                            ExitCode::FAILURE
                        }
                    } else {
                        println!("{} Configured Repository Aliases:", theme.gold_bold("★"));
                        let mut has_any = false;
                        for entry in &catalog {
                            if !entry.aliases.is_empty() {
                                has_any = true;
                                println!(
                                    "  • {} ({}) -> {}",
                                    theme.bold(&entry.name),
                                    theme.cyan(&entry.slug),
                                    theme.yellow(&entry.aliases.join(", "))
                                );
                            }
                        }
                        if !has_any {
                            println!("  {}", theme.dim("No aliases configured in catalog."));
                        }
                        ExitCode::SUCCESS
                    }
                }
                None => {
                    if let Some(ref r) = alias_args.repo {
                        if !alias_args.aliases.is_empty() {
                            match cccp::catalog::add_repo_aliases(
                                r,
                                &alias_args.aliases,
                                Some(&config),
                            ) {
                                Ok(entry) => {
                                    println!(
                                        "{} Comrade! Associated aliases to '{}' ({}):",
                                        theme.gold_bold("★"),
                                        theme.bold(&entry.name),
                                        theme.cyan(&entry.slug)
                                    );
                                    println!(
                                        "  • Current Aliases: {}",
                                        theme.yellow(&entry.aliases.join(", "))
                                    );
                                    ExitCode::SUCCESS
                                }
                                Err(e) => {
                                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                                    ExitCode::FAILURE
                                }
                            }
                        } else {
                            let catalog = cccp::catalog::load_catalog(Some(&config), None);
                            let clean_r = r.trim();
                            if let Some(entry) = catalog.iter().find(|e| {
                                e.slug.eq_ignore_ascii_case(clean_r)
                                    || e.name.eq_ignore_ascii_case(clean_r)
                                    || e.aliases.iter().any(|a| a.eq_ignore_ascii_case(clean_r))
                            }) {
                                println!(
                                    "{} Repository '{}' ({}):",
                                    theme.gold_bold("★"),
                                    theme.bold(&entry.name),
                                    theme.cyan(&entry.slug)
                                );
                                if entry.aliases.is_empty() {
                                    println!("  • Aliases: {}", theme.dim("(none)"));
                                } else {
                                    println!(
                                        "  • Aliases: {}",
                                        theme.yellow(&entry.aliases.join(", "))
                                    );
                                }
                                ExitCode::SUCCESS
                            } else {
                                eprintln!(
                                    "{}: Repositório '{}' não encontrado no catálogo.",
                                    theme.red_bold("Error"),
                                    clean_r
                                );
                                ExitCode::FAILURE
                            }
                        }
                    } else {
                        eprintln!(
                            "{}: Repositório alvo não fornecido.",
                            theme.red_bold("Error")
                        );
                        ExitCode::FAILURE
                    }
                }
            },
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
                        || catalog_args.filter.ci.is_some()
                        || catalog_args.filter.dirty
                        || catalog_args.filter.unpushed
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
                    if catalog_args.filter.ci_details {
                        let details = cccp::catalog::format_catalog_ci_details(&summaries, &theme);
                        print!("{}", details);
                    }
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

                    if merged_filter.status.is_some()
                        || merged_filter.min_tickets.is_some()
                        || merged_filter.ci.is_some()
                        || merged_filter.dirty
                        || merged_filter.unpushed
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
                    if merged_filter.ci_details {
                        let details = cccp::catalog::format_catalog_ci_details(&summaries, &theme);
                        print!("{}", details);
                    }
                }
                ExitCode::SUCCESS
            }
            Some(cli::CatalogSubcommands::Init(newrepo_args)) => {
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
            Some(cli::CatalogSubcommands::Assemble {
                repo,
                dry_run,
                auto_commit,
            }) => {
                let entries = cccp::catalog::load_catalog(Some(&config), None);
                if let Some(ref target) = repo {
                    match cccp::catalog::assemble_catalog_repo(
                        &entries,
                        target,
                        dry_run,
                        auto_commit,
                        Some(&config),
                        &theme,
                    ) {
                        Ok(()) => {
                            println!(
                                "{} Especificação GOST montada com sucesso para '{}'.",
                                theme.green("✓"),
                                target
                            );
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!(
                                "{} Falha ao montar GOST para '{}': {}",
                                theme.red("✗"),
                                target,
                                e
                            );
                            ExitCode::FAILURE
                        }
                    }
                } else {
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
            }
            Some(cli::CatalogSubcommands::Remove { repo, delete_dir }) => {
                match cccp::catalog::remove_repo_from_catalog_ext(&repo, delete_dir, Some(&config))
                {
                    Ok((entry, path, purged_dir)) => {
                        if let Some(dir) = purged_dir {
                            println!(
                                "{} Comrade! Removed repository '{}' ({}) from State Catalog ({}) and purged clean directory ({})",
                                theme.gold_bold("★"),
                                theme.bold(&entry.name),
                                theme.cyan(&entry.slug),
                                path.display(),
                                theme.green(&dir.display().to_string())
                            );
                        } else {
                            println!(
                                "{} Comrade! Removed repository '{}' ({}) from State Catalog ({})",
                                theme.gold_bold("★"),
                                theme.bold(&entry.name),
                                theme.cyan(&entry.slug),
                                path.display()
                            );
                        }
                        ExitCode::SUCCESS
                    }
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            }
            Some(cli::CatalogSubcommands::UpdateTickets { repos, all }) => {
                let sync_all = all || repos.is_empty();
                match cccp::catalog::update_catalog_tickets(&repos, sync_all, Some(&config)) {
                    Ok(report) => {
                        println!(
                            "{} Comrade! Updated ticket count for {} repository(ies):",
                            theme.gold_bold("★"),
                            report.updated.len()
                        );
                        for entry in &report.updated {
                            println!(
                                "  {} {} ({}) -> {} open ticket(s)",
                                theme.gold_bold("•"),
                                theme.bold(&entry.name),
                                theme.cyan(&entry.slug),
                                theme.bold(&entry.open_tickets.unwrap_or(0).to_string())
                            );
                        }
                        for (target, err) in &report.failed {
                            eprintln!("  {} Failed for '{}': {}", theme.red("✗"), target, err);
                        }
                        if report.updated.is_empty() && !report.failed.is_empty() {
                            ExitCode::FAILURE
                        } else {
                            ExitCode::SUCCESS
                        }
                    }
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            }
        },
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
        Some(Commands::Update(update_args)) => {
            if update_args.check {
                match cccp::update::cmd_check_update(update_args.channel.as_deref()) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
                }
            } else {
                match cccp::update::cmd_update(&update_args) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(err) => {
                        eprintln!("{}: {}", theme.red_bold("Error"), err);
                        ExitCode::FAILURE
                    }
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
                    // Automatically reconcile VERSION if diverged (e.g. parallel branches)
                    if let Err(e) = cccp::hook_runner::reconcile_merge_version(&repo, &theme) {
                        eprintln!(
                            "  {} Notice: Could not reconcile VERSION after merge: {}",
                            theme.yellow("!"),
                            e
                        );
                    }
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
        Some(Commands::Epic(ref epic_cmd)) => {
            let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };
            let res = match epic_cmd {
                cli::EpicCommand::Start(args) => {
                    cccp::governance::cmd_epic_start(&repo, &args.slug, &theme)
                }
                cli::EpicCommand::List => {
                    cccp::governance::cmd_epic_list(&repo, &theme).map(|_| ())
                }
                cli::EpicCommand::Finish(args) => {
                    cccp::governance::cmd_epic_finish(&repo, &args.slug, &theme)
                }
            };
            match res {
                Ok(_) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Inbox(ref inbox_cmd)) => {
            let repo = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    return ExitCode::FAILURE;
                }
            };
            let res = match inbox_cmd {
                cli::InboxCommand::List => {
                    cccp::governance::cmd_inbox_list(&repo, &theme).map(|_| ())
                }
                cli::InboxCommand::Review(args) => {
                    cccp::governance::cmd_inbox_review(&repo, &args.id, &theme).map(|_| ())
                }
                cli::InboxCommand::Accept(args) => {
                    cccp::governance::cmd_inbox_accept(&repo, &args.id, &theme)
                }
            };
            match res {
                Ok(_) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("{}: {}", theme.red_bold("Error"), e);
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Site(ref site_cmd)) => {
            let root = match cccp::git::GitRepo::open(args.dir.as_deref()) {
                Ok(r) => r.root_dir,
                Err(_) => args.dir.clone().unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
                }),
            };
            match site_cmd {
                cli::SiteCommand::Banner(banner_args) => {
                    let opts = cccp::site_banner::BannerOptions {
                        title: banner_args.title.clone(),
                        tagline: banner_args.tagline.clone(),
                        output: banner_args.output.clone(),
                    };
                    match cccp::site_banner::generate_site_banner(&root, &opts, &theme) {
                        Ok(_) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("{}: {}", theme.red_bold("Error"), e);
                            ExitCode::FAILURE
                        }
                    }
                }
            }
        }
        Some(Commands::Divan(ref divan_args)) => {
            run_divan_cmd(divan_args, args.dir.as_deref(), &config, &theme)
        }
        None => {
            let mut cmd = Cli::command();
            let _ = cmd.print_help();
            println!();
            ExitCode::SUCCESS
        }
    }
}

fn print_comrades(theme: &Theme, format: &str) {
    if format == "json" {
        println!(
            r#"[{{ "name": "Lumen Pink", "email": "hi@lumen.pink", "role": "Hero of Socialist Code & Project Architect" }}]"#
        );
    } else {
        println!();
        println!(
            "{}",
            theme.bold("★ CCCP Honorary Stakhanovite Brigade (Comrades of the Revolution) ★")
        );
        println!(
            "{}",
            theme.dim(
                "────────────────────────────────────────────────────────────────────────────"
            )
        );
        println!(
            "  • {} <hi@lumen.pink> — {}",
            theme.gold_bold("Lumen Pink"),
            theme.cyan("Hero of Socialist Code & Project Architect")
        );
        println!(
            "{}",
            theme.dim(
                "────────────────────────────────────────────────────────────────────────────"
            )
        );
        println!("Glory to the workers who maintain clean commits and strict SemVer!");
        println!();
    }
}

fn run_divan_cmd(
    diva: &cli::DivanArgs,
    dir: Option<&std::path::Path>,
    config: &config::Config,
    theme: &Theme,
) -> ExitCode {
    let start_path = dir;

    // 1. Determine target repo and prompt from arguments
    let (target_repo_arg, mut prompt_arg) = diva.resolve_repo_and_prompt();

    // 2. Resolve target forge and repository
    let mut git_repo = cccp::git::GitRepo::open(dir).ok();
    let mut target_dir: std::path::PathBuf = if let Some(ref r) = git_repo {
        r.root_dir.clone()
    } else if let Some(p) = dir {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    };
    let mut detected_forge = if let Some(ref target) = target_repo_arg {
        match cccp::catalog::resolve_target_repo_workspace(target, Some(config), start_path) {
            Some((info, local_path)) => {
                if let Some(path) = local_path {
                    git_repo = cccp::git::GitRepo::open(Some(&path)).ok();
                    target_dir = path;
                } else {
                    git_repo = None;
                }
                Some(info)
            }
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
        let catalog_entries = cccp::catalog::load_catalog(Some(config), start_path);
        if !catalog_entries.is_empty() && std::io::stdin().is_terminal() {
            println!(
                "{} No git repository detected in current directory.",
                theme.yellow("★ [GOSPLAN]")
            );
            println!("Select target repository from your catalog:");
            match inquire::Select::new("Target repository:", catalog_entries).prompt() {
                Ok(selected) => {
                    let local_path =
                        cccp::catalog::find_local_repo_path_with_config(&selected, Some(config));
                    if let Some(path) = local_path {
                        git_repo = cccp::git::GitRepo::open(Some(&path)).ok();
                        target_dir = path;
                    } else {
                        git_repo = None;
                    }
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
            if cccp::forges::resolve_token_for_host(&f.host, f.provider, Some(config)).is_none() {
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
    let mut client = cccp::ai::AiClient::from_config(config);
    if diva.local {
        client = client.with_provider(cccp::ai::AiProvider::Local);
        if diva.model.is_none() {
            client = client.with_model(&config.local_ai_model);
        }
        if diva.host.is_none() {
            client = client.with_host(&config.local_ai_host);
        }
    } else if diva.ollama {
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
                        cccp::ai::AiProvider::Local => &config.local_ai_model,
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

    let mut resume_from_tickets: Option<(String, Vec<cccp::ai::DecomposedTicket>, usize)> = None;

    // Check intentional manual drafts first (Ctrl+R / Vim swap style)
    if let Ok(Some(draft)) = cccp::draft::DivanDraft::load(&target_dir) {
        let stage_desc = match draft.stage {
            cccp::draft::DraftStage::Prompt => "rascunho de prompt preliminar".to_string(),
            cccp::draft::DraftStage::TicketsReview => {
                format!(
                    "rascunho com {} ticket(s) pendente(s) de revisão",
                    draft.tickets.len()
                )
            }
        };

        let choice = if std::io::stdin().is_terminal() {
            println!(
                "{} Encontrado {} para este repositório.",
                theme.yellow("★ [GOSPLAN]"),
                stage_desc
            );
            loop {
                print!("  [c] Continuar rascunho  [m] Manter e novo  [d] Descartar e novo → ");
                let _ = std::io::stdout().flush();
                let key = cccp::ai::read_single_key().unwrap_or('c');
                println!();
                match key {
                    'c' | 'C' => break 'c',
                    'm' | 'M' => break 'm',
                    'd' | 'D' => break 'd',
                    _ => {}
                }
            }
        } else {
            'c'
        };

        match choice {
            'c' => {
                println!(
                    "  {} {}",
                    theme.green("✓"),
                    theme.bold("Rascunho restaurado com sucesso.")
                );
                match draft.stage {
                    cccp::draft::DraftStage::Prompt => {
                        prompt_arg = Some(draft.prompt);
                    }
                    cccp::draft::DraftStage::TicketsReview => {
                        resume_from_tickets =
                            Some((draft.prompt, draft.tickets, draft.current_index));
                    }
                }
            }
            'm' => {
                println!("  {} Rascunho mantido no disco.", theme.dim("ℹ"));
            }
            'd' => {
                println!("  {} Rascunho descartado do disco.", theme.dim("ℹ"));
                let _ = cccp::draft::DivanDraft::clear(&target_dir);
            }
            _ => {}
        }
    }

    // If no draft was resumed, check unintentional crash checkpoints
    if resume_from_tickets.is_none() && prompt_arg.is_none() {
        if let Ok(Some(cp)) = cccp::checkpoint::DivanCheckpoint::load(&target_dir) {
            let stage_desc = match cp.stage {
                cccp::checkpoint::CheckpointStage::PromptEntered => {
                    "interrompida antes da chamada à IA (prompt preservado)".to_string()
                }
                cccp::checkpoint::CheckpointStage::TicketsPendingReview => {
                    format!("com {} ticket(s) pendente(s) de revisão", cp.tickets.len())
                }
            };

            let restore_prompt = format!(
                "Sessão anterior encontrada para este repositório ({})",
                stage_desc
            );

            let should_restore = if std::io::stdin().is_terminal() {
                print!(
                    "{} {}. Restaurar sessão? [Y/n] → ",
                    theme.yellow("★ [GOSPLAN]"),
                    restore_prompt
                );
                let _ = std::io::stdout().flush();
                let key = cccp::ai::read_single_key().unwrap_or('y');
                println!();
                key != 'n' && key != 'N'
            } else {
                true
            };

            if should_restore {
                println!(
                    "  {} {}",
                    theme.green("✓"),
                    theme.bold("Sessão anterior restaurada com sucesso.")
                );
                match cp.stage {
                    cccp::checkpoint::CheckpointStage::PromptEntered => {
                        prompt_arg = Some(cp.prompt);
                    }
                    cccp::checkpoint::CheckpointStage::TicketsPendingReview => {
                        resume_from_tickets = Some((cp.prompt, cp.tickets, cp.current_index));
                    }
                }
            } else {
                println!("  {} Sessão anterior descartada.", theme.dim("ℹ"));
                let _ = cccp::checkpoint::DivanCheckpoint::clear(&target_dir);
            }
        }
    }

    let repo_ctx = cccp::ai::build_enriched_context_with_provider(
        git_repo.as_ref(),
        detected_forge.as_ref(),
        config,
        Some(client.provider),
    );

    // Handle Divan comment mode on existing ticket
    if let Some(target_issue_num) = diva.comment {
        let forge_info = match detected_forge {
            Some(ref f) => f.clone(),
            None => {
                eprintln!(
                            "{}: Cannot comment on ticket #{}: No remote git forge detected for repository '{}'.",
                            theme.red_bold("Error"),
                            target_issue_num,
                            target_repo_arg.as_deref().unwrap_or("current")
                        );
                return ExitCode::FAILURE;
            }
        };

        println!(
            "{} Loading ticket #{} and comments from {} ({}/{})...",
            theme.cyan("★ [GOSPLAN]"),
            target_issue_num,
            forge_info.provider,
            forge_info.owner,
            forge_info.repo
        );

        let forge_client = cccp::forges::ForgeClient::new();
        let issue_comments = forge_client
            .fetch_issue_comments(&forge_info, target_issue_num)
            .unwrap_or_default();

        let mut issue_ctx = format!(
            "Target Ticket #{} on {} ({}/{})\n",
            target_issue_num, forge_info.provider, forge_info.owner, forge_info.repo
        );

        if !issue_comments.is_empty() {
            issue_ctx.push_str(&format!(
                "\nPrevious Comments ({}):\n",
                issue_comments.len()
            ));
            for c in &issue_comments {
                issue_ctx.push_str(&format!(
                    "- @{} ({}):\n  {}\n",
                    c.author, c.created_at, c.body
                ));
            }
        }

        let user_prompt = if let Some(p) = prompt_arg.take() {
            p
        } else if std::io::stdin().is_terminal() {
            let initial = format!(
                        "# The Divan — Comment on Ticket #{}\n# {}\n# Describe what you want the AI to analyze, propose or draft for this ticket.\n# Lines starting with '#' will be ignored.\n\n",
                        target_issue_num,
                        issue_ctx.replace('\n', "\n# ")
                    );
            match cccp::editor::edit_prompt("Divan - Ticket Comment Prompt", &initial) {
                Ok(Some(edited)) => {
                    let filtered: Vec<&str> = edited
                        .lines()
                        .filter(|l| !l.trim().starts_with('#'))
                        .collect();
                    filtered.join("\n").trim().to_string()
                }
                _ => {
                    eprintln!(
                        "{}: No comment instructions provided. Exiting.",
                        theme.yellow("Notice")
                    );
                    return ExitCode::SUCCESS;
                }
            }
        } else {
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap_or_default();
            line.trim().to_string()
        };

        if user_prompt.is_empty() {
            eprintln!(
                "{}: No comment instructions provided. Exiting.",
                theme.yellow("Notice")
            );
            return ExitCode::SUCCESS;
        }

        let ai_prompt = format!(
                    "You are a helpful software engineering assistant in the CCCP (Conventional Commits Compliance Program).\n\
                     The user wants to comment on ticket #{} in repository {}.\n\n\
                     {}\n\n\
                     User request/question regarding this ticket:\n\
                     {}\n\n\
                     Draft a clear, concise, constructive technical comment response directly addressing the user request in the context of this ticket.",
                    target_issue_num, forge_info.repo, issue_ctx, user_prompt
                );

        println!(
            "{} Consulting {} ({}) to generate comment for ticket #{}...",
            theme.yellow("★ [GOSPLAN]"),
            theme.cyan(&client.provider.to_string()),
            theme.yellow(&client.model),
            target_issue_num
        );

        let generated_body = match client.triage(&ai_prompt, Some(&repo_ctx)) {
            Ok(tickets) if !tickets.is_empty() => tickets[0].body.clone(),
            _ => user_prompt.clone(),
        };

        let editor_title = format!("Ticket #{} — Review Comment", target_issue_num);
        let edited_comment = if std::io::stdin().is_terminal() {
            match cccp::editor::edit_text(&editor_title, &generated_body) {
                Ok(Some(c)) => c.trim().to_string(),
                _ => {
                    println!("  {} Comment aborted by user.", theme.dim("ℹ"));
                    return ExitCode::SUCCESS;
                }
            }
        } else {
            generated_body.trim().to_string()
        };

        if edited_comment.is_empty() {
            eprintln!("{}: Comment is empty. Exiting.", theme.yellow("Notice"));
            return ExitCode::SUCCESS;
        }

        if diva.dry_run {
            println!(
                "  {} [DRY-RUN] Comment verified for ticket #{}. Not published:\n{}",
                theme.green("✓"),
                target_issue_num,
                edited_comment
            );
            return ExitCode::SUCCESS;
        }

        println!(
            "  -> Publishing comment to ticket #{} on {}...",
            target_issue_num, forge_info.provider
        );
        match forge_client.create_comment(&forge_info, target_issue_num, &edited_comment) {
            Ok(c) => {
                println!(
                    "  {} Comment posted successfully to ticket #{} (comment ID: {}).",
                    theme.green("✓"),
                    target_issue_num,
                    c.id
                );
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("{}: Failed to post comment: {}", theme.red_bold("Error"), e);
                return ExitCode::FAILURE;
            }
        }
    }

    let (prompt, mut tickets, mut idx) = if let Some((p, t, i)) = resume_from_tickets {
        println!(
            "{} Retomando revisão a partir do ticket {}/{}...",
            theme.yellow("★ [GOSPLAN]"),
            i + 1,
            t.len()
        );
        (p, t, i)
    } else {
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
            let target_dir_draft = target_dir.clone();
            let draft_saver = move |content: &str| {
                let filtered: Vec<&str> = content
                    .lines()
                    .filter(|line| !line.trim().starts_with('#'))
                    .collect();
                let prompt_text = filtered.join("\n").trim().to_string();
                if !prompt_text.is_empty() {
                    cccp::draft::DivanDraft::save_prompt(&target_dir_draft, &prompt_text)
                } else {
                    Ok(())
                }
            };
            match cccp::editor::edit_prompt_with_draft(
                "Divan - Triage Prompt",
                &initial_prompt,
                draft_saver,
            ) {
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

        let _ = cccp::checkpoint::DivanCheckpoint::save_prompt(&target_dir, &prompt);

        let tickets = if diva.direct {
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
            let _ = cccp::checkpoint::DivanCheckpoint::clear(&target_dir);
            println!(
                "{} No tickets were identified by the AI model.",
                theme.yellow("★ [GOSPLAN]")
            );
            return ExitCode::SUCCESS;
        }

        let _ = cccp::checkpoint::DivanCheckpoint::save_tickets(&target_dir, &prompt, &tickets, 0);
        (prompt, tickets, 0)
    };

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

    while idx < tickets.len() {
        let _ =
            cccp::checkpoint::DivanCheckpoint::save_tickets(&target_dir, &prompt, &tickets, idx);

        println!(
            "{} Opening ticket {}/{} in CCCP Editor for review and submission...",
            theme.yellow("★ [GOSPLAN]"),
            idx + 1,
            tickets.len()
        );

        let initial_content = format!("{}\n\n{}", tickets[idx].title, tickets[idx].body);
        let editor_title = format!("Ticket {}/{} — Review & Submit", idx + 1, tickets.len());

        let target_dir_draft = target_dir.clone();
        let prompt_draft = prompt.clone();
        let tickets_draft = tickets.clone();
        let current_idx = idx;
        let draft_saver = move |content: &str| {
            let mut d_tickets = tickets_draft.clone();
            if let Ok((t, b)) = cccp::editor::parse_ticket_buffer(content) {
                d_tickets[current_idx].title = t;
                d_tickets[current_idx].body = b;
            }
            cccp::draft::DivanDraft::save_tickets(
                &target_dir_draft,
                &prompt_draft,
                &d_tickets,
                current_idx,
            )
        };

        match cccp::editor::edit_ticket_with_draft(&editor_title, &initial_content, draft_saver) {
            Ok(Some(edited)) => match cccp::editor::parse_ticket_buffer(&edited) {
                Ok((title, body)) => {
                    match cccp::editor::validate_ticket_submission(&title, &body, config) {
                        Ok(()) => {
                            tickets[idx].title = title;
                            tickets[idx].body = body;

                            if diva.dry_run {
                                println!(
                                    "  {} {}",
                                    theme.green("✓"),
                                    theme.dim(&format!(
                                        "[DRY-RUN] Ticket verified: '{}'. Not published.",
                                        tickets[idx].title
                                    ))
                                );
                                idx += 1;
                                continue;
                            }

                            if let Some(ref forge_info) = detected_forge {
                                println!(
                                    "  -> Publishing ticket to {} ({}/{})...",
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
                                        print!(
                                            "  [r] Retry in editor  [s] Skip ticket  [q] Quit → "
                                        );
                                        let _ = std::io::stdout().flush();
                                        let key = cccp::ai::read_single_key().unwrap_or('s');
                                        if key == 'q' {
                                            break;
                                        } else if key == 's' {
                                            idx += 1;
                                        }
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
                        Err(val_err) => {
                            eprintln!(
                                "\n  {}: {}",
                                theme.red_bold("Invalid ticket format"),
                                val_err
                            );
                            print!("  [e] Reopen in editor  [s] Skip ticket  [q] Quit → ");
                            let _ = std::io::stdout().flush();
                            let key = cccp::ai::read_single_key().unwrap_or('e');
                            if key == 'q' {
                                break;
                            } else if key == 's' {
                                idx += 1;
                            }
                        }
                    }
                }
                Err(parse_err) => {
                    eprintln!(
                        "\n  {}: {}",
                        theme.red_bold("Ticket parse error"),
                        parse_err
                    );
                    print!("  [e] Reopen in editor  [s] Skip ticket  [q] Quit → ");
                    let _ = std::io::stdout().flush();
                    let key = cccp::ai::read_single_key().unwrap_or('e');
                    if key == 'q' {
                        break;
                    } else if key == 's' {
                        idx += 1;
                    }
                }
            },
            Ok(None) => {
                println!(
                    "\n  {} Ticket review cancelled by user (not published).",
                    theme.yellow("Notice")
                );
                print!("  [e] Reopen in editor  [r] Refine with AI  [s] Skip ticket  [q] Quit → ");
                let _ = std::io::stdout().flush();
                let key = cccp::ai::read_single_key().unwrap_or('s');
                match key {
                    'q' => break,
                    's' => idx += 1,
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
                                    let _ = cccp::checkpoint::DivanCheckpoint::save_tickets(
                                        &target_dir,
                                        &prompt,
                                        &tickets,
                                        0,
                                    );
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
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("\n  {}: {}", theme.red_bold("Editor error"), e);
                idx += 1;
            }
        }
    }

    if idx >= tickets.len() {
        let _ = cccp::checkpoint::DivanCheckpoint::clear(&target_dir);
        let _ = cccp::draft::DivanDraft::clear(&target_dir);
    }

    if created_count > 0 {
        println!();
        let target_git_repo = if let Some(ref f) = detected_forge {
            let is_current = git_repo
                .as_ref()
                .and_then(|r| cccp::forges::detect_forge_from_repo(r).ok())
                .map(|cf| {
                    cf.repo.eq_ignore_ascii_case(&f.repo) && cf.owner.eq_ignore_ascii_case(&f.owner)
                })
                .unwrap_or(false);

            if is_current {
                git_repo.clone()
            } else {
                let catalog = cccp::catalog::load_catalog(Some(config), start_path);
                let slug = format!("{}/{}", f.owner, f.repo);
                let local_path = catalog
                    .iter()
                    .find(|e| {
                        e.slug.eq_ignore_ascii_case(&slug) || e.name.eq_ignore_ascii_case(&f.repo)
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
            let _ = cccp::tickets::sync_tickets(&tickets_args, repo, config, theme);
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
