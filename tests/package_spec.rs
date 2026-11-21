use cccp::package::{build_generic_package_url, package_dir_to_tarball};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_build_generic_package_url() {
    let url =
        build_generic_package_url("gambi.2lp.in", "lumen", "travesti", "1.0.0", "site.tar.gz");
    assert_eq!(
        url,
        "https://gambi.2lp.in/api/packages/lumen/generic/travesti/1.0.0/site.tar.gz"
    );
}

#[test]
fn test_package_dir_to_tarball_creates_valid_gzip_archive() {
    let dir = tempdir().unwrap();
    let site_dir = dir.path().join("site");
    fs::create_dir_all(site_dir.join("sub")).unwrap();
    fs::write(site_dir.join("index.html"), b"<h1>Hello World</h1>").unwrap();
    fs::write(site_dir.join("sub/style.css"), b"body { margin: 0; }").unwrap();

    let tar_out = dir.path().join("site.tar.gz");
    let size = package_dir_to_tarball(&site_dir, &tar_out).expect("tarball creation must succeed");

    assert!(tar_out.exists());
    assert!(size > 0);

    // Valida magic bytes de gzip: 0x1f 0x8b
    let bytes = fs::read(&tar_out).unwrap();
    assert_eq!(bytes[0], 0x1f);
    assert_eq!(bytes[1], 0x8b);
}

#[test]
fn test_upload_package_fails_if_target_does_not_exist() {
    let cfg = cccp::package::PackageUploadConfig {
        host: "gambi.2lp.in".to_string(),
        owner: "lumen".to_string(),
        app: "travesti".to_string(),
        version: "1.0.0".to_string(),
        token: "secret".to_string(),
    };
    let non_existent = std::path::Path::new("/tmp/non-existent-artifact-123456789.tar.gz");
    let res = cccp::package::upload_package(non_existent, &cfg);
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("não encontrado"));
}

#[test]
fn test_cli_parsing_package_upload() {
    use cccp::cli::{Cli, Commands, PackageCommand};
    use clap::Parser;

    let args = Cli::parse_from([
        "cccp",
        "package",
        "upload",
        "dist/site",
        "--app",
        "travesti",
        "--version",
        "2.0.0",
        "--filename",
        "site.tar.gz",
    ]);

    match args.command {
        Some(Commands::Package(PackageCommand::Upload(up))) => {
            assert_eq!(up.path, std::path::PathBuf::from("dist/site"));
            assert_eq!(up.app.as_deref(), Some("travesti"));
            assert_eq!(up.version.as_deref(), Some("2.0.0"));
            assert_eq!(up.filename.as_deref(), Some("site.tar.gz"));
        }
        _ => panic!("Expected Commands::Package(PackageCommand::Upload)"),
    }
}

#[test]
fn test_resolve_package_upload_config_fails_without_git_repo() {
    let empty_dir = tempdir().unwrap();
    let res = cccp::package::resolve_package_upload_config(empty_dir.path(), None, None);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("origin")
            || err.contains("Falha ao executar")
            || err.contains("Nenhum remote")
    );
}

#[test]
fn test_is_mutable_package_version_allows_dev_nightly_and_protects_stable() {
    use cccp::package::is_mutable_package_version;

    assert!(is_mutable_package_version("0.1.0-dev.42+20261121.abc1234"));
    assert!(is_mutable_package_version("nightly"));
    assert!(is_mutable_package_version("latest"));
    assert!(is_mutable_package_version("1.2.0-rc.1"));

    // Versões estáveis permanecem imutáveis
    assert!(!is_mutable_package_version("1.0.0"));
    assert!(!is_mutable_package_version("3.2.1"));
}
