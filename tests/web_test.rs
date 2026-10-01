use cccp::web::{cmd_cockpit, export_assets, generate_token, CockpitOptions};
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_generate_token() {
    let t1 = generate_token();
    let t2 = generate_token();

    assert!(t1.len() >= 16);
    assert!(t2.len() >= 16);
    assert_ne!(t1, t2);
}

#[test]
fn test_export_assets_content_and_security() {
    let dir = tempdir().unwrap();
    let out_dir = dir.path().join("assets");
    let token = "gosplan-secret-key-1922";

    let opts = CockpitOptions {
        port: 8088,
        token: Some(token.to_string()),
        ..CockpitOptions::default()
    };

    let index_file = export_assets(&out_dir, &opts).unwrap();
    assert!(index_file.is_file());

    let content = fs::read_to_string(&index_file).unwrap();
    assert!(content.contains("CCCP"));
    assert!(content.contains("auth-modal"));
    assert!(content.contains("repo-select"));
    assert!(content.contains("8088"));
    assert!(content.contains("tab-matraca"));
    assert!(content.contains("tab-diva"));
    assert!(content.contains("tab-status"));
    assert!(content.contains("tab-tickets"));

    // Security: secret token must NOT be leaked into the static HTML
    assert!(!content.contains(token));
}

#[test]
fn test_export_assets_tab_filtering() {
    let dir = tempdir().unwrap();

    // 1. Without matraca
    let out_no_matraca = dir.path().join("no_matraca");
    let opts_no_matraca = CockpitOptions {
        enable_matraca: false,
        ..CockpitOptions::default()
    };
    export_assets(&out_no_matraca, &opts_no_matraca).unwrap();
    let c1 = fs::read_to_string(out_no_matraca.join("index.html")).unwrap();
    assert!(!c1.contains("tab-matraca"));
    assert!(!c1.contains("nav-matraca"));
    assert!(c1.contains("tab-diva"));

    // 2. Without diva
    let out_no_diva = dir.path().join("no_diva");
    let opts_no_diva = CockpitOptions {
        enable_diva: false,
        ..CockpitOptions::default()
    };
    export_assets(&out_no_diva, &opts_no_diva).unwrap();
    let c2 = fs::read_to_string(out_no_diva.join("index.html")).unwrap();
    assert!(!c2.contains("tab-diva"));
    assert!(!c2.contains("nav-diva"));
    assert!(c2.contains("tab-matraca"));

    // 3. All tabs disabled -> error
    let out_none = dir.path().join("none");
    let opts_none = CockpitOptions {
        enable_matraca: false,
        enable_diva: false,
        enable_status: false,
        enable_tickets: false,
        ..CockpitOptions::default()
    };
    let err = export_assets(&out_none, &opts_none);
    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .contains("At least one tab must be enabled"));
}

#[test]
fn test_cmd_cockpit_with_mock_mayak() {
    let _lock = ENV_LOCK.lock().unwrap();

    let dir = tempdir().unwrap();
    let bin_dir = dir.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    let args_log = dir.path().join("mk_args.log");
    let mock_mk = bin_dir.join("mk");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script = format!(
            "#!/bin/sh\necho \"MOCK_MK_ARGS: $@\" > '{}'\nexit 0\n",
            args_log.display()
        );
        fs::write(&mock_mk, script).unwrap();
        let mut perms = fs::metadata(&mock_mk).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_mk, perms).unwrap();
    }

    let orig_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", bin_dir.display(), orig_path));

    // 1. Basic execution
    let opts = CockpitOptions {
        port: 9091,
        host: "127.0.0.1".to_string(),
        token: Some("test-token-777".to_string()),
        open: false,
        quiet: true,
        ..CockpitOptions::default()
    };
    let res = cmd_cockpit(opts);
    assert!(res.is_ok());

    let logged = fs::read_to_string(&args_log).unwrap();
    assert!(logged.contains("MOCK_MK_ARGS:"));
    assert!(logged.contains("-p 9091"));
    assert!(logged.contains("--host 127.0.0.1"));
    assert!(logged.contains("--bridge"));
    assert!(logged.contains("--auth test-token-777"));
    assert!(logged.contains("--cors"));
    assert!(!logged.contains("--open"));

    // 2. With --open flag
    let opts_open = CockpitOptions {
        port: 9092,
        token: Some("token-open".to_string()),
        open: true,
        quiet: true,
        ..CockpitOptions::default()
    };
    cmd_cockpit(opts_open).unwrap();
    let logged_open = fs::read_to_string(&args_log).unwrap();
    assert!(logged_open.contains("--open"));

    // 3. Reads CCCP_TOKEN environment variable when not specified
    std::env::set_var("CCCP_TOKEN", "env-secret-888");
    let opts_env = CockpitOptions {
        port: 9093,
        token: None,
        quiet: true,
        ..CockpitOptions::default()
    };
    cmd_cockpit(opts_env).unwrap();
    let logged_env = fs::read_to_string(&args_log).unwrap();
    assert!(logged_env.contains("--auth env-secret-888"));

    std::env::remove_var("CCCP_TOKEN");
    std::env::set_var("PATH", orig_path);
}
