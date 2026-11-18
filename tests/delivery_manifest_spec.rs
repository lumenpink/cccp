use std::fs;
use std::path::Path;

#[test]
fn test_cccp_delivery_manifest_exists_and_declares_binaries() {
    let manifest_path = Path::new(".gost/delivery.toml");
    assert!(manifest_path.exists(), ".gost/delivery.toml deve existir");

    let content = fs::read_to_string(manifest_path).unwrap();
    assert!(
        content.contains("[binaries]"),
        ".gost/delivery.toml deve conter a seção [binaries]"
    );
}
