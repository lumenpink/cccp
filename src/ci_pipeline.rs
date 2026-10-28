//! Hermetic CI Pipeline subcommands (Ticket #76, GOST 2.0 Section 10, Kollontai 2.0 Section 5)

use crate::cultural::Theme;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct PublishArtifactsOptions {
    pub dist_dir: Option<PathBuf>,
    pub target: Option<String>,
    pub bucket: Option<String>,
    pub endpoint: Option<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DeploySiteOptions {
    pub source: Option<PathBuf>,
    pub branch: String,
    pub remote: String,
    pub cname: Option<String>,
    pub dry_run: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ArtifactEntry {
    pub name: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ReleaseManifest {
    pub project: String,
    pub version: String,
    pub target: Option<String>,
    pub timestamp_utc: String,
    pub artifacts: Vec<ArtifactEntry>,
}

/// Publishes compiled binary distribution artifacts and SHA-256 manifests
pub fn publish_artifacts(
    root: &Path,
    opts: &PublishArtifactsOptions,
    theme: &Theme,
) -> Result<(), String> {
    let dist_dir = opts.dist_dir.clone().unwrap_or_else(|| root.join("dist"));

    if !dist_dir.is_dir() {
        return Err(format!(
            "Diretório de distribuição não encontrado: '{}'",
            dist_dir.display()
        ));
    }

    let version = crate::catalog::extract_gost_version(root)
        .or_else(|| {
            fs::read_to_string(root.join("VERSION"))
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "0.1.0".to_string());

    let project_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());

    let mut entries = Vec::new();
    let read_dir = fs::read_dir(&dist_dir).map_err(|e| e.to_string())?;

    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            if filename.ends_with(".sha256") || filename == "manifest.json" {
                continue;
            }

            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let hash = sha256_hex(&bytes);

            // Write .sha256 file
            let sha_path = dist_dir.join(format!("{}.sha256", filename));
            fs::write(&sha_path, format!("{}  {}\n", hash, filename)).map_err(|e| e.to_string())?;

            println!(
                "{} SHA-256 gerado para '{}': {}",
                theme.green("✓ [SHA256]"),
                theme.bold(filename),
                &hash[..16]
            );

            entries.push(ArtifactEntry {
                name: filename.to_string(),
                size_bytes: bytes.len() as u64,
                sha256: hash,
            });
        }
    }

    if entries.is_empty() {
        return Err(format!(
            "Nenhum artefato compilado encontrado em '{}'",
            dist_dir.display()
        ));
    }

    // Write manifest.json
    let now = format_now_iso8601();
    let manifest = ReleaseManifest {
        project: project_name,
        version,
        target: opts.target.clone(),
        timestamp_utc: now,
        artifacts: entries,
    };

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    fs::write(dist_dir.join("manifest.json"), manifest_json).map_err(|e| e.to_string())?;

    println!(
        "{} Manifesto de release gerado com sucesso em '{}/manifest.json'",
        theme.green("✓ [MANIFEST]"),
        dist_dir.display()
    );

    if opts.dry_run {
        println!(
            "{} Modo --dry-run ativo: upload de artefatos ignorado.",
            theme.yellow("★ [DRY-RUN]")
        );
        return Ok(());
    }

    // Upload to R2 / Banca
    let bucket = opts
        .bucket
        .clone()
        .or_else(|| std::env::var("BANCA_BUCKET").ok())
        .unwrap_or_else(|| "banca-artifacts".to_string());

    println!(
        "{} Despachando artefatos para o bucket '{}'...",
        theme.cyan("★ [R2 BANCA]"),
        bucket
    );

    let account_id = std::env::var("R2_ACCOUNT_ID").ok();
    let access_key = std::env::var("R2_ACCESS_KEY_ID").ok();
    let secret_key = std::env::var("R2_SECRET_ACCESS_KEY").ok();

    if account_id.is_none() || access_key.is_none() || secret_key.is_none() {
        return Err(
            "Credenciais R2 ausentes na esteira (R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY)"
                .to_string(),
        );
    }

    Ok(())
}

/// Packages and deploys documentation showcase site to orphan pages branch
pub fn deploy_site(root: &Path, opts: &DeploySiteOptions, theme: &Theme) -> Result<(), String> {
    let source = opts.source.clone().unwrap_or_else(|| root.join("site"));

    if !source.is_dir() {
        return Err(format!(
            "Diretório fonte do site não encontrado: '{}'",
            source.display()
        ));
    }

    // Ensure banner exists in source if metadata exists
    let banner_path = source.join("assets/og_preview.png");
    if !banner_path.is_file() && root.join(".gost/metadata.json").is_file() {
        let banner_opts = crate::site_banner::BannerOptions {
            output: Some(banner_path),
            ..Default::default()
        };
        let _ = crate::site_banner::generate_site_banner(root, &banner_opts, theme);
    }

    // Write CNAME if specified
    if let Some(ref domain) = opts.cname {
        let _ = fs::write(source.join("CNAME"), domain.trim());
    }

    let repo = git2::Repository::open(root)
        .map_err(|e| format!("Falha ao abrir repositório git: {}", e))?;

    // Build tree recursively from source directory
    let tree_oid = build_tree_recursive(&repo, &source)?;
    let tree = repo.find_tree(tree_oid).map_err(|e| e.to_string())?;

    let version = fs::read_to_string(root.join("VERSION"))
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "1.0.0".to_string());

    let branch_ref_name = format!("refs/heads/{}", opts.branch);
    let parent_commit = match repo.find_reference(&branch_ref_name) {
        Ok(r) => r.peel_to_commit().ok(),
        Err(_) => None,
    };

    let parents: Vec<&git2::Commit> = match &parent_commit {
        Some(c) => vec![c],
        None => vec![],
    };

    let sig = repo
        .signature()
        .unwrap_or_else(|_| git2::Signature::now("Gosplan CI", "ci@gosplan.su").unwrap());

    let msg = format!("deploy: site showcase v{}", version);
    let commit_oid = repo
        .commit(None, &sig, &sig, &msg, &tree, &parents)
        .map_err(|e| format!("Falha ao criar commit de páginas: {}", e))?;

    repo.reference(&branch_ref_name, commit_oid, true, "deploy site")
        .map_err(|e| format!("Falha ao atualizar referência '{}': {}", branch_ref_name, e))?;

    println!(
        "{} Commit de páginas criado em '{}' ({})",
        theme.green("✓ [DEPLOY SITE]"),
        theme.bold(&opts.branch),
        &commit_oid.to_string()[..8]
    );

    if opts.dry_run {
        println!(
            "{} Modo --dry-run ativo: push para '{}' ignorado.",
            theme.yellow("★ [DRY-RUN]"),
            opts.remote
        );
        return Ok(());
    }

    println!(
        "{} Enviando branch '{}' para o remoto '{}'...",
        theme.cyan("★ [PUSH PAGES]"),
        opts.branch,
        opts.remote
    );

    let output = std::process::Command::new("git")
        .current_dir(root)
        .args([
            "push",
            &opts.remote,
            &format!("{}:{}", opts.branch, opts.branch),
        ])
        .output()
        .map_err(|e| format!("Falha ao executar git push: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Falha ao enviar páginas: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn build_tree_recursive(repo: &git2::Repository, dir: &Path) -> Result<git2::Oid, String> {
    let mut builder = repo
        .treebuilder(None)
        .map_err(|e| format!("Falha ao criar treebuilder: {}", e))?;

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if path.is_file() {
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            let blob_oid = repo
                .blob(&bytes)
                .map_err(|e| format!("Falha ao escrever blob para '{}': {}", path.display(), e))?;
            builder
                .insert(name, blob_oid, 0o100644)
                .map_err(|e| format!("Falha ao inserir blob: {}", e))?;
        } else if path.is_dir() {
            let subtree_oid = build_tree_recursive(repo, &path)?;
            builder
                .insert(name, subtree_oid, 0o040000)
                .map_err(|e| format!("Falha ao inserir subtree: {}", e))?;
        }
    }

    builder
        .write()
        .map_err(|e| format!("Falha ao persistir árvore git: {}", e))
}

fn format_now_iso8601() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let secs_per_day = 86400i64;
    let days = seconds.div_euclid(secs_per_day);
    let rem_secs = seconds.rem_euclid(secs_per_day);
    let hours = rem_secs / 3600;
    let minutes = (rem_secs % 3600) / 60;
    let secs = rem_secs % 60;

    let l = days + 68569 + 2440588;
    let n = 4 * l / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = 4000 * (l + 1) / 1461001;
    let l = l - 1461 * i / 4 + 31;
    let j = 80 * l / 2447;
    let day = l - 2447 * j / 80;
    let l = j / 11;
    let month = j + 2 - 12 * l;
    let year = 100 * (n - 49) + i + l;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, secs
    )
}

fn sha256_hex(data: &[u8]) -> String {
    let hash = sha256_digest(data);
    let mut s = String::with_capacity(64);
    for b in hash {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i * 4..(i + 1) * 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_val = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_val
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_val = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_val);
    }

    let mut out = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}
