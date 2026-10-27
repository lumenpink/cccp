//! Tests for DD-TDD Phase 7 & 8 tools: `cccp mutate` and `cccp hygiene` / `cccp fix`
//! Validates toolchain resolution, CLI argument parsing, and command matrix.

use cccp::hygiene::{resolve_hygiene_commands, resolve_mutate_command};

#[test]
fn test_resolve_mutate_command() {
    let rust_cmd = resolve_mutate_command("rust", "main");
    assert_eq!(rust_cmd, Some(vec!["cargo", "mutants", "--in-diff"]));

    let php_cmd = resolve_mutate_command("php", "main");
    assert_eq!(php_cmd, Some(vec!["vendor/bin/infection", "--git-diff-lines"]));

    let js_cmd = resolve_mutate_command("javascript", "main");
    assert_eq!(js_cmd, Some(vec!["npx", "stryker", "run", "--diff"]));

    let py_cmd = resolve_mutate_command("python", "main");
    assert_eq!(py_cmd, Some(vec!["mutatest"]));

    let unknown = resolve_mutate_command("unknown", "main");
    assert_eq!(unknown, None);
}

#[test]
fn test_resolve_hygiene_commands_check_mode() {
    let rust_cmds = resolve_hygiene_commands("rust", false);
    assert_eq!(
        rust_cmds,
        vec![
            vec!["cargo", "fmt", "--check"],
            vec!["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
        ]
    );

    let shell_cmds = resolve_hygiene_commands("shell", false);
    assert_eq!(
        shell_cmds,
        vec![
            vec!["shfmt", "-d", "-i", "2", "-ci", "-bn"],
            vec!["shellcheck"],
        ]
    );

    let py_cmds = resolve_hygiene_commands("python", false);
    assert_eq!(
        py_cmds,
        vec![
            vec!["ruff", "check"],
            vec!["ruff", "format", "--check"],
        ]
    );
}

#[test]
fn test_resolve_hygiene_commands_fix_mode() {
    let rust_fix = resolve_hygiene_commands("rust", true);
    assert_eq!(rust_fix, vec![vec!["cargo", "fmt"]]);

    let shell_fix = resolve_hygiene_commands("shell", true);
    assert_eq!(shell_fix, vec![vec!["shfmt", "-w", "-i", "2", "-ci", "-bn"]]);

    let js_fix = resolve_hygiene_commands("javascript", true);
    assert_eq!(
        js_fix,
        vec![vec!["npx", "@biomejs/biome", "check", "--write"]]
    );
}

#[test]
fn test_cli_parsing_mutate_and_hygiene() {
    use cccp::cli::{Cli, Commands};
    use clap::Parser;

    // cccp mutate
    let cli = Cli::try_parse_from(["cccp", "mutate"]).expect("should parse cccp mutate");
    match cli.command {
        Some(Commands::Mutate(m)) => {
            assert_eq!(m.base, None);
            assert!(!m.dry_run);
        }
        _ => panic!("Expected Commands::Mutate"),
    }

    // cccp mutate --base develop --dry-run
    let cli2 = Cli::try_parse_from(["cccp", "mutate", "--base", "develop", "--dry-run"])
        .expect("should parse cccp mutate with flags");
    match cli2.command {
        Some(Commands::Mutate(m)) => {
            assert_eq!(m.base, Some("develop".to_string()));
            assert!(m.dry_run);
        }
        _ => panic!("Expected Commands::Mutate"),
    }

    // cccp hygiene
    let cli3 = Cli::try_parse_from(["cccp", "hygiene"]).expect("should parse cccp hygiene");
    match cli3.command {
        Some(Commands::Hygiene(h)) => {
            assert!(!h.fix);
        }
        _ => panic!("Expected Commands::Hygiene"),
    }

    // cccp fix
    let cli4 = Cli::try_parse_from(["cccp", "fix"]).expect("should parse cccp fix");
    match cli4.command {
        Some(Commands::Fix(f)) => {
            assert!(f.fix);
        }
        _ => panic!("Expected Commands::Fix"),
    }
}
