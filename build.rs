fn main() {
    println!("cargo:rerun-if-changed=VERSION");

    let version = std::fs::read_to_string("VERSION")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    println!("cargo:rustc-env=CCCP_CANONICAL_VERSION={}", version);
}
