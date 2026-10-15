use cccp::cli::{Cli, Commands, TicketAction};
use cccp::forges::{ForgeComment, ForgeIssue};
use cccp::tickets::{format_ticket_markdown, generate_tickets_markdown};
use clap::Parser;

#[test]
fn test_forge_comment_serialization_and_fields() {
    let comment = ForgeComment {
        id: 42,
        author: "lumenpink".to_string(),
        body: "Trabalhando na implementação do suporte a comentários".to_string(),
        created_at: "2026-10-14T20:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&comment).expect("Serialization must succeed");
    let deserialized: ForgeComment =
        serde_json::from_str(&json).expect("Deserialization must succeed");

    assert_eq!(deserialized.id, 42);
    assert_eq!(deserialized.author, "lumenpink");
    assert_eq!(
        deserialized.body,
        "Trabalhando na implementação do suporte a comentários"
    );
    assert_eq!(deserialized.created_at, "2026-10-14T20:00:00Z");
}

#[test]
fn test_format_ticket_markdown_with_and_without_comments() {
    // 1. Issue sem comentários
    let issue_no_comments = ForgeIssue {
        number: 148,
        title: "feat(tickets): suportar comentários".to_string(),
        state: "open".to_string(),
        author: "lumenpink".to_string(),
        labels: vec!["feature".to_string()],
        body: "Descrição detalhada da issue".to_string(),
        html_url: "https://codeberg.org/lumenpink/cccp/issues/148".to_string(),
        created_at: "2026-10-14T10:00:00Z".to_string(),
        comments: Vec::new(),
    };

    let md1 = format_ticket_markdown(&issue_no_comments);
    assert!(md1.contains("## [#148] feat(tickets): suportar comentários"));
    assert!(md1.contains("Descrição detalhada da issue"));
    assert!(!md1.contains("### Comments"));

    // 2. Issue com comentários
    let mut issue_with_comments = issue_no_comments.clone();
    issue_with_comments.comments = vec![
        ForgeComment {
            id: 101,
            author: "camarada".to_string(),
            body: "Apoio a proposta do Gosplan.".to_string(),
            created_at: "2026-10-14T12:00:00Z".to_string(),
        },
        ForgeComment {
            id: 102,
            author: "lumenpink".to_string(),
            body: "Iniciando a especificação e os testes falhos.".to_string(),
            created_at: "2026-10-14T13:00:00Z".to_string(),
        },
    ];

    let md2 = format_ticket_markdown(&issue_with_comments);
    assert!(md2.contains("### Comments (2)"));
    assert!(md2.contains("**@camarada** (2026-10-14T12:00:00Z):"));
    assert!(md2.contains("Apoio a proposta do Gosplan."));
    assert!(md2.contains("**@lumenpink** (2026-10-14T13:00:00Z):"));
    assert!(md2.contains("Iniciando a especificação e os testes falhos."));
}

#[test]
fn test_generate_tickets_markdown_respects_max_bytes_ceiling() {
    let issue = ForgeIssue {
        number: 1,
        title: "Issue com comentários longos".to_string(),
        state: "open".to_string(),
        author: "author".to_string(),
        labels: vec![],
        body: "Corpo base".to_string(),
        html_url: "https://example.com/1".to_string(),
        created_at: "2026-10-14T10:00:00Z".to_string(),
        comments: vec![ForgeComment {
            id: 1,
            author: "user".to_string(),
            body: "Comentário moderadamente longo para testar a cota de bytes".to_string(),
            created_at: "2026-10-14T10:05:00Z".to_string(),
        }],
    };

    let issues = vec![issue.clone(), issue.clone(), issue];

    // Limite muito pequeno (deve parar antes de incluir todas)
    let (md_small, count, stopped) = generate_tickets_markdown("repo_teste", &issues, 400);
    assert!(stopped, "Deve ter parado antecipadamente pelo teto");
    assert!(count < 3, "Não deve incluir todas as issues");
    assert!(md_small.len() <= 400, "Tamanho deve respeitar teto");
}

#[test]
fn test_cli_parsing_ticket_comment_action() {
    let parsed = Cli::parse_from([
        "cccp",
        "ticket",
        "comment",
        "148",
        "-m",
        "Atualização técnica do progresso",
    ]);

    match parsed.command {
        Some(Commands::Ticket(t)) => match t.action {
            Some(TicketAction::Comment { id, message }) => {
                assert_eq!(id, 148);
                assert_eq!(
                    message.as_deref(),
                    Some("Atualização técnica do progresso")
                );
            }
            _ => panic!("Expected TicketAction::Comment"),
        },
        _ => panic!("Expected Commands::Ticket"),
    }
}

#[test]
fn test_cli_parsing_divan_comment_flag() {
    let parsed = Cli::parse_from([
        "cccp",
        "divan",
        "-c",
        "148",
        "Sugerir testes de integração",
    ]);

    match parsed.command {
        Some(Commands::Divan(d)) => {
            assert_eq!(d.comment, Some(148));
            let (_, prompt) = d.resolve_repo_and_prompt();
            assert_eq!(prompt.as_deref(), Some("Sugerir testes de integração"));
        }
        _ => panic!("Expected Commands::Divan"),
    }
}
