use cccp::cli::UpdateArgs;
use cccp::config::Config;
use cccp::update::{
    check_auto_update, check_hook_version, cmd_check_update, cmd_update, get_current_version,
    get_update_cache_file, semver_is_newer,
};
use std::env;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_semver_is_newer_comprehensive() {
    // Major
    assert!(semver_is_newer("2.0.0", "1.9.9"));
    assert!(!semver_is_newer("1.9.9", "2.0.0"));

    // Minor
    assert!(semver_is_newer("1.3.0", "1.2.9"));
    assert!(!semver_is_newer("1.2.9", "1.3.0"));

    // Patch
    assert!(semver_is_newer("1.2.5", "1.2.4"));
    assert!(!semver_is_newer("1.2.4", "1.2.5"));

    // Equality
    assert!(!semver_is_newer("1.2.3", "1.2.3"));

    // 'v' prefix
    assert!(semver_is_newer("v2.0.0", "1.9.9"));
    assert!(semver_is_newer("v1.2.4", "v1.2.3"));

    // Pre-release vs release
    assert!(semver_is_newer("1.0.0", "1.0.0-rc.1"));
    assert!(!semver_is_newer("1.0.0-rc.1", "1.0.0"));

    // Build metadata
    assert!(semver_is_newer("1.0.1+build.1", "1.0.0+build.2"));
}

#[test]
fn test_get_current_version_env_override() {
    let _lock = ENV_LOCK.lock().unwrap();
    let old_v = env::var("CCCP_VERSION").ok();

    env::set_var("CCCP_VERSION", "2.0.0");
    assert_eq!(get_current_version(), "2.0.0");

    env::remove_var("CCCP_VERSION");
    assert_eq!(get_current_version(), env!("CARGO_PKG_VERSION"));

    if let Some(v) = old_v {
        env::set_var("CCCP_VERSION", v);
    }
}

#[test]
fn test_get_update_cache_file() {
    let _lock = ENV_LOCK.lock().unwrap();
    let old_xdg = env::var("XDG_CACHE_HOME").ok();
    let old_home = env::var("HOME").ok();

    env::set_var("XDG_CACHE_HOME", "/tmp/custom_cache");
    assert_eq!(
        get_update_cache_file().to_str().unwrap(),
        "/tmp/custom_cache/cccp/update_cache"
    );

    env::remove_var("XDG_CACHE_HOME");
    env::set_var("HOME", "/tmp/custom_home");
    assert_eq!(
        get_update_cache_file().to_str().unwrap(),
        "/tmp/custom_home/.cache/cccp/update_cache"
    );

    if let Some(x) = old_xdg {
        env::set_var("XDG_CACHE_HOME", x);
    } else {
        env::remove_var("XDG_CACHE_HOME");
    }
    if let Some(h) = old_home {
        env::set_var("HOME", h);
    }
}

#[test]
fn test_check_hook_version() {
    let _lock = ENV_LOCK.lock().unwrap();
    let old_v = env::var("CCCP_VERSION").ok();
    env::set_var("CCCP_VERSION", "1.5.0");

    let temp = tempdir().unwrap();
    let hooks_dir = temp.path().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir).unwrap();

    let commit_msg = hooks_dir.join("commit-msg");

    // Case 1: Matching version
    fs::write(
        &commit_msg,
        "#!/bin/sh\n# cccp-hook-version: 1.5.0\nexec cccp hook commit-msg \"$@\"\n",
    )
    .unwrap();
    let warns = check_hook_version(temp.path());
    assert!(warns.is_empty());

    // Case 2: Divergent version, unmodified hook -> auto-updates silently/safely
    fs::write(
        &commit_msg,
        "#!/bin/sh\n# cccp-hook-version: 1.0.0\nexec cccp hook commit-msg \"$@\"\n",
    )
    .unwrap();
    let warns = check_hook_version(temp.path());
    assert!(warns.is_empty());
    let updated = fs::read_to_string(&commit_msg).unwrap();
    assert!(updated.contains("# cccp-hook-version: 1.5.0"));

    // Case 3: Divergent version, manually modified hook -> preserves and warns
    fs::write(
        &commit_msg,
        "#!/bin/sh\n# cccp-hook-version: 1.0.0\n# cccp-hook-checksum: 1234\necho 'custom local'\nexec cccp hook commit-msg \"$@\"\n",
    )
    .unwrap();
    let warns = check_hook_version(temp.path());
    assert_eq!(warns.len(), 1);
    assert!(warns[0].contains("customizado") || warns[0].contains("Custom"));

    // Case 4: Non-cccp hook -> ignored
    fs::write(&commit_msg, "#!/bin/sh\necho 'custom git hook'\n").unwrap();
    let warns = check_hook_version(temp.path());
    assert!(warns.is_empty());

    if let Some(v) = old_v {
        env::set_var("CCCP_VERSION", v);
    } else {
        env::remove_var("CCCP_VERSION");
    }
}

#[test]
fn test_check_auto_update_behavior() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let old_cache = env::var("XDG_CACHE_HOME").ok();
    let old_check = env::var("CHECK_UPDATES").ok();
    let old_pinned = env::var("PINNED_VERSION").ok();
    let old_v = env::var("CCCP_VERSION").ok();

    env::set_var("XDG_CACHE_HOME", temp.path().to_str().unwrap());
    env::set_var("CCCP_VERSION", "1.5.0");
    env::remove_var("PINNED_VERSION");

    let cache_file = temp.path().join("cccp").join("update_cache");
    fs::create_dir_all(cache_file.parent().unwrap()).unwrap();

    let cfg = Config::default();

    // 1. CHECK_UPDATES=0 skips
    env::set_var("CHECK_UPDATES", "0");
    let notice = check_auto_update(&cfg).unwrap();
    assert!(notice.is_none());

    // 2. Cached newer version produces notice
    env::set_var("CHECK_UPDATES", "1");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    fs::write(
        &cache_file,
        format!("last_check_timestamp={}\nlatest_version=2.0.0\n", now),
    )
    .unwrap();

    let notice = check_auto_update(&cfg).unwrap();
    assert!(notice.is_some());
    let msg = notice.unwrap();
    assert!(msg.contains("Notice: A newer version of cccp is available (2.0.0 vs current 1.5.0)"));

    // 3. Cached same version produces no notice
    fs::write(
        &cache_file,
        format!("last_check_timestamp={}\nlatest_version=1.5.0\n", now),
    )
    .unwrap();
    let notice = check_auto_update(&cfg).unwrap();
    assert!(notice.is_none());

    // 4. PINNED_VERSION suppresses update notice
    fs::write(
        &cache_file,
        format!("last_check_timestamp={}\nlatest_version=2.0.0\n", now),
    )
    .unwrap();
    env::set_var("PINNED_VERSION", "1.5.0");
    let notice = check_auto_update(&cfg).unwrap();
    assert!(notice.is_none());

    // Restore env
    if let Some(c) = old_cache {
        env::set_var("XDG_CACHE_HOME", c);
    } else {
        env::remove_var("XDG_CACHE_HOME");
    }
    if let Some(c) = old_check {
        env::set_var("CHECK_UPDATES", c);
    } else {
        env::remove_var("CHECK_UPDATES");
    }
    if let Some(p) = old_pinned {
        env::set_var("PINNED_VERSION", p);
    } else {
        env::remove_var("PINNED_VERSION");
    }
    if let Some(v) = old_v {
        env::set_var("CCCP_VERSION", v);
    } else {
        env::remove_var("CCCP_VERSION");
    }
}

#[test]
fn test_cmd_update_pinning_and_validation() {
    let _lock = ENV_LOCK.lock().unwrap();
    let temp = tempdir().unwrap();
    let old_xdg = env::var("XDG_CONFIG_HOME").ok();
    let old_pinned = env::var("PINNED_VERSION").ok();
    let old_v = env::var("CCCP_VERSION").ok();

    env::set_var("XDG_CONFIG_HOME", temp.path().to_str().unwrap());
    env::set_var("CCCP_VERSION", "2.0.0");
    env::set_var("CCCP_TEST_NO_DOWNLOAD", "1");
    env::remove_var("PINNED_VERSION");

    // Invalid channel fails
    let args_invalid = UpdateArgs {
        channel: "invalid_channel".to_string(),
        pin: None,
        unpin: false,
        target: None,
    };
    assert!(cmd_update(&args_invalid).is_err());

    // Pinning to current version succeeds without download
    let args_pin_default = UpdateArgs {
        channel: "stable".to_string(),
        pin: Some("".to_string()),
        unpin: false,
        target: None,
    };
    assert!(cmd_update(&args_pin_default).is_ok());
    assert_eq!(env::var("PINNED_VERSION").unwrap(), "2.0.0");

    // Once pinned, standard update is blocked
    let args_standard = UpdateArgs {
        channel: "stable".to_string(),
        pin: None,
        unpin: false,
        target: None,
    };
    let err = cmd_update(&args_standard).unwrap_err();
    assert!(err
        .to_string()
        .contains("Version is pinned to 2.0.0 by Gosplan directive"));

    // Unpin clears pin
    let args_unpin = UpdateArgs {
        channel: "stable".to_string(),
        pin: None,
        unpin: true,
        target: None,
    };
    assert!(cmd_update(&args_unpin).is_ok());
    assert!(env::var("PINNED_VERSION").is_err());

    // Restore env
    if let Some(x) = old_xdg {
        env::set_var("XDG_CONFIG_HOME", x);
    } else {
        env::remove_var("XDG_CONFIG_HOME");
    }
    if let Some(p) = old_pinned {
        env::set_var("PINNED_VERSION", p);
    } else {
        env::remove_var("PINNED_VERSION");
    }
    if let Some(v) = old_v {
        env::set_var("CCCP_VERSION", v);
    } else {
        env::remove_var("CCCP_VERSION");
    }
    env::remove_var("CCCP_TEST_NO_DOWNLOAD");
}

#[test]
fn test_cmd_check_update_runs() {
    let _lock = ENV_LOCK.lock().unwrap();
    let old_v = env::var("CCCP_VERSION").ok();
    env::set_var("CCCP_VERSION", "3.0.0");

    // Command should run and return Ok(())
    assert!(cmd_check_update(Some("stable")).is_ok());

    if let Some(v) = old_v {
        env::set_var("CCCP_VERSION", v);
    } else {
        env::remove_var("CCCP_VERSION");
    }
}

#[test]
fn test_distribution_urls_banca_and_no_codeberg() {
    let (stable_url, nightly_url, ver_url) = cccp::update::get_distribution_endpoints("stable");
    assert!(
        stable_url.contains("banca.2lp.in"),
        "Stable URL must point to banca.2lp.in: {}",
        stable_url
    );
    assert!(
        !stable_url.contains("codeberg.org"),
        "Stable URL must NOT contain codeberg: {}",
        stable_url
    );
    assert!(
        nightly_url.contains("banca.2lp.in"),
        "Nightly URL must point to banca.2lp.in: {}",
        nightly_url
    );
    assert!(
        !nightly_url.contains("codeberg.org"),
        "Nightly URL must NOT contain codeberg: {}",
        nightly_url
    );
    assert!(
        ver_url.contains("banca.2lp.in"),
        "Version URL must point to banca.2lp.in: {}",
        ver_url
    );
    assert!(
        !ver_url.contains("codeberg.org"),
        "Version URL must NOT contain codeberg: {}",
        ver_url
    );
}
