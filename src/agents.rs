//! CCCP Agent Directives & Variable Interpolation Engine (`cmd_agent`)
//! Sychronizes `AGENTS.md` from canonical template, extracts `{{VAR}}` directives,
//! and manages agent configuration variables in `.cccprc` and environment.

use crate::cultural::Theme;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

pub const DEFAULT_AGENTS_MD_URL: &str =
    "https://codeberg.org/lumenpink/cccp/raw/branch/main/src/base/AGENTS_template.md";

/// Fetches agent template from local path, URL, or fallback repository path
pub fn fetch_agents_template(url: &str, repo_root: Option<&Path>) -> Result<String, String> {
    if let Some(stripped) = url.strip_prefix("file://") {
        return fs::read_to_string(stripped)
            .map_err(|e| format!("Failed to read local template file '{}': {}", stripped, e));
    }

    if Path::new(url).is_file() {
        return fs::read_to_string(url)
            .map_err(|e| format!("Failed to read local template file '{}': {}", url, e));
    }

    if url.starts_with("http://") || url.starts_with("https://") {
        let resp = ureq::get(url)
            .call()
            .map_err(|e| format!("Failed to fetch template from '{}': {}", url, e))?;

        let body = resp
            .into_body()
            .read_to_string()
            .map_err(|e| format!("Failed to read response from '{}': {}", url, e))?;

        if body.trim().is_empty() {
            return Err(format!("Received empty response from '{}'.", url));
        }

        let lower = body.to_ascii_lowercase();
        if lower.contains("<html")
            || lower.contains("<!doctype")
            || lower.contains("<head")
            || lower.contains("<body")
        {
            return Err(format!(
                "Response from '{}' does not appear to be a valid Markdown template (looks like HTML).",
                url
            ));
        }

        return Ok(body);
    }

    // Fallback: check repository src/base/AGENTS_template.md
    if let Some(root) = repo_root {
        let local_fallback = root.join("src/base/AGENTS_template.md");
        if local_fallback.is_file() {
            return fs::read_to_string(&local_fallback).map_err(|e| {
                format!(
                    "Failed to read local fallback template '{}': {}",
                    local_fallback.display(),
                    e
                )
            });
        }
    }

    Err(format!(
        "Template '{}' not found locally and cannot be fetched.",
        url
    ))
}

/// Extracts unique variable names from template text (matching `{{VAR}}` and `${VAR}`)
pub fn extract_template_vars(template_text: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let bytes = template_text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Check for {{VAR}}
        if i + 3 < len && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            let start = i + 2;
            let mut end = start;
            while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end + 1 < len && bytes[end] == b'}' && bytes[end + 1] == b'}' {
                let var_name = &template_text[start..end];
                if !var_name.is_empty() && !vars.iter().any(|v| v == var_name) {
                    vars.push(var_name.to_string());
                }
                i = end + 2;
                continue;
            }
        }

        // Check for ${VAR}
        if i + 2 < len && bytes[i] == b'$' && bytes[i + 1] == b'{' {
            let start = i + 2;
            let mut end = start;
            while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end < len && bytes[end] == b'}' {
                let var_name = &template_text[start..end];
                if !var_name.is_empty() && !vars.iter().any(|v| v == var_name) {
                    vars.push(var_name.to_string());
                }
                i = end + 1;
                continue;
            }
        }

        i += 1;
    }

    vars.sort();
    vars
}

/// Resolves variable value checking environment variables, local `.cccprc`, and global `~/.cccprc`
pub fn resolve_agent_var(var_name: &str, repo_root: Option<&Path>) -> Option<String> {
    let norm = var_name
        .to_ascii_lowercase()
        .replace('-', "_");
    let stripped = norm
        .strip_prefix("agents_")
        .or_else(|| norm.strip_prefix("agent_"))
        .unwrap_or(&norm);

    let candidates = vec![
        norm.clone(),
        stripped.to_string(),
        format!("agent_{}", stripped),
        format!("agents_{}", stripped),
    ];

    // 1. Environment variables (AGENT_VAR_*)
    for cand in &candidates {
        let upper = cand.to_ascii_uppercase();
        let env_key = format!("AGENT_VAR_{}", upper);
        if let Ok(val) = std::env::var(&env_key) {
            if !val.is_empty() {
                return Some(val);
            }
        }
        let env_key_exact = format!("AGENT_VAR_{}", cand);
        if let Ok(val) = std::env::var(&env_key_exact) {
            if !val.is_empty() {
                return Some(val);
            }
        }
    }

    // 2. Local .cccprc
    if let Some(root) = repo_root {
        let local_cfg = root.join(".cccprc");
        if local_cfg.is_file() {
            if let Ok(content) = fs::read_to_string(&local_cfg) {
                if let Some(val) = find_key_in_config(&content, &candidates) {
                    return Some(val);
                }
            }
        }
    }

    // 3. Global ~/.cccprc
    if let Ok(home) = std::env::var("HOME") {
        let global_cfg = PathBuf::from(home).join(".cccprc");
        if global_cfg.is_file() {
            if let Ok(content) = fs::read_to_string(&global_cfg) {
                if let Some(val) = find_key_in_config(&content, &candidates) {
                    return Some(val);
                }
            }
        }
    }

    None
}

fn find_key_in_config(content: &str, candidates: &[String]) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if let Some((key_raw, val_raw)) = trimmed.split_once('=') {
            let key = key_raw.trim().to_ascii_lowercase().replace('-', "_");
            let clean_val = val_raw.trim().trim_matches('"').trim_matches('\'').to_string();

            for cand in candidates {
                if key == *cand || key == format!("agent_var_{}", cand) {
                    return Some(clean_val);
                }
            }
        }
    }
    None
}

/// Configures an agent variable in `.cccprc`
pub fn write_agent_var(key: &str, val: &str, repo_root: &Path) -> Result<(String, String), String> {
    let local_cfg = repo_root.join(".cccprc");
    let norm_key = key
        .strip_prefix("agent_var_")
        .unwrap_or(key)
        .to_ascii_lowercase()
        .replace('-', "_");

    let (target_key, display_key) = match norm_key.as_str() {
        "url" | "template_url" | "agent_url" | "agents_url" | "agent_md_url" | "agents_md_url" => {
            ("agent_md_url".to_string(), "agent_md_url".to_string())
        }
        "file" | "dest" | "destination" | "agent_file" | "agents_file" | "agent_md_file"
        | "agents_md_file" => ("agent_md_file".to_string(), "agent_md_file".to_string()),
        _ => (format!("agent_var_{}", norm_key), norm_key.clone()),
    };

    let mut lines = Vec::new();
    let mut updated = false;

    if local_cfg.is_file() {
        let content = fs::read_to_string(&local_cfg)
            .map_err(|e| format!("Failed to read '{}': {}", local_cfg.display(), e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with(';') {
                lines.push(line.to_string());
                continue;
            }
            if let Some((k, _)) = trimmed.split_once('=') {
                let clean_k = k.trim().to_ascii_lowercase().replace('-', "_");
                if clean_k == target_key || (target_key == "agent_md_url" && clean_k == "agent_var_url") {
                    if !updated {
                        lines.push(format!("{} = {}", target_key, val));
                        updated = true;
                    }
                    continue;
                }
            }
            lines.push(line.to_string());
        }
    }

    if !updated {
        lines.push(format!("{} = {}", target_key, val));
    }

    fs::write(&local_cfg, lines.join("\n") + "\n")
        .map_err(|e| format!("Failed to write '{}': {}", local_cfg.display(), e))?;

    Ok((target_key, display_key))
}

/// Unsets an agent variable in `.cccprc`
pub fn unset_agent_var(key: &str, repo_root: &Path) -> Result<String, String> {
    let local_cfg = repo_root.join(".cccprc");
    if !local_cfg.is_file() {
        return Ok(key.to_string());
    }

    let norm_key = key
        .strip_prefix("agent_var_")
        .unwrap_or(key)
        .to_ascii_lowercase()
        .replace('-', "_");
    let target_key = format!("agent_var_{}", norm_key);

    let content = fs::read_to_string(&local_cfg)
        .map_err(|e| format!("Failed to read '{}': {}", local_cfg.display(), e))?;

    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some((k, _)) = trimmed.split_once('=') {
            let clean_k = k.trim().to_ascii_lowercase().replace('-', "_");
            if clean_k == target_key || clean_k == norm_key {
                continue;
            }
        }
        lines.push(line.to_string());
    }

    fs::write(&local_cfg, lines.join("\n") + "\n")
        .map_err(|e| format!("Failed to write '{}': {}", local_cfg.display(), e))?;

    Ok(norm_key)
}

/// Ensures `.antigravity/` is listed in `.gitignore`
pub fn ensure_agent_gitignored(repo_root: &Path, target_file: &Path) {
    let gitignore = repo_root.join(".gitignore");
    let target_str = target_file.to_string_lossy();

    let entry = if target_str.contains(".antigravity") {
        ".antigravity/"
    } else {
        return;
    };

    if gitignore.is_file() {
        if let Ok(content) = fs::read_to_string(&gitignore) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == entry || trimmed == ".antigravity" {
                    return;
                }
            }
            let mut new_content = content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(entry);
            new_content.push('\n');
            let _ = fs::write(&gitignore, new_content);
            return;
        }
    }

    let _ = fs::write(&gitignore, format!("{}\n", entry));
}

/// Resolves the configured template URL
pub fn resolve_template_url(cli_url: Option<&str>, repo_root: Option<&Path>) -> String {
    if let Some(u) = cli_url {
        return u.to_string();
    }

    if let Some(root) = repo_root {
        let local_cfg = root.join(".cccprc");
        if local_cfg.is_file() {
            if let Ok(content) = fs::read_to_string(&local_cfg) {
                if let Some(u) = find_key_in_config(
                    &content,
                    &[
                        "agent_md_url".to_string(),
                        "agents_md_url".to_string(),
                        "agent_url".to_string(),
                    ],
                ) {
                    return u;
                }
            }
        }
    }

    if let Ok(val) = std::env::var("AGENT_MD_URL").or_else(|_| std::env::var("AGENTS_MD_URL")) {
        if !val.is_empty() {
            return val;
        }
    }

    DEFAULT_AGENTS_MD_URL.to_string()
}

/// Resolves the configured output file path
pub fn resolve_output_file(cli_file: Option<&str>, repo_root: Option<&Path>) -> PathBuf {
    if let Some(f) = cli_file {
        if let Some(root) = repo_root {
            return root.join(f);
        }
        return PathBuf::from(f);
    }

    if let Some(root) = repo_root {
        let local_cfg = root.join(".cccprc");
        if local_cfg.is_file() {
            if let Ok(content) = fs::read_to_string(&local_cfg) {
                if let Some(f) = find_key_in_config(
                    &content,
                    &["agent_md_file".to_string(), "agents_md_file".to_string()],
                ) {
                    return root.join(f);
                }
            }
        }
    }

    if let Ok(val) = std::env::var("AGENT_MD_FILE").or_else(|_| std::env::var("AGENTS_MD_FILE")) {
        if !val.is_empty() {
            if let Some(root) = repo_root {
                return root.join(&val);
            }
            return PathBuf::from(val);
        }
    }

    if let Some(root) = repo_root {
        root.join(".antigravity/AGENTS.md")
    } else {
        PathBuf::from(".antigravity/AGENTS.md")
    }
}

/// Subcommand: cccp agent check
pub fn cmd_agent_check(
    cli_url: Option<&str>,
    cli_file: Option<&str>,
    repo_root: Option<&Path>,
    theme: &Theme,
) -> Result<bool, String> {
    let target_url = resolve_template_url(cli_url, repo_root);
    let target_file = resolve_output_file(cli_file, repo_root);

    println!("\nTemplate: {}", target_url);
    println!("Destination: {}\n", target_file.display());

    let template_content = fetch_agents_template(&target_url, repo_root)?;
    let vars = extract_template_vars(&template_content);

    if vars.is_empty() {
        println!("No customizable variables detected in template.");
        return Ok(true);
    }

    println!("Detected Variables:");
    let mut missing_count = 0;
    for v in &vars {
        let norm_v = v.to_ascii_lowercase().replace('-', "_");
        let stripped_v = norm_v
            .strip_prefix("agents_")
            .or_else(|| norm_v.strip_prefix("agent_"))
            .unwrap_or(&norm_v);

        if let Some(val) = resolve_agent_var(v, repo_root) {
            println!("  {} {} = {}", theme.green("[✓]"), v, val);
        } else {
            println!(
                "  {} {} = (missing - configure via: cccp agent set {} \"<val>\")",
                theme.red("[✗]"),
                v,
                stripped_v
            );
            missing_count += 1;
        }
    }

    println!();
    if missing_count > 0 {
        println!(
            "Status: {} variable(s) missing. Cannot generate {}.",
            missing_count,
            target_file.display()
        );
        Ok(false)
    } else {
        println!("Status: All variables configured! Ready to run 'cccp agent'.");
        Ok(true)
    }
}

/// Subcommand: cccp agent update (default)
pub fn cmd_agent_update(
    cli_url: Option<&str>,
    cli_file: Option<&str>,
    repo_root: Option<&Path>,
    _theme: &Theme,
) -> Result<(), String> {
    let target_url = resolve_template_url(cli_url, repo_root);
    let target_file = resolve_output_file(cli_file, repo_root);

    // If explicit --url provided, persist it to local .cccprc
    if cli_url.is_some() {
        if let Some(root) = repo_root {
            let _ = write_agent_var("agent_md_url", &target_url, root);
        }
    }

    println!("Fetching AGENTS template from: {}", target_url);
    let template_content = fetch_agents_template(&target_url, repo_root)?;
    let vars = extract_template_vars(&template_content);

    let mut missing_list = String::new();
    let mut missing_count = 0;
    for v in &vars {
        if resolve_agent_var(v, repo_root).is_none() {
            let norm_v = v.to_ascii_lowercase().replace('-', "_");
            let stripped_v = norm_v
                .strip_prefix("agents_")
                .or_else(|| norm_v.strip_prefix("agent_"))
                .unwrap_or(&norm_v);
            missing_list.push_str(&format!(
                "\n  - {} (configure with: cccp agent set {} \"<value>\")",
                stripped_v, stripped_v
            ));
            missing_count += 1;
        }
    }

    let mut result = template_content;
    let mut interpolated_count = 0;
    for v in &vars {
        if let Some(val) = resolve_agent_var(v, repo_root) {
            let m_tag = format!("{{{{{}}}}}", v);
            let s_tag = format!("${{{}}}", v);
            result = result.replace(&m_tag, &val);
            result = result.replace(&s_tag, &val);
            interpolated_count += 1;
        }
    }

    if let Some(parent) = target_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
    }

    let tmp_out = target_file.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&tmp_out, &result)
        .map_err(|e| format!("Failed to write temporary file '{}': {}", tmp_out.display(), e))?;
    fs::rename(&tmp_out, &target_file)
        .map_err(|e| format!("Failed to save target file '{}': {}", target_file.display(), e))?;

    if let Some(root) = repo_root {
        ensure_agent_gitignored(root, &target_file);
    }

    println!(
        "Done! Generated {} with {} variable(s) substituted.",
        target_file.display(),
        interpolated_count
    );

    if missing_count > 0 {
        eprintln!(
            "Warning: {} variable(s) missing in {}:{}",
            missing_count,
            target_file.display(),
            missing_list
        );
        eprintln!("Run 'cccp agent fill' to interactively provide missing variables.");
    }

    Ok(())
}

/// Subcommand: cccp agent set <key> <val>
pub fn cmd_agent_set(key: &str, val: &str, repo_root: Option<&Path>) -> Result<(), String> {
    let root = repo_root.ok_or_else(|| {
        "Error: Not inside a git repository. Cannot save to local .cccprc.".to_string()
    })?;

    let (target_key, display_key) = write_agent_var(key, val, root)?;
    let local_cfg = root.join(".cccprc");

    if target_key == "agent_md_url" {
        println!(
            "Configured agent template URL 'agent_md_url' = '{}' in {}",
            val,
            local_cfg.display()
        );
    } else if target_key == "agent_md_file" {
        println!(
            "Configured agent destination file 'agent_md_file' = '{}' in {}",
            val,
            local_cfg.display()
        );
    } else {
        println!(
            "Configured agent variable '{}' = '{}' in {}",
            display_key,
            val,
            local_cfg.display()
        );
    }

    Ok(())
}

/// Subcommand: cccp agent get <key>
pub fn cmd_agent_get(key: &str, repo_root: Option<&Path>) -> Result<(), String> {
    let norm = key
        .strip_prefix("agent_var_")
        .unwrap_or(key)
        .to_ascii_lowercase()
        .replace('-', "_");

    match resolve_agent_var(&norm, repo_root) {
        Some(val) => {
            println!("{}", val);
            Ok(())
        }
        None => Err(format!("Error: Variable '{}' is not set.", norm)),
    }
}

/// Subcommand: cccp agent unset <key>
pub fn cmd_agent_unset(key: &str, repo_root: Option<&Path>) -> Result<(), String> {
    let root = repo_root.ok_or_else(|| "Error: Not inside a git repository.".to_string())?;

    let unset_key = unset_agent_var(key, root)?;
    let local_cfg = root.join(".cccprc");
    println!(
        "Unset agent variable '{}' from {}",
        unset_key,
        local_cfg.display()
    );
    Ok(())
}

/// Subcommand: cccp agent fill
pub fn cmd_agent_fill(
    cli_url: Option<&str>,
    cli_file: Option<&str>,
    repo_root: Option<&Path>,
    theme: &Theme,
) -> Result<(), String> {
    let root = repo_root.ok_or_else(|| {
        "Error: Not inside a git repository. Cannot save to local .cccprc.".to_string()
    })?;

    let target_url = resolve_template_url(cli_url, repo_root);
    let target_file = resolve_output_file(cli_file, repo_root);

    let template_content = fetch_agents_template(&target_url, repo_root)?;
    let vars = extract_template_vars(&template_content);

    if vars.is_empty() {
        println!("No customizable variables detected in template. Nothing to fill.");
        return Ok(());
    }

    let mut filled_count = 0;
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();

    for v in &vars {
        let norm_v = v.to_ascii_lowercase().replace('-', "_");
        let stripped_v = norm_v
            .strip_prefix("agents_")
            .or_else(|| norm_v.strip_prefix("agent_"))
            .unwrap_or(&norm_v);

        if let Some(existing) = resolve_agent_var(v, repo_root) {
            println!("  [✓] {} = {}  (already set, skipping)", stripped_v, existing);
            continue;
        }

        let mut val = String::new();
        while val.trim().is_empty() {
            print!("  {}: ", stripped_v);
            let _ = std::io::stdout().flush();
            val.clear();
            if reader.read_line(&mut val).is_err() {
                break;
            }
            if val.trim().is_empty() {
                println!("    ↳ Value cannot be empty.");
            }
        }

        let _ = write_agent_var(stripped_v, val.trim(), root);
        filled_count += 1;
    }

    if filled_count > 0 {
        println!(
            "\nSaved {} variable(s). Regenerating {}...",
            filled_count,
            target_file.display()
        );
        cmd_agent_update(cli_url, cli_file, repo_root, theme)?;
    } else {
        println!("No missing variables — already complete. Run 'cccp agent' to regenerate.");
    }

    Ok(())
}
