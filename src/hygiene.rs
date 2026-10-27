//! DD-TDD Phase 7 & 8 tools: `cccp mutate` and `cccp hygiene` / `cccp fix`.
//! Implements canonical toolchain resolution and execution for mutation testing and Fast Hygiene.

use crate::cli::{HygieneArgs, MutateArgs};
use crate::cultural::Theme;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

/// Resolves the mutation testing command for the specified language.
pub fn resolve_mutate_command<'a>(lang: &str, _base: &str) -> Option<Vec<&'a str>> {
    match lang.trim().to_lowercase().as_str() {
        "rust" => Some(vec!["cargo", "mutants", "--in-diff"]),
        "php" => Some(vec!["vendor/bin/infection", "--git-diff-lines"]),
        "javascript" | "typescript" => Some(vec!["npx", "stryker", "run", "--diff"]),
        "go" => Some(vec!["gremlins", "unleash", "--diff"]),
        "python" => Some(vec!["mutatest"]),
        _ => None,
    }
}

/// Resolves the Fast Hygiene / linter & formatter commands for the specified language.
pub fn resolve_hygiene_commands<'a>(lang: &str, fix: bool) -> Vec<Vec<&'a str>> {
    match lang.trim().to_lowercase().as_str() {
        "rust" => {
            if fix {
                vec![vec!["cargo", "fmt"]]
            } else {
                vec![
                    vec!["cargo", "fmt", "--check"],
                    vec!["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
                ]
            }
        }
        "php" => {
            if fix {
                vec![vec!["vendor/bin/pint"]]
            } else {
                vec![
                    vec!["vendor/bin/pint", "--test"],
                    vec!["vendor/bin/phpstan", "analyse", "--level=max"],
                ]
            }
        }
        "shell" => {
            if fix {
                vec![vec!["shfmt", "-w", "-i", "2", "-ci", "-bn"]]
            } else {
                vec![
                    vec!["shfmt", "-d", "-i", "2", "-ci", "-bn"],
                    vec!["shellcheck"],
                ]
            }
        }
        "javascript" | "typescript" => {
            if fix {
                vec![vec!["npx", "@biomejs/biome", "check", "--write"]]
            } else {
                vec![vec!["npx", "@biomejs/biome", "check"]]
            }
        }
        "python" => {
            if fix {
                vec![vec!["ruff", "check", "--fix"], vec!["ruff", "format"]]
            } else {
                vec![
                    vec!["ruff", "check"],
                    vec!["ruff", "format", "--check"],
                ]
            }
        }
        _ => vec![],
    }
}

/// Reads configured languages from GOSTVER in root directory.
pub fn get_gost_languages(root: &Path) -> Vec<String> {
    let gostver_path = root.join("GOSTVER");
    if let Ok(content) = fs::read_to_string(&gostver_path) {
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("GOST_LANGUAGES=") {
                let unquoted = rest.trim().trim_matches('"').trim_matches('\'');
                return unquoted
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    vec!["rust".to_string()]
}

/// Dispatches `cccp mutate`
pub fn handle_mutate(args: &MutateArgs, dir: Option<&Path>, theme: &Theme) -> ExitCode {
    let root = dir.unwrap_or(Path::new("."));
    let langs = get_gost_languages(root);
    let base = args.base.as_deref().unwrap_or("main");

    println!(
        "{} [GOSPLAN DD-TDD] Executando Fase 7 — Teste de Mutação no Diff (base: {}):",
        theme.gold_bold("★"),
        theme.cyan(base)
    );

    let mut any_executed = false;
    for lang in &langs {
        if let Some(cmd_parts) = resolve_mutate_command(lang, base) {
            any_executed = true;
            println!("  Linguagem : {}", theme.green(lang));
            println!("  Comando   : {}", theme.yellow(&cmd_parts.join(" ")));

            if args.dry_run {
                println!("  [dry-run] Comando não disparado.");
                continue;
            }

            let mut cmd = Command::new(cmd_parts[0]);
            cmd.args(&cmd_parts[1..]);
            if let Some(d) = dir {
                cmd.current_dir(d);
            }

            match cmd.status() {
                Ok(status) if status.success() => {
                    println!("✓ Mutação concluída com sucesso para '{}'!", lang);
                }
                Ok(status) => {
                    eprintln!(
                        "{}: Teste de mutação falhou para '{}' (código: {:?})",
                        theme.red_bold("Falha na Fase 7"),
                        lang,
                        status.code()
                    );
                    return ExitCode::FAILURE;
                }
                Err(err) => {
                    eprintln!(
                        "{}: Não foi possível executar '{}': {}",
                        theme.red_bold("Erro de Execução"),
                        cmd_parts[0],
                        err
                    );
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    if !any_executed {
        println!(
            "  Nenhum executor de mutação aplicável para as linguagens declaradas ({:?}).",
            langs
        );
    }

    ExitCode::SUCCESS
}

/// Dispatches `cccp hygiene` / `cccp fix`
pub fn handle_hygiene(args: &HygieneArgs, dir: Option<&Path>, theme: &Theme) -> ExitCode {
    let root = dir.unwrap_or(Path::new("."));
    let langs = if let Some(ref l_str) = args.lang {
        l_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        get_gost_languages(root)
    };

    println!(
        "{} [GOSPLAN DD-TDD] Executando Fase 8 — Fast Hygiene & Quality Gate (modo: {}):",
        theme.gold_bold("★"),
        if args.fix {
            theme.green("auto-fix")
        } else {
            theme.cyan("verificação estrita")
        }
    );

    for lang in &langs {
        let commands = resolve_hygiene_commands(lang, args.fix);
        if commands.is_empty() {
            continue;
        }

        println!("  • Stack: {}", theme.green(lang));
        for cmd_parts in commands {
            println!("    Executando: {}", theme.yellow(&cmd_parts.join(" ")));

            let mut cmd = Command::new(cmd_parts[0]);
            cmd.args(&cmd_parts[1..]);
            if let Some(d) = dir {
                cmd.current_dir(d);
            }

            match cmd.status() {
                Ok(status) if status.success() => {
                    // sucesso
                }
                Ok(_) => {
                    eprintln!(
                        "{}: Comando de higiene '{}' reportou violações ou falhou.",
                        theme.red_bold("Portão de Qualidade Rejeitado"),
                        cmd_parts.join(" ")
                    );
                    return ExitCode::FAILURE;
                }
                Err(err) => {
                    eprintln!(
                        "{}: Falha ao executar utilitário '{}': {}",
                        theme.red_bold("Erro"),
                        cmd_parts[0],
                        err
                    );
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    println!(
        "✓ {} Todas as verificações de higiene passaram sem warnings!",
        theme.green("Concluído:")
    );
    ExitCode::SUCCESS
}
