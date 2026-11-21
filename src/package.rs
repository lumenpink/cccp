use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use tar::Builder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageUploadConfig {
    pub host: String,
    pub owner: String,
    pub app: String,
    pub version: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageUploadReport {
    pub url: String,
    pub filename: String,
    pub size_bytes: u64,
}

pub fn build_generic_package_url(
    host: &str,
    owner: &str,
    app: &str,
    version: &str,
    filename: &str,
) -> String {
    format!(
        "https://{}/api/packages/{}/generic/{}/{}/{}",
        host.trim_end_matches('/'),
        owner.trim_matches('/'),
        app.trim_matches('/'),
        version.trim_matches('/'),
        filename.trim_matches('/')
    )
}

/// Compacta recursivamente o conteúdo de `dir_path` em um arquivo gzip `.tar.gz`
pub fn package_dir_to_tarball(dir_path: &Path, output_path: &Path) -> Result<u64, io::Error> {
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Diretório para empacotamento não encontrado: {:?}",
                dir_path
            ),
        ));
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tar_gz = File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all(".", dir_path)?;
    let mut enc = tar.into_inner()?;
    enc.try_finish()?;

    let metadata = fs::metadata(output_path)?;
    Ok(metadata.len())
}

/// Identifica versões mutáveis (dev/nightly/rolling) onde um rerun de CI pode
/// sobrescrever com segurança um pacote já existente via DELETE + PUT em caso de 409 Conflict.
pub fn is_mutable_package_version(version: &str) -> bool {
    let lower = version.trim().to_ascii_lowercase();
    lower.contains("-dev")
        || lower.contains("nightly")
        || lower == "latest"
        || lower.contains("-alpha")
        || lower.contains("-beta")
        || lower.contains("-rc")
        || lower.contains("canary")
}

/// Envia arquivo ou diretório empacotado para o Generic Package Registry da forja
pub fn upload_package(
    target_path: &Path,
    config: &PackageUploadConfig,
) -> Result<PackageUploadReport, Box<dyn std::error::Error>> {
    if !target_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Alvo de upload não encontrado: {:?}", target_path),
        )
        .into());
    }

    let (upload_file, filename, _tmp_guard) = if target_path.is_dir() {
        let dir_name = target_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("bundle");
        let archive_filename = format!("{}.tar.gz", dir_name);
        let tmp_file = tempfile::Builder::new()
            .prefix("cccp-pkg-")
            .suffix(".tar.gz")
            .tempfile()?;
        let tmp_path = tmp_file.path().to_path_buf();

        package_dir_to_tarball(target_path, &tmp_path)?;
        (tmp_path, archive_filename, Some(tmp_file))
    } else {
        let name = target_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Nome de arquivo inválido"))?
            .to_string();
        (target_path.to_path_buf(), name, None)
    };

    let data = fs::read(&upload_file)?;
    let size_bytes = data.len() as u64;
    let url = build_generic_package_url(
        &config.host,
        &config.owner,
        &config.app,
        &config.version,
        &filename,
    );

    let auth_header = format!("token {}", config.token);
    let send_put = || {
        ureq::put(&url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/octet-stream")
            .send(&data)
    };

    let res = match send_put() {
        Ok(r) => r,
        Err(ureq::Error::StatusCode(409)) if is_mutable_package_version(&config.version) => {
            let _ = ureq::delete(&url)
                .header("Authorization", &auth_header)
                .call();
            send_put()?
        }
        Err(e) => return Err(e.into()),
    };

    let status = res.status();
    if status == 200 || status == 201 {
        Ok(PackageUploadReport {
            url,
            filename,
            size_bytes,
        })
    } else {
        Err(io::Error::other(format!(
            "Forja retornou status HTTP {} ao enviar pacote para {}",
            status, url
        ))
        .into())
    }
}

/// Autodetecta coordenadas da forja, app, versão e token a partir do repositório local
pub fn resolve_package_upload_config(
    repo_dir: &Path,
    override_app: Option<&str>,
    override_version: Option<&str>,
) -> Result<PackageUploadConfig, String> {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("Falha ao executar 'git remote get-url origin': {}", e))?;

    if !output.status.success() {
        return Err("Nenhum remote 'origin' encontrado no repositório".to_string());
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let forge_info = crate::forges::parse_remote_url(&url)
        .ok_or_else(|| format!("Não foi possível analisar remote URL '{}'", url))?;

    let host = forge_info.host.clone();
    let owner = forge_info.owner.clone();
    let app = override_app
        .map(|s| s.to_string())
        .unwrap_or_else(|| forge_info.repo.clone());

    let version = if let Some(v) = override_version {
        v.to_string()
    } else {
        // Tenta ler VERSION ou Cargo.toml
        let version_file = repo_dir.join("VERSION");
        if version_file.exists() {
            fs::read_to_string(&version_file)
                .unwrap_or_default()
                .trim()
                .to_string()
        } else {
            "0.1.0".to_string()
        }
    };

    let token = crate::forges::resolve_token_for_host(&host, forge_info.provider, None)
        .ok_or_else(|| {
            format!(
                "Nenhum token encontrado para a forja '{}'. Defina {} no ambiente.",
                host,
                crate::forges::host_to_token_env_var(&host)
            )
        })?;

    Ok(PackageUploadConfig {
        host,
        owner,
        app,
        version,
        token,
    })
}
