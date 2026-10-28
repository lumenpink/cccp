//! Integration and contract tests for CCCP Site Banner Generator (Ticket #75, GOST-20, GOST-26)

use cccp::cultural::Theme;
use cccp::site_banner::{generate_site_banner, BannerOptions};
use std::fs;
use tempfile::tempdir;

fn read_png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 {
        return None;
    }
    // PNG Signature
    if &bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    // IHDR chunk
    if &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((width, height))
}

#[test]
fn test_generate_site_banner_defaults() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let theme = Theme::with_color(false);

    // Mock .gost/metadata.json
    fs::create_dir_all(root.join(".gost")).unwrap();
    let metadata = r##"{
  "name": "meu-projeto",
  "title": "Meu Projeto Soberano",
  "tagline": "Ferramenta de alta performance para o ecossistema",
  "theme": {
    "bg_color": "#111317",
    "accent_color": "#ef4444"
  }
}"##;
    fs::write(root.join(".gost/metadata.json"), metadata).unwrap();

    let opts = BannerOptions::default();
    let res = generate_site_banner(root, &opts, &theme);
    assert!(
        res.is_ok(),
        "Banner generation must succeed: {:?}",
        res.err()
    );

    let target = root.join("site/assets/og_preview.png");
    assert!(
        target.is_file(),
        "site/assets/og_preview.png must be created"
    );

    let bytes = fs::read(&target).unwrap();
    let (width, height) = read_png_dimensions(&bytes).expect("Valid PNG with dimensions");
    assert_eq!(width, 1200, "Banner width must be exactly 1200");
    assert_eq!(height, 630, "Banner height must be exactly 630");
}

#[test]
fn test_generate_site_banner_custom_options() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let theme = Theme::with_color(false);

    let custom_output = root.join("custom/banner.png");
    let opts = BannerOptions {
        title: Some("Custom Title".to_string()),
        tagline: Some("A very custom tagline override".to_string()),
        output: Some(custom_output.clone()),
    };

    let res = generate_site_banner(root, &opts, &theme);
    assert!(
        res.is_ok(),
        "Custom banner generation must succeed: {:?}",
        res.err()
    );

    assert!(
        custom_output.is_file(),
        "Custom output path must be created"
    );
    let bytes = fs::read(&custom_output).unwrap();
    let (width, height) = read_png_dimensions(&bytes).expect("Valid PNG dimensions");
    assert_eq!(width, 1200);
    assert_eq!(height, 630);
}

#[test]
fn test_generate_site_banner_mutation_edge_cases() {
    let temp = tempdir().unwrap();
    let root = temp.path();
    let theme = Theme::with_color(false);

    // 1. Missing metadata.json: should succeed using defaults
    let opts = BannerOptions::default();
    let res = generate_site_banner(root, &opts, &theme);
    assert!(
        res.is_ok(),
        "Banner generation without metadata must succeed"
    );
    assert!(root.join("site/assets/og_preview.png").is_file());

    // 2. Metadata with malformed hex colors and excessive lengths
    fs::create_dir_all(root.join(".gost")).unwrap();
    let malformed_metadata = r##"{
  "name": "super-long-name-that-definitely-exceeds-the-title-length-threshold",
  "tagline": "This is an extraordinarily long tagline designed to verify that truncation behaves deterministically and appends an ellipsis when exceeding sixty-five characters limit in site banners",
  "theme": {
    "bg_color": "not-a-color",
    "accent_color": "#123"
  }
}"##;
    fs::write(root.join(".gost/metadata.json"), malformed_metadata).unwrap();

    let output_nested = root.join("nested/deep/dir/banner.png");
    let opts2 = BannerOptions {
        title: None,
        tagline: None,
        output: Some(output_nested.clone()),
    };
    let res2 = generate_site_banner(root, &opts2, &theme);
    assert!(
        res2.is_ok(),
        "Must handle malformed hex and deep nested paths"
    );
    assert!(output_nested.is_file(), "Deep nested output must exist");

    let bytes = fs::read(&output_nested).unwrap();
    let (w, h) = read_png_dimensions(&bytes).expect("Valid PNG dimensions");
    assert_eq!(w, 1200);
    assert_eq!(h, 630);
}
