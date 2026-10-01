use cccp::cultural::Theme;
use cccp::serve::{
    cmd_serve, detect_target_dir, find_free_port, guess_mime_type, is_port_in_use,
    run_static_server, ServeOptions,
};
use std::fs;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_detect_target_dir_precedence() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // 1. None of site/dist/public exist -> returns root
    assert_eq!(detect_target_dir(None, root), root);

    // 2. public exists -> returns public
    fs::create_dir_all(root.join("public")).unwrap();
    assert_eq!(detect_target_dir(None, root), root.join("public"));

    // 3. dist exists -> takes precedence over public
    fs::create_dir_all(root.join("dist")).unwrap();
    assert_eq!(detect_target_dir(None, root), root.join("dist"));

    // 4. site exists -> takes highest precedence over dist and public
    fs::create_dir_all(root.join("site")).unwrap();
    assert_eq!(detect_target_dir(None, root), root.join("site"));

    // 5. Explicit directory takes precedence over site
    fs::create_dir_all(root.join("custom_docs")).unwrap();
    assert_eq!(
        detect_target_dir(Some("custom_docs"), root),
        root.join("custom_docs")
    );
}

#[test]
fn test_mime_type_detection() {
    assert_eq!(
        guess_mime_type(std::path::Path::new("index.html")),
        "text/html; charset=utf-8"
    );
    assert_eq!(
        guess_mime_type(std::path::Path::new("app.css")),
        "text/css; charset=utf-8"
    );
    assert_eq!(
        guess_mime_type(std::path::Path::new("bundle.js")),
        "application/javascript; charset=utf-8"
    );
    assert_eq!(
        guess_mime_type(std::path::Path::new("data.json")),
        "application/json; charset=utf-8"
    );
    assert_eq!(
        guess_mime_type(std::path::Path::new("logo.svg")),
        "image/svg+xml"
    );
    assert_eq!(
        guess_mime_type(std::path::Path::new("image.png")),
        "image/png"
    );
    assert_eq!(
        guess_mime_type(std::path::Path::new("module.wasm")),
        "application/wasm"
    );
}

#[test]
fn test_find_free_port() {
    // Bind a listener on an ephemeral port
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let occupied_port = listener.local_addr().unwrap().port();

    assert!(is_port_in_use(occupied_port, "127.0.0.1"));

    // find_free_port starting at occupied_port should find another port
    let free = find_free_port(occupied_port, "127.0.0.1");
    assert_ne!(free, occupied_port);
    assert!(!is_port_in_use(free, "127.0.0.1"));
}

#[test]
fn test_http_server_request_handling_and_security() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("index.html"), "<h1>Welcome to CCCP</h1>").unwrap();
    fs::write(root.join("style.css"), "body { color: red; }").unwrap();

    let sub = root.join("assets");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("hello.txt"), "Soviet Tech Standard").unwrap();

    // Pick a free port
    let dummy = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = dummy.local_addr().unwrap().port();
    drop(dummy);

    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown_flag);
    let root_path = root.to_path_buf();

    let server_handle = thread::spawn(move || {
        let _ = run_static_server(&root_path, "127.0.0.1", port, true, Some(shutdown_clone));
    });

    // Wait for server to become active
    let mut ready = false;
    for _ in 0..50 {
        if TcpListener::bind(("127.0.0.1", port)).is_err() {
            ready = true;
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(ready, "Server failed to bind in time");

    let base_url = format!("http://127.0.0.1:{}", port);

    // 1. GET / -> returns index.html
    let resp_root = ureq::get(&format!("{}/", base_url)).call().unwrap();
    assert_eq!(resp_root.status(), 200);
    assert_eq!(
        resp_root.headers().get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );
    let body_root = resp_root.into_body().read_to_string().unwrap();
    assert!(body_root.contains("<h1>Welcome to CCCP</h1>"));

    // 2. GET /style.css -> returns style.css with text/css
    let resp_css = ureq::get(&format!("{}/style.css", base_url))
        .call()
        .unwrap();
    assert_eq!(resp_css.status(), 200);
    assert_eq!(
        resp_css.headers().get("content-type").unwrap(),
        "text/css; charset=utf-8"
    );
    let body_css = resp_css.into_body().read_to_string().unwrap();
    assert!(body_css.contains("color: red;"));

    // 3. GET /assets/hello.txt -> subfolder file
    let resp_sub = ureq::get(&format!("{}/assets/hello.txt", base_url))
        .call()
        .unwrap();
    assert_eq!(resp_sub.status(), 200);
    let body_sub = resp_sub.into_body().read_to_string().unwrap();
    assert_eq!(body_sub, "Soviet Tech Standard");

    // 4. GET /missing.html -> returns 404 Not Found
    let resp_404 = ureq::get(&format!("{}/missing.html", base_url)).call();
    match resp_404 {
        Err(ureq::Error::StatusCode(404)) => {}
        other => panic!("Expected 404, got: {:?}", other),
    }

    // 5. Directory traversal attack -> returns 403 Forbidden
    let resp_traversal = ureq::get(&format!("{}/../Cargo.toml", base_url)).call();
    match resp_traversal {
        Err(ureq::Error::StatusCode(403)) => {}
        other => panic!("Expected 403, got: {:?}", other),
    }

    // Shut down server
    shutdown_flag.store(true, Ordering::Relaxed);
    let _ = server_handle.join();
}

#[test]
fn test_dev_mode_delegates_to_scripts_dev() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let scripts_dir = root.join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();

    let marker_file = root.join("dev_executed.marker");
    let dev_script = scripts_dir.join("dev.sh");
    fs::write(
        &dev_script,
        format!("#!/bin/sh\necho 'OK' > '{}'\n", marker_file.display()),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dev_script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dev_script, perms).unwrap();
    }

    // Change current directory to root during test
    let prev_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(root).unwrap();

    let theme = Theme::init(true);
    let opts = ServeOptions {
        dev: true,
        quiet: true,
        ..ServeOptions::default()
    };

    let res = cmd_serve(opts, &theme);
    let _ = std::env::set_current_dir(prev_dir);

    assert!(res.is_ok());
    assert!(marker_file.is_file());
    assert_eq!(fs::read_to_string(marker_file).unwrap().trim(), "OK");
}
