use cccp::ai::{clean_json_markdown, AiClient, AiProvider};
use cccp::config::Config;

#[test]
fn test_ai_provider_parsing_and_display() {
    assert_eq!("ollama".parse::<AiProvider>().unwrap(), AiProvider::Ollama);
    assert_eq!("local".parse::<AiProvider>().unwrap(), AiProvider::Ollama);
    assert_eq!("gemini".parse::<AiProvider>().unwrap(), AiProvider::Gemini);
    assert_eq!("google".parse::<AiProvider>().unwrap(), AiProvider::Gemini);

    assert!("unknown_provider".parse::<AiProvider>().is_err());

    assert_eq!(AiProvider::Ollama.to_string(), "ollama");
    assert_eq!(AiProvider::Gemini.to_string(), "gemini");
}

#[test]
fn test_clean_json_markdown_fences() {
    let input1 = "```json\n{\"tickets\": []}\n```";
    assert_eq!(clean_json_markdown(input1), "{\"tickets\": []}");

    let input2 = "```\n[{\"title\": \"feat: test\"}]\n```";
    assert_eq!(clean_json_markdown(input2), "[{\"title\": \"feat: test\"}]");

    let input3 = "  {\"title\": \"fix: raw\"}  ";
    assert_eq!(clean_json_markdown(input3), "{\"title\": \"fix: raw\"}");
}

#[test]
fn test_parse_tickets_json_envelope() {
    let json = r###"{
        "tickets": [
            {
                "title": "feat(core): implement Ollama client",
                "rationale": "Allows running local LLMs",
                "labels": ["ai", "enhancement"],
                "body": "## Description\nImplement local AI client.\n"
            },
            {
                "title": "test(ai): add unit test suite",
                "rationale": "Verifies JSON extraction",
                "labels": ["test"],
                "body": "## Description\nTests for AI module.\n"
            }
        ]
    }"###;

    let tickets = AiClient::parse_tickets_json(json).expect("Failed to parse tickets envelope");
    assert_eq!(tickets.len(), 2);
    assert_eq!(tickets[0].title, "feat(core): implement Ollama client");
    assert_eq!(
        tickets[0].rationale.as_deref(),
        Some("Allows running local LLMs")
    );
    assert_eq!(tickets[0].labels, vec!["ai", "enhancement"]);
    assert_eq!(tickets[1].title, "test(ai): add unit test suite");
}

#[test]
fn test_parse_tickets_json_array() {
    let json = r###"[
        {
            "title": "fix(cli): resolve flag parsing bug",
            "body": "## Description\nFixes issue.\n"
        }
    ]"###;

    let tickets = AiClient::parse_tickets_json(json).expect("Failed to parse array format");
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].title, "fix(cli): resolve flag parsing bug");
    assert!(tickets[0].labels.is_empty());
}

#[test]
fn test_parse_tickets_json_single_object() {
    let json = r###"{
        "title": "chore(deps): update ureq to 3.4",
        "rationale": "Security update",
        "labels": ["dependencies"],
        "body": "## Description\nBump dependency.\n"
    }"###;

    let tickets = AiClient::parse_tickets_json(json).expect("Failed to parse single object");
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].title, "chore(deps): update ureq to 3.4");
}

#[test]
fn test_parse_tickets_json_empty_envelope() {
    let json = r###"{"tickets": []}"###;
    let tickets = AiClient::parse_tickets_json(json).expect("Failed to parse empty tickets");
    assert_eq!(tickets.len(), 0);
}

#[test]
fn test_config_ai_cascade_parsing() {
    let cccprc_content = r#"
ai_provider = ollama
ollama_host = http://127.0.0.1:11434
ollama_model = qwen2.5-coder
ollama_temperature = 0.4
gemini_model = gemini-1.5-pro
gemini_api_key = test_secret_key_123
"#;

    let mut config = Config::default();
    config.merge_from_str(cccprc_content);

    assert_eq!(config.ai_provider, "ollama");
    assert_eq!(config.ollama_host, "http://127.0.0.1:11434");
    assert_eq!(config.ollama_model, "qwen2.5-coder");
    assert!((config.ollama_temperature - 0.4).abs() < 1e-4);
    assert_eq!(config.gemini_model, "gemini-1.5-pro");
    assert_eq!(
        config.gemini_api_key.as_deref(),
        Some("test_secret_key_123")
    );

    let client = AiClient::from_config(&config);
    assert_eq!(client.provider, AiProvider::Ollama);
    assert_eq!(client.model, "qwen2.5-coder");
    assert_eq!(client.host, "http://127.0.0.1:11434");
    assert!((client.temperature - 0.4).abs() < 1e-4);
}

#[test]
fn test_ollama_unreachable_host_error_message() {
    let config = Config {
        ai_provider: "ollama".into(),
        ollama_host: "http://127.0.0.1:59999".into(),
        ..Default::default()
    };

    let client = AiClient::from_config(&config);
    let result = client.triage("problem with compiler", None);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("Failed to connect to Ollama") || err.contains("ollama serve"),
        "Unexpected error message: {}",
        err
    );
}

#[test]
fn test_gemini_missing_api_key_error() {
    let config = Config {
        ai_provider: "gemini".into(),
        gemini_api_key: None,
        ..Default::default()
    };

    let client = AiClient::from_config(&config);
    let result = client.triage("problem with auth", None);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("GEMINI_API_KEY is not configured"),
        "Unexpected error message: {}",
        err
    );
}

#[test]
fn test_is_generative_model_filtering() {
    assert!(AiClient::is_generative_model("llama3.2:1b"));
    assert!(AiClient::is_generative_model("qwen2.5-coder:3b"));
    assert!(AiClient::is_generative_model("mistral:7b"));
    assert!(!AiClient::is_generative_model("nomic-embed-text:latest"));
    assert!(!AiClient::is_generative_model("bge-m3:latest"));
    assert!(!AiClient::is_generative_model("all-minilm-bert"));
}

#[test]
fn test_build_enriched_context() {
    let forge = cccp::forges::RepoForgeInfo {
        provider: cccp::forges::ForgeType::Codeberg,
        host: "codeberg.org".into(),
        owner: "lumenpink".into(),
        repo: "cccp".into(),
    };
    let config = Config {
        commit_types: vec!["feat".into(), "fix".into()],
        commit_scopes: vec!["cli".into(), "ai".into()],
        ..Default::default()
    };

    let ctx = cccp::ai::build_enriched_context(None, Some(&forge), &config);
    assert!(ctx.contains("Target Forge: Codeberg (lumenpink/cccp) at codeberg.org"));
    assert!(ctx.contains("Configured commit types: feat, fix"));
    assert!(ctx.contains("Configured commit scopes: cli, ai"));
}
