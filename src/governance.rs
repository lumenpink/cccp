//! Persistent Epic Integration Branches and Border Quarantine Governance
//! Implements Regra GOST-25 and Section 15 of GOST 2.0 / Kollontai 2.0.

use crate::cultural::Theme;
use crate::git::GitRepo;
use std::process::Command;

/// Starts a persistent epic integration branch (epic/<slug>)
pub fn cmd_epic_start(repo: &GitRepo, slug: &str, theme: &Theme) -> Result<(), String> {
    let clean_slug = slug.trim();
    if clean_slug.is_empty() {
        return Err("O slug do épico não pode estar vazio".to_string());
    }

    if !clean_slug
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "Slug de épico inválido '{}': utilize apenas letras, números, hífens ou underscores",
            clean_slug
        ));
    }

    let branch_name = if clean_slug.starts_with("epic/") {
        clean_slug.to_string()
    } else {
        format!("epic/{}", clean_slug)
    };

    // Check if branch already exists locally
    let list_out = Command::new("git")
        .args(["branch", "--list", &branch_name])
        .current_dir(&repo.root_dir)
        .output()
        .map_err(|e| format!("Falha ao verificar branches: {}", e))?;

    let exists = !list_out.stdout.is_empty();

    if exists {
        let status = Command::new("git")
            .args(["checkout", &branch_name])
            .current_dir(&repo.root_dir)
            .status()
            .map_err(|e| format!("Falha ao alternar para branch épica: {}", e))?;

        if !status.success() {
            return Err(format!("Falha ao alternar para branch '{}'", branch_name));
        }
    } else {
        let status = Command::new("git")
            .args(["checkout", "-b", &branch_name])
            .current_dir(&repo.root_dir)
            .status()
            .map_err(|e| format!("Falha ao criar branch épica: {}", e))?;

        if !status.success() {
            return Err(format!("Falha ao criar branch épica '{}'", branch_name));
        }
    }

    println!(
        "{} Branch épica '{}' ativada com sucesso.",
        theme.green("★ [GOSPLAN]"),
        branch_name
    );

    Ok(())
}

/// Lists all active persistent epic integration branches (epic/*)
pub fn cmd_epic_list(repo: &GitRepo, theme: &Theme) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["branch", "--list", "epic/*"])
        .current_dir(&repo.root_dir)
        .output()
        .map_err(|e| format!("Falha ao listar branches épicas: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut epics = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim().trim_start_matches('*').trim();
        if !trimmed.is_empty() {
            epics.push(trimmed.to_string());
        }
    }

    if epics.is_empty() {
        println!("{} Nenhuma branch épica ativa encontrada.", theme.dim("○"));
    } else {
        println!("{} Branches épicas ativas:", theme.cyan("★ [GOSPLAN]"));
        for epic in &epics {
            println!("  • {}", theme.bold(epic));
        }
    }

    Ok(epics)
}

/// Finishes a persistent epic integration branch and integrates it into base
pub fn cmd_epic_finish(repo: &GitRepo, slug: &str, theme: &Theme) -> Result<(), String> {
    let clean_slug = slug.trim();
    let branch_name = if clean_slug.starts_with("epic/") {
        clean_slug.to_string()
    } else {
        format!("epic/{}", clean_slug)
    };

    // Check clean working tree
    let status_out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo.root_dir)
        .output()
        .map_err(|e| format!("Falha ao verificar status: {}", e))?;

    if !status_out.stdout.is_empty() {
        return Err("Working tree possui modificações pendentes. Salve ou descarte as alterações antes de finalizar o épico.".to_string());
    }

    // Checkout base branch (main)
    let checkout = Command::new("git")
        .args(["checkout", "main"])
        .current_dir(&repo.root_dir)
        .status()
        .map_err(|e| format!("Falha ao alternar para branch main: {}", e))?;

    if !checkout.success() {
        return Err("Falha ao alternar para a branch 'main'".to_string());
    }

    // Merge epic branch
    let merge = Command::new("git")
        .args(["merge", &branch_name])
        .current_dir(&repo.root_dir)
        .status()
        .map_err(|e| format!("Falha ao mesclar branch épica: {}", e))?;

    if !merge.success() {
        return Err(format!(
            "Falha ao mesclar branch épica '{}' na main",
            branch_name
        ));
    }

    println!(
        "{} Épico '{}' finalizado e integrado à main com sucesso.",
        theme.green("✓ [GOSPLAN]"),
        branch_name
    );

    Ok(())
}

/// Lists all branches quarantined under refs/heads/inbox/* or pr/*
pub fn cmd_inbox_list(repo: &GitRepo, theme: &Theme) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["branch", "--list", "inbox/*"])
        .current_dir(&repo.root_dir)
        .output()
        .map_err(|e| format!("Falha ao listar quarentena: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim().trim_start_matches('*').trim();
        if !trimmed.is_empty() {
            items.push(trimmed.to_string());
        }
    }

    if items.is_empty() {
        println!(
            "{} Nenhuma contribuição em quarentena (inbox/*).",
            theme.dim("○")
        );
    } else {
        println!(
            "{} Contribuições em quarentena de fronteira:",
            theme.cyan("★ [INBOX]")
        );
        for item in &items {
            println!("  • {}", theme.bold(item));
        }
    }

    Ok(items)
}

/// Reviews quarantined contribution diff and commits
pub fn cmd_inbox_review(repo: &GitRepo, id: &str, theme: &Theme) -> Result<String, String> {
    let clean_id = id.trim();
    let branch_name = if clean_id.starts_with("inbox/") {
        clean_id.to_string()
    } else {
        format!("inbox/{}", clean_id)
    };

    let log_out = Command::new("git")
        .args(["log", "-n", "5", "--oneline", &branch_name])
        .current_dir(&repo.root_dir)
        .output()
        .map_err(|e| format!("Falha ao inspecionar commits de '{}': {}", branch_name, e))?;

    if !log_out.status.success() {
        return Err(format!(
            "Branch de quarentena '{}' não encontrada",
            branch_name
        ));
    }

    let diff_out = Command::new("git")
        .args(["diff", "--stat", "main...", &branch_name])
        .current_dir(&repo.root_dir)
        .output()
        .map_err(|e| format!("Falha ao inspecionar diff de '{}': {}", branch_name, e))?;

    let log_str = String::from_utf8_lossy(&log_out.stdout);
    let diff_str = String::from_utf8_lossy(&diff_out.stdout);

    let summary = format!(
        "Revisão de Quarentena: {}\n\nCommits Recentes:\n{}\nDiff Summary:\n{}",
        branch_name, log_str, diff_str
    );

    println!("{}", theme.cyan(&summary));

    Ok(summary)
}

/// Accepts quarantined contribution into the active branch and enforces GOST-25 pushurl safety
pub fn cmd_inbox_accept(repo: &GitRepo, id: &str, theme: &Theme) -> Result<(), String> {
    let clean_id = id.trim();
    let branch_name = if clean_id.starts_with("inbox/") {
        clean_id.to_string()
    } else {
        format!("inbox/{}", clean_id)
    };

    // 1. Enforce GOST-25: Ensure mirrors have pushurl = no_push
    enforce_mirror_pushurl_safety(repo)?;

    // 2. Merge quarantined branch
    let merge_status = Command::new("git")
        .args(["merge", &branch_name])
        .current_dir(&repo.root_dir)
        .status()
        .map_err(|e| format!("Falha ao mesclar quarentena: {}", e))?;

    if !merge_status.success() {
        return Err(format!(
            "Falha ao mesclar '{}' na branch ativa",
            branch_name
        ));
    }

    println!(
        "{} Contribuição '{}' aceita e integrada com sucesso.",
        theme.green("✓ [INBOX]"),
        branch_name
    );

    Ok(())
}

/// Enforces GOST-25 remote safety: sets pushurl = no_push on secondary mirrors (codeberg, github)
pub fn enforce_mirror_pushurl_safety(repo: &GitRepo) -> Result<(), String> {
    let remotes_out = Command::new("git")
        .args(["remote"])
        .current_dir(&repo.root_dir)
        .output()
        .map_err(|e| format!("Falha ao listar remotes: {}", e))?;

    let remotes_str = String::from_utf8_lossy(&remotes_out.stdout);
    for remote in remotes_str
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if remote == "codeberg" || remote == "github" || remote == "mirror" {
            let _ = Command::new("git")
                .args(["config", &format!("remote.{}.pushurl", remote), "no_push"])
                .current_dir(&repo.root_dir)
                .status();
        }
    }

    Ok(())
}
