//! CCCP Matraca — Mobile & Offline-Tolerant Web REPL for Antigravity CLI (agy)
//! Filesystem-based FIFO Queue without external database dependencies.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Matraca Queue Job record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatracaJob {
    pub id: String,
    pub client_id: String,
    pub status: String,
    pub workspace: String,
    pub prompt: String,
    pub response: String,
    pub exit_code: i32,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Resolves Matraca data directory and ensures queue subdirectories exist
pub fn get_data_dir(override_dir: Option<&Path>) -> PathBuf {
    let base = match override_dir {
        Some(d) => d.to_path_buf(),
        None => {
            if let Ok(val) = std::env::var("MATRACA_DATA_DIR") {
                if !val.is_empty() {
                    PathBuf::from(val)
                } else {
                    default_data_dir()
                }
            } else {
                default_data_dir()
            }
        }
    };

    let _ = fs::create_dir_all(base.join("clients"));
    let _ = fs::create_dir_all(base.join("queue/pending"));
    let _ = fs::create_dir_all(base.join("queue/processing"));
    let _ = fs::create_dir_all(base.join("queue/done"));
    let _ = fs::create_dir_all(base.join("queue/error"));

    base
}

fn default_data_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("cccp/matraca");
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share/cccp/matraca")
    } else {
        PathBuf::from("/tmp/cccp_matraca")
    }
}

/// Returns current Unix timestamp in seconds
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Generates a short random string using system time and process ID
fn generate_random_token() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    format!("{:x}{:x}", (now & 0xFFFFFF) as u32, pid)
}

/// Triggers background worker execution non-blockingly
pub fn trigger_background_worker(data_dir: Option<&Path>) {
    if std::env::var("MATRACA_NO_AUTO_WORKER").as_deref() == Ok("1") {
        return;
    }

    let d = data_dir.map(|p| p.to_path_buf());
    std::thread::spawn(move || {
        let _ = cmd_matraca_worker(false, None, d.as_deref());
    });
}

/// Enqueues prompt into FIFO queue with strict idempotency
pub fn cmd_matraca_send(
    client_id: Option<&str>,
    workspace: Option<&str>,
    prompt: &str,
    data_dir: Option<&Path>,
) -> Result<String, String> {
    let trimmed_prompt = prompt.trim();
    if trimmed_prompt.is_empty() {
        let err_json = serde_json::json!({
            "ok": false,
            "error": "Prompt cannot be empty"
        });
        return Err(err_json.to_string());
    }

    let mdir = get_data_dir(data_dir);
    let now = current_timestamp();

    let resolved_client_id = match client_id {
        Some(cid) if !cid.is_empty() => cid.to_string(),
        _ => format!("cli_{}_{}", now, generate_random_token()),
    };

    // Idempotency check: if client_id already submitted, return existing job
    let client_file = mdir.join("clients").join(&resolved_client_id);
    if client_file.is_file() {
        if let Ok(existing_job_id) = fs::read_to_string(&client_file) {
            let clean_job_id = existing_job_id.trim();
            for sub in ["done", "processing", "pending", "error"] {
                let candidate = mdir
                    .join("queue")
                    .join(sub)
                    .join(format!("{}.json", clean_job_id));
                if candidate.is_file() {
                    if let Ok(content) = fs::read_to_string(&candidate) {
                        return Ok(content);
                    }
                }
            }
        }
    }

    let resolved_workspace = match workspace {
        Some(w) if !w.is_empty() => w.to_string(),
        _ => std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string()),
    };

    let job_id = format!("{}_{}_{}", now, generate_random_token(), std::process::id());

    let job = MatracaJob {
        id: job_id.clone(),
        client_id: resolved_client_id.clone(),
        status: "pending".to_string(),
        workspace: resolved_workspace,
        prompt: trimmed_prompt.to_string(),
        response: String::new(),
        exit_code: 0,
        created_at: now,
        updated_at: now,
    };

    let json_bytes = serde_json::to_vec_pretty(&job)
        .map_err(|e| format!("Failed to serialize job JSON: {}", e))?;

    let tmp_job = mdir.join("queue").join(format!("tmp_{}.json", job_id));
    let target_job = mdir.join("queue/pending").join(format!("{}.json", job_id));

    fs::write(&tmp_job, &json_bytes)
        .map_err(|e| format!("Failed to write temporary job file: {}", e))?;
    fs::rename(&tmp_job, &target_job)
        .map_err(|e| format!("Failed to move job to pending queue: {}", e))?;

    // Record client_id mapping
    let _ = fs::write(&client_file, &job_id);

    trigger_background_worker(data_dir);

    let ack = serde_json::json!({
        "ok": true,
        "id": job_id,
        "client_id": resolved_client_id,
        "status": "pending",
        "created_at": now
    });

    Ok(ack.to_string())
}

/// Polls status and response of a job
pub fn cmd_matraca_poll(query_id: &str, data_dir: Option<&Path>) -> Result<String, String> {
    let mdir = get_data_dir(data_dir);

    let mut target_id = query_id.to_string();
    let client_file = mdir.join("clients").join(query_id);
    if client_file.is_file() {
        if let Ok(cid_job_id) = fs::read_to_string(&client_file) {
            target_id = cid_job_id.trim().to_string();
        }
    }

    for sub in ["done", "processing", "pending", "error"] {
        let f = mdir
            .join("queue")
            .join(sub)
            .join(format!("{}.json", target_id));
        if f.is_file() {
            if let Ok(content) = fs::read_to_string(&f) {
                return Ok(content);
            }
        }
    }

    let err_json = serde_json::json!({
        "ok": false,
        "status": "not_found",
        "error": format!("Job '{}' not found", query_id)
    });
    Err(err_json.to_string())
}

/// Processes a single pending job file
fn process_pending_file(pending_file: &Path, mdir: &Path) {
    if !pending_file.is_file() {
        return;
    }

    let fname = match pending_file.file_name() {
        Some(f) => f.to_os_string(),
        None => return,
    };

    let processing_file = mdir.join("queue/processing").join(&fname);

    // Atomic claim via rename
    if fs::rename(pending_file, &processing_file).is_err() {
        return;
    }

    let job_content = match fs::read_to_string(&processing_file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut job: MatracaJob = match serde_json::from_str(&job_content) {
        Ok(j) => j,
        Err(_) => return,
    };

    let start_time = current_timestamp();
    let (agent_output, agent_exit) = if let Ok(runner) = std::env::var("MATRACA_RUNNER") {
        match Command::new("sh").arg("-c").arg(&runner).output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let combined = if stderr.is_empty() {
                    stdout.to_string()
                } else if stdout.is_empty() {
                    stderr.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };
                (combined, out.status.code().unwrap_or(1))
            }
            Err(e) => (format!("Failed to execute MATRACA_RUNNER: {}", e), 1),
        }
    } else {
        // Native agy invocation
        let agy_found = Command::new("agy").arg("--version").output().is_ok();
        if agy_found {
            match Command::new("agy")
                .args(["--continue", "--print", &job.prompt])
                .current_dir(&job.workspace)
                .output()
            {
                Ok(out) => {
                    let combined = String::from_utf8_lossy(&out.stdout).to_string();
                    (combined, out.status.code().unwrap_or(1))
                }
                Err(e) => (format!("Failed to run agy: {}", e), 1),
            }
        } else {
            (
                "Erro: Binário 'agy' (Antigravity CLI) não encontrado no PATH.".to_string(),
                127,
            )
        }
    };

    let finish_time = current_timestamp();
    job.response = agent_output.trim().to_string();
    job.exit_code = agent_exit;
    job.created_at = start_time;
    job.updated_at = finish_time;

    let (final_status, dest_dir) = if agent_exit == 0 {
        ("done", mdir.join("queue/done"))
    } else {
        ("error", mdir.join("queue/error"))
    };
    job.status = final_status.to_string();

    let tmp_dest = mdir
        .join("queue")
        .join(format!("tmp_final_{}", fname.to_string_lossy()));
    if let Ok(bytes) = serde_json::to_vec_pretty(&job) {
        let _ = fs::write(&tmp_dest, bytes);
        let _ = fs::rename(&tmp_dest, dest_dir.join(&fname));
    }

    let _ = fs::remove_file(&processing_file);
}

/// Worker loop or single-pass processing
pub fn cmd_matraca_worker(
    daemon: bool,
    target_job_id: Option<&str>,
    data_dir: Option<&Path>,
) -> Result<(), String> {
    let mdir = get_data_dir(data_dir);

    if let Some(jid) = target_job_id {
        let single_target = mdir.join("queue/pending").join(format!("{}.json", jid));
        process_pending_file(&single_target, &mdir);
        return Ok(());
    }

    let pending_dir = mdir.join("queue/pending");

    if daemon {
        loop {
            let mut found = false;
            if let Ok(entries) = fs::read_dir(&pending_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("json") {
                        found = true;
                        process_pending_file(&p, &mdir);
                    }
                }
            }
            if !found {
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    } else if let Ok(entries) = fs::read_dir(&pending_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                process_pending_file(&p, &mdir);
            }
        }
    }

    Ok(())
}

/// Retrieves list of recently processed or active jobs
pub fn cmd_matraca_history(limit: usize, data_dir: Option<&Path>) -> Result<String, String> {
    let mdir = get_data_dir(data_dir);
    let mut messages: Vec<serde_json::Value> = Vec::new();

    for sub in ["done", "error", "processing", "pending"] {
        let dir = mdir.join("queue").join(sub);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&p) {
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
                            messages.push(json_val);
                        }
                    }
                    if messages.len() >= limit {
                        break;
                    }
                }
            }
        }
        if messages.len() >= limit {
            break;
        }
    }

    let res = serde_json::json!({
        "ok": true,
        "messages": messages
    });

    Ok(res.to_string())
}

/// Computes queue status metrics
pub fn cmd_matraca_status(as_json: bool, data_dir: Option<&Path>) -> Result<String, String> {
    let mdir = get_data_dir(data_dir);

    let count_dir = |sub: &str| -> usize {
        let d = mdir.join("queue").join(sub);
        fs::read_dir(&d)
            .map(|entries| {
                entries
                    .flatten()
                    .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
                    .count()
            })
            .unwrap_or(0)
    };

    let pending_count = count_dir("pending");
    let processing_count = count_dir("processing");
    let done_count = count_dir("done");
    let error_count = count_dir("error");

    let clients_count = fs::read_dir(mdir.join("clients"))
        .map(|entries| entries.flatten().filter(|e| e.path().is_file()).count())
        .unwrap_or(0);

    if as_json {
        let json_val = serde_json::json!({
            "ok": true,
            "pending": pending_count,
            "processing": processing_count,
            "done": done_count,
            "error": error_count,
            "clients": clients_count,
            "data_dir": mdir.display().to_string()
        });
        Ok(json_val.to_string())
    } else {
        let mut out = String::new();
        out.push_str("🦜 CCCP Matraca — Queue & Spool Status\n");
        out.push_str("======================================\n");
        out.push_str(&format!("  Diretório : {}\n", mdir.display()));
        out.push_str(&format!("  Na fila   : {}\n", pending_count));
        out.push_str(&format!("  Executando: {}\n", processing_count));
        out.push_str(&format!("  Concluídos: {}\n", done_count));
        out.push_str(&format!("  Erros     : {}\n", error_count));
        out.push_str(&format!("  Clientes  : {}\n", clients_count));
        Ok(out)
    }
}

/// Cleans old jobs older than `days` from `done` and `error` queues
pub fn cmd_matraca_clean(days: u64, data_dir: Option<&Path>) -> Result<String, String> {
    let mdir = get_data_dir(data_dir);
    let cutoff_secs = days * 86400;
    let now = current_timestamp();

    let mut removed_count = 0;
    for sub in ["done", "error"] {
        let dir = mdir.join("queue").join(sub);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(metadata) = fs::metadata(&p) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(m_secs) = modified.duration_since(UNIX_EPOCH) {
                                if now.saturating_sub(m_secs.as_secs()) > cutoff_secs
                                    && fs::remove_file(&p).is_ok()
                                {
                                    removed_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(format!(
        "Fila do Matraca limpa com sucesso ({} arquivos > {} dias removidos).",
        removed_count, days
    ))
}
