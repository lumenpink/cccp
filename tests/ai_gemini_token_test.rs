//! Integration and contract tests for Gemini token precedence (Ticket #79)

use cccp::ai::{resolve_gemini_api_key, GEMINI_ENV_KEYS};
use cccp::config::Config;
use std::env;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard<'a> {
    _lock: std::sync::MutexGuard<'a, ()>,
    keys: Vec<(&'static str, Option<String>)>,
}

impl<'a> EnvGuard<'a> {
    fn new() -> Self {
        let lock = ENV_LOCK.lock().unwrap();
        let keys = GEMINI_ENV_KEYS
            .iter()
            .map(|&k| (k, env::var(k).ok()))
            .collect();
        for &k in GEMINI_ENV_KEYS {
            env::remove_var(k);
        }
        Self { _lock: lock, keys }
    }
}

impl<'a> Drop for EnvGuard<'a> {
    fn drop(&mut self) {
        for (k, val) in &self.keys {
            if let Some(v) = val {
                env::set_var(k, v);
            } else {
                env::remove_var(k);
            }
        }
    }
}

#[test]
fn test_cccp_gemini_token_precedence_over_all() {
    let _guard = EnvGuard::new();

    env::set_var("CCCP_GEMINI_TOKEN", "cccp-token-123");
    env::set_var("GEMINI_API_KEY", "gemini-api-key-456");
    env::set_var("GOOGLE_API_KEY", "google-api-key-789");

    let mut cfg = Config::default();
    cfg.merge_from_env();
    assert_eq!(cfg.gemini_api_key.as_deref(), Some("cccp-token-123"));

    let resolved = resolve_gemini_api_key(&cfg);
    assert_eq!(resolved.as_deref(), Some("cccp-token-123"));
}

#[test]
fn test_gemini_token_precedence_over_google_api_key() {
    let _guard = EnvGuard::new();

    env::set_var("GEMINI_TOKEN", "gemini-token-abc");
    env::set_var("GOOGLE_API_KEY", "google-api-key-xyz");

    let mut cfg = Config::default();
    cfg.merge_from_env();
    assert_eq!(cfg.gemini_api_key.as_deref(), Some("gemini-token-abc"));

    let resolved = resolve_gemini_api_key(&cfg);
    assert_eq!(resolved.as_deref(), Some("gemini-token-abc"));
}

#[test]
fn test_google_api_key_fallback() {
    let _guard = EnvGuard::new();

    env::set_var("GOOGLE_API_KEY", "fallback-google-key");

    let mut cfg = Config::default();
    cfg.merge_from_env();
    assert_eq!(cfg.gemini_api_key.as_deref(), Some("fallback-google-key"));

    let resolved = resolve_gemini_api_key(&cfg);
    assert_eq!(resolved.as_deref(), Some("fallback-google-key"));
}
