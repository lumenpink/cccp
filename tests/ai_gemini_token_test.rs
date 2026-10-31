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

#[test]
fn test_gemini_token_whitespace_skips_to_next() {
    let _guard = EnvGuard::new();

    env::set_var("CCCP_GEMINI_TOKEN", "   ");
    env::set_var("CCCP_GEMINI_API_KEY", "");
    env::set_var("GEMINI_API_KEY", "  valid-gemini-key  ");

    let mut cfg = Config::default();
    cfg.merge_from_env();
    assert_eq!(cfg.gemini_api_key.as_deref(), Some("valid-gemini-key"));

    let resolved = resolve_gemini_api_key(&cfg);
    assert_eq!(resolved.as_deref(), Some("valid-gemini-key"));
}

#[test]
fn test_cccp_gemini_api_key_variant() {
    let _guard = EnvGuard::new();

    env::set_var("CCCP_GEMINI_API_KEY", "cccp-api-key-variant");
    env::set_var("GEMINI_API_KEY", "lower-prio-key");

    let mut cfg = Config::default();
    cfg.merge_from_env();
    assert_eq!(cfg.gemini_api_key.as_deref(), Some("cccp-api-key-variant"));

    let resolved = resolve_gemini_api_key(&cfg);
    assert_eq!(resolved.as_deref(), Some("cccp-api-key-variant"));
}

#[test]
fn test_gemini_total_absence_returns_none() {
    let _guard = EnvGuard::new();

    let mut cfg = Config::default();
    cfg.merge_from_env();
    assert_eq!(cfg.gemini_api_key, None);

    let resolved = resolve_gemini_api_key(&cfg);
    assert_eq!(resolved, None);
}

#[test]
fn test_config_parsing_cccp_gemini_key_precedence_over_generic() {
    let _guard = EnvGuard::new();

    // Case A: config string with generic GEMINI_API_KEY before CCCP_GEMINI_API_KEY
    let mut cfg1 = Config::default();
    cfg1.merge_from_str("GEMINI_API_KEY=deactivated_key\nCCCP_GEMINI_API_KEY=active_cccp_key\n");
    assert_eq!(
        resolve_gemini_api_key(&cfg1).as_deref(),
        Some("active_cccp_key")
    );

    // Case B: config string in reverse order
    let mut cfg2 = Config::default();
    cfg2.merge_from_str("CCCP_GEMINI_API_KEY=active_cccp_key\nGEMINI_API_KEY=deactivated_key\n");
    assert_eq!(
        resolve_gemini_api_key(&cfg2).as_deref(),
        Some("active_cccp_key")
    );

    // Case C: config string with CCCP_GEMINI_TOKEN over GEMINI_API_KEY
    let mut cfg3 = Config::default();
    cfg3.merge_from_str("GEMINI_API_KEY=generic_key\nCCCP_GEMINI_TOKEN=token_key\n");
    assert_eq!(
        resolve_gemini_api_key(&cfg3).as_deref(),
        Some("token_key")
    );

    // Case D: GEMINI_API_KEY in OS env, but CCCP_GEMINI_API_KEY in loaded config file
    env::set_var("GEMINI_API_KEY", "env_generic_key");
    let mut cfg4 = Config::default();
    cfg4.merge_from_str("CCCP_GEMINI_API_KEY=config_cccp_key\n");
    cfg4.merge_from_env();
    assert_eq!(
        resolve_gemini_api_key(&cfg4).as_deref(),
        Some("config_cccp_key")
    );
}

#[test]
fn test_config_merging_preserves_specific_key_against_generic_env() {
    let _guard = EnvGuard::new();

    // System has generic GEMINI_API_KEY exported
    env::set_var("GEMINI_API_KEY", "system-generic-key");

    // User's config file (like ~/.env or .cccprc) has CCCP_GEMINI_API_KEY
    let mut cfg = Config::default();
    cfg.merge_from_str("CCCP_GEMINI_API_KEY=my-isolated-cccp-key\nGEMINI_API_KEY=system-generic-key\n");
    cfg.merge_from_env();

    assert_eq!(cfg.gemini_api_key.as_deref(), Some("my-isolated-cccp-key"));
    assert_eq!(
        resolve_gemini_api_key(&cfg).as_deref(),
        Some("my-isolated-cccp-key")
    );
}

#[test]
fn test_mutation_config_parsing_whitespace_and_format_variants() {
    let _guard = EnvGuard::new();

    // Mutation 1: Whitespace in CCCP_GEMINI_API_KEY must fall back to GEMINI_API_KEY
    let mut cfg1 = Config::default();
    cfg1.merge_from_str("CCCP_GEMINI_API_KEY=   \nGEMINI_API_KEY=fallback-key\n");
    assert_eq!(
        resolve_gemini_api_key(&cfg1).as_deref(),
        Some("fallback-key"),
        "Whitespace in specific key must fallback to generic key"
    );

    // Mutation 2: Hyphenated and mixed-case keys (cccp-gemini-token)
    let mut cfg2 = Config::default();
    cfg2.merge_from_str("cccp-gemini-token = my-hyphen-token\ngemini-api-key = other\n");
    assert_eq!(
        resolve_gemini_api_key(&cfg2).as_deref(),
        Some("my-hyphen-token"),
        "Hyphenated keys must be normalized and prioritized"
    );

    // Mutation 3: Export prefix in .env file
    let mut cfg3 = Config::default();
    cfg3.merge_from_str("export CCCP_GEMINI_API_KEY=\"quoted-token\"\n");
    assert_eq!(
        resolve_gemini_api_key(&cfg3).as_deref(),
        Some("quoted-token"),
        "Export prefix and quotes must be stripped cleanly"
    );
}



