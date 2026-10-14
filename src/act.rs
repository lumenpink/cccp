//! CCCP Act Local CI Runner & Pre-Push Gate
//! Local CI verification via 'act' with live progress spinner,
//! structured logging to `.git/act-logs/`, and graceful degradation.

use crate::cultural::Theme;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Formats total seconds into human-readable duration (e.g. "0m 47s", "1m 34s")
pub fn format_duration(total_sec: u64) -> String {
    let min = total_sec / 60;
    let sec = total_sec % 60;
    format!("{}m {:02}s", min, sec)
}

/// Formats current UTC timestamp for structured log filename (YYYY-MM-DD_HH-MM)
pub fn format_log_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    let (year, month, day) = crate::tickets::days_to_ymd(days);
    format!("{:04}-{:02}-{:02}_{:02}-{:02}", year, month, day, h, m)
}

/// Checks availability of `act` binary and Docker daemon.
/// Returns (act_available, docker_available).
pub fn check_prerequisites(custom_bin: Option<&str>) -> (bool, bool) {
    let act_bin = custom_bin
        .map(String::from)
        .or_else(|| std::env::var("CCCP_ACT_BIN").ok())
        .unwrap_or_else(|| "act".to_string());

    let act_ok = Command::new(&act_bin)
        .arg("--version")
        .output()
        .map(|o| o.status.success() || !o.stdout.is_empty())
        .unwrap_or(false);

    let docker_ok = Command::new("docker")
        .args(["info"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or_else(|_| Path::new("/var/run/docker.sock").exists());

    (act_ok, docker_ok)
}

/// Locates active CI workflow file in repository (.forgejo/workflows or .github/workflows)
pub fn find_workflow(repo_root: &Path) -> Option<PathBuf> {
    let candidates = [
        ".forgejo/workflows/ci.yml",
        ".forgejo/workflows/ci.yaml",
        ".forgejo/workflows/test.yml",
        ".forgejo/workflows/test.yaml",
        ".github/workflows/ci.yml",
        ".github/workflows/ci.yaml",
        ".github/workflows/test.yml",
        ".github/workflows/test.yaml",
    ];

    for rel in &candidates {
        let candidate = repo_root.join(rel);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    for dir_rel in &[".forgejo/workflows", ".github/workflows"] {
        let dir = repo_root.join(dir_rel);
        if dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                let mut matches: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.is_file()
                            && p.extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext == "yml" || ext == "yaml")
                                .unwrap_or(false)
                    })
                    .collect();
                matches.sort();
                if let Some(first) = matches.into_iter().next() {
                    return Some(first);
                }
            }
        }
    }

    None
}

/// Sanitizes Forgejo full GitHub action URLs (`uses: https://github.com/...`) for act compatibility
pub fn sanitize_workflow_if_needed(wf_path: &Path) -> (PathBuf, Option<tempfile::NamedTempFile>) {
    if let Ok(content) = std::fs::read_to_string(wf_path) {
        if content.contains("uses: https://github.com/") {
            let sanitized = content.replace("uses: https://github.com/", "uses: ");
            if let Ok(mut tmp) = tempfile::Builder::new().suffix(".yml").tempfile() {
                if tmp.write_all(sanitized.as_bytes()).is_ok() {
                    return (tmp.path().to_path_buf(), Some(tmp));
                }
            }
        }
    }
    (wf_path.to_path_buf(), None)
}

/// Returns or creates `.git/act-logs` directory
pub fn get_log_dir(repo_root: &Path) -> PathBuf {
    let dir = repo_root.join(".git").join("act-logs");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn extract_step_from_log(logfile: &Path) -> Option<String> {
    let content = std::fs::read_to_string(logfile).ok()?;
    for line in content.lines().rev().take(15) {
        if line.contains("⭐ Run") || (line.contains('[') && line.contains(']')) {
            let clean = strip_ansi(line.trim());
            if !clean.is_empty() {
                return Some(clean);
            }
        }
    }
    None
}

fn extract_failed_step_from_log(logfile: &Path) -> Option<String> {
    let content = std::fs::read_to_string(logfile).ok()?;
    for line in content.lines().rev() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("##[error]")
            || lower.contains("error:")
            || lower.contains("fatal:")
            || lower.contains("failure:")
            || lower.contains("failed:")
        {
            let clean = strip_ansi(line.trim());
            if !clean.is_empty() {
                let truncated: String = clean.chars().take(70).collect();
                return Some(truncated);
            }
        }
    }
    None
}

/// Pre-push CI Gate Runner
/// Returns Ok(true) if gate passed or skipped, Ok(false) if CI failed and push should be blocked.
pub fn act_ci_gate(
    repo_root: &Path,
    remote: &str,
    branch: &str,
    theme: &Theme,
    _verbose: u8,
) -> Result<bool, String> {
    // 1. Conscious bypass check via environment
    if std::env::var("CCCP_ACT_SKIP").is_ok() || std::env::var("ACT_SKIP").is_ok() {
        println!(
            "{}",
            theme.yellow("[pre-push] CI gate bypassed via environment directive.")
        );
        return Ok(true);
    }

    // 2. Check if repository has CI workflow
    let raw_wf = match find_workflow(repo_root) {
        Some(w) => w,
        None => return Ok(true),
    };

    // 3. Prerequisites check: act & docker (Graceful degradation)
    let (act_ok, docker_ok) = check_prerequisites(None);
    if !act_ok || !docker_ok {
        println!(
            "{}",
            theme.yellow("[pre-push] Aviso: 'act' ou Docker indisponível. Pulando CI gate local (degradação graciosa).")
        );
        return Ok(true);
    }

    // 4. Sanitize workflow if needed
    let (clean_wf, _tmp_file) = sanitize_workflow_if_needed(&raw_wf);

    // 5. Prepare structured log file path (.git/act-logs/<YYYY-MM-DD>_<HH-MM>_<branch>_<short-sha>.log)
    let log_dir = get_log_dir(repo_root);
    let branch_clean = branch.replace('/', "-");
    let sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo_root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "0000000".to_string());
    let ts = format_log_timestamp();
    let logfile = log_dir.join(format!("{}_{}_{}.log", ts, branch_clean, sha));

    let job_name = std::env::var("CCCP_ACT_JOB").unwrap_or_else(|_| "test".to_string());
    let act_bin = std::env::var("CCCP_ACT_BIN").unwrap_or_else(|_| "act".to_string());

    let file_out = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&logfile)
        .map_err(|e| format!("Failed to create log file '{}': {}", logfile.display(), e))?;
    let file_err = file_out
        .try_clone()
        .map_err(|e| format!("Failed to clone log file handle: {}", e))?;

    let start_time = std::time::Instant::now();
    let is_interactive = std::io::stdout().is_terminal()
        && std::env::var("CI").as_deref() != Ok("true")
        && std::env::var("SHELLSPEC").as_deref() != Ok("1");

    let _runner_guard = RunnerPauseGuard::new(theme);

    let mut cmd = Command::new(&act_bin);
    cmd.args([
        "-W",
        clean_wf.to_str().unwrap_or(""),
        "-j",
        &job_name,
        "--pull=false",
    ])
    .current_dir(repo_root)
    .stdout(file_out)
    .stderr(file_err);

    let exit_success = if is_interactive {
        println!("{}", theme.cyan("[pre-push] Running CI gate locally..."));
        println!("  Job: {}", job_name);
        println!("  Log: {}", logfile.display());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn act runner '{}': {}", act_bin, e))?;

        let spinner_chars = ['|', '/', '-', '\\'];
        let mut spin_idx = 0;

        while child.try_wait().map_err(|e| e.to_string())?.is_none() {
            let elapsed = start_time.elapsed().as_secs();
            let time_disp = format_duration(elapsed);
            let spin_char = spinner_chars[spin_idx % spinner_chars.len()];
            spin_idx += 1;

            let step_name = extract_step_from_log(&logfile)
                .unwrap_or_else(|| "Executing workflow...".to_string());
            let step_trunc: String = step_name.chars().take(35).collect();

            eprint!(
                "\r  {} Job: {} › {:<35} [{}]   ",
                theme.cyan(&spin_char.to_string()),
                job_name,
                step_trunc,
                time_disp
            );
            let _ = std::io::stderr().flush();
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        eprint!("\r{:80}\r", "");
        let status = child.wait().map_err(|e| e.to_string())?;
        status.success()
    } else {
        let status = cmd
            .status()
            .map_err(|e| format!("Failed to execute act runner '{}': {}", act_bin, e))?;
        status.success()
    };

    let elapsed_fmt = format_duration(start_time.elapsed().as_secs());

    if exit_success {
        println!(
            "{}",
            theme.green(&format!(
                "✅ CI gate passed ({}) — pushing to {}/{}",
                elapsed_fmt, remote, branch
            ))
        );
        Ok(true)
    } else {
        let failed_step = extract_failed_step_from_log(&logfile)
            .unwrap_or_else(|| format!("job '{}' failed", job_name));

        eprintln!(
            "\n{}",
            theme.red_bold(&format!(
                "❌ CI gate failed ({}) — push aborted",
                elapsed_fmt
            ))
        );
        eprintln!("   → Step failed: {}", failed_step);
        eprintln!("   → Log: {}", logfile.display());
        eprintln!(
            "   → Bypass (conscious): {}",
            theme.bold("git push --no-verify")
        );
        Ok(false)
    }
}

/// Executes `cccp act status`
pub fn run_act_status(repo_root: &Path) {
    println!("========================================================");
    println!(" ★ CCCP Act Local CI Runner Status ★");
    println!("========================================================");

    let act_bin = std::env::var("CCCP_ACT_BIN").unwrap_or_else(|_| "act".to_string());
    let (act_ok, docker_ok) = check_prerequisites(Some(&act_bin));

    if act_ok {
        println!(" Act Runner Binary: {} (available)", act_bin);
    } else {
        println!(" Act Runner Binary: not installed (optional, runs in Docker)");
    }

    if docker_ok {
        println!(" Docker Daemon:     running (operational)");
    } else {
        println!(" Docker Daemon:     not reachable / stopped");
    }

    if let Some(wf) = find_workflow(repo_root) {
        println!(" Active Workflow:   {}", wf.display());
    } else {
        println!(" Active Workflow:   none found");
    }

    let log_dir = get_log_dir(repo_root);
    let log_count = std::fs::read_dir(&log_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "log")
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0);

    println!(
        " Recorded CI Logs:  {} logs in {}",
        log_count,
        log_dir.display()
    );
    println!("========================================================");
}

/// Executes `cccp act log [-f|--follow]`
pub fn run_act_log(repo_root: &Path, follow: bool, theme: &Theme) -> Result<(), String> {
    let log_dir = get_log_dir(repo_root);
    let mut logs: Vec<PathBuf> = std::fs::read_dir(&log_dir)
        .map_err(|e| format!("Failed to read log directory: {}", e))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .collect();

    logs.sort();
    let latest_log = match logs.last() {
        Some(l) => l,
        None => {
            return Err(format!("No act CI logs found in {}", log_dir.display()));
        }
    };

    println!(
        "{}",
        theme.bold(&format!(
            "=== Most Recent CI Log: {} ===",
            latest_log.display()
        ))
    );

    let content = std::fs::read_to_string(latest_log)
        .map_err(|e| format!("Failed to read log file: {}", e))?;
    print!("{}", content);

    if follow {
        let _ = Command::new("tail")
            .args(["-f", latest_log.to_str().unwrap_or("")])
            .status();
    }

    Ok(())
}

/// Executes `cccp act run [-j|--job <job>]`
pub fn run_act_run(
    repo_root: &Path,
    job: Option<&str>,
    theme: &Theme,
    verbose: u8,
) -> Result<(), String> {
    if let Some(j) = job {
        std::env::set_var("CCCP_ACT_JOB", j);
    }
    let passed = act_ci_gate(repo_root, "local", "manual", theme, verbose)?;
    if passed {
        Ok(())
    } else {
        Err("Local CI execution failed.".to_string())
    }
}

/// RAII Guard that pauses forgejo-runner during local act CI execution
pub struct RunnerPauseGuard {
    docker_bin: String,
    paused_containers: Vec<String>,
    theme: Theme,
}

impl RunnerPauseGuard {
    pub fn new(theme: &Theme) -> Self {
        let docker_bin = std::env::var("CCCP_DOCKER_BIN").unwrap_or_else(|_| "docker".to_string());
        Self::with_docker_bin(theme, &docker_bin)
    }

    pub fn with_docker_bin(theme: &Theme, docker_bin: &str) -> Self {
        let mut paused = Vec::new();

        // Allow bypassing pause via CCCP_NO_PAUSE_RUNNER=1
        if std::env::var("CCCP_NO_PAUSE_RUNNER").as_deref() == Ok("1") {
            return Self {
                docker_bin: docker_bin.to_string(),
                paused_containers: paused,
                theme: theme.clone(),
            };
        }

        // Query running containers matching forgejo-runner
        let output = Command::new(docker_bin)
            .args([
                "ps",
                "--filter",
                "name=forgejo-runner",
                "--filter",
                "status=running",
                "--format",
                "{{.Names}}",
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                for line in String::from_utf8_lossy(&out.stdout).lines() {
                    let name = line.trim();
                    if !name.is_empty() {
                        let pause_res = Command::new(docker_bin).args(["pause", name]).output();
                        if let Ok(p_out) = pause_res {
                            if p_out.status.success() {
                                eprintln!(
                                    "{}",
                                    theme.yellow(&format!(
                                        "[pre-push] ⏸ Pausando {} durante CI local...",
                                        name
                                    ))
                                );
                                paused.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        Self {
            docker_bin: docker_bin.to_string(),
            paused_containers: paused,
            theme: theme.clone(),
        }
    }

    pub fn paused_containers(&self) -> &[String] {
        &self.paused_containers
    }
}

impl Drop for RunnerPauseGuard {
    fn drop(&mut self) {
        for name in &self.paused_containers {
            let _ = Command::new(&self.docker_bin)
                .args(["unpause", name])
                .output();
            eprintln!(
                "{}",
                self.theme
                    .cyan(&format!("[pre-push] ▶ Retomando {}...", name))
            );
        }
    }
}


