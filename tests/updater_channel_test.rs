use cccp::config::Config;
use cccp::update::get_distribution_endpoints;
use std::fs;
use tempfile::tempdir;

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
    assert_eq!(
        version_url,
        "https://banca.2lp.in/downloads/cccp/version"
    );

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
