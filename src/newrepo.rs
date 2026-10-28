//! CCCP Repository Scaffolding Engine (GOST Standard)
//! Creates new repositories or aligns existing directories with GOST v1.1.0 and CCCP conventions.

use crate::cultural::Theme;
use crate::hooks_install::install_hooks;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const DEFAULT_GOST_VERSION: &str = "2.0.0";
pub const DEFAULT_PROFILE_URL: &str = "profiles/lumen.md";
pub const DEFAULT_LANGUAGES: &str = "shell";
pub const DEFAULT_LICENSE: &str = "GPL-3.0";
pub const DEFAULT_VERSION_CONTENT: &str = "0.1.0-dev.1\n";

/// Options configuring repository scaffolding
#[derive(Debug, Clone)]
pub struct NewrepoOptions {
    pub name: Option<String>,
    pub root: Option<PathBuf>,
    pub desc: Option<String>,
    pub lang: String,
    pub license: String,
    pub profile: String,
    pub git: bool,
    pub hooks: bool,
    pub ci: bool,
    pub sync: bool,
    pub quiet: bool,
}

impl Default for NewrepoOptions {
    fn default() -> Self {
        Self {
            name: None,
            root: None,
            desc: None,
            lang: DEFAULT_LANGUAGES.to_string(),
            license: DEFAULT_LICENSE.to_string(),
            profile: DEFAULT_PROFILE_URL.to_string(),
            git: true,
            hooks: true,
            ci: true,
            sync: true,
            quiet: false,
        }
    }
}

/// Generates standard SPDX license text
pub fn generate_license_text(spdx: &str, project_name: &str) -> String {
    let year = "2026";

    match spdx.trim() {
        "CC0-1.0" | "CC0" => "CC0 1.0 Universal (CC0 1.0) Public Domain Dedication\n\n\
The person who associated a work with this deed has dedicated the work to the\n\
public domain by waiving all of his or her rights to the work worldwide under\n\
copyright law, including all related and neighboring rights, to the extent\n\
allowed by law.\n\n\
You can copy, modify, distribute and perform the work, even for commercial\n\
purposes, all without asking permission.\n"
            .to_string(),
        "MIT" => {
            format!(
                "MIT License\n\n\
Copyright (c) {} {}\n\n\
Permission is hereby granted, free of charge, to any person obtaining a copy\n\
of this software and associated documentation files (the \"Software\"), to deal\n\
in the Software without restriction, including without limitation the rights\n\
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell\n\
copies of the Software, and to permit persons to whom the Software is\n\
furnished to do so, subject to the following conditions:\n\n\
The above copyright notice and this permission notice shall be included in all\n\
copies or substantial portions of the Software.\n\n\
THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR\n\
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,\n\
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE\n\
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER\n\
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,\n\
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE\n\
SOFTWARE.\n",
                year, project_name
            )
        }
        "CC-BY-SA-4.0" | "CC-BY-SA" => {
            "Creative Commons Attribution-ShareAlike 4.0 International (CC-BY-SA 4.0)\n\n\
You are free to:\n\
- Share — copy and redistribute the material in any medium or format\n\
- Adapt — remix, transform, and build upon the material for any purpose, even commercially.\n\n\
Under the following terms:\n\
- Attribution — You must give appropriate credit, provide a link to the license,\n\
  and indicate if changes were made.\n\
- ShareAlike — If you remix, transform, or build upon the material, you must\n\
  distribute your contributions under the same license as the original.\n"
                .to_string()
        }
        "GPL-3.0" | "GPLv3" => {
            format!(
                "GNU GENERAL PUBLIC LICENSE\n\
Version 3, 29 June 2007\n\n\
Copyright (C) {} {}\n\n\
This program is free software: you can redistribute it and/or modify\n\
it under the terms of the GNU General Public License as published by\n\
the Free Software Foundation, either version 3 of the License, or\n\
(at your option) any later version.\n\n\
This program is distributed in the hope that it will be useful,\n\
but WITHOUT ANY WARRANTY; without even the implied warranty of\n\
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the\n\
GNU General Public License for more details.\n",
                year, project_name
            )
        }
        "Apache-2.0" | "Apache" => {
            format!(
                "Apache License\n\
Version 2.0, January 2004\n\
http://www.apache.org/licenses/\n\n\
Copyright {} {}\n\n\
Licensed under the Apache License, Version 2.0 (the \"License\");\n\
you may not use this file except in compliance with the License.\n\
You may obtain a copy of the License at\n\n\
    http://www.apache.org/licenses/LICENSE-2.0\n\n\
Unless required by applicable law or agreed to in writing, software\n\
distributed under the License is distributed on an \"AS IS\" BASIS,\n\
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.\n\
See the License for the specific language governing permissions and\n\
limitations under the License.\n",
                year, project_name
            )
        }
        _ => {
            format!(
                "Software License: {}\nCopyright (c) {} {}\n",
                spdx, year, project_name
            )
        }
    }
}

/// Helper: Write file idempotently (preserves existing files)
pub fn write_file_if_missing(
    target: &Path,
    content: &str,
    label: &str,
    quiet: bool,
    theme: &Theme,
) -> Result<bool, String> {
    if target.is_file() {
        if !quiet {
            println!("  {} {} (preservado)", theme.yellow("[EXISTE]"), label);
        }
        return Ok(false);
    }

    if let Some(parent) = target.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
        }
    }

    fs::write(target, content)
        .map_err(|e| format!("Failed to write file '{}': {}", target.display(), e))?;

    if !quiet {
        println!("  {} {}", theme.green("[CRIADO]"), label);
    }
    Ok(true)
}

/// Fetches a GOST specification resource from local repository or remote Codeberg
pub fn fetch_gost_resource(
    target_res: &str,
    target_dest: &Path,
    version: &str,
) -> Result<(), String> {
    // 1. Check local paths
    let local_candidates = [
        std::env::var("GOST_LOCAL_PATH").ok().map(PathBuf::from),
        Some(PathBuf::from("/home/lumen/src/gost")),
        Some(PathBuf::from("../gost")),
    ];

    for candidate in local_candidates.into_iter().flatten() {
        if candidate.is_dir() {
            let local_file = candidate.join(target_res);
            if local_file.is_file() {
                if let Some(parent) = target_dest.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if fs::copy(&local_file, target_dest).is_ok() {
                    return Ok(());
                }
            }
        }
    }

    // 2. Direct HTTP URL
    if target_res.starts_with("http://") || target_res.starts_with("https://") {
        if let Ok(resp) = ureq::get(target_res).call() {
            if let Ok(body) = resp.into_body().read_to_string() {
                let lower = body.to_ascii_lowercase();
                if !lower.contains("<html") && !lower.contains("<!doctype") {
                    if let Some(parent) = target_dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if fs::write(target_dest, body).is_ok() {
                        return Ok(());
                    }
                }
            }
        }
    }

    // 3. Remote candidate URLs (sovereign forge git.2lp.in and gost.2lp.in)
    let remote_candidates = [
        format!("https://gost.2lp.in/{}", target_res),
        format!(
            "https://git.2lp.in/lumen/gost/raw/tag/v{}/{}",
            version, target_res
        ),
        format!(
            "https://git.2lp.in/lumen/gost/raw/tag/{}/{}",
            version, target_res
        ),
        format!(
            "https://git.2lp.in/lumen/gost/raw/branch/main/{}",
            target_res
        ),
    ];

    for url in &remote_candidates {
        if let Ok(resp) = ureq::get(url).call() {
            if let Ok(body) = resp.into_body().read_to_string() {
                let lower = body.to_ascii_lowercase();
                if !lower.contains("<html") && !lower.contains("<!doctype") {
                    if let Some(parent) = target_dest.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if fs::write(target_dest, body).is_ok() {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(format!(
        "Resource '{}' could not be fetched locally or remotely.",
        target_res
    ))
}

/// Synchronizes GOST standard and language specs into `.gost/`
pub fn sync_gost_specs(
    root_dir: &Path,
    profile_url: &str,
    languages: &str,
    version: &str,
    quiet: bool,
    theme: &Theme,
) {
    let gost_dir = root_dir.join(".gost");
    let lang_dir = gost_dir.join("languages");
    let _ = fs::create_dir_all(&lang_dir);

    // 1. SPEC.md -> .gost/GOST.md
    let _ = fetch_gost_resource("SPEC.md", &gost_dir.join("GOST.md"), version);

    // 2. Profile URL -> .gost/PROFILE.md
    if !profile_url.is_empty() {
        let _ = fetch_gost_resource(profile_url, &gost_dir.join("PROFILE.md"), version);
    }

    // 3. specs/rules.json -> .gost/rules.json
    let _ = fetch_gost_resource("specs/rules.json", &gost_dir.join("rules.json"), version);

    // 4. Languages specs
    for lang in languages.split(',') {
        let l = lang.trim();
        if !l.is_empty() {
            let res_name = format!("specs/languages/{}.md", l);
            let dest_file = lang_dir.join(format!("{}.md", l));
            let _ = fetch_gost_resource(&res_name, &dest_file, version);
        }
    }

    if !quiet && gost_dir.is_dir() {
        println!(
            "  {} Especificações GOST sincronizadas em .gost/",
            theme.green("[GOST]")
        );
    }
}

/// Executes repository scaffolding
pub fn cmd_newrepo(opts: NewrepoOptions, theme: &Theme) -> Result<(), String> {
    if opts.name.is_none() && opts.root.is_none() {
        return Err(
            "Erro: Informe o nome do repositório a ser criado (ex: cccp newrepo meu-app)."
                .to_string(),
        );
    }

    let target_root = match opts.root {
        Some(ref r) => r.clone(),
        None => PathBuf::from(opts.name.as_ref().unwrap()),
    };

    let project_name = match opts.name {
        Some(ref n) => n.clone(),
        None => target_root
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "projeto".to_string()),
    };

    let desc = opts.desc.unwrap_or_else(|| {
        "Repositório estruturado em estrita conformidade com o padrão GOST v1.1.0".to_string()
    });

    if !target_root.exists() {
        fs::create_dir_all(&target_root).map_err(|e| {
            format!(
                "Failed to create target root directory '{}': {}",
                target_root.display(),
                e
            )
        })?;
    }

    if !opts.quiet {
        println!(
            "{}",
            theme.bold(&format!(
                "☭ [ГОСТ / GOSPLAN] Scaffolding de Repositório: {}",
                project_name
            ))
        );
        println!(
            "Diretório Alvo: {} | Linguagens: {} | Licença: {}",
            target_root.display(),
            opts.lang,
            opts.license
        );
        println!("────────────────────────────────────────────────────────────────────────────");
    }

    // 1. Git Repository Initialization
    if opts.git && !target_root.join(".git").is_dir() {
        let init_status = Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(&target_root)
            .output()
            .or_else(|_| {
                Command::new("git")
                    .arg("init")
                    .current_dir(&target_root)
                    .output()
            });

        if let Ok(out) = init_status {
            if out.status.success() && !opts.quiet {
                println!(
                    "  {} Repositório Git inicializado na branch main",
                    theme.green("[GIT]")
                );
            }
        }
    }

    // 2. GOSTVER (GOST-16)
    let gostver_content = format!(
        "# GOSTVER — Declaração de Conformidade com o Padrão GOST\n\
GOST_VERSION=\"{}\"\n\
GOST_PROFILE_URL=\"{}\"\n\
GOST_LANGUAGES=\"{}\"\n",
        DEFAULT_GOST_VERSION, opts.profile, opts.lang
    );
    write_file_if_missing(
        &target_root.join("GOSTVER"),
        &gostver_content,
        "GOSTVER (v1.1.0)",
        opts.quiet,
        theme,
    )?;

    // 3. .editorconfig (GOST-02)
    let editorconfig_content = "root = true\n\n\
[*]\n\
charset = utf-8\n\
end_of_line = lf\n\
insert_final_newline = true\n\
trim_trailing_whitespace = true\n\
indent_style = space\n\
indent_size = 2\n\n\
[*.md]\n\
trim_trailing_whitespace = false\n";
    write_file_if_missing(
        &target_root.join(".editorconfig"),
        editorconfig_content,
        ".editorconfig",
        opts.quiet,
        theme,
    )?;

    // 4. .gitignore (GOST-02)
    let gitignore_content = "# Dependências e compilação\n\
/target/\n\
/vendor/\n\
/node_modules/\n\
/dist/\n\
/build/\n\n\
# Caches e arquivos temporários\n\
.cache/\n\
*.tmp\n\
*.log\n\
.DS_Store\n\n\
# Metadados de agentes locais\n\
.antigravity/\n";
    write_file_if_missing(
        &target_root.join(".gitignore"),
        gitignore_content,
        ".gitignore",
        opts.quiet,
        theme,
    )?;

    // 5. VERSION (GOST-03)
    write_file_if_missing(
        &target_root.join("VERSION"),
        DEFAULT_VERSION_CONTENT,
        "VERSION (0.1.0-dev.1)",
        opts.quiet,
        theme,
    )?;

    // 6. CHANGELOG.md (GOST-02)
    let changelog_content = "# Changelog\n\n\
All notable changes to this project will be documented in this file.\n\n\
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\n\
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n\
## [Unreleased]\n\n\
### Added\n\
- Inicialização do repositório em conformidade com o padrão GOST.\n";
    write_file_if_missing(
        &target_root.join("CHANGELOG.md"),
        changelog_content,
        "CHANGELOG.md",
        opts.quiet,
        theme,
    )?;

    // 7. LICENSE (GOST-01)
    let license_body = generate_license_text(&opts.license, &project_name);
    write_file_if_missing(
        &target_root.join("LICENSE"),
        &license_body,
        &format!("LICENSE ({})", opts.license),
        opts.quiet,
        theme,
    )?;

    // 8. README.md (GOST-02)
    let readme_content = format!(
        "# {}\n\n\
{}\n\n\
## 📋 Conformidade GOST\n\n\
Este projeto adota a norma **[GOST](https://gost.2lp.in)** (Gosplan Standard for Technical Software).\n\
- **Versão GOST:** `{}`\n\
- **Linguagens declaradas:** `{}`\n\
- **Licença:** `{}`\n\n\
## 🚀 Como Iniciar\n\n\
```bash\n\
# Clonar o repositório\n\
git clone <url-do-repositorio>\n\
cd {}\n\n\
# Verificar conformidade\n\
cccp check\n\
```\n\n\
## 📄 Licença\n\n\
Distribuído sob a licença [{}](./LICENSE).\n",
        project_name, desc, DEFAULT_GOST_VERSION, opts.lang, opts.license, project_name, opts.license
    );
    write_file_if_missing(
        &target_root.join("README.md"),
        &readme_content,
        "README.md",
        opts.quiet,
        theme,
    )?;

    // 9. site/_headers e site/index.html (GOST-09)
    let headers_content = "/*\n\
  X-Content-Type-Options: nosniff\n\
  X-Frame-Options: DENY\n\
  Referrer-Policy: strict-origin-when-cross-origin\n";
    write_file_if_missing(
        &target_root.join("site/_headers"),
        headers_content,
        "site/_headers",
        opts.quiet,
        theme,
    )?;

    let site_index_content = format!(
        "<!DOCTYPE html>\n\
<html lang=\"pt-BR\">\n\
<head>\n\
  <meta charset=\"UTF-8\">\n\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n\
  <title>{} — Documentação</title>\n\
  <style>\n\
    body {{ font-family: monospace, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; line-height: 1.6; }}\n\
    h1 {{ border-bottom: 2px solid #333; padding-bottom: 8px; }}\n\
    .badge {{ display: inline-block; padding: 2px 8px; background: #eee; border: 1px solid #ccc; font-size: 0.9em; }}\n\
  </style>\n\
</head>\n\
<body>\n\
  <h1>{}</h1>\n\
  <p>{}</p>\n\
  <p><span class=\"badge\">GOST v{}</span> <span class=\"badge\">{}</span></p>\n\
</body>\n\
</html>\n",
        project_name, project_name, desc, DEFAULT_GOST_VERSION, opts.license
    );
    write_file_if_missing(
        &target_root.join("site/index.html"),
        &site_index_content,
        "site/index.html",
        opts.quiet,
        theme,
    )?;

    // 10. Workflows de CI (GOST-10)
    if opts.ci {
        let ci_workflow_content = "name: CI\n\n\
on:\n\
  push:\n\
    branches: [main]\n\
  pull_request:\n\n\
jobs:\n\
  check:\n\
    runs-on: ubuntu-latest\n\
    steps:\n\
      - uses: actions/checkout@v4\n\
      - name: GOST Compliance Audit\n\
        run: |\n\
          if command -v cccp >/dev/null 2>&1; then\n\
            cccp check --strict\n\
          fi\n";
        write_file_if_missing(
            &target_root.join(".forgejo/workflows/ci.yml"),
            ci_workflow_content,
            ".forgejo/workflows/ci.yml (CI)",
            opts.quiet,
            theme,
        )?;
    }

    // 11. Sincronização GOST (.gost/) (GOST-17)
    if opts.sync {
        sync_gost_specs(
            &target_root,
            &opts.profile,
            &opts.lang,
            DEFAULT_GOST_VERSION,
            opts.quiet,
            theme,
        );
    }

    // 12. Instalação de hooks semânticos CCCP
    if opts.hooks && target_root.join(".git").is_dir() {
        let hooks_dir = target_root.join(".git").join("hooks");
        if install_hooks(&hooks_dir, &target_root, false, theme).is_ok() && !opts.quiet {
            println!(
                "  {} Git hooks instalados via 'cccp install'",
                theme.green("[HOOKS]")
            );
        }
    }

    if !opts.quiet {
        println!("────────────────────────────────────────────────────────────────────────────");
        println!(
            "{}",
            theme.green(&format!(
                "✔ Repositório '{}' criado com sucesso com 100% de conformidade GOST!",
                project_name
            ))
        );
        println!(
            "\nPara iniciar o desenvolvimento:\n  cd {}\n  cccp check\n",
            target_root.display()
        );
    }

    Ok(())
}
