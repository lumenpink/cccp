use cccp::cli::{Cli, Commands, PromptArgs};
use clap::Parser;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_parse_prompt_cli_options() {
    let cli = Cli::try_parse_from(["cccp", "prompt", "--short", "--no-color"])
        .expect("should parse cccp prompt -s --no-color");
    match cli.command {
        Some(Commands::Prompt(args)) => {
            assert!(args.short);
            assert!(args.no_color);
            assert!(!args.json);
            assert!(!args.bash);
            assert!(!args.zsh);
        }
        _ => panic!("Expected Commands::Prompt"),
    }

    let cli = Cli::try_parse_from(["cccp", "prompt", "--json", "--status"])
        .expect("should parse cccp prompt --json --status");
    match cli.command {
        Some(Commands::Prompt(args)) => {
            assert!(args.json);
            assert!(args.status);
        }
        _ => panic!("Expected Commands::Prompt"),
    }

    let cli = Cli::try_parse_from(["cccp", "prompt", "--bash"])
        .expect("should parse cccp prompt --bash");
    match cli.command {
        Some(Commands::Prompt(args)) => {
            assert!(args.bash);
        }
        _ => panic!("Expected Commands::Prompt"),
    }
}

#[test]
fn test_prompt_generation_in_git_repo() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // 1. Outside git repo -> returns None / empty
    let empty_args = PromptArgs::default();
    let out = cccp::prompt::generate_prompt(Some(root), &empty_args);
    assert_eq!(out, None, "Outside git repo should return None");

    // 2. Initialize git repository
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Gosplan Test"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@gosplan.su"])
        .current_dir(root)
        .output()
        .unwrap();

    fs::write(root.join("README.md"), "# Test\n").unwrap();
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore: initial commit"])
        .current_dir(root)
        .output()
        .unwrap();

    // Clean state on main
    let prompt_clean = cccp::prompt::generate_prompt(Some(root), &PromptArgs {
        no_color: true,
        short: true,
        ..Default::default()
    });
    assert!(prompt_clean.is_some());
    let txt_clean = prompt_clean.unwrap();
    assert!(txt_clean.contains("main"), "Clean prompt should contain branch 'main': {}", txt_clean);
    assert!(!txt_clean.contains('*'), "Clean prompt should not contain '*': {}", txt_clean);

    // Dirty state on main
    fs::write(root.join("README.md"), "# Modified\n").unwrap();
    let prompt_dirty = cccp::prompt::generate_prompt(Some(root), &PromptArgs {
        no_color: true,
        short: true,
        ..Default::default()
    });
    assert!(prompt_dirty.is_some());
    let txt_dirty = prompt_dirty.unwrap();
    assert!(txt_dirty.contains('*'), "Dirty prompt should contain '*': {}", txt_dirty);

    // Switch to DD-TDD feature branch
    Command::new("git")
        .args(["checkout", "-b", "feat/67-prompt-shell"])
        .current_dir(root)
        .output()
        .unwrap();

    let prompt_ddtdd = cccp::prompt::generate_prompt(Some(root), &PromptArgs {
        no_color: true,
        ..Default::default()
    });
    assert!(prompt_ddtdd.is_some());
    let txt_ddtdd = prompt_ddtdd.unwrap();
    assert!(txt_ddtdd.contains("#67"), "DD-TDD prompt should contain ticket #67: {}", txt_ddtdd);
    assert!(txt_ddtdd.contains("ph3"), "Initial phase should be ph3: {}", txt_ddtdd);

    // JSON mode
    let prompt_json = cccp::prompt::generate_prompt(Some(root), &PromptArgs {
        json: true,
        ..Default::default()
    });
    assert!(prompt_json.is_some());
    let json_str = prompt_json.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Valid JSON");
    assert_eq!(parsed["branch"], "feat/67-prompt-shell");
    assert_eq!(parsed["dirty"], true);
    assert_eq!(parsed["ticket"], 67);
    assert_eq!(parsed["phase"], "ph3");

    // Bash ANSI wrapping
    let prompt_bash = cccp::prompt::generate_prompt(Some(root), &PromptArgs {
        bash: true,
        ..Default::default()
    });
    assert!(prompt_bash.is_some());
    let bash_txt = prompt_bash.unwrap();
    assert!(bash_txt.contains("\\["), "Bash mode should wrap escapes with \\[: {}", bash_txt);
    assert!(bash_txt.contains("\\]"), "Bash mode should wrap escapes with \\]: {}", bash_txt);

    // Zsh ANSI wrapping
    let prompt_zsh = cccp::prompt::generate_prompt(Some(root), &PromptArgs {
        zsh: true,
        ..Default::default()
    });
    assert!(prompt_zsh.is_some());
    let zsh_txt = prompt_zsh.unwrap();
    assert!(zsh_txt.contains("%{"), "Zsh mode should wrap escapes with %{{: {}", zsh_txt);
    assert!(zsh_txt.contains("%}"), "Zsh mode should wrap escapes with %}}: {}", zsh_txt);

    // Status mode with local tickets
    let tickets_dir = root.join(".antigravity");
    fs::create_dir_all(&tickets_dir).unwrap();
    fs::write(
        tickets_dir.join("tickets.md"),
        "## [#10] [open] task 1\n## [#11] [open] task 2\n## [#12] [closed] task 3\n",
    )
    .unwrap();

    let prompt_status = cccp::prompt::generate_prompt(Some(root), &PromptArgs {
        status: true,
        no_color: true,
        ..Default::default()
    });
    assert!(prompt_status.is_some());
    let status_txt = prompt_status.unwrap();
    assert!(status_txt.contains("⚑2"), "Status mode should show ⚑2 open tickets: {}", status_txt);
}
