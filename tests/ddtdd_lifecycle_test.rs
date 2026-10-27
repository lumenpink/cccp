//! Tests for DD-TDD Phase and Lifecycle Engine (GOST 2.0 / Kollontai 2.0)
//! Validates branch inference, phase deduction state machine, message formatting,
//! phase guards, and resolution report validations.

use cccp::ddtdd::{
    DdtddPhase, format_close_commit, format_phase_commit, infer_next_phase,
    infer_ticket_from_branch, validate_phase_guards, validate_resolution_report,
};

#[test]
fn test_infer_ticket_from_branch() {
    let b1 = "feat/72-start-commit-close-ddtdd";
    let res1 = infer_ticket_from_branch(b1);
    assert_eq!(
        res1,
        Some((72, "feat".to_string(), "start-commit-close-ddtdd".to_string()))
    );

    let b2 = "fix/133-fechar-tickets-merge-trava-closes";
    let res2 = infer_ticket_from_branch(b2);
    assert_eq!(
        res2,
        Some((
            133,
            "fix".to_string(),
            "fechar-tickets-merge-trava-closes".to_string()
        ))
    );

    let b3 = "docs/42-spec-gost";
    let res3 = infer_ticket_from_branch(b3);
    assert_eq!(
        res3,
        Some((42, "docs".to_string(), "spec-gost".to_string()))
    );

    let b_invalid = "main";
    assert_eq!(infer_ticket_from_branch(b_invalid), None);

    let b_invalid2 = "random-branch-without-id";
    assert_eq!(infer_ticket_from_branch(b_invalid2), None);
}

#[test]
fn test_infer_next_phase_state_machine() {
    // 1. Branch vazia -> ph3 (Doc/Spec)
    let history_empty: Vec<&str> = vec![];
    assert_eq!(infer_next_phase(&history_empty), DdtddPhase::Ph3);

    // 2. Teve ph3 -> ph4 (Red Test)
    let history_ph3 = vec!["docs(spec): specify ddtdd lifecycle subcommands (ref #72 - ph3)"];
    assert_eq!(infer_next_phase(&history_ph3), DdtddPhase::Ph4);

    // 3. Teve ph4 -> ph5 (Green Implement)
    let history_ph4 = vec![
        "test: add failing tests for ddtdd lifecycle (ref #72 - ph4)",
        "docs(spec): specify ddtdd lifecycle subcommands (ref #72 - ph3)",
    ];
    assert_eq!(infer_next_phase(&history_ph4), DdtddPhase::Ph5);

    // 4. Teve ph5 -> ph6 (Refactor)
    let history_ph5 = vec![
        "feat(ddtdd): implement start commit and close (ref #72 - ph5)",
        "test: add failing tests (ref #72 - ph4)",
        "docs(spec): specify (ref #72 - ph3)",
    ];
    assert_eq!(infer_next_phase(&history_ph5), DdtddPhase::Ph6);

    // 5. Teve ph6 -> ph7 (Mutation)
    let history_ph6 = vec![
        "refactor(ddtdd): clean up code (ref #72 - ph6)",
        "feat(ddtdd): implement (ref #72 - ph5)",
    ];
    assert_eq!(infer_next_phase(&history_ph6), DdtddPhase::Ph7);

    // 6. Teve ph7 -> ph8 (Quality Gate / Hygiene)
    let history_ph7 = vec![
        "test(ddtdd): eliminate mutants on phase deduction (ref #72 - ph7)",
        "refactor(ddtdd): clean up (ref #72 - ph6)",
    ];
    assert_eq!(infer_next_phase(&history_ph7), DdtddPhase::Ph8);

    // 7. Teve ph8 -> ph9 (Finish & Close)
    let history_ph8 = vec![
        "chore(ddtdd): quality gate passed (ref #72 - ph8)",
        "test(ddtdd): eliminate mutants (ref #72 - ph7)",
    ];
    assert_eq!(infer_next_phase(&history_ph8), DdtddPhase::Ph9);
}

#[test]
fn test_format_phase_commit_message() {
    let msg1 = format_phase_commit("feat", Some("ddtdd"), "subcomandos start e close", 72, DdtddPhase::Ph5);
    assert_eq!(msg1, "feat(ddtdd): subcomandos start e close (ref #72 - ph5)");

    let msg2 = format_phase_commit("docs", None, "especificar contratos", 72, DdtddPhase::Ph3);
    assert_eq!(msg2, "docs: especificar contratos (ref #72 - ph3)");

    let msg3 = format_close_commit("feat", Some("ddtdd"), "concluir ciclo de desenvolvimento", 72);
    assert_eq!(msg3, "feat(ddtdd): concluir ciclo de desenvolvimento (closes #72 - ph9)");
}

#[test]
fn test_validate_phase_guards() {
    // ph3 bloqueia código de produção em src/
    let staged_src = vec!["src/main.rs", "docs/spec.md"];
    assert!(validate_phase_guards(DdtddPhase::Ph3, &staged_src).is_err());

    let staged_doc_only = vec!["docs/spec-72.md", "README.md", ".gost/GOST.md"];
    assert!(validate_phase_guards(DdtddPhase::Ph3, &staged_doc_only).is_ok());

    // ph4, ph5, ph6 permitem código em src/
    assert!(validate_phase_guards(DdtddPhase::Ph4, &staged_src).is_ok());
    assert!(validate_phase_guards(DdtddPhase::Ph5, &staged_src).is_ok());
}

#[test]
fn test_validate_resolution_report() {
    let valid_report = r#"
[Resumo]:
Implementação dos subcomandos cccp start, commit e close com travas das 9 fases.

[Solução]:
Criado o módulo ddtdd com máquina de estados para inferência de fases e integradas as travas.

[Validação]:
Suíte de testes de integração ddtdd_lifecycle_test cobrindo todas as fases e formatação.
"#;
    assert!(validate_resolution_report(valid_report).is_ok());

    // Faltando cabeçalho [Validação]:
    let missing_section = r#"
[Resumo]:
Implementação dos subcomandos cccp start, commit e close com travas das 9 fases.

[Solução]:
Criado o módulo ddtdd com máquina de estados para inferência de fases e integradas as travas.
"#;
    assert!(validate_resolution_report(missing_section).is_err());

    // Curto demais (< 80 caracteres)
    let too_short = "[Resumo]: a\n[Solução]: b\n[Validação]: c";
    assert!(validate_resolution_report(too_short).is_err());
}

#[test]
fn test_cli_parsing_start_and_close() {
    use cccp::cli::{Cli, Commands};
    use clap::Parser;

    // cccp start 72
    let cli = Cli::try_parse_from(["cccp", "start", "72"]).expect("should parse cccp start 72");
    match cli.command {
        Some(Commands::Start(s)) => {
            assert_eq!(s.id, 72);
            assert_eq!(s.base, None);
        }
        _ => panic!("Expected Commands::Start"),
    }

    // cccp start 42 --base develop
    let cli2 = Cli::try_parse_from(["cccp", "start", "42", "--base", "develop"])
        .expect("should parse cccp start 42 --base develop");
    match cli2.command {
        Some(Commands::Start(s)) => {
            assert_eq!(s.id, 42);
            assert_eq!(s.base, Some("develop".to_string()));
        }
        _ => panic!("Expected Commands::Start"),
    }

    // cccp close "finalizar feature"
    let cli3 = Cli::try_parse_from(["cccp", "close", "finalizar feature"])
        .expect("should parse cccp close");
    match cli3.command {
        Some(Commands::Close(c)) => {
            assert_eq!(c.summary, "finalizar feature");
            assert_eq!(c.body, None);
            assert!(!c.editor);
        }
        _ => panic!("Expected Commands::Close"),
    }

    // cccp commit "mensagem direta" --phase ph4
    let cli4 = Cli::try_parse_from(["cccp", "commit", "mensagem direta", "--phase", "ph4"])
        .expect("should parse cccp commit with phase");
    match cli4.command {
        Some(Commands::Commit(c)) => {
            assert_eq!(c.summary, Some("mensagem direta".to_string()));
            assert_eq!(c.phase, Some("ph4".to_string()));
        }
        _ => panic!("Expected Commands::Commit"),
    }
}

