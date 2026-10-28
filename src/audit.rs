//! CCCP GOST Standards Compliance & Quality Audit Engine
//! Implements strict repository inspection under Gosplan Technical Standard (GOST).

use crate::cultural::Theme;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone)]
pub struct AuditCheck {
    pub code: &'static str,
    pub title: &'static str,
    pub detail: String,
    pub status: CheckStatus,
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub gost_version: String,
    pub profile: String,
    pub languages: String,
    pub checks: Vec<AuditCheck>,
    pub passed: usize,
    pub warnings: usize,
    pub failures: usize,
    pub percentage: usize,
    pub is_success: bool,
}

/// Parse GOSTVER file if present
pub fn read_gostver(root: &Path) -> (String, String, String) {
    let gostver_file = root.join("GOSTVER");
    let mut ver = String::new();
    let mut prof = String::new();
    let mut langs = String::new();

    if let Ok(content) = fs::read_to_string(gostver_file) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("GOST_VERSION=") {
                ver = rest.trim_matches('"').trim_matches('\'').to_string();
            } else if let Some(rest) = trimmed.strip_prefix("GOST_PROFILE_URL=") {
                prof = rest.trim_matches('"').trim_matches('\'').to_string();
            } else if let Some(rest) = trimmed.strip_prefix("GOST_LANGUAGES=") {
                langs = rest.trim_matches('"').trim_matches('\'').to_string();
            }
        }
    }

    (
        if ver.is_empty() {
            "Não declarado".to_string()
        } else {
            ver
        },
        if prof.is_empty() {
            "Nenhum".to_string()
        } else {
            prof
        },
        if langs.is_empty() {
            "Nenhuma".to_string()
        } else {
            langs
        },
    )
}

/// Executes full GOST compliance check on the specified root directory
pub fn run_gost_audit(root: &Path, strict: bool, theme: &Theme) -> AuditReport {
    let (gost_ver, gost_prof, gost_langs) = read_gostver(root);
    let mut checks = Vec::new();

    // 1. GOST-16-GOSTVER
    if root.join("GOSTVER").is_file() && gost_ver != "Não declarado" {
        checks.push(AuditCheck {
            code: "GOST-16-GOSTVER",
            title: "Declaração de Versão via GOSTVER",
            detail: format!("Versão ativa ({})", gost_ver),
            status: CheckStatus::Pass,
        });
    } else {
        checks.push(AuditCheck {
            code: "GOST-16-GOSTVER",
            title: "Declaração de Versão via GOSTVER",
            detail: "Arquivo GOSTVER ausente ou sem GOST_VERSION declarada".to_string(),
            status: CheckStatus::Fail,
        });
    }

    // 2. GOST-01-LICENSE
    let license_path = root.join("LICENSE");
    if license_path.is_file() && fs::metadata(&license_path).map_or(0, |m| m.len()) > 0 {
        checks.push(AuditCheck {
            code: "GOST-01-LICENSE",
            title: "Licença de Código Aberto",
            detail: "Arquivo LICENSE declarado e não-vazio".to_string(),
            status: CheckStatus::Pass,
        });
    } else {
        checks.push(AuditCheck {
            code: "GOST-01-LICENSE",
            title: "Licença de Código Aberto",
            detail: "Arquivo LICENSE ausente ou vazio".to_string(),
            status: CheckStatus::Fail,
        });
    }

    // 3. GOST-02-ANATOMY-ROOT
    let required_root_files = [
        "README.md",
        "VERSION",
        "CHANGELOG.md",
        ".gitignore",
        ".editorconfig",
    ];
    let mut missing_root = Vec::new();
    for f in &required_root_files {
        if !root.join(f).is_file() {
            missing_root.push(*f);
        }
    }

    if missing_root.is_empty() {
        checks.push(AuditCheck {
            code: "GOST-02-ANATOMY-ROOT",
            title: "Anatomia Básica da Raiz",
            detail: "Arquivos básicos da raiz presentes".to_string(),
            status: CheckStatus::Pass,
        });
    } else {
        checks.push(AuditCheck {
            code: "GOST-02-ANATOMY-ROOT",
            title: "Anatomia Básica da Raiz",
            detail: format!("Arquivos ausentes na raiz: {}", missing_root.join(", ")),
            status: CheckStatus::Fail,
        });
    }

    // 4. GOST-03-VERSION-SEMVER
    let version_path = root.join("VERSION");
    if version_path.is_file() {
        let v_content = fs::read_to_string(&version_path)
            .unwrap_or_default()
            .trim()
            .to_string();
        let is_semver = semver::Version::parse(&v_content).is_ok()
            || semver::Version::parse(v_content.trim_start_matches('v')).is_ok();
        if is_semver && !v_content.is_empty() {
            checks.push(AuditCheck {
                code: "GOST-03-VERSION-SEMVER",
                title: "Versionamento Semântico Estrito",
                detail: format!("Arquivo VERSION SemVer 2.0.0 ({})", v_content),
                status: CheckStatus::Pass,
            });
        } else {
            checks.push(AuditCheck {
                code: "GOST-03-VERSION-SEMVER",
                title: "Versionamento Semântico Estrito",
                detail: format!("Versão '{}' não é SemVer 2.0.0 estrito", v_content),
                status: CheckStatus::Fail,
            });
        }
    } else {
        checks.push(AuditCheck {
            code: "GOST-03-VERSION-SEMVER",
            title: "Versionamento Semântico Estrito",
            detail: "Arquivo VERSION inexistente".to_string(),
            status: CheckStatus::Fail,
        });
    }

    // 5. GOST-04-TICKETS-COMMITS
    if root.join(".git").is_dir() {
        let repo = crate::git::GitRepo::open(Some(root)).ok();
        if let Some(r) = repo {
            if r.commits_since_tag.is_empty() {
                checks.push(AuditCheck {
                    code: "GOST-04-TICKETS-COMMITS",
                    title: "Rastreabilidade de Tickets",
                    detail: "Repositório novo (sem novos commits para auditar)".to_string(),
                    status: CheckStatus::Pass,
                });
            } else {
                let audited_commits: Vec<_> = r
                    .commits_since_tag
                    .iter()
                    .filter(|c| {
                        let lower = c.summary.to_ascii_lowercase();
                        !(lower.starts_with("chore:") || lower.starts_with("chore("))
                    })
                    .collect();

                let sample_size = audited_commits.len().min(5);
                let valid_count = audited_commits
                    .iter()
                    .take(sample_size)
                    .filter(|c| {
                        let s = &c.summary;
                        s.contains('#')
                            || s.contains("(ref")
                            || s.contains("(closes")
                            || s.contains("(resolves")
                            || s.contains("(fixes")
                    })
                    .count();

                if sample_size == 0 {
                    checks.push(AuditCheck {
                        code: "GOST-04-TICKETS-COMMITS",
                        title: "Rastreabilidade de Tickets",
                        detail:
                            "Commits recentes são tarefas de manutenção (chores isentos de ticket)"
                                .to_string(),
                        status: CheckStatus::Pass,
                    });
                } else if valid_count == sample_size {
                    checks.push(AuditCheck {
                        code: "GOST-04-TICKETS-COMMITS",
                        title: "Rastreabilidade de Tickets",
                        detail: format!(
                            "Rastreabilidade estrita de tickets ({}/{} commits recentes auditados, chores isentos)",
                            valid_count, sample_size
                        ),
                        status: CheckStatus::Pass,
                    });
                } else {
                    checks.push(AuditCheck {
                        code: "GOST-04-TICKETS-COMMITS",
                        title: "Rastreabilidade de Tickets",
                        detail: format!(
                            "Commits recentes sem referência de ticket ({}/{} conformes)",
                            valid_count, sample_size
                        ),
                        status: CheckStatus::Fail,
                    });
                }
            }
        } else {
            checks.push(AuditCheck {
                code: "GOST-04-TICKETS-COMMITS",
                title: "Rastreabilidade de Tickets",
                detail: "Repositório Git inacessível".to_string(),
                status: CheckStatus::Warn,
            });
        }
    } else {
        checks.push(AuditCheck {
            code: "GOST-04-TICKETS-COMMITS",
            title: "Rastreabilidade de Tickets",
            detail: "Não é um repositório Git".to_string(),
            status: CheckStatus::Warn,
        });
    }

    // 6. GOST-05-BRANCH-TICKET
    if root.join(".git").is_dir() {
        let repo = crate::git::GitRepo::open(Some(root)).ok();
        if let Some(r) = repo {
            let branch = r.branch.as_deref().unwrap_or("HEAD");
            if branch == "main" || branch == "master" {
                checks.push(AuditCheck {
                    code: "GOST-05-BRANCH-TICKET",
                    title: "Padrão de Branching",
                    detail: format!("Branch canônica/release ({})", branch),
                    status: CheckStatus::Pass,
                });
            } else if branch.starts_with("feat/")
                || branch.starts_with("fix/")
                || branch.starts_with("docs/")
                || branch.starts_with("refactor/")
                || branch.starts_with("test/")
                || branch.starts_with("chore/")
            {
                checks.push(AuditCheck {
                    code: "GOST-05-BRANCH-TICKET",
                    title: "Padrão de Branching",
                    detail: format!("Branch dedicada por ticket ({})", branch),
                    status: CheckStatus::Pass,
                });
            } else {
                checks.push(AuditCheck {
                    code: "GOST-05-BRANCH-TICKET",
                    title: "Padrão de Branching",
                    detail: format!("Nome da branch '{}' fora do padrão GOST", branch),
                    status: CheckStatus::Fail,
                });
            }
        } else {
            checks.push(AuditCheck {
                code: "GOST-05-BRANCH-TICKET",
                title: "Padrão de Branching",
                detail: "Git indisponível para auditar branches".to_string(),
                status: CheckStatus::Warn,
            });
        }
    } else {
        checks.push(AuditCheck {
            code: "GOST-05-BRANCH-TICKET",
            title: "Padrão de Branching",
            detail: "Não é um repositório Git".to_string(),
            status: CheckStatus::Warn,
        });
    }

    // 7. GOST-09-SITE-HEADERS
    let site_dir = root.join("site");
    if site_dir.is_dir() {
        let headers_file = site_dir.join("_headers");
        if headers_file.is_file() {
            let content = fs::read_to_string(&headers_file).unwrap_or_default();
            if content.contains("X-Content-Type-Options: nosniff")
                && (content.contains("X-Frame-Options: DENY")
                    || content.contains("X-Frame-Options: SAMEORIGIN"))
            {
                checks.push(AuditCheck {
                    code: "GOST-09-SITE-HEADERS",
                    title: "Segurança de Cabeçalhos Web",
                    detail: "Cabeçalhos HTTP de segurança presentes em site/_headers".to_string(),
                    status: CheckStatus::Pass,
                });
            } else {
                checks.push(AuditCheck {
                    code: "GOST-09-SITE-HEADERS",
                    title: "Segurança de Cabeçalhos Web",
                    detail: "site/_headers não contém nosniff ou DENY/SAMEORIGIN".to_string(),
                    status: CheckStatus::Warn,
                });
            }
        } else {
            checks.push(AuditCheck {
                code: "GOST-09-SITE-HEADERS",
                title: "Segurança de Cabeçalhos Web",
                detail: "Diretório site/ presente sem site/_headers".to_string(),
                status: CheckStatus::Warn,
            });
        }
    } else {
        checks.push(AuditCheck {
            code: "GOST-09-SITE-HEADERS",
            title: "Segurança de Cabeçalhos Web",
            detail: "Vitrine web não declarada (N/A)".to_string(),
            status: CheckStatus::Pass,
        });
    }

    // 8. GOST-10-CI-SEPARATION
    if root.join(".forgejo/workflows").is_dir() || root.join(".github/workflows").is_dir() {
        checks.push(AuditCheck {
            code: "GOST-10-CI-SEPARATION",
            title: "Automação Contínua (CI)",
            detail: "Workflows de automação CI presentes".to_string(),
            status: CheckStatus::Pass,
        });
    } else {
        checks.push(AuditCheck {
            code: "GOST-10-CI-SEPARATION",
            title: "Automação Contínua (CI)",
            detail: "Nenhum workflow de CI (.forgejo/.github) detectado".to_string(),
            status: CheckStatus::Warn,
        });
    }

    // 9. GOST-17-LANGUAGE-SPECS
    if gost_langs != "Nenhuma" && !gost_langs.is_empty() {
        let mut missing_specs = Vec::new();
        for l in gost_langs.split(',') {
            let lang = l.trim();
            if !lang.is_empty() && !root.join(format!(".gost/languages/{}.md", lang)).is_file() {
                missing_specs.push(lang);
            }
        }

        if missing_specs.is_empty() {
            checks.push(AuditCheck {
                code: "GOST-17-LANGUAGE-SPECS",
                title: "Especificações de Linguagem",
                detail: "Especificações modulares sincronizadas em .gost/languages/".to_string(),
                status: CheckStatus::Pass,
            });
        } else {
            checks.push(AuditCheck {
                code: "GOST-17-LANGUAGE-SPECS",
                title: "Especificações de Linguagem",
                detail: format!(
                    "Especificações ausentes: {} (execute 'cccp sync')",
                    missing_specs.join(", ")
                ),
                status: CheckStatus::Warn,
            });
        }
    } else {
        checks.push(AuditCheck {
            code: "GOST-17-LANGUAGE-SPECS",
            title: "Especificações de Linguagem",
            detail: "Nenhuma especificação de linguagem declarada".to_string(),
            status: CheckStatus::Pass,
        });
    }

    // 10. GOST-20-SITE-TYPE-AND-SSOT-DOCS (GOST 2.0)
    checks.push(check_gost20_docs(root));

    // 11. GOST-24-ZERO-BINARIES-IN-GIT (GOST 2.0)
    checks.push(check_gost24_binaries(root));

    // 12. GOST-26-METADATA-MANIFEST-SSOT (GOST 2.0)
    checks.push(check_gost26_metadata(root));

    // 13. GOST-27-DDD-HEXAGONAL-PURITY (GOST 2.0)
    checks.push(check_gost27_domain_purity(root));

    let passed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Pass)
        .count();
    let warnings = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Warn)
        .count();
    let failures = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let total = passed + warnings + failures;
    let percentage = (passed * 100).checked_div(total).unwrap_or(100);
    let is_success = failures == 0 && (!strict || warnings == 0);

    let report = AuditReport {
        gost_version: gost_ver,
        profile: gost_prof,
        languages: gost_langs,
        checks,
        passed,
        warnings,
        failures,
        percentage,
        is_success,
    };

    display_audit_report(&report, strict, theme);
    report
}

/// Renders the Gosplan Quality Audit Report to standard output
pub fn display_audit_report(report: &AuditReport, strict: bool, theme: &Theme) {
    println!();
    println!(
        "{}",
        theme.bold("☭ [ГОСТ / GOSPLAN] COMISSÃO DE CONTROLE DE QUALIDADE GOST")
    );
    println!(
        "Norma: {} | Perfil: {} | Linguagens: {}",
        report.gost_version, report.profile, report.languages
    );
    println!(
        "{}",
        theme.dim("────────────────────────────────────────────────────────────────────────────")
    );

    for check in &report.checks {
        let status_badge = match check.status {
            CheckStatus::Pass => theme.green("[PASS]"),
            CheckStatus::Warn => theme.yellow("[WARN]"),
            CheckStatus::Fail => theme.red_bold("[FAIL]"),
        };
        println!("  {} {:<25} : {}", status_badge, check.code, check.detail);
    }

    println!(
        "{}",
        theme.dim("────────────────────────────────────────────────────────────────────────────")
    );
    println!(
        "Total de verificações: {} | Conformes: {} | Avisos: {} | Falhas: {}",
        report.checks.len(),
        theme.green(&report.passed.to_string()),
        theme.yellow(&report.warnings.to_string()),
        if report.failures > 0 {
            theme.red_bold(&report.failures.to_string())
        } else {
            theme.green("0")
        }
    );
    println!(
        "Índice de Conformidade Gosplan: {}",
        theme.bold(&format!("{}%", report.percentage))
    );
    println!();

    if report.failures > 0 {
        eprintln!(
            "{}",
            theme.red_bold(
                "✘ Inconformidades críticas detectadas. Corrija os itens marcados com [FAIL]."
            )
        );
    } else if strict && report.warnings > 0 {
        eprintln!(
            "{}",
            theme.red_bold("✘ Modo estrito ativado: avisos tratados como falhas bloqueantes.")
        );
    } else {
        println!(
            "{}",
            theme.green("✔ Projeto aprovado pela Comissão de Normas Técnicas GOST.")
        );
    }
}

fn check_gost20_docs(root: &Path) -> AuditCheck {
    let readme = root.join("README.md");
    if !readme.is_file() {
        return AuditCheck {
            code: "GOST-20-SITE-TYPE-AND-SSOT-DOCS",
            title: "Topologia de Documentação e README",
            detail: "Arquivo README.md ausente na raiz".to_string(),
            status: CheckStatus::Fail,
        };
    }
    let content = fs::read_to_string(&readme).unwrap_or_default();
    let line_count = content.lines().count();
    if line_count > 50 {
        return AuditCheck {
            code: "GOST-20-SITE-TYPE-AND-SSOT-DOCS",
            title: "Topologia de Documentação e README",
            detail: format!(
                "README.md excede o teto normativo de 50 linhas (possui {} linhas)",
                line_count
            ),
            status: CheckStatus::Fail,
        };
    }
    if !root.join("site/index.html").is_file() {
        return AuditCheck {
            code: "GOST-20-SITE-TYPE-AND-SSOT-DOCS",
            title: "Topologia de Documentação e README",
            detail: "Diretório ou arquivo site/index.html ausente".to_string(),
            status: CheckStatus::Fail,
        };
    }
    AuditCheck {
        code: "GOST-20-SITE-TYPE-AND-SSOT-DOCS",
        title: "Topologia de Documentação e README",
        detail: format!(
            "README enxuto ({} linhas) e documentação site/ presente",
            line_count
        ),
        status: CheckStatus::Pass,
    }
}

fn check_gost24_binaries(root: &Path) -> AuditCheck {
    let forbidden_extensions = [
        ".exe",
        ".so",
        ".dll",
        ".dylib",
        ".a",
        ".tar.gz",
        ".tgz",
        ".zip",
        ".whl",
        ".deb",
        ".rpm",
        ".apk",
        ".AppImage",
    ];
    let mut found_binaries = Vec::new();
    fn scan_dir(dir: &Path, forbidden: &[&str], found: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "target"
                    || file_name == ".git"
                    || file_name == "node_modules"
                    || file_name == "vendor"
                {
                    continue;
                }
                if path.is_dir() {
                    scan_dir(&path, forbidden, found);
                } else {
                    let lower = file_name.to_lowercase();
                    for ext in forbidden {
                        if lower.ends_with(ext) {
                            found.push(file_name.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }
    scan_dir(root, &forbidden_extensions, &mut found_binaries);

    if found_binaries.is_empty() {
        AuditCheck {
            code: "GOST-24-ZERO-BINARIES-IN-GIT",
            title: "Veto a Binários no Repositório",
            detail: "Nenhum binário ou pacote compilado detectado".to_string(),
            status: CheckStatus::Pass,
        }
    } else {
        AuditCheck {
            code: "GOST-24-ZERO-BINARIES-IN-GIT",
            title: "Veto a Binários no Repositório",
            detail: format!(
                "Binários compilados detectados: {}",
                found_binaries.join(", ")
            ),
            status: CheckStatus::Fail,
        }
    }
}

fn check_gost26_metadata(root: &Path) -> AuditCheck {
    let meta_path = root.join(".gost/metadata.json");
    if !meta_path.is_file() {
        return AuditCheck {
            code: "GOST-26-METADATA-MANIFEST-SSOT",
            title: "Manifesto de Metadados e Anti-Jargão",
            detail: "Manifesto .gost/metadata.json não declarado (opcional)".to_string(),
            status: CheckStatus::Pass,
        };
    }

    let content = match fs::read_to_string(&meta_path) {
        Ok(c) => c,
        Err(_) => {
            return AuditCheck {
                code: "GOST-26-METADATA-MANIFEST-SSOT",
                title: "Manifesto de Metadados e Anti-Jargão",
                detail: "Falha ao ler .gost/metadata.json".to_string(),
                status: CheckStatus::Fail,
            };
        }
    };

    let forbidden_jargon = [
        "soberano",
        "soberana",
        "hexagonal",
        "ssot",
        "rust puro",
        "em rust",
        "em php",
        "wasm",
    ];
    let lower = content.to_lowercase();
    for jargon in &forbidden_jargon {
        if lower.contains(jargon) {
            return AuditCheck {
                code: "GOST-26-METADATA-MANIFEST-SSOT",
                title: "Manifesto de Metadados e Anti-Jargão",
                detail: format!(
                    "Jargão técnico proibido '{}' detectado em .gost/metadata.json",
                    jargon
                ),
                status: CheckStatus::Fail,
            };
        }
    }

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return AuditCheck {
                code: "GOST-26-METADATA-MANIFEST-SSOT",
                title: "Manifesto de Metadados e Anti-Jargão",
                detail: format!("JSON malformado em .gost/metadata.json: {}", e),
                status: CheckStatus::Fail,
            };
        }
    };

    let required = [
        "slug",
        "title",
        "site_type",
        "tagline",
        "summary",
        "abstract",
        "tags",
        "icon",
        "theme",
    ];
    for field in &required {
        if json.get(field).is_none() {
            return AuditCheck {
                code: "GOST-26-METADATA-MANIFEST-SSOT",
                title: "Manifesto de Metadados e Anti-Jargão",
                detail: format!(
                    "Campo obrigatório '{}' ausente no .gost/metadata.json",
                    field
                ),
                status: CheckStatus::Fail,
            };
        }
    }

    AuditCheck {
        code: "GOST-26-METADATA-MANIFEST-SSOT",
        title: "Manifesto de Metadados e Anti-Jargão",
        detail: "Manifesto .gost/metadata.json íntegro e livre de jargões".to_string(),
        status: CheckStatus::Pass,
    }
}

fn check_gost27_domain_purity(root: &Path) -> AuditCheck {
    let domain_dir = root.join("src/domain");
    if !domain_dir.is_dir() {
        return AuditCheck {
            code: "GOST-27-DDD-HEXAGONAL-PURITY",
            title: "Pureza Arquitetural do Domínio",
            detail: "Diretório src/domain/ não declarado (N/A)".to_string(),
            status: CheckStatus::Pass,
        };
    }

    let forbidden_patterns = [
        "std::fs",
        "std::net",
        "std::process",
        "tokio::fs",
        "tokio::net",
        "reqwest",
        "rusqlite",
        "sqlx",
        "clap",
    ];

    let mut violations = Vec::new();
    fn scan_domain(dir: &Path, forbidden: &[&str], violations: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    scan_domain(&path, forbidden, violations);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if let Ok(code) = fs::read_to_string(&path) {
                        for pat in forbidden {
                            if code.contains(pat) {
                                let name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("arquivo");
                                violations.push(format!("{}: {}", name, pat));
                            }
                        }
                    }
                }
            }
        }
    }

    scan_domain(&domain_dir, &forbidden_patterns, &mut violations);

    if violations.is_empty() {
        AuditCheck {
            code: "GOST-27-DDD-HEXAGONAL-PURITY",
            title: "Pureza Arquitetural do Domínio",
            detail: "Camada de domínio pura (zero I/O ou chamadas de processo)".to_string(),
            status: CheckStatus::Pass,
        }
    } else {
        AuditCheck {
            code: "GOST-27-DDD-HEXAGONAL-PURITY",
            title: "Pureza Arquitetural do Domínio",
            detail: format!("I/O proibido no core de domínio: {}", violations.join(", ")),
            status: CheckStatus::Fail,
        }
    }
}
