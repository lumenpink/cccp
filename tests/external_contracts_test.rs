use std::fs;

#[test]
fn test_external_gossnab_schema_exists() {
    let spec = fs::read_to_string("docs/external/gossnab.yaml")
        .expect("docs/external/gossnab.yaml deve existir");
    assert!(spec.contains("openapi: 3."), "Schema deve ser OpenAPI 3.x");
    assert!(spec.contains("/api/v1/apps/{slug}/update"), "Deve conter rota de update");
    assert!(spec.contains("/api/v1/queue"), "Deve conter rota de queue");
}

#[test]
fn test_fail_fast_gossnab_models_deserialization() {
    let valid_update = r#"{
        "app": "cccp",
        "channel": "stable",
        "latest_version": "3.2.0",
        "download_url": "https://banca.2lp.in/downloads/cccp/cccp-latest-x86_64-linux.tar.gz",
        "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "published_at": "2026-11-08T11:50:00Z"
    }"#;
    let parsed: cccp::external::gossnab::AppUpdateResponse = serde_json::from_str(valid_update).unwrap();
    assert_eq!(parsed.app, "cccp");

    let bad_update = r#"{
        "app": "cccp",
        "channel": "stable",
        "latest_version": "3.2.0",
        "download_url": "https://banca.2lp.in/downloads/cccp/cccp-latest-x86_64-linux.tar.gz",
        "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "published_at": "2026-11-08T11:50:00Z",
        "campo_invalido": true
    }"#;
    let res: Result<cccp::external::gossnab::AppUpdateResponse, _> = serde_json::from_str(bad_update);
    assert!(res.is_err(), "Deve rejeitar campos nao declarados no contrato");
}
