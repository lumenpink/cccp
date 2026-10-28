//! Deterministic Open Graph Preview Banner Generator (1200x630, Safe Zone 4:3)
//! Compliant with GOST-20, GOST-26, and Section 6.2 of Kollontai 2.0.

use crate::cultural::Theme;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const BANNER_WIDTH: u32 = 1200;
const BANNER_HEIGHT: u32 = 630;

/// Options for configuring site banner generation
#[derive(Debug, Clone, Default)]
pub struct BannerOptions {
    pub title: Option<String>,
    pub tagline: Option<String>,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct BannerConfig {
    pub title: String,
    pub tagline: String,
    pub bg_color: (u8, u8, u8),
    pub accent_color: (u8, u8, u8),
    pub text_color: (u8, u8, u8),
    pub gost_version: String,
}

/// Generates the deterministic Open Graph preview banner
pub fn generate_site_banner(
    root: &Path,
    opts: &BannerOptions,
    theme: &Theme,
) -> Result<PathBuf, String> {
    let config = resolve_banner_config(root, opts)?;

    let target_path = opts
        .output
        .clone()
        .unwrap_or_else(|| root.join("site/assets/og_preview.png"));

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Falha ao criar diretório para o banner '{}': {}",
                parent.display(),
                e
            )
        })?;
    }

    let png_bytes = render_banner_png(&config);
    fs::write(&target_path, png_bytes).map_err(|e| {
        format!(
            "Falha ao salvar banner em '{}': {}",
            target_path.display(),
            e
        )
    })?;

    println!(
        "{} Banner Open Graph gerado com sucesso: {}",
        theme.green("✓ [BANNER]"),
        theme.bold(&target_path.display().to_string())
    );

    Ok(target_path)
}

fn resolve_banner_config(root: &Path, opts: &BannerOptions) -> Result<BannerConfig, String> {
    let meta_path = root.join(".gost/metadata.json");
    let mut meta_title = None;
    let mut meta_tagline = None;
    let mut meta_bg = None;
    let mut meta_accent = None;
    let mut meta_gost_ver = None;

    if meta_path.is_file() {
        if let Ok(content) = fs::read_to_string(&meta_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(t) = json.get("title").and_then(|v| v.as_str()) {
                    meta_title = Some(t.to_string());
                } else if let Some(n) = json.get("name").and_then(|v| v.as_str()) {
                    meta_title = Some(n.to_string());
                }
                if let Some(tag) = json.get("tagline").and_then(|v| v.as_str()) {
                    meta_tagline = Some(tag.to_string());
                }
                if let Some(th) = json.get("theme") {
                    if let Some(bg) = th.get("bg_color").and_then(|v| v.as_str()) {
                        meta_bg = parse_hex_color(bg);
                    }
                    if let Some(acc) = th.get("accent_color").and_then(|v| v.as_str()) {
                        meta_accent = parse_hex_color(acc);
                    }
                }
                if let Some(gv) = json.get("gost_version").and_then(|v| v.as_str()) {
                    meta_gost_ver = Some(gv.to_string());
                }
            }
        }
    }

    let gost_version = meta_gost_ver.unwrap_or_else(|| {
        crate::catalog::extract_gost_version(root)
            .unwrap_or_else(|| crate::newrepo::DEFAULT_GOST_VERSION.to_string())
    });

    let default_title = root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Projeto Gosplan".to_string());

    let mut title = opts.title.clone().or(meta_title).unwrap_or(default_title);
    if title.chars().count() > 22 {
        title = title.chars().take(19).collect::<String>() + "...";
    }

    let default_tagline = extract_readme_tagline(root)
        .unwrap_or_else(|| "Especificação e Arquitetura Aberta".to_string());

    let mut tagline = opts
        .tagline
        .clone()
        .or(meta_tagline)
        .unwrap_or(default_tagline);

    if tagline.chars().count() > 65 {
        tagline = tagline.chars().take(62).collect::<String>() + "...";
    }

    let bg_color = meta_bg.unwrap_or((17, 19, 23)); // #111317
    let accent_color = meta_accent.unwrap_or((239, 68, 68)); // #ef4444
    let text_color = (244, 244, 245); // #f4f4f5

    Ok(BannerConfig {
        title,
        tagline,
        bg_color,
        accent_color,
        text_color,
        gost_version,
    })
}

fn extract_readme_tagline(root: &Path) -> Option<String> {
    let readme = root.join("README.md");
    let content = fs::read_to_string(readme).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#')
            || trimmed.starts_with("<!--")
            || trimmed.starts_with('>')
            || trimmed.is_empty()
        {
            continue;
        }
        return Some(trimmed.to_string());
    }
    None
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some((r, g, b))
}

fn render_banner_png(config: &BannerConfig) -> Vec<u8> {
    let w = BANNER_WIDTH as usize;
    let h = BANNER_HEIGHT as usize;
    let mut buffer = vec![0u8; w * h * 3];

    // 1. Fill Background
    for pixel in buffer.as_chunks_mut::<3>().0 {
        pixel[0] = config.bg_color.0;
        pixel[1] = config.bg_color.1;
        pixel[2] = config.bg_color.2;
    }

    let mut canvas = Canvas::new(&mut buffer, w, h);

    // 2. Draw Safe Zone Card (840x500 centered at (180, 65))
    let card_x = 180;
    let card_y = 65;
    let card_w = 840;
    let card_h = 500;

    // Card background slightly lighter / darker
    let card_bg = (
        config.bg_color.0.saturating_add(6),
        config.bg_color.1.saturating_add(6),
        config.bg_color.2.saturating_add(8),
    );
    canvas.draw_rect(card_x, card_y, card_w, card_h, card_bg);

    // Accent top border (4px)
    canvas.draw_rect(card_x, card_y, card_w, 4, config.accent_color);

    // Subtle outline around card
    canvas.draw_rect_outline(card_x, card_y, card_w, card_h, 1, (40, 44, 52));

    // 3. Draw Badge [ГОСТ vX.X.X]
    let badge_text = format!("ГОСТ v{}", config.gost_version);
    let badge_x = card_x + 60;
    let badge_y = card_y + 60;
    canvas.draw_text(badge_x, badge_y, &badge_text, 2, config.accent_color);

    // 4. Draw Title (large font, scale 4)
    let title_x = card_x + 60;
    let title_y = card_y + 130;
    canvas.draw_text(title_x, title_y, &config.title, 4, config.text_color);

    // 5. Draw Separator Line
    let sep_y = card_y + 240;
    canvas.draw_rect(card_x + 60, sep_y, card_w - 120, 2, (50, 54, 64));

    // 6. Draw Tagline (scale 2)
    let tagline_x = card_x + 60;
    let tagline_y = card_y + 280;
    canvas.draw_text(tagline_x, tagline_y, &config.tagline, 2, (180, 185, 195));

    // 7. Footer inside card
    let footer_text = "SOVEREIGN OPEN SOURCE • GOSPLAN VERIFIED";
    let footer_x = card_x + 60;
    let footer_y = card_y + card_h - 60;
    canvas.draw_text(footer_x, footer_y, footer_text, 2, (100, 105, 115));

    encode_rgb_png(BANNER_WIDTH, BANNER_HEIGHT, &buffer)
}

struct Canvas<'a> {
    buffer: &'a mut [u8],
    width: usize,
    height: usize,
}

impl<'a> Canvas<'a> {
    fn new(buffer: &'a mut [u8], width: usize, height: usize) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    fn draw_rect(&mut self, rx: usize, ry: usize, rw: usize, rh: usize, color: (u8, u8, u8)) {
        let x_end = (rx + rw).min(self.width);
        let y_end = (ry + rh).min(self.height);

        for y in ry..y_end {
            let row_start = y * self.width * 3;
            for x in rx..x_end {
                let idx = row_start + x * 3;
                self.buffer[idx] = color.0;
                self.buffer[idx + 1] = color.1;
                self.buffer[idx + 2] = color.2;
            }
        }
    }

    fn draw_rect_outline(
        &mut self,
        rx: usize,
        ry: usize,
        rw: usize,
        rh: usize,
        thickness: usize,
        color: (u8, u8, u8),
    ) {
        // Top
        self.draw_rect(rx, ry, rw, thickness, color);
        // Bottom
        self.draw_rect(
            rx,
            (ry + rh).saturating_sub(thickness),
            rw,
            thickness,
            color,
        );
        // Left
        self.draw_rect(rx, ry, thickness, rh, color);
        // Right
        self.draw_rect(
            (rx + rw).saturating_sub(thickness),
            ry,
            thickness,
            rh,
            color,
        );
    }

    fn draw_text(
        &mut self,
        start_x: usize,
        start_y: usize,
        text: &str,
        scale: usize,
        color: (u8, u8, u8),
    ) {
        let mut cursor_x = start_x;
        for ch in text.chars() {
            if cursor_x + (8 * scale) >= self.width {
                break;
            }
            self.draw_char(cursor_x, start_y, ch, scale, color);
            cursor_x += (8 + 1) * scale;
        }
    }

    fn draw_char(&mut self, cx: usize, cy: usize, ch: char, scale: usize, color: (u8, u8, u8)) {
        let glyph = get_glyph(ch);
        for (row, &byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                if (byte >> (7 - col)) & 1 != 0 {
                    let px = cx + col * scale;
                    let py = cy + row * scale;
                    self.draw_rect(px, py, scale, scale, color);
                }
            }
        }
    }
}

/// Fallback 8x8 font matrix for ASCII 32..=126 and basic Cyrillic/Portuguese chars
fn get_glyph(ch: char) -> [u8; 8] {
    match ch {
        'A' | 'a' => [0x18, 0x24, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
        'B' | 'b' => [0x7C, 0x42, 0x42, 0x7C, 0x42, 0x42, 0x7C, 0x00],
        'C' | 'c' => [0x3C, 0x42, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00],
        'D' | 'd' => [0x78, 0x44, 0x42, 0x42, 0x42, 0x44, 0x78, 0x00],
        'E' | 'e' => [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x7E, 0x00],
        'F' | 'f' => [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x40, 0x00],
        'G' | 'g' => [0x3C, 0x42, 0x40, 0x4E, 0x42, 0x42, 0x3C, 0x00],
        'H' | 'h' => [0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00],
        'I' | 'i' => [0x3E, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x3E, 0x00],
        'J' | 'j' => [0x1F, 0x0E, 0x0E, 0x0E, 0x0E, 0x4E, 0x38, 0x00],
        'K' | 'k' => [0x42, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x00],
        'L' | 'l' => [0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00],
        'M' | 'm' => [0x42, 0x66, 0x5A, 0x42, 0x42, 0x42, 0x42, 0x00],
        'N' | 'n' => [0x42, 0x62, 0x52, 0x4A, 0x46, 0x42, 0x42, 0x00],
        'O' | 'o' => [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'P' | 'p' => [0x7C, 0x42, 0x42, 0x7C, 0x40, 0x40, 0x40, 0x00],
        'Q' | 'q' => [0x3C, 0x42, 0x42, 0x42, 0x4A, 0x44, 0x3A, 0x00],
        'R' | 'r' => [0x7C, 0x42, 0x42, 0x7C, 0x48, 0x44, 0x42, 0x00],
        'S' | 's' => [0x3C, 0x42, 0x40, 0x3C, 0x02, 0x42, 0x3C, 0x00],
        'T' | 't' => [0x7F, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x00],
        'U' | 'u' => [0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'V' | 'v' => [0x42, 0x42, 0x42, 0x42, 0x24, 0x24, 0x18, 0x00],
        'W' | 'w' => [0x42, 0x42, 0x42, 0x5A, 0x5A, 0x66, 0x42, 0x00],
        'X' | 'x' => [0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x42, 0x00],
        'Y' | 'y' => [0x42, 0x42, 0x24, 0x18, 0x18, 0x18, 0x18, 0x00],
        'Z' | 'z' => [0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x7E, 0x00],
        '0' => [0x3C, 0x46, 0x4A, 0x52, 0x62, 0x42, 0x3C, 0x00],
        '1' => [0x18, 0x28, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00],
        '2' => [0x3C, 0x42, 0x02, 0x0C, 0x30, 0x40, 0x7E, 0x00],
        '3' => [0x3C, 0x42, 0x02, 0x1C, 0x02, 0x42, 0x3C, 0x00],
        '4' => [0x0C, 0x14, 0x24, 0x44, 0x7E, 0x04, 0x04, 0x00],
        '5' => [0x7E, 0x40, 0x7C, 0x02, 0x02, 0x42, 0x3C, 0x00],
        '6' => [0x3C, 0x40, 0x7C, 0x42, 0x42, 0x42, 0x3C, 0x00],
        '7' => [0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x20, 0x00],
        '8' => [0x3C, 0x42, 0x42, 0x3C, 0x42, 0x42, 0x3C, 0x00],
        '9' => [0x3C, 0x42, 0x42, 0x3E, 0x02, 0x02, 0x3C, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x08, 0x10],
        ':' => [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00],
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00],
        '/' => [0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x00, 0x00],
        '•' => [0x00, 0x18, 0x3C, 0x3C, 0x18, 0x00, 0x00, 0x00],
        '[' => [0x3C, 0x20, 0x20, 0x20, 0x20, 0x20, 0x3C, 0x00],
        ']' => [0x3C, 0x04, 0x04, 0x04, 0x04, 0x04, 0x3C, 0x00],
        'Г' | 'г' => [0x7E, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x00],
        'О' | 'о' => [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00],
        'С' | 'с' => [0x3C, 0x42, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00],
        'Т' | 'т' => [0x7F, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x00],
        _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    }
}

fn encode_rgb_png(width: u32, height: u32, rgb_data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    let mut ihdr_data = Vec::with_capacity(13);
    ihdr_data.extend_from_slice(&width.to_be_bytes());
    ihdr_data.extend_from_slice(&height.to_be_bytes());
    ihdr_data.push(8); // bit depth 8
    ihdr_data.push(2); // color type 2: RGB
    ihdr_data.push(0); // compression method 0
    ihdr_data.push(0); // filter method 0
    ihdr_data.push(0); // interlace 0
    write_chunk(&mut out, b"IHDR", &ihdr_data);

    let stride = (width * 3) as usize;
    let mut uncompressed = Vec::with_capacity((height as usize) * (stride + 1));
    for y in 0..height {
        uncompressed.push(0); // filter: None
        let start = (y as usize) * stride;
        let end = start + stride;
        uncompressed.extend_from_slice(&rgb_data[start..end]);
    }

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&uncompressed).unwrap();
    let compressed = encoder.finish().unwrap();
    write_chunk(&mut out, b"IDAT", &compressed);

    write_chunk(&mut out, b"IEND", &[]);

    out
}

fn write_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);

    let mut crc = crc32_update(0xffffffff, chunk_type);
    crc = crc32_update(crc, data) ^ 0xffffffff;
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_update(mut crc: u32, data: &[u8]) -> u32 {
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xedb88320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}
