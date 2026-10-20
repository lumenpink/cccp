//! CCCP Update and Version Verification Bureau.
//! Handles remote version querying, SemVer comparisons, hook version auditing,
//! non-blocking auto-update checks, and atomic binary self-updating.

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::cli::UpdateArgs;
use crate::config::{self, Config};

/// Returns the currently running CCCP version string.
/// Respects `CCCP_VERSION` environment variable override if present,
/// otherwise returns compile-time `CARGO_PKG_VERSION`.
pub fn get_current_version() -> String {
    env::var("CCCP_VERSION")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

/// Returns path to the update cache file.
/// Respects `XDG_CACHE_HOME` or falls back to `$HOME/.cache/cccp/update_cache`.
pub fn get_update_cache_file() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_CACHE_HOME") {
        if !xdg.trim().is_empty() {
            return PathBuf::from(xdg).join("cccp").join("update_cache");
        }
    }
    if let Ok(home) = env::var("HOME") {
        if !home.trim().is_empty() {
            return PathBuf::from(home)
                .join(".cache")
                .join("cccp")
                .join("update_cache");
        }
    }
    PathBuf::from(".cache/cccp/update_cache")
}

/// Returns the official Banca distribution endpoints: (stable_url, nightly_url, version_url)
pub fn get_distribution_endpoints(channel: &str) -> (String, String, String) {
    let stable = "https://banca.2lp.in/downloads/cccp/cccp-stable".to_string();
    let nightly = "https://banca.2lp.in/downloads/cccp/cccp-nightly".to_string();
    let ver = if channel == "nightly" {
        "https://banca.2lp.in/downloads/cccp/nightly-version".to_string()
    } else {
        "https://banca.2lp.in/downloads/cccp/version".to_string()
    };
    (stable, nightly, ver)
}

/// Helper: resolve distribution channel taking into account explicit argument, environment, and config
pub fn resolve_channel(channel_override: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let cfg = config::Config::load_hierarchical(None);
    let raw = if let Some(ch) = channel_override {
        ch.to_string()
    } else if let Ok(v) = env::var("UPDATE_CHANNEL").or_else(|_| env::var("CCCP_UPDATE_CHANNEL")) {
        v.trim().to_string()
    } else {
        cfg.channel
    };
    let clean = raw.trim().to_ascii_lowercase();
    config::Config::validate_channel(&clean)?;
    Ok(clean)
}

/// Compares two version strings according to SemVer precedence.
/// Returns true if `remote` is newer than `current`.
pub fn semver_is_newer(remote: &str, current: &str) -> bool {
    let r_clean = remote
        .trim()
        .strip_prefix('v')
        .or_else(|| remote.trim().strip_prefix('V'))
        .unwrap_or(remote.trim());
    let c_clean = current
        .trim()
        .strip_prefix('v')
        .or_else(|| current.trim().strip_prefix('V'))
        .unwrap_or(current.trim());

    if let (Ok(rv), Ok(cv)) = (
        semver::Version::parse(r_clean),
        semver::Version::parse(c_clean),
    ) {
        return rv > cv;
    }

    // Fallback parser matching shell implementation
    let r_base = r_clean
        .split('-')
        .next()
        .unwrap_or("")
        .split('+')
        .next()
        .unwrap_or("");
    let c_base = c_clean
        .split('-')
        .next()
        .unwrap_or("")
        .split('+')
        .next()
        .unwrap_or("");

    let parse_digits = |s: &str| -> u64 {
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.parse::<u64>().unwrap_or(0)
    };

    let mut r_parts = r_base.split('.');
    let r_maj = parse_digits(r_parts.next().unwrap_or("0"));
    let r_min = parse_digits(r_parts.next().unwrap_or("0"));
    let r_pat = parse_digits(r_parts.next().unwrap_or("0"));

    let mut c_parts = c_base.split('.');
    let c_maj = parse_digits(c_parts.next().unwrap_or("0"));
    let c_min = parse_digits(c_parts.next().unwrap_or("0"));
    let c_pat = parse_digits(c_parts.next().unwrap_or("0"));

    if r_maj > c_maj {
        return true;
    }
    if r_maj < c_maj {
        return false;
    }
    if r_min > c_min {
        return true;
    }
    if r_min < c_min {
        return false;
    }
    if r_pat > c_pat {
        return true;
    }
    if r_pat < c_pat {
        return false;
    }

    if c_clean.contains('-') && !r_clean.contains('-') {
        return true;
    }

    false
}

/// Checks installed Git hooks in `repo_dir`, auto-updates unmodified hooks,
/// and warns only if a locally customized hook has a divergent version.
pub fn check_hook_version(repo_dir: &Path) -> Vec<String> {
    use crate::hooks_install::{
        compute_hook_checksum, generate_hook_stub, make_executable, CURRENT_HOOK_TEMPLATE_VERSION,
    };

    let mut warnings = Vec::new();
    let hooks_dir = repo_dir.join(".git").join("hooks");
    if !hooks_dir.is_dir() {
        return warnings;
    }

    let curr_ver = get_current_version();
    for hook_name in &["commit-msg", "post-commit"] {
        let hook_file = hooks_dir.join(hook_name);
        if !hook_file.is_file() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&hook_file) {
            let mut hook_ver = String::new();
            let mut template_ver: Option<u32> = None;
            let mut recorded_checksum = String::new();

            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("# cccp-hook-version:") {
                    hook_ver = rest.trim().to_string();
                } else if let Some(rest) = trimmed.strip_prefix("# cccp-hook-template-version:") {
                    template_ver = rest.trim().parse::<u32>().ok();
                } else if let Some(rest) = trimmed.strip_prefix("# cccp-hook-checksum:") {
                    recorded_checksum = rest.trim().to_string();
                }
            }

            // Se não for um hook gerenciado pelo CCCP, ignorar
            if hook_ver.is_empty() {
                continue;
            }

            let t_ver = template_ver.unwrap_or(0);
            if t_ver == CURRENT_HOOK_TEMPLATE_VERSION && hook_ver == curr_ver {
                continue;
            }

            // Extrair corpo atual sem cabeçalho de comentários
            let mut body_lines = Vec::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("#!") || !trimmed.starts_with('#') {
                    body_lines.push(line);
                }
            }
            let actual_body = body_lines.join("\n");
            let actual_checksum = compute_hook_checksum(&actual_body);

            // Verifica se o hook foi modificado manualmente:
            // 1) Se tinha checksum gravado e ele não bate com o conteúdo real
            // 2) Se era template legado (template_version = 0) mas tem linhas extras/diferentes
            let is_modified = if !recorded_checksum.is_empty() {
                recorded_checksum != actual_checksum
            } else {
                // Em templates legados, verificamos se o corpo contém personalizações manuais
                // ou se é estritamente a chamada padrão do cccp
                let is_clean_legacy = content.contains("exec cccp hook")
                    || content.contains("exec \"$candidate\" hook");
                content.contains("echo 'Custom")
                    || content.contains("Custom pre-validation")
                    || !is_clean_legacy
            };

            if is_modified {
                let warn_msg = format!(
                    "[cccp] Aviso: Git hook '{}' foi customizado localmente e mantido intacto (versão: {}, atual: {}).",
                    hook_name, hook_ver, curr_ver
                );
                eprintln!("{}", warn_msg);
                warnings.push(warn_msg);
            } else {
                // Não modificado: auto-atualização segura silenciosa/transparente
                let new_stub = generate_hook_stub(hook_name, repo_dir, &curr_ver);
                if fs::write(&hook_file, new_stub).is_ok() {
                    let _ = make_executable(&hook_file);
                }
            }
        }
    }
    warnings
}

/// Non-blocking auto-check for updates every N days.
pub fn check_auto_update(_config: &Config) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Check if auto-updates are disabled
    if let Ok(check_val) = env::var("CHECK_UPDATES") {
        if check_val == "0" || check_val.eq_ignore_ascii_case("false") {
            return Ok(None);
        }
    }

    // Check if version is pinned
    let pinned = env::var("PINNED_VERSION")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            if let Some(cfg_path) = config::find_global_config_path() {
                if let Ok(content) = fs::read_to_string(cfg_path) {
                    for line in content.lines() {
                        let clean = line.trim().strip_prefix("export ").unwrap_or(line.trim());
                        if let Some((k, v)) = clean.split_once('=') {
                            if k.trim().eq_ignore_ascii_case("pinned_version") {
                                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                                if !val.is_empty() {
                                    return Some(val.to_string());
                                }
                            }
                        }
                    }
                }
            }
            None
        });

    if pinned.is_some() {
        return Ok(None);
    }

    let interval_days: u64 = env::var("UPDATE_INTERVAL_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let interval_sec = interval_days * 86400;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let cache_file = get_update_cache_file();
    let mut last_check: u64 = 0;
    let mut cached_latest = String::new();

    if cache_file.is_file() {
        if let Ok(content) = fs::read_to_string(&cache_file) {
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("last_check_timestamp=") {
                    last_check = val.trim().parse().unwrap_or(0);
                } else if let Some(val) = line.strip_prefix("latest_version=") {
                    cached_latest = val.trim().to_string();
                }
            }
        }
    }

    let cfg = config::Config::load_hierarchical(None);
    let channel = env::var("UPDATE_CHANNEL")
        .or_else(|_| env::var("CCCP_UPDATE_CHANNEL"))
        .ok()
        .filter(|c| !c.trim().is_empty())
        .unwrap_or(cfg.channel);

    if now.saturating_sub(last_check) >= interval_sec {
        let (_s, _n, ver_url) = get_distribution_endpoints(&channel);

        let ureq_cfg = ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(2)))
            .build();
        let agent = ureq::Agent::new_with_config(ureq_cfg);

        if let Ok(resp) = agent.get(ver_url).call() {
            if let Ok(body) = resp.into_body().read_to_string() {
                if !body.trim().is_empty() {
                    let remote_ver = extract_version_from_response(&body);
                    if !remote_ver.is_empty() {
                        cached_latest = remote_ver;
                        if let Some(parent) = cache_file.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let cache_content = format!(
                            "last_check_timestamp={}\nlatest_version={}\n",
                            now, cached_latest
                        );
                        let _ = fs::write(&cache_file, cache_content);
                    }
                }
            }
        }
    }

    let curr_ver = get_current_version();
    if !cached_latest.is_empty() {
        if cached_latest == "nightly" {
            if channel == "nightly" {
                let notice = "[cccp] Notice: Running on nightly channel. Run 'cccp update' to pull latest changes.".to_string();
                eprintln!("{}", notice);
                return Ok(Some(notice));
            }
        } else if semver_is_newer(&cached_latest, &curr_ver) {
            let notice = format!(
                "[cccp] Notice: A newer version of cccp is available ({} vs current {}).\n[cccp] Run 'cccp update' to update to the latest {} release.",
                cached_latest, curr_ver, channel
            );
            eprintln!("{}", notice);
            return Ok(Some(notice));
        }
    }

    Ok(None)
}

/// Helper: extracts version from raw text or JSON response.
fn extract_version_from_response(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.contains("\"tag_name\":") {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(tag) = val.get("tag_name").and_then(|t| t.as_str()) {
                return tag.trim_start_matches('v').trim().to_string();
            }
        }
        // Fallback string matching if json parsing fails
        if let Some(idx) = trimmed.find("\"tag_name\":") {
            let sub = &trimmed[idx + 11..];
            let clean = sub.trim().trim_start_matches('"');
            if let Some(end) = clean.find('"') {
                return clean[..end].trim_start_matches('v').trim().to_string();
            }
        }
    }

    trimmed
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .trim_start_matches('v')
        .to_string()
}

/// Check update status command: `cccp check-update`.
pub fn cmd_check_update(channel_override: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let curr_ver = get_current_version();
    let channel = resolve_channel(channel_override)?;

    let pinned = env::var("PINNED_VERSION")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            if let Some(cfg_path) = config::find_global_config_path() {
                if let Ok(content) = fs::read_to_string(cfg_path) {
                    for line in content.lines() {
                        let clean = line.trim().strip_prefix("export ").unwrap_or(line.trim());
                        if let Some((k, v)) = clean.split_once('=') {
                            if k.trim().eq_ignore_ascii_case("pinned_version") {
                                let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                                if !val.is_empty() {
                                    return Some(val.to_string());
                                }
                            }
                        }
                    }
                }
            }
            None
        });

    println!("★ CCCP Update Verification Bureau ★");
    println!("Installed Version : {}", curr_ver);
    println!("Release Channel   : {}", channel);
    if let Some(ref pin) = pinned {
        println!("Pinned Version    : {} (Gosplan Directive Active)", pin);
    } else {
        println!(
            "Pinned Version    : none (tracking latest {} releases)",
            channel
        );
    }

    let (_stable_url, _nightly_url, ver_url) = get_distribution_endpoints(&channel);

    let ureq_cfg = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(3)))
        .build();
    let agent = ureq::Agent::new_with_config(ureq_cfg);

    let mut remote_ver = String::new();
    if let Ok(resp) = agent.get(&ver_url).call() {
        if let Ok(body) = resp.into_body().read_to_string() {
            if !body.trim().is_empty() {
                remote_ver = extract_version_from_response(&body);
            }
        }
    }

    if remote_ver.is_empty() {
        println!("Remote Version    : unavailable (unable to reach distribution endpoint)");
        println!(
            "Status            : Offline or telemetry unreachable. Verify network connection."
        );
        return Ok(());
    }

    println!("Remote Version    : {}", remote_ver);

    if let Some(ref pin) = pinned {
        if pin == &curr_ver {
            println!(
                "Status            : Pinned to {}. Updates are frozen by Gosplan decree.",
                pin
            );
        } else {
            println!(
                "Status            : Pinned to {} (currently running {}). Use 'cccp update --pin {}' to align or 'cccp update --unpin' to release.",
                pin, curr_ver, pin
            );
        }
    } else if channel == "nightly" {
        println!("Status            : Following nightly stream. Run 'cccp update' to pull latest changes.");
    } else if semver_is_newer(&remote_ver, &curr_ver) {
        println!(
            "Status            : Update available ({} -> {}). Execute 'cccp update' to upgrade.",
            curr_ver, remote_ver
        );
    } else {
        println!(
            "Status            : Up to date. The collective is operating on the latest standard."
        );
    }

    Ok(())
}

/// Executes CCCP update or pin configuration.
pub fn cmd_update(args: &UpdateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let channel = resolve_channel(args.channel.as_deref())?;

    let global_cfg = config::get_or_create_global_config_path();

    // Handle unpinning
    if args.unpin {
        let _ = config::unset_config_key(&global_cfg, "pinned_version");
        let _ = config::unset_config_key(&global_cfg, "pin_version");
        env::remove_var("PINNED_VERSION");
        println!(
            "Gosplan directive lifted: Version pin removed. Tracking {} releases.",
            channel
        );
    }

    // Handle pinning
    let mut is_pinned_now = false;
    let mut current_pin = None;

    if let Some(ref pin_arg) = args.pin {
        is_pinned_now = true;
        let clean_pin = if pin_arg.trim().is_empty() {
            get_current_version()
        } else {
            pin_arg
                .trim()
                .strip_prefix('v')
                .unwrap_or(pin_arg.trim())
                .to_string()
        };

        config::write_config_key(&global_cfg, "pinned_version", &clean_pin)?;
        env::set_var("PINNED_VERSION", &clean_pin);
        println!(
            "Gosplan directive enacted: Version pinned to {}.",
            clean_pin
        );

        if clean_pin == get_current_version() {
            return Ok(());
        }
        current_pin = Some(clean_pin);
    } else if !args.unpin {
        // Check existing pin
        if let Ok(val) = env::var("PINNED_VERSION") {
            if !val.trim().is_empty() {
                current_pin = Some(val.trim().to_string());
            }
        } else if global_cfg.is_file() {
            if let Ok(content) = fs::read_to_string(&global_cfg) {
                for line in content.lines() {
                    let clean = line.trim().strip_prefix("export ").unwrap_or(line.trim());
                    if let Some((k, v)) = clean.split_once('=') {
                        if k.trim().eq_ignore_ascii_case("pinned_version") {
                            let val = v.trim().trim_matches('"').trim_matches('\'').trim();
                            if !val.is_empty() {
                                current_pin = Some(val.to_string());
                            }
                        }
                    }
                }
            }
        }

        if let Some(pin) = current_pin {
            return Err(format!(
                "Error: Version is pinned to {} by Gosplan directive.\nTo upgrade anyway or release the pin, run 'cccp update --unpin' or 'cccp update --pin <version>'.",
                pin
            ).into());
        }
    }

    // Hermetic test bypass to prevent downloading in isolated test environments
    if env::var("CCCP_TEST_NO_DOWNLOAD").as_deref() == Ok("1") {
        return Ok(());
    }

    // Determine download URL from official Banca distribution
    let download_url = if is_pinned_now {
        let pin = current_pin.unwrap_or_else(get_current_version);
        format!("https://banca.2lp.in/downloads/cccp/releases/v{}/cccp", pin)
    } else if channel == "nightly" {
        "https://banca.2lp.in/downloads/cccp/cccp-nightly".to_string()
    } else {
        "https://banca.2lp.in/downloads/cccp/cccp-stable".to_string()
    };

    let target = if let Some(ref t) = args.target {
        t.clone()
    } else if let Ok(exe) = env::current_exe() {
        if exe
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s == "cccp" || s == "cccp.exe")
            .unwrap_or(false)
        {
            exe
        } else if let Ok(xdg_bin) = env::var("XDG_BIN_HOME") {
            PathBuf::from(xdg_bin).join("cccp")
        } else if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".local").join("bin").join("cccp")
        } else {
            PathBuf::from("/usr/local/bin/cccp")
        }
    } else if let Ok(xdg_bin) = env::var("XDG_BIN_HOME") {
        PathBuf::from(xdg_bin).join("cccp")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".local").join("bin").join("cccp")
    } else {
        PathBuf::from("/usr/local/bin/cccp")
    };

    println!("Updating cccp from channel '{}'...", channel);
    println!("Downloading from: {}", download_url);

    let ureq_cfg = ureq::config::Config::builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .build();
    let agent = ureq::Agent::new_with_config(ureq_cfg);

    let mut download_bytes = Vec::new();
    let mut downloaded = false;

    if let Ok(resp) = agent.get(&download_url).call() {
        if let Ok(bytes) = resp.into_body().read_to_vec() {
            if !bytes.is_empty() {
                download_bytes = bytes;
                downloaded = true;
            }
        }
    }

    if !downloaded || download_bytes.is_empty() {
        return Err(format!("Failed to download update from {}", download_url).into());
    }

    // Atomic update
    let temp_dir = target.parent().unwrap_or_else(|| Path::new("."));
    let temp_file = tempfile::Builder::new()
        .prefix("cccp-update-")
        .tempfile_in(temp_dir)?;

    let (mut file, temp_path) = temp_file.keep()?;
    file.write_all(&download_bytes)?;
    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755));
    }

    if target.is_file() {
        let bak_path = target.with_extension("bak");
        let _ = fs::copy(&target, &bak_path);
        println!("Backup saved to: {}", bak_path.display());
    } else if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    if fs::rename(&temp_path, &target).is_err() {
        fs::copy(&temp_path, &target)?;
        let _ = fs::remove_file(&temp_path);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o755));
    }

    // Refresh cache timestamp
    let cache_file = get_update_cache_file();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Some(parent) = cache_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(
        cache_file,
        format!("last_check_timestamp={}\nlatest_version=updated\n", now),
    );

    println!("Successfully updated cccp at: {}", target.display());
    Ok(())
}
