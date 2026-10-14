use cccp::ai::DecomposedTicket;
use cccp::checkpoint::{CheckpointStage, DivanCheckpoint};
use tempfile::tempdir;

#[test]
fn test_checkpoint_prompt_save_load_clear() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let repo_path = tmp.path();

    // 1. Initially no checkpoint exists
    let initial =
        DivanCheckpoint::load(repo_path).expect("Loading non-existent checkpoint must succeed");
    assert!(initial.is_none(), "Initial checkpoint must be None");

    // 2. Save prompt checkpoint
    let prompt = "Refatorar sistema de autenticação e adicionar suporte a tokens OAuth2";
    DivanCheckpoint::save_prompt(repo_path, prompt).expect("Saving prompt checkpoint must succeed");

    // 3. Load prompt checkpoint
    let loaded = DivanCheckpoint::load(repo_path)
        .expect("Loading prompt checkpoint must succeed")
        .expect("Must have saved checkpoint");

    assert_eq!(loaded.stage, CheckpointStage::PromptEntered);
    assert_eq!(loaded.prompt, prompt);
    assert!(loaded.tickets.is_empty());
    assert_eq!(loaded.current_index, 0);

    // 4. Clear checkpoint
    DivanCheckpoint::clear(repo_path).expect("Clearing checkpoint must succeed");
    let after_clear = DivanCheckpoint::load(repo_path).expect("Loading after clear must succeed");
    assert!(after_clear.is_none(), "Checkpoint must be None after clear");
}

#[test]
fn test_checkpoint_tickets_save_load_and_advance() {
    let tmp = tempdir().expect("Failed to create tempdir");
    let repo_path = tmp.path();

    let prompt = "Implementar exportação de relatórios em CSV e JSON";
    let tickets = vec![
        DecomposedTicket {
            title: "feat(export): exportar relatorio em CSV".to_string(),
            rationale: Some("Exportador CSV".to_string()),
            labels: vec!["feature".to_string()],
            body: "## Description\nExportador CSV".to_string(),
        },
        DecomposedTicket {
            title: "feat(export): exportar relatorio em JSON".to_string(),
            rationale: Some("Exportador JSON".to_string()),
            labels: vec!["feature".to_string()],
            body: "## Description\nExportador JSON".to_string(),
        },
    ];

    // 1. Save tickets checkpoint at index 0
    DivanCheckpoint::save_tickets(repo_path, prompt, &tickets, 0)
        .expect("Saving tickets checkpoint must succeed");

    let loaded = DivanCheckpoint::load(repo_path)
        .expect("Loading must succeed")
        .expect("Checkpoint must exist");

    assert_eq!(loaded.stage, CheckpointStage::TicketsPendingReview);
    assert_eq!(loaded.prompt, prompt);
    assert_eq!(loaded.tickets.len(), 2);
    assert_eq!(loaded.current_index, 0);
    assert_eq!(
        loaded.tickets[0].title,
        "feat(export): exportar relatorio em CSV"
    );

    // 2. Advance to index 1
    DivanCheckpoint::save_tickets(repo_path, prompt, &tickets, 1)
        .expect("Updating checkpoint index must succeed");

    let loaded_advanced = DivanCheckpoint::load(repo_path)
        .expect("Loading must succeed")
        .expect("Checkpoint must exist");

    assert_eq!(loaded_advanced.current_index, 1);

    // 3. Clean up
    DivanCheckpoint::clear(repo_path).expect("Clearing checkpoint must succeed");
}

#[test]
fn test_checkpoint_isolation_between_different_repositories() {
    let tmp1 = tempdir().expect("Failed to create tempdir 1");
    let tmp2 = tempdir().expect("Failed to create tempdir 2");
    let repo1 = tmp1.path();
    let repo2 = tmp2.path();

    DivanCheckpoint::save_prompt(repo1, "Prompt do repo 1").expect("Save repo1");
    DivanCheckpoint::save_prompt(repo2, "Prompt do repo 2").expect("Save repo2");

    let cp1 = DivanCheckpoint::load(repo1).unwrap().unwrap();
    let cp2 = DivanCheckpoint::load(repo2).unwrap().unwrap();

    assert_eq!(cp1.prompt, "Prompt do repo 1");
    assert_eq!(cp2.prompt, "Prompt do repo 2");

    DivanCheckpoint::clear(repo1).unwrap();
    assert!(DivanCheckpoint::load(repo1).unwrap().is_none());
    assert!(DivanCheckpoint::load(repo2).unwrap().is_some());

    DivanCheckpoint::clear(repo2).unwrap();
}
