use cccp::ai::DecomposedTicket;
use cccp::draft::{DivanDraft, DraftStage};
use cccp::editor::Editor;
use tempfile::tempdir;

#[test]
fn test_draft_prompt_save_load_clear() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let repo_path = tmp.path();

    // 1. Initially no draft exists
    let initial = DivanDraft::load(repo_path).expect("Loading non-existent draft must succeed");
    assert!(initial.is_none(), "Initial draft must be None");

    // 2. Save prompt draft
    let prompt = "Implementar novo endpoint de métricas com Prometheus";
    DivanDraft::save_prompt(repo_path, prompt).expect("Saving prompt draft must succeed");

    // 3. Load prompt draft
    let loaded = DivanDraft::load(repo_path)
        .expect("Loading prompt draft must succeed")
        .expect("Must have saved draft");

    assert_eq!(loaded.stage, DraftStage::Prompt);
    assert_eq!(loaded.prompt, prompt);
    assert!(loaded.tickets.is_empty());
    assert_eq!(loaded.current_index, 0);

    // 4. Clear draft
    DivanDraft::clear(repo_path).expect("Clearing draft must succeed");
    let after_clear = DivanDraft::load(repo_path).expect("Loading after clear must succeed");
    assert!(after_clear.is_none(), "Draft must be None after clear");
}

#[test]
fn test_draft_tickets_save_load_and_index() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let repo_path = tmp.path();

    let prompt = "Adicionar suporte a exportação de rascunhos manuais";
    let tickets = vec![
        DecomposedTicket {
            title: "feat(draft): salvar rascunho de prompt".to_string(),
            rationale: Some("Rascunho de prompt".to_string()),
            labels: vec!["feature".to_string()],
            body: "## Description\nSalvar rascunho do prompt".to_string(),
        },
        DecomposedTicket {
            title: "feat(draft): salvar rascunho de tickets".to_string(),
            rationale: Some("Rascunho de tickets".to_string()),
            labels: vec!["feature".to_string()],
            body: "## Description\nSalvar rascunho dos tickets".to_string(),
        },
    ];

    // 1. Save tickets draft at index 1
    DivanDraft::save_tickets(repo_path, prompt, &tickets, 1)
        .expect("Saving tickets draft must succeed");

    let loaded = DivanDraft::load(repo_path)
        .expect("Loading draft must succeed")
        .expect("Draft must exist");

    assert_eq!(loaded.stage, DraftStage::TicketsReview);
    assert_eq!(loaded.prompt, prompt);
    assert_eq!(loaded.tickets.len(), 2);
    assert_eq!(loaded.current_index, 1);
    assert_eq!(
        loaded.tickets[0].title,
        "feat(draft): salvar rascunho de prompt"
    );

    // 2. Clean up
    DivanDraft::clear(repo_path).expect("Clearing draft must succeed");
    assert!(DivanDraft::load(repo_path).unwrap().is_none());
}

#[test]
fn test_draft_isolation_between_repositories() {
    let tmp1 = tempdir().expect("Failed to create tempdir 1");
    let tmp2 = tempdir().expect("Failed to create tempdir 2");
    let repo1 = tmp1.path();
    let repo2 = tmp2.path();

    DivanDraft::save_prompt(repo1, "Draft do repositório 1").expect("Save repo1");
    DivanDraft::save_prompt(repo2, "Draft do repositório 2").expect("Save repo2");

    let d1 = DivanDraft::load(repo1).unwrap().unwrap();
    let d2 = DivanDraft::load(repo2).unwrap().unwrap();

    assert_eq!(d1.prompt, "Draft do repositório 1");
    assert_eq!(d2.prompt, "Draft do repositório 2");

    // Limpar repo1 não afeta repo2
    DivanDraft::clear(repo1).expect("Clear repo1");
    assert!(DivanDraft::load(repo1).unwrap().is_none());
    assert!(DivanDraft::load(repo2).unwrap().is_some());

    DivanDraft::clear(repo2).expect("Clear repo2");
}

#[test]
fn test_editor_draft_shortcut_and_status_feedback() {
    let mut editor = Editor::new("Divan Prompt", "Conteúdo preliminar");
    let mut saved_content = String::new();

    editor.set_draft_handler(Box::new(move |content: &str| {
        saved_content.push_str(content);
        Ok(())
    }));

    // Trigger draft action
    let result = editor.trigger_draft();
    assert!(result.is_ok(), "Draft trigger should succeed");
    assert_eq!(
        editor.status_msg,
        "★ Rascunho salvo com sucesso [Ctrl+R]"
    );
}
