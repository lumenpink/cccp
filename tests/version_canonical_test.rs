use cccp::update::get_current_version;

#[test]
fn test_get_current_version_uses_canonical_version_from_version_file() {
    let version_on_disk = std::fs::read_to_string("VERSION")
        .expect("VERSION file must exist in repo root")
        .trim()
        .to_string();

    let curr_ver = get_current_version();

    // Must NOT be the stale Cargo.toml version (3.1.0)
    assert_ne!(
        curr_ver, "3.1.0",
        "CCCP version must not be hardcoded 3.1.0 tag from Cargo.toml"
    );

    // Must match canonical version from VERSION file
    assert_eq!(
        curr_ver, version_on_disk,
        "Current version must match canonical VERSION"
    );
}

#[test]
fn test_gostver_extraction_sanitizes_multiline_content() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let sample_gostver = r#"# GOSTVER — Declaração de Conformidade com o Padrão GOST
GOST_VERSION="1.5.0"
GOST_PROFILE_URL="https://gostlumen.2lp.in/profile.md"
GOST_LANGUAGES="rust"
"#;
    fs::write(dir.path().join("GOSTVER"), sample_gostver).unwrap();

    let extracted = cccp::catalog::extract_gost_version(dir.path());
    assert_eq!(extracted.as_deref(), Some("1.5.0"));
}

#[test]
fn test_mutation_gostver_parsing_edge_cases() {
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();

    // Single quotes
    fs::write(dir.path().join("GOSTVER"), "GOST_VERSION='1.4.0'\n").unwrap();
    assert_eq!(
        cccp::catalog::extract_gost_version(dir.path()).as_deref(),
        Some("1.4.0")
    );

    // Empty file
    fs::write(dir.path().join("GOSTVER"), "").unwrap();
    assert_eq!(cccp::catalog::extract_gost_version(dir.path()), None);

    // No GOST_VERSION key
    fs::write(dir.path().join("GOSTVER"), "PROFILE=default\n").unwrap();
    assert_eq!(cccp::catalog::extract_gost_version(dir.path()), None);
}

#[test]
fn test_mutation_cli_command_version_matches_canonical() {
    use clap::CommandFactory;

    let clap_ver = cccp::cli::Cli::command()
        .get_version()
        .map(|s| s.to_string());
    let expected = env!("CCCP_CANONICAL_VERSION");

    assert_eq!(
        clap_ver.as_deref(),
        Some(expected),
        "Clap metadata version (-V/--version) must match CCCP_CANONICAL_VERSION"
    );
}
