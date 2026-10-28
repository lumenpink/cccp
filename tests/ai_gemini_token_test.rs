//! Integration and contract tests for Gemini token precedence (Ticket #79)

use cccp::ai::resolve_gemini_api_key;
use cccp::config::Config;
use std::env;

struct EnvGuard {
    keys: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn new(vars: &[&'static str]) -> Self {
        let keys = vars.iter().map(|&k| (k, env::var(k).ok())).collect();
        for &k in vars {
            env::remove_var(k);
        }
        Self { keys }
    }
}

impl Drop for EnvGuard {
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
    let _guard = EnvGuard::new(&[
        "CCCP_GEMINI_TOKEN",
        "CCCP_GEMINI_API_KEY",
        "GEMINI_TOKEN",
        "GEMINI_API_KEY",
        "GOOGLE_API_KEY",
    ]);

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
    let _guard = EnvGuard::new(&[
        "CCCP_GEMINI_TOKEN",
        "CCCP_GEMINI_API_KEY",
        "GEMINI_TOKEN",
        "GEMINI_API_KEY",
        "GOOGLE_API_KEY",
    ]);

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
    let _guard = EnvGuard::new(&[
        "CCCP_GEMINI_TOKEN",
        "CCCP_GEMINI_API_KEY",
        "GEMINI_TOKEN",
        "GEMINI_API_KEY",
        "GOOGLE_API_KEY",
    ]);

    env::set_var("GOOGLE_API_KEY", "fallback-google-key");

    let mut cfg = Config::default();
    cfg.merge_from_env();
    assert_eq!(cfg.gemini_api_key.as_deref(), Some("fallback-google-key"));

    let resolved = resolve_gemini_api_key(&cfg);
    assert_eq!(resolved.as_deref(), Some("fallback-google-key"));
}
