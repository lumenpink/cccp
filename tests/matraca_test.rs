use cccp::matraca::{
    cmd_matraca_clean, cmd_matraca_history, cmd_matraca_poll, cmd_matraca_send,
    cmd_matraca_status, cmd_matraca_worker, MatracaJob,
};
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_matraca_send_and_poll() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::set_var("MATRACA_NO_AUTO_WORKER", "1");

    let dir = tempdir().unwrap();
    let data_dir = dir.path();

    // 1. Rejects empty prompt
    let err = cmd_matraca_send(None, None, "   ", Some(data_dir));
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("Prompt cannot be empty"));

    // 2. Normal send creates job
    let send_res = cmd_matraca_send(
        Some("cli-test-100"),
        None,
        "List all branches",
        Some(data_dir),
    )
    .unwrap();
    assert!(send_res.contains("\"ok\":true") || send_res.contains("\"ok\": true"));
    assert!(send_res.contains("cli-test-100"));
    assert!(send_res.contains("pending"));

    // 3. Idempotency on duplicate client_id
    let dup_res = cmd_matraca_send(
        Some("cli-test-100"),
        None,
        "Another prompt text",
        Some(data_dir),
    )
    .unwrap();
    assert!(dup_res.contains("List all branches"));
    assert!(!dup_res.contains("Another prompt text"));

    // 4. Poll by client_id
    let poll_res = cmd_matraca_poll("cli-test-100", Some(data_dir)).unwrap();
    let job: MatracaJob = serde_json::from_str(&poll_res).unwrap();
    assert_eq!(job.client_id, "cli-test-100");
    assert_eq!(job.status, "pending");
    assert_eq!(job.prompt, "List all branches");

    // 5. Poll nonexistent job
    let poll_none = cmd_matraca_poll("non-existent-xyz", Some(data_dir));
    assert!(poll_none.is_err());
    assert!(poll_none.unwrap_err().contains("not_found"));

    std::env::remove_var("MATRACA_NO_AUTO_WORKER");
}

#[test]
fn test_matraca_worker_success_and_error() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::set_var("MATRACA_NO_AUTO_WORKER", "1");

    let dir = tempdir().unwrap();
    let data_dir = dir.path();

    // 1. Process successful job with mock runner
    std::env::set_var("MATRACA_RUNNER", "echo 'Mocked agent response 42'");
    cmd_matraca_send(Some("cli-work-1"), None, "Do work", Some(data_dir)).unwrap();
    cmd_matraca_worker(false, None, Some(data_dir)).unwrap();

    let poll_done = cmd_matraca_poll("cli-work-1", Some(data_dir)).unwrap();
    let job_done: MatracaJob = serde_json::from_str(&poll_done).unwrap();
    assert_eq!(job_done.status, "done");
    assert_eq!(job_done.exit_code, 0);
    assert!(job_done.response.contains("Mocked agent response 42"));

    // 2. Process failed job with error runner
    std::env::set_var("MATRACA_RUNNER", "echo 'Falha simulada' >&2; exit 2");
    cmd_matraca_send(Some("cli-err-1"), None, "Fail task", Some(data_dir)).unwrap();
    cmd_matraca_worker(false, None, Some(data_dir)).unwrap();

    let poll_err = cmd_matraca_poll("cli-err-1", Some(data_dir)).unwrap();
    let job_err: MatracaJob = serde_json::from_str(&poll_err).unwrap();
    assert_eq!(job_err.status, "error");
    assert_eq!(job_err.exit_code, 2);
    assert!(job_err.response.contains("Falha simulada"));

    std::env::remove_var("MATRACA_RUNNER");
    std::env::remove_var("MATRACA_NO_AUTO_WORKER");
}

#[test]
fn test_matraca_status_history_and_clean() {
    let _lock = ENV_LOCK.lock().unwrap();
    std::env::set_var("MATRACA_NO_AUTO_WORKER", "1");
    std::env::set_var("MATRACA_RUNNER", "echo 'res'");

    let dir = tempdir().unwrap();
    let data_dir = dir.path();

    cmd_matraca_send(Some("c1"), None, "p1", Some(data_dir)).unwrap();
    cmd_matraca_send(Some("c2"), None, "p2", Some(data_dir)).unwrap();
    cmd_matraca_worker(false, None, Some(data_dir)).unwrap();

    // Status JSON
    let status_json = cmd_matraca_status(true, Some(data_dir)).unwrap();
    let v: serde_json::Value = serde_json::from_str(&status_json).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["done"], 2);
    assert_eq!(v["pending"], 0);
    assert_eq!(v["clients"], 2);

    // Status text
    let status_text = cmd_matraca_status(false, Some(data_dir)).unwrap();
    assert!(status_text.contains("CCCP Matraca"));
    assert!(status_text.contains("Concluídos: 2"));

    // History
    let history_json = cmd_matraca_history(10, Some(data_dir)).unwrap();
    let hist_v: serde_json::Value = serde_json::from_str(&history_json).unwrap();
    assert_eq!(hist_v["ok"], true);
    let msgs = hist_v["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 2);

    // Clean
    let clean_res = cmd_matraca_clean(0, Some(data_dir)).unwrap();
    assert!(clean_res.contains("Fila do Matraca limpa"));

    std::env::remove_var("MATRACA_RUNNER");
    std::env::remove_var("MATRACA_NO_AUTO_WORKER");
}
