//! Canonical release tagging, SemVer synchronization and CHANGELOG generator.

use crate::cultural::Theme;
use crate::git::GitRepo;
use crate::semver_calc::{calculate_version, write_version_file};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct ReleaseTagOptions {
    pub version: Option<String>,
    pub title: Option<String>,
    pub message: Option<String>,
    pub no_v: bool,
    pub sign: bool,
}

#[derive(Debug, Clone)]
pub struct ChangelogOptions {
    pub output: PathBuf,
    pub dry_run: bool,
    pub range: Option<String>,
}

/// Extrai mensagens de commits convencionais no range especificado
pub fn fetch_commits_in_range(
    repo_root: &Path,
    range: Option<&str>,
) -> Result<Vec<String>, String> {
    let mut cmd = Command::new("git");
    cmd.arg("log").arg("--format=%s").current_dir(repo_root);

    if let Some(r) = range {
        cmd.arg(r);
    } else {
        // Tenta obter da última tag anotada até HEAD
        let latest_tag_out = Command::new("git")
            .args(["describe", "--tags", "--abbrev=0"])
            .current_dir(repo_root)
            .output();

        if let Ok(out) = latest_tag_out {
            if out.status.success() {
                let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !tag.is_empty() {
                    cmd.arg(format!("{tag}..HEAD"));
                }
            }
        }
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Falha ao executar git log: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(lines)
}

/// Agrupa commits convencionais e formata a seção do CHANGELOG.md (Keep a Changelog)
pub fn generate_changelog(version: &str, date: &str, commits: &[impl AsRef<str>]) -> String {
    let mut feats = Vec::new();
    let mut fixes = Vec::new();
    let mut docs = Vec::new();
    let mut perfs = Vec::new();
    let mut refactors = Vec::new();
    let mut tests = Vec::new();
    let mut chores = Vec::new();

    for commit_line in commits {
        let line = commit_line.as_ref().trim();
        if line.is_empty() {
            continue;
        }

        let formatted = format_commit_entry(line);

        if line.starts_with("feat") {
            feats.push(formatted);
        } else if line.starts_with("fix") {
            fixes.push(formatted);
        } else if line.starts_with("docs") {
            docs.push(formatted);
        } else if line.starts_with("perf") {
            perfs.push(formatted);
        } else if line.starts_with("refactor") {
            refactors.push(formatted);
        } else if line.starts_with("test") {
            tests.push(formatted);
        } else {
            chores.push(formatted);
        }
    }

    let mut out = String::new();
    out.push_str(&format!("## [{version}] - {date}\n\n"));

    if !feats.is_empty() {
        out.push_str("### Features\n");
        for f in feats {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }

    if !fixes.is_empty() {
        out.push_str("### Bug Fixes\n");
        for f in fixes {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }

    if !perfs.is_empty() {
        out.push_str("### Performance\n");
        for f in perfs {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }

    if !refactors.is_empty() {
        out.push_str("### Refactoring\n");
        for f in refactors {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }

    if !docs.is_empty() {
        out.push_str("### Documentation\n");
        for f in docs {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }

    if !tests.is_empty() {
        out.push_str("### Testing\n");
        for f in tests {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }

    if !chores.is_empty() {
        out.push_str("### Maintenance\n");
        for f in chores {
            out.push_str(&format!("- {f}\n"));
        }
        out.push('\n');
    }

    out
}

fn format_commit_entry(line: &str) -> String {
    if let Some((prefix, desc)) = line.split_once(':') {
        let desc = desc.trim();
        if let Some(open) = prefix.find('(') {
            if let Some(close) = prefix.find(')') {
                let scope = &prefix[open + 1..close];
                return format!("**{scope}:** {desc}");
            }
        }
        return desc.to_string();
    }
    line.to_string()
}

/// Atualiza ou insere a seção no arquivo CHANGELOG.md de forma não-destrutiva
pub fn update_changelog_file(changelog_path: &Path, section_content: &str) -> Result<(), String> {
    let existing = if changelog_path.exists() {
        fs::read_to_string(changelog_path)
            .map_err(|e| format!("Falha ao ler {}: {e}", changelog_path.display()))?
    } else {
        "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\nand this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n".to_string()
    };

    let insertion_index = if let Some(pos) = existing.find("## [") {
        pos
    } else {
        existing.len()
    };

    let mut new_content = String::new();
    new_content.push_str(&existing[..insertion_index]);
    if !new_content.ends_with("\n\n") {
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push('\n');
    }
    new_content.push_str(section_content.trim_end());
    new_content.push_str("\n\n");
    new_content.push_str(&existing[insertion_index..]);

    fs::write(changelog_path, new_content.trim_start())
        .map_err(|e| format!("Falha ao escrever {}: {e}", changelog_path.display()))?;
    Ok(())
}

/// Executa a orquestração completa do subcomando cccp tag
pub fn execute_release_tag(
    repo_root: &Path,
    options: &ReleaseTagOptions,
) -> Result<String, String> {
    // 1. Determinar versão limpa
    let clean_version = if let Some(ref v) = options.version {
        v.trim_start_matches('v').trim().to_string()
    } else {
        let repo = GitRepo::open(Some(repo_root))
            .map_err(|e| format!("Não foi possível abrir repositório git: {e}"))?;
        let predicted = calculate_version(&repo, "0.1.0", options.no_v, None)?;
        predicted.target_version
    };

    if clean_version.contains("-dev") || clean_version.contains('+') {
        return Err(format!(
            "Versão inválida para release tag: '{clean_version}'. Não pode conter sufixos -dev ou metadados de build."
        ));
    }

    let tag_name = if options.no_v {
        clean_version.clone()
    } else {
        format!("v{clean_version}")
    };

    // 2. Escrever versão limpa no arquivo VERSION
    write_version_file(repo_root, &clean_version)
        .map_err(|e| format!("Falha ao escrever VERSION: {e}"))?;

    // 3. Gerar e atualizar CHANGELOG.md
    let today = {
        let out = Command::new("date").arg("+%Y-%m-%d").output();
        if let Ok(o) = out {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        } else {
            "2026-10-14".to_string()
        }
    };

    let commits = fetch_commits_in_range(repo_root, None).unwrap_or_default();
    let changelog_section = generate_changelog(&clean_version, &today, &commits);
    let changelog_file = repo_root.join("CHANGELOG.md");
    let _ = update_changelog_file(&changelog_file, &changelog_section);

    // 4. Git add & commit
    let mut add_cmd = Command::new("git");
    add_cmd.args(["add", "VERSION"]).current_dir(repo_root);
    if changelog_file.exists() {
        add_cmd.arg("CHANGELOG.md");
    }
    let add_status = add_cmd
        .status()
        .map_err(|e| format!("git add failed: {e}"))?;
    if !add_status.success() {
        return Err("Falha ao adicionar arquivos de release no git".to_string());
    }

    let commit_msg = if let Some(ref t) = options.title {
        format!("chore(release): v{clean_version} - {t}")
    } else {
        format!("chore(release): v{clean_version}")
    };

    let commit_status = Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(repo_root)
        .status()
        .map_err(|e| format!("git commit failed: {e}"))?;
    if !commit_status.success() {
        return Err("Falha ao criar commit de release".to_string());
    }

    // 5. Git tag
    let mut tag_cmd = Command::new("git");
    tag_cmd.arg("tag");

    let has_signing_key = {
        let key_out = Command::new("git")
            .args(["config", "user.signingkey"])
            .current_dir(repo_root)
            .output();
        key_out
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false)
    };

    if options.sign || has_signing_key {
        tag_cmd.arg("-s");
    } else {
        tag_cmd.arg("-a");
    }

    tag_cmd.arg(&tag_name);

    let annotation = if let Some(ref m) = options.message {
        m.clone()
    } else if let Some(ref t) = options.title {
        format!("Release {tag_name}: {t}")
    } else {
        format!("Release {tag_name}")
    };

    tag_cmd.args(["-m", &annotation]);
    tag_cmd.current_dir(repo_root);

    let tag_status = tag_cmd
        .status()
        .map_err(|e| format!("git tag failed: {e}"))?;
    if !tag_status.success() {
        return Err(format!("Falha ao criar git tag '{tag_name}'"));
    }

    Ok(tag_name)
}

/// Handler CLI para cccp changelog
pub fn cmd_changelog(
    repo_root: &Path,
    options: &ChangelogOptions,
    theme: &Theme,
) -> Result<(), String> {
    let commits = fetch_commits_in_range(repo_root, options.range.as_deref())?;
    let today = {
        let out = Command::new("date").arg("+%Y-%m-%d").output();
        if let Ok(o) = out {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        } else {
            "2026-10-14".to_string()
        }
    };

    let target_version = {
        let repo = GitRepo::open(Some(repo_root)).ok();
        repo.and_then(|r| calculate_version(&r, "0.1.0", false, None).ok())
            .map(|p| p.target_version)
            .unwrap_or_else(|| "Unreleased".to_string())
    };

    let content = generate_changelog(&target_version, &today, &commits);

    if options.dry_run {
        println!("{}", theme.bold("★ CHANGELOG PREVIEW ★\n"));
        println!("{content}");
    } else {
        let dest = repo_root.join(&options.output);
        update_changelog_file(&dest, &content)?;
        println!(
            "{} Changelog gravado com sucesso em: {}",
            theme.green("✓"),
            dest.display()
        );
    }

    Ok(())
}
