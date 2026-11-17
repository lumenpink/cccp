use cccp::package::{build_generic_package_url, package_dir_to_tarball};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_build_generic_package_url() {
    let url = build_generic_package_url(
        "gambi.2lp.in",
        "lumen",
        "travesti",
        "1.0.0",
        "site.tar.gz",
    );
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
