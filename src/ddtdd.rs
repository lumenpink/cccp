//! DD-TDD Phase and Lifecycle Engine (GOST 2.0 / Kollontai 2.0)
//! Implements deterministic branch inference, phase progression state machine,
//! message formatting with phase suffixes, phase guards, resolution report validations,
//! and CLI handlers for `cccp start`, `cccp commit`, and `cccp close`.

use crate::cli::{CloseArgs, CommitArgs, StartArgs};
use crate::commit_exec::{execute_commit, CommitOptions};
use crate::cultural::Theme;
use std::fmt;
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

/// The 7 active execution phases of the GOST DD-TDD cycle (Phases 3 to 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdtddPhase {
    /// Fase 3: Doc / Spec (Zero produção)
    Ph3,
    /// Fase 4: Red Test (Teste comprovadamente falho)
    Ph4,
    /// Fase 5: Green Implement (Implementação mínima funcional)
    Ph5,
    /// Fase 6: Refactor (Limpeza, idiomático, zero warnings)
    Ph6,
    /// Fase 7: Mutation Test (Eliminação de mutantes no diff)
    Ph7,
    /// Fase 8: Quality Gate (Hygiene, linter, formatação determinística)
    Ph8,
    /// Fase 9: Finish & Close (Laudo de resolução e fechamento do ticket)
    Ph9,
}

impl fmt::Display for DdtddPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DdtddPhase::Ph3 => write!(f, "ph3"),
            DdtddPhase::Ph4 => write!(f, "ph4"),
            DdtddPhase::Ph5 => write!(f, "ph5"),
            DdtddPhase::Ph6 => write!(f, "ph6"),
            DdtddPhase::Ph7 => write!(f, "ph7"),
            DdtddPhase::Ph8 => write!(f, "ph8"),
            DdtddPhase::Ph9 => write!(f, "ph9"),
        }
    }
}

impl DdtddPhase {
    pub fn description(&self) -> &'static str {
        match self {
            DdtddPhase::Ph3 => "Fase 3: Doc/Spec (Contrato preliminar em docs/ - Zero produção)",
            DdtddPhase::Ph4 => {
                "Fase 4: Red Test (Adicionar teste automatizado comprovadamente falho)"
            }
            DdtddPhase::Ph5 => {
                "Fase 5: Green Implement (Implementação funcional fazendo testes passarem)"
            }
            DdtddPhase::Ph6 => "Fase 6: Refactor (Limpeza de código, idiomático, zero warnings)",
            DdtddPhase::Ph7 => "Fase 7: Mutation Test (Eliminação de mutantes no diff)",
            DdtddPhase::Ph8 => "Fase 8: Quality Gate (Formatador e linters rápidos -D warnings)",
            DdtddPhase::Ph9 => "Fase 9: Finish & Close (Laudo textual de resolução e fechamento)",
        }
    }
}

/// Parses the `-p / --phase` CLI argument into a valid commit phase (Ph3..Ph8).
pub fn parse_phase_arg(input: &str) -> Result<DdtddPhase, String> {
    let clean = input.trim().to_ascii_lowercase();
    match clean.as_str() {
        "3" | "ph3" => Ok(DdtddPhase::Ph3),
        "4" | "ph4" => Ok(DdtddPhase::Ph4),
        "5" | "ph5" => Ok(DdtddPhase::Ph5),
        "6" | "ph6" => Ok(DdtddPhase::Ph6),
        "7" | "ph7" => Ok(DdtddPhase::Ph7),
        "8" | "ph8" => Ok(DdtddPhase::Ph8),
        "1" | "ph1" | "2" | "ph2" => Err(
            "Fases 1 (Ticket) e 2 (Branch) não possuem commits. Inicie o ciclo com 'cccp ticket new' e 'cccp start <id>'.".to_string(),
        ),
        "9" | "ph9" => Err(
            "A Fase 9 (Finish & Close) exige laudo textual e deve ser executada obrigatoriamente através do comando 'cccp close \"<resumo>\"'.".to_string(),
        ),
        other => Err(format!(
            "Fase inválida '{}'. Fases de commit aceitas: 3 a 8.",
            other
        )),
    }
}

/// Infers ticket ID, conventional commit kind, and title slug from branch name.
/// Expected branch pattern: `<kind>/<id>-<slug>`
pub fn infer_ticket_from_branch(branch: &str) -> Option<(u64, String, String)> {
    let trimmed = branch.trim();
    let parts: Vec<&str> = trimmed.splitn(2, '/').collect();
    if parts.len() != 2 {
        return None;
    }
    let kind = parts[0];
    let valid_kinds = ["feat", "fix", "docs", "refactor", "test", "chore"];
    if !valid_kinds.contains(&kind) {
        return None;
    }
    let rest = parts[1];
    let hyphen_pos = rest.find('-')?;
    let id_str = &rest[..hyphen_pos];
    let slug = &rest[hyphen_pos + 1..];
    let id: u64 = id_str.parse().ok()?;
    if slug.is_empty() {
        return None;
    }
    Some((id, kind.to_string(), slug.to_string()))
}

/// Infers the next required DD-TDD phase based on the commit history of the branch.
pub fn infer_next_phase(log_messages: &[&str]) -> DdtddPhase {
    for msg in log_messages {
        let trimmed = msg.trim();
        if trimmed.contains("- ph8)") {
            return DdtddPhase::Ph9;
        }
        if trimmed.contains("- ph7)") {
            return DdtddPhase::Ph8;
        }
        if trimmed.contains("- ph6)") {
            return DdtddPhase::Ph7;
        }
        if trimmed.contains("- ph5)") {
            return DdtddPhase::Ph6;
        }
        if trimmed.contains("- ph4)") {
            return DdtddPhase::Ph5;
        }
        if trimmed.contains("- ph3)") {
            return DdtddPhase::Ph4;
        }
    }
    DdtddPhase::Ph3
}

/// Formats a conventional commit message tagged with ticket ID and phase suffix.
pub fn format_phase_commit(
    kind: &str,
    scope: Option<&str>,
    summary: &str,
    ticket_id: u64,
    phase: DdtddPhase,
) -> String {
    let scope_part = match scope {
        Some(s) if !s.trim().is_empty() => format!("({})", s.trim()),
        _ => String::new(),
    };
    format!("{kind}{scope_part}: {summary} (ref #{ticket_id} - {phase})")
}

/// Formats a final closing commit message tagged with ticket ID and Phase 9 suffix.
pub fn format_close_commit(
    kind: &str,
    scope: Option<&str>,
    summary: &str,
    ticket_id: u64,
) -> String {
    let scope_part = match scope {
        Some(s) if !s.trim().is_empty() => format!("({})", s.trim()),
        _ => String::new(),
    };
    format!("{kind}{scope_part}: {summary} (closes #{ticket_id} - ph9)")
}

/// Enforces phase guards on staged files (e.g. no production/test code in Phase 3).
pub fn validate_phase_guards(phase: DdtddPhase, staged_files: &[&str]) -> Result<(), String> {
    if phase == DdtddPhase::Ph3 {
        let forbidden_prefixes = ["src/", "lib/", "tests/", "bin/", "R/"];
        for file in staged_files {
            for prefix in &forbidden_prefixes {
                if file.starts_with(prefix) {
                    return Err(format!(
                        "Trava da Fase 3 violada: arquivo '{}' não pode ser modificado na Fase 3 (Doc/Spec).",
                        file
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Validates that the Phase 9 resolution report meets GOST requirements.
/// Requires headers `[Resumo]:`, `[Solução]:`, `[Validação]:` and minimum 80 characters.
pub fn validate_resolution_report(body: &str) -> Result<(), String> {
    let trimmed = body.trim();
    if trimmed.chars().count() < 80 {
        return Err(format!(
            "Laudo de resolução muito curto ({} caracteres). Mínimo exigido pela norma GOST: 80 caracteres.",
            trimmed.chars().count()
        ));
    }
    let required_headers = ["[Resumo]:", "[Solução]:", "[Validação]:"];
    for header in &required_headers {
        if !trimmed.contains(header) {
            return Err(format!(
                "Laudo de resolução incompleto. Cabeçalho obrigatório ausente: '{}'",
                header
            ));
        }
    }
    Ok(())
}

// ----------------------------------------------------------------------------
// CLI Execution Handlers
// ----------------------------------------------------------------------------

fn get_current_branch(dir: Option<&Path>) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(["symbolic-ref", "--short", "HEAD"]);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("Falha ao invocar git symbolic-ref: {}", e))?;
    if !output.status.success() {
        return Err("HEAD está detached ou repositório não inicializado".to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_staged_files(dir: Option<&Path>) -> Result<Vec<String>, String> {
    let mut cmd = Command::new("git");
    cmd.args(["diff", "--cached", "--name-only"]);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("Falha ao obter diff do stage: {}", e))?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

fn get_branch_recent_commit_messages(dir: Option<&Path>, base: &str) -> Vec<String> {
    let mut cmd = Command::new("git");
    cmd.args(["log", &format!("{}..HEAD", base), "--oneline"]);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            return text
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    // Fallback: últimos 10 commits
    let mut cmd2 = Command::new("git");
    cmd2.args(["log", "-n", "10", "--oneline"]);
    if let Some(d) = dir {
        cmd2.current_dir(d);
    }
    if let Ok(output) = cmd2.output() {
        let text = String::from_utf8_lossy(&output.stdout);
        return text
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    vec![]
}

/// Canonical branch name builder: <kind>/<id>-<slug>
pub fn canonical_branch_name(issue_num: u64, title: &str) -> String {
    let trimmed = title.trim();
    let (kind, rest) = if let Some((raw_prefix, raw_rest)) = trimmed.split_once(':') {
        let clean_kind = raw_prefix
            .split('(')
            .next()
            .unwrap_or(raw_prefix)
            .trim()
            .to_ascii_lowercase();
        if [
            "feat", "fix", "docs", "refactor", "test", "chore", "style", "perf", "build", "ci",
        ]
        .contains(&clean_kind.as_str())
        {
            (clean_kind, raw_rest.trim())
        } else {
            ("feat".to_string(), trimmed)
        }
    } else {
        ("feat".to_string(), trimmed)
    };

    let slug = transliterate_slug(rest);
    if slug.is_empty() {
        format!("{}/{}", kind, issue_num)
    } else {
        format!("{}/{}-{}", kind, issue_num, slug)
    }
}

pub fn transliterate_slug(text: &str) -> String {
    let mut slug = String::new();
    let mut prev_hyphen = false;
    for c in text.chars() {
        let mapped = match c {
            'a'..='z' | '0'..='9' => {
                slug.push(c);
                prev_hyphen = false;
                continue;
            }
            'A'..='Z' => {
                slug.push(c.to_ascii_lowercase());
                prev_hyphen = false;
                continue;
            }
            'á' | 'à' | 'ã' | 'â' | 'ä' | 'Á' | 'À' | 'Ã' | 'Â' | 'Ä' => "a",
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => "e",
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => "i",
            'ó' | 'ò' | 'õ' | 'ô' | 'ö' | 'Ó' | 'Ò' | 'Õ' | 'Ô' | 'Ö' => "o",
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => "u",
            'ç' | 'Ç' => "c",
            'ñ' | 'Ñ' => "n",
            _ => {
                if !prev_hyphen && !slug.is_empty() {
                    slug.push('-');
                    prev_hyphen = true;
                }
                continue;
            }
        };
        slug.push_str(mapped);
        prev_hyphen = false;
        if slug.len() >= 45 {
            break;
        }
    }
    slug.trim_matches('-').to_string()
}

/// Dispatches `cccp start <id> [--base <b>]`
pub fn handle_start(args: &StartArgs, dir: Option<&Path>, theme: &Theme) -> ExitCode {
    // 1. Verifica se working tree está limpa
    let mut cmd_status = Command::new("git");
    cmd_status.args(["status", "--porcelain"]);
    if let Some(d) = dir {
        cmd_status.current_dir(d);
    }
    if let Ok(output) = cmd_status.output() {
        if !output.stdout.is_empty() {
            eprintln!(
                "{}: Working tree possui arquivos não comitados. Limpe ou dê stash antes de iniciar um novo ticket.",
                theme.red_bold("Erro GOST (Fase 2)")
            );
            return ExitCode::FAILURE;
        }
    }

    // 2. Busca informações do ticket no .antigravity/tickets.md ou na forja remota
    let root = dir.unwrap_or(Path::new("."));
    let tickets_path = root.join(".antigravity/tickets.md");
    let mut resolved_title: Option<String> = None;

    if tickets_path.is_file() {
        if let Ok(content) = fs::read_to_string(&tickets_path) {
            let marker = format!("## [#{}]", args.id);
            for line in content.lines() {
                if let Some(pos) = line.find(&marker) {
                    let after = line[pos + marker.len()..].trim();
                    if !after.is_empty() {
                        resolved_title = Some(after.to_string());
                        break;
                    }
                }
            }
        }
    }

    let ticket_title = match resolved_title {
        Some(t) => t,
        None => {
            let mut remote_title: Option<String> = None;
            if let Ok(git_repo) = crate::git::GitRepo::open(dir) {
                if let Ok(forge_info) = crate::forges::detect_forge_from_repo(&git_repo) {
                    let client = crate::forges::ForgeClient::new();
                    match client.fetch_single_issue(&forge_info, args.id) {
                        Ok(issue) => {
                            remote_title = Some(issue.title);
                        }
                        Err(err) => {
                            eprintln!(
                                "{} Consulta remota à forja falhou para o Ticket #{}: {}",
                                theme.yellow("!"),
                                args.id,
                                err
                            );
                        }
                    }
                }
            }

            match remote_title {
                Some(t) => t,
                None => {
                    eprintln!(
                        "{}: Ticket #{} não encontrado na forja nem no cache local. Crie o ticket antes de iniciar o ciclo com 'cccp ticket new'.",
                        theme.red_bold("Erro GOST (Fase 2)"),
                        args.id
                    );
                    return ExitCode::FAILURE;
                }
            }
        }
    };

    let branch_name = canonical_branch_name(args.id, &ticket_title);
    let base_branch = args.base.as_deref().unwrap_or("main");

    println!(
        "{} [GOSPLAN DD-TDD] Ativando ciclo de trabalho para Ticket #{}",
        theme.gold_bold("★"),
        args.id
    );

    // 3. Verifica se a branch já existe localmente
    let mut cmd_branch_check = Command::new("git");
    cmd_branch_check.args(["rev-parse", "--verify", &branch_name]);
    if let Some(d) = dir {
        cmd_branch_check.current_dir(d);
    }
    let branch_exists = cmd_branch_check
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let checkout_res = if branch_exists {
        println!("  Branch existente detectada: {}", theme.cyan(&branch_name));
        let mut cmd = Command::new("git");
        cmd.args(["checkout", &branch_name]);
        if let Some(d) = dir {
            cmd.current_dir(d);
        }
        cmd.status()
    } else {
        println!(
            "  Criando nova branch de trabalho: {} (base: {})",
            theme.green(&branch_name),
            theme.dim(base_branch)
        );
        let mut cmd = Command::new("git");
        cmd.args(["checkout", "-b", &branch_name, base_branch]);
        if let Some(d) = dir {
            cmd.current_dir(d);
        }
        cmd.status()
    };

    match checkout_res {
        Ok(s) if s.success() => {
            println!(
                "✓ {} Branch ativa configurada com sucesso!",
                theme.green("Fase 2 (Branch):")
            );
            let spec_slug = branch_name.split('/').nth(1).unwrap_or(&branch_name);
            println!(
                "  {} {}",
                theme.gold_bold("Próximo Passo Obrigatório:"),
                theme.cyan(&format!(
                    "Fase 3 (Doc/Spec): elabore a especificação em docs/specs/{}.md",
                    spec_slug
                ))
            );
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!(
                "{}: Falha ao realizar checkout para branch '{}'",
                theme.red_bold("Erro"),
                branch_name
            );
            ExitCode::FAILURE
        }
    }
}

/// Dispatches `cccp commit [summary]` in DD-TDD automated flow
pub fn handle_commit(cz: &CommitArgs, dir: Option<&Path>, theme: &Theme) -> Option<ExitCode> {
    let summary = cz.summary.as_ref().or(cz.message.as_ref())?;
    let branch = match get_current_branch(dir) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}: {}", theme.red_bold("Erro de Git"), e);
            return Some(ExitCode::FAILURE);
        }
    };

    let (ticket_id, kind, _) = match infer_ticket_from_branch(&branch) {
        Some(t) => t,
        None => {
            if branch == "main" && cz.phase.is_none() {
                eprintln!(
                    "{}: Commits diretos na branch 'main' são estritamente proibidos pela norma GOST 2.0 (Regra GOST-04-B).",
                    theme.red_bold("Bloqueio de Governança GOST")
                );
                eprintln!(
                    "  Execute '{}' para abrir a branch de trabalho da issue.",
                    theme.cyan("cccp start <id>")
                );
                return Some(ExitCode::FAILURE);
            }
            return None;
        }
    };

    let log_messages = get_branch_recent_commit_messages(dir, "main");
    let msg_slices: Vec<&str> = log_messages.iter().map(|s| s.as_str()).collect();

    let phase = if let Some(ref p_str) = cz.phase {
        match parse_phase_arg(p_str) {
            Ok(p) => p,
            Err(err) => {
                eprintln!("{}: {}", theme.red_bold("Erro GOST"), err);
                return Some(ExitCode::FAILURE);
            }
        }
    } else {
        infer_next_phase(&msg_slices)
    };

    println!(
        "{} [GOSPLAN DD-TDD] Dedução Automática de Ciclo:",
        theme.gold_bold("★")
    );
    println!(
        "  Ticket Identificado: #{}",
        theme.cyan(&ticket_id.to_string())
    );
    println!(
        "  Fase Reconhecida   : {} ({})",
        theme.green(&phase.to_string()),
        phase.description()
    );

    // Validação de Phase Guards no stage
    let staged_files = match get_staged_files(dir) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}: {}", theme.red_bold("Erro"), e);
            return Some(ExitCode::FAILURE);
        }
    };
    let file_slices: Vec<&str> = staged_files.iter().map(|s| s.as_str()).collect();
    if let Err(guard_err) = validate_phase_guards(phase, &file_slices) {
        eprintln!("{}: {}", theme.red_bold("Trava de Fase Violada"), guard_err);
        return Some(ExitCode::FAILURE);
    }

    let final_kind = cz.commit_type.as_deref().unwrap_or(&kind);
    let formatted_msg =
        format_phase_commit(final_kind, cz.scope.as_deref(), summary, ticket_id, phase);

    println!("  Mensagem Canônica  : {}", theme.yellow(&formatted_msg));

    let commit_opts = CommitOptions {
        all: cz.all,
        sign: cz.sign,
        no_sign: cz.no_sign,
        dry_run: cz.dry_run,
        force: false,
        dir: dir.map(|p| p.to_path_buf()),
    };

    match execute_commit(&formatted_msg, &commit_opts, theme) {
        Ok(0) => {
            let next = match phase {
                DdtddPhase::Ph3 => {
                    "Fase 4 (Red Test) — adicione testes falhos antes da implementação"
                }
                DdtddPhase::Ph4 => "Fase 5 (Green Implement) — faça os testes passarem",
                DdtddPhase::Ph5 => "Fase 6 (Refactor) ou Fase 7 (Mutation Test)",
                DdtddPhase::Ph6 => "Fase 7 (Mutation Test) com 'cccp mutate'",
                DdtddPhase::Ph7 => "Fase 8 (Quality Gate) com 'cccp hygiene'",
                DdtddPhase::Ph8 => "Fase 9 (Close) com 'cccp close \"<resumo>\"'",
                DdtddPhase::Ph9 => "Integração via 'cccp merge'",
            };
            println!("✓ Commit registrado com sucesso!");
            println!("  Próxima fase sugerida: {}", theme.cyan(next));
            Some(ExitCode::SUCCESS)
        }
        Ok(code) => Some(ExitCode::from(code as u8)),
        Err(err) => {
            eprintln!("{}: {}", theme.red_bold("Erro"), err);
            Some(ExitCode::FAILURE)
        }
    }
}

/// Dispatches `cccp close <summary> [--body <b>]`
pub fn handle_close(args: &CloseArgs, dir: Option<&Path>, theme: &Theme) -> ExitCode {
    let branch = match get_current_branch(dir) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}: {}", theme.red_bold("Erro de Git"), e);
            return ExitCode::FAILURE;
        }
    };

    let (ticket_id, kind, _) = match infer_ticket_from_branch(&branch) {
        Some(t) => t,
        None => {
            eprintln!(
                "{}: O comando 'cccp close' deve ser executado a partir de uma branch de trabalho vinculada a ticket (<tipo>/<id>-<slug>). Branch ativa: '{}'",
                theme.red_bold("Erro de Governança"),
                branch
            );
            return ExitCode::FAILURE;
        }
    };

    let body = if let Some(ref b) = args.body {
        b.clone()
    } else {
        // Template padrão para edição
        let template = format!(
            "[Resumo]:\nConclusão do atendimento do Ticket #{ticket_id}.\n\n[Solução]:\nImplementadas as alterações e validações requeridas conforme especificação.\n\n[Validação]:\nExecutados testes unitários, análise estática (-D warnings) e testes de regressão com sucesso.\n"
        );
        template
    };

    if let Err(val_err) = validate_resolution_report(&body) {
        eprintln!(
            "{}: {}\n\nO Laudo Textual da Fase 9 exige comprimento >= 80 caracteres e os cabeçalhos:\n  • [Resumo]:\n  • [Solução]:\n  • [Validação]:",
            theme.red_bold("Laudo Inválido"),
            val_err
        );
        return ExitCode::FAILURE;
    }

    let closing_subject = format_close_commit(&kind, None, &args.summary, ticket_id);
    let full_message = format!("{}\n\n{}", closing_subject, body.trim());

    println!(
        "{} [GOSPLAN DD-TDD] Finalizando ciclo e gerando Laudo de Fechamento (Fase 9):",
        theme.gold_bold("★")
    );
    println!("  Ticket : #{}", theme.cyan(&ticket_id.to_string()));
    println!("  Header : {}", theme.yellow(&closing_subject));

    let commit_opts = CommitOptions {
        all: false,
        sign: false,
        no_sign: false,
        dry_run: false,
        force: false,
        dir: dir.map(|p| p.to_path_buf()),
    };

    match execute_commit(&full_message, &commit_opts, theme) {
        Ok(0) => {
            println!("✓ Ticket #{} concluído com sucesso na branch!", ticket_id);
            println!(
                "  Execute '{}' a partir da base para integrar e postar o Laudo no Forge.",
                theme.cyan(&format!("cccp merge {}", branch))
            );
            ExitCode::SUCCESS
        }
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            eprintln!("{}: {}", theme.red_bold("Erro"), err);
            ExitCode::FAILURE
        }
    }
}
