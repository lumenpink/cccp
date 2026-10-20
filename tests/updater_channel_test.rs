use cccp::cli::UpdateArgs;
use cccp::config::Config;
use cccp::update::{cmd_update, get_distribution_endpoints, resolve_channel};
use std::env;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_get_distribution_endpoints_banca() {
    let (stable_url, nightly_url, version_url) = get_distribution_endpoints("stable");
    assert_eq!(
        stable_url,
        "https://banca.2lp.in/downloads/cccp/cccp-stable"
    );
    assert_eq!(
        nightly_url,
        "https://banca.2lp.in/downloads/cccp/cccp-nightly"
    );
    assert_eq!(version_url, "https://banca.2lp.in/downloads/cccp/version");

    let (_, _, nightly_ver_url) = get_distribution_endpoints("nightly");
    assert_eq!(
        nightly_ver_url,
        "https://banca.2lp.in/downloads/cccp/nightly-version"
    );
}

#[test]
fn test_config_channel_reading_from_cccprc() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let cccprc_path = tmp.path().join(".cccprc");

    // Write .cccprc with channel = nightly
    fs::write(&cccprc_path, "channel = nightly\n").expect("write .cccprc");

    let cfg = Config::load_hierarchical(Some(tmp.path()));
    assert_eq!(cfg.channel, "nightly");
}

#[test]
fn test_config_channel_reading_from_cccprs_fallback() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let cccprs_path = tmp.path().join(".cccprs");

    // Write .cccprs with channel = stable
    fs::write(&cccprs_path, "channel = stable\n").expect("write .cccprs");

    let cfg = Config::load_hierarchical(Some(tmp.path()));
    assert_eq!(cfg.channel, "stable");
}

#[test]
fn test_config_validate_channel() {
    assert!(Config::validate_channel("stable").is_ok());
    assert!(Config::validate_channel("nightly").is_ok());
    assert!(Config::validate_channel("STABLE").is_ok());
    assert!(Config::validate_channel("NIGHTLY").is_ok());

    assert!(Config::validate_channel("beta").is_err());
    assert!(Config::validate_channel("edge").is_err());
    assert!(Config::validate_channel("").is_err());
    assert!(Config::validate_channel("invalid").is_err());
}

#[test]
fn test_resolve_channel_precedence() {
    let _lock = ENV_LOCK.lock().unwrap();

    let old_update_channel = env::var("UPDATE_CHANNEL").ok();
    let old_cccp_channel = env::var("CCCP_UPDATE_CHANNEL").ok();

    // 1. Explicit override wins
    env::set_var("UPDATE_CHANNEL", "stable");
    let ch = resolve_channel(Some("nightly")).expect("resolve channel");
    assert_eq!(ch, "nightly");

    // 2. Env variable wins when no override
    let ch = resolve_channel(None).expect("resolve channel");
    assert_eq!(ch, "stable");

    env::set_var("UPDATE_CHANNEL", "nightly");
    let ch = resolve_channel(None).expect("resolve channel");
    assert_eq!(ch, "nightly");

    // 3. Invalid channel produces error
    assert!(resolve_channel(Some("canary")).is_err());

    // Restore env
    if let Some(v) = old_update_channel {
        env::set_var("UPDATE_CHANNEL", v);
    } else {
        env::remove_var("UPDATE_CHANNEL");
    }
    if let Some(v) = old_cccp_channel {
        env::set_var("CCCP_UPDATE_CHANNEL", v);
    } else {
        env::remove_var("CCCP_UPDATE_CHANNEL");
    }
}

#[test]
fn test_cmd_update_channel_resolution_and_validation() {
    let _lock = ENV_LOCK.lock().unwrap();
    let old_xdg = env::var("XDG_CONFIG_HOME").ok();
    let old_v = env::var("CCCP_VERSION").ok();
    let old_no_download = env::var("CCCP_TEST_NO_DOWNLOAD").ok();

    let temp = tempdir().expect("tempdir");
    env::set_var("XDG_CONFIG_HOME", temp.path().to_str().unwrap());
    env::set_var("CCCP_VERSION", "2.0.0");
    env::set_var("CCCP_TEST_NO_DOWNLOAD", "1");
    env::remove_var("PINNED_VERSION");
    env::remove_var("UPDATE_CHANNEL");

    // 1. Valid channel nightly executes without download
    let args_nightly = UpdateArgs {
        channel: Some("nightly".to_string()),
        pin: None,
        unpin: false,
        target: None,
    };
    assert!(cmd_update(&args_nightly).is_ok());

    // 2. Invalid channel fails with error
    let args_invalid = UpdateArgs {
        channel: Some("unsupported_channel".to_string()),
        pin: None,
        unpin: false,
        target: None,
    };
    let err = cmd_update(&args_invalid).unwrap_err();
    assert!(err.to_string().contains("Invalid update channel"));

    // Restore
    if let Some(x) = old_xdg {
        env::set_var("XDG_CONFIG_HOME", x);
    } else {
        env::remove_var("XDG_CONFIG_HOME");
    }
    if let Some(v) = old_v {
        env::set_var("CCCP_VERSION", v);
    } else {
        env::remove_var("CCCP_VERSION");
    }
    if let Some(d) = old_no_download {
        env::set_var("CCCP_TEST_NO_DOWNLOAD", d);
    } else {
        env::remove_var("CCCP_TEST_NO_DOWNLOAD");
    }
}
