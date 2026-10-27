//! Push manager with automatic origin tracking branch parity check and fast-forward reconciliation.

use crate::cli::PushArgs;
use crate::cultural::Theme;
use std::path::Path;
use std::process::{Command, ExitCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParityAction {
    ReadyToPush,
    FastForwarded,
    NoRemote,
    AlreadyUpToDate,
}

/// Checks parity with the upstream tracking branch of the current Git repository.
///
/// If local branch is behind origin and has no local divergence (ahead == 0),
/// automatically fast-forwards to reconcile without requiring manual git pull.
/// If divergent history is detected (ahead > 0 && behind > 0), returns an error
/// preventing push rejection.
pub fn check_and_reconcile_parity(repo_path: &Path, theme: &Theme) -> Result<ParityAction, String> {
    // 1. Get current branch name
    let branch_out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Falha ao obter branch atual: {}", e))?;

    if !branch_out.status.success() {
        return Err("Não foi possível identificar a branch atual".to_string());
    }

    let branch = String::from_utf8_lossy(&branch_out.stdout)
        .trim()
        .to_string();
    if branch == "HEAD" || branch.is_empty() {
        return Ok(ParityAction::ReadyToPush);
    }

    // 2. Resolve tracking upstream
    let upstream_out = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "@{u}"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Falha ao verificar upstream: {}", e))?;

    let upstream = if upstream_out.status.success() {
        String::from_utf8_lossy(&upstream_out.stdout)
            .trim()
            .to_string()
    } else {
        // No upstream tracking branch configured
        let remote_out = Command::new("git")
            .args(["remote"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Falha ao verificar remotes: {}", e))?;

        let remotes = String::from_utf8_lossy(&remote_out.stdout);
        if !remotes.lines().any(|r| r.trim() == "origin") {
            return Ok(ParityAction::NoRemote);
        }

        // Check if branch exists on origin
        let ls_out = Command::new("git")
            .args(["ls-remote", "--heads", "origin", &branch])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Falha ao verificar existência da branch no remote: {}", e))?;

        if !ls_out.status.success() || ls_out.stdout.is_empty() {
            return Ok(ParityAction::ReadyToPush);
        }

        format!("origin/{}", branch)
    };

    // 3. Extract remote name (e.g. "origin" from "origin/main")
    let parts: Vec<&str> = upstream.splitn(2, '/').collect();
    let (remote_name, remote_branch) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        ("origin", branch.as_str())
    };

    // 4. Fetch silently from remote
    let fetch_res = Command::new("git")
        .args(["fetch", remote_name, remote_branch])
        .current_dir(repo_path)
        .output()
        .map_err(|e| {
            format!(
                "Falha ao executar 'git fetch {} {}': {}",
                remote_name, remote_branch, e
            )
        })?;

    if !fetch_res.status.success() {
        let err_msg = String::from_utf8_lossy(&fetch_res.stderr);
        return Err(format!(
            "Falha ao sincronizar com remote '{}': {}",
            remote_name,
            err_msg.trim()
        ));
    }

    // 5. Compare commit counts: ahead and behind
    let count_out = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("HEAD...{}", upstream),
        ])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Falha ao calcular divergência com '{}': {}", upstream, e))?;

    if !count_out.status.success() {
        return Err(format!("Não foi possível comparar HEAD com '{}'", upstream));
    }

    let count_str = String::from_utf8_lossy(&count_out.stdout)
        .trim()
        .to_string();
    let parts: Vec<&str> = count_str.split_whitespace().collect();
    if parts.len() != 2 {
        return Ok(ParityAction::ReadyToPush);
    }

    let ahead: usize = parts[0].parse().unwrap_or(0);
    let behind: usize = parts[1].parse().unwrap_or(0);

    if behind == 0 {
        if ahead == 0 {
            Ok(ParityAction::AlreadyUpToDate)
        } else {
            Ok(ParityAction::ReadyToPush)
        }
    } else if ahead == 0 {
        println!(
            "{} Sincronizando automaticamente {} commit(s) de '{}' (fast-forward)...",
            theme.cyan("•"),
            behind,
            upstream
        );
        let ff_res = Command::new("git")
            .args(["merge", "--ff-only", &upstream])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Falha ao avançar branch via fast-forward: {}", e))?;

        if !ff_res.status.success() {
            let err = String::from_utf8_lossy(&ff_res.stderr);
            return Err(format!(
                "Falha ao reconciliar fast-forward com '{}': {}",
                upstream,
                err.trim()
            ));
        }

        println!(
            "{} Repositório local atualizado com sucesso.",
            theme.green("✓")
        );
        Ok(ParityAction::FastForwarded)
    } else {
        Err(format!(
            "Divergência detectada com o tracking branch '{}': {} commit(s) à frente e {} commit(s) atrás. Reconcilie o histórico (rebase ou merge) antes do envio.",
            upstream, ahead, behind
        ))
    }
}

/// Main entry point for `cccp push` command.
pub fn handle_push(push: &PushArgs, dir: Option<&Path>, theme: &Theme) -> ExitCode {
    let repo_dir = match dir {
        Some(d) => d.to_path_buf(),
        None => std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf()),
    };

    println!(
        "{} Inspecionando paridade com o tracking branch remoto...",
        theme.cyan("•")
    );

    match check_and_reconcile_parity(&repo_dir, theme) {
        Ok(ParityAction::AlreadyUpToDate) => {
            println!(
                "{} Tudo sincronizado com o remote (nada a enviar).",
                theme.green("✓")
            );
            return ExitCode::SUCCESS;
        }
        Ok(ParityAction::FastForwarded) => {
            println!(
                "{} Paridade reestabelecida via fast-forward.",
                theme.green("✓")
            );
        }
        Ok(ParityAction::ReadyToPush) => {
            println!(
                "{} Paridade confirmada. Pronto para envio.",
                theme.green("✓")
            );
        }
        Ok(ParityAction::NoRemote) => {
            println!(
                "{} Nenhum remote configurado para verificação de paridade.",
                theme.yellow("!")
            );
        }
        Err(e) => {
            eprintln!("\n{}", theme.red_bold("★ GOSPLAN PUSH ABORTADO ★"));
            eprintln!("{}\n", theme.red(&e));
            return ExitCode::FAILURE;
        }
    }

    let mut cmd = Command::new("git");
    cmd.arg("push");
    cmd.env("CCCP_MAESTRO_ACTIVE", "1");

    if push.dry_run {
        cmd.arg("--dry-run");
    }
    if !push.git_args.is_empty() {
        cmd.args(&push.git_args);
    }
    cmd.current_dir(&repo_dir);

    match cmd.status() {
        Ok(status) if status.success() => {
            println!(
                "{} Push concluído com sucesso via Maestro.",
                theme.green("✓")
            );
            ExitCode::SUCCESS
        }
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(e) => {
            eprintln!("Erro ao executar git push: {}", e);
            ExitCode::FAILURE
        }
    }
}
