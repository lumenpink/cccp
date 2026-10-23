use cccp::ai::{clean_json_markdown, AiClient, AiProvider};
use cccp::config::Config;

#[test]
fn test_ai_provider_parsing_and_display() {
    assert_eq!("ollama".parse::<AiProvider>().unwrap(), AiProvider::Ollama);
    assert_eq!("local".parse::<AiProvider>().unwrap(), AiProvider::Local);
    assert_eq!("gemini".parse::<AiProvider>().unwrap(), AiProvider::Gemini);
    assert_eq!("google".parse::<AiProvider>().unwrap(), AiProvider::Gemini);

    assert!("unknown_provider".parse::<AiProvider>().is_err());

    assert_eq!(AiProvider::Ollama.to_string(), "ollama");
    assert_eq!(AiProvider::Gemini.to_string(), "gemini");
    assert_eq!(AiProvider::Local.to_string(), "local");
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
    let saved_key = std::env::var("GEMINI_API_KEY").ok();
    let saved_google = std::env::var("GOOGLE_API_KEY").ok();
    std::env::remove_var("GEMINI_API_KEY");
    std::env::remove_var("GOOGLE_API_KEY");

    let config = Config {
        ai_provider: "gemini".into(),
        gemini_api_key: None,
        ..Default::default()
    };

    let client = AiClient::from_config(&config);
    let result = client.triage("problem with auth", None);

    if let Some(key) = saved_key {
        std::env::set_var("GEMINI_API_KEY", key);
    }
    if let Some(key) = saved_google {
        std::env::set_var("GOOGLE_API_KEY", key);
    }

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
    let forge = cccp::forges::RepoForgeInfo::new(
        cccp::forges::ForgeType::Codeberg,
        "codeberg.org".into(),
        "lumenpink".into(),
        "cccp".into(),
    );
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

#[test]
fn test_check_access_gemini() {
    let config = Config::default();
    let client_no_key = AiClient::from_config(&config)
        .with_provider(AiProvider::Gemini)
        .with_api_key_opt(None);
    let err = client_no_key.check_access().unwrap_err();
    assert!(err.contains("GEMINI_API_KEY is not configured"));

    let client_with_key = client_no_key.with_api_key("some_valid_key");
    assert!(client_with_key.check_access().is_ok());
}

#[test]
fn test_check_access_ollama_offline_fail_fast() {
    let config = Config::default();
    let client_offline = AiClient::from_config(&config)
        .with_provider(AiProvider::Ollama)
        .with_host("http://127.0.0.1:59999"); // Unused port to trigger connection error
    let err = client_offline.check_access().unwrap_err();
    assert!(err.contains("Cannot connect to Ollama at http://127.0.0.1:59999"));
}

#[test]
fn test_extract_project_manifest_cargo() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cargo_toml = r#"
[package]
name = "my-awesome-tool"
version = "2.1.0"
description = "A fast CLI tool"

[dependencies]
clap = "4.5"
serde = "1.0"
"#;
    std::fs::write(temp.path().join("Cargo.toml"), cargo_toml).unwrap();

    let manifest = cccp::ai::extract_project_manifest(temp.path()).expect("manifest should parse");
    assert!(manifest.contains("Rust crate: my-awesome-tool v2.1.0"));
    assert!(manifest.contains("A fast CLI tool"));
    assert!(manifest.contains("[deps: clap, serde]"));
}

#[test]
fn test_extract_project_manifest_package_json() {
    let temp = tempfile::tempdir().expect("tempdir");
    let pkg_json = r#"{
        "name": "web-portal",
        "version": "1.0.4",
        "description": "Frontend app",
        "dependencies": {
            "react": "^18.0.0",
            "vite": "^5.0.0"
        }
    }"#;
    std::fs::write(temp.path().join("package.json"), pkg_json).unwrap();

    let manifest = cccp::ai::extract_project_manifest(temp.path()).expect("manifest should parse");
    assert!(manifest.contains("Node package: web-portal v1.0.4"));
    assert!(manifest.contains("Frontend app"));
    assert!(manifest.contains("react"));
}

#[test]
fn test_extract_codebase_symbols_rust() {
    let temp = tempfile::tempdir().expect("tempdir");
    let src_dir = temp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    let rust_code = r#"
pub struct UserSession {
    id: String,
}

pub enum AuthState {
    LoggedIn,
    LoggedOut,
}

pub trait Authenticator {
    fn verify(&self) -> bool;
}

pub fn login(user: &str) -> bool {
    true
}

fn private_helper() {}
"#;
    std::fs::write(src_dir.join("auth.rs"), rust_code).unwrap();

    let symbols = cccp::ai::extract_codebase_symbols(temp.path(), 5, 5);
    assert_eq!(symbols.len(), 1);
    let line = &symbols[0];
    assert!(line.contains("src/auth.rs"));
    assert!(line.contains("UserSession"));
    assert!(line.contains("AuthState"));
    assert!(line.contains("Authenticator"));
    assert!(line.contains("login()"));
    assert!(!line.contains("private_helper"));
}

#[test]
fn test_extract_project_summary_and_caching() {
    let temp = tempfile::tempdir().expect("tempdir");
    let git_dir = temp.path().join(".git");
    let src_dir = temp.path().join("src");
    std::fs::create_dir_all(&git_dir).unwrap();
    std::fs::create_dir_all(&src_dir).unwrap();

    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"cached-project\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(
        src_dir.join("main.rs"),
        "pub fn run_pipeline() {}\npub struct Pipeline;\n",
    )
    .unwrap();

    // First call: generates summary and caches it
    let summary1 = cccp::ai::extract_project_summary(temp.path(), Some("abc1234"), true);
    assert!(summary1.contains("cached-project"));
    assert!(summary1.contains("run_pipeline()"));
    assert!(summary1.contains("Pipeline"));

    let cache_file = git_dir.join("cccp-context.cache");
    assert!(cache_file.is_file(), "Cache file must be created");

    // Second call with same commit: reads from cache
    let summary2 = cccp::ai::extract_project_summary(temp.path(), Some("abc1234"), true);
    assert_eq!(summary1, summary2);

    // Call with different commit: cache miss, updates cache
    let summary3 = cccp::ai::extract_project_summary(temp.path(), Some("def5678"), true);
    assert!(summary3.contains("cached-project"));
    let cache_content = std::fs::read_to_string(&cache_file).unwrap();
    assert!(cache_content.starts_with("HEAD:def5678"));
}

#[test]
fn test_build_enriched_context_with_provider_budget() {
    let temp = tempfile::tempdir().expect("tempdir");
    let src_dir = temp.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    for i in 0..12 {
        std::fs::write(
            src_dir.join(format!("module_{}.rs", i)),
            format!("pub fn fn_{}() {{}}\npub struct S_{};\n", i, i),
        )
        .unwrap();
    }

    let symbols_ollama = cccp::ai::extract_codebase_symbols(temp.path(), 8, 4);
    let symbols_gemini = cccp::ai::extract_codebase_symbols(temp.path(), 15, 6);

    // Ollama adheres to Gosplan austerity (max 8 files)
    assert!(symbols_ollama.len() <= 8);
    // Gemini allows more context
    assert!(symbols_gemini.len() > symbols_ollama.len());
}

#[test]
fn test_render_ticket_card_borders_and_metadata() {
    let ticket = cccp::ai::DecomposedTicket {
        title: "feat(auth): add OAuth2 provider authentication".into(),
        rationale: Some("Allows offline user authentication".into()),
        labels: vec!["auth".into(), "security".into()],
        body:
            "## Description\nImplements JWT token decoding and validation with fail-fast checks.\n"
                .into(),
    };
    let theme = cccp::cultural::Theme::init(false);

    let card =
        cccp::ai::render_ticket_card(&ticket, 0, 2, Some("Codeberg (lumenpink/cccp)"), &theme);

    // Verify box-drawing borders
    assert!(card.contains('┌'), "must contain top-left border");
    assert!(card.contains('┐'), "must contain top-right border");
    assert!(card.contains('├'), "must contain mid border");
    assert!(card.contains('┤'), "must contain mid border");
    assert!(card.contains('│'), "must contain vertical border");
    assert!(card.contains('└'), "must contain bottom-left border");
    assert!(card.contains('┘'), "must contain bottom-right border");

    // Verify content
    assert!(card.contains("TICKET 1/2"));
    assert!(card.contains("feat(auth): add OAuth2 provider authentication"));
    assert!(card.contains("Destination"));
    assert!(card.contains("Codeberg (lumenpink/cccp)"));
    assert!(card.contains("Rationale"));
    assert!(card.contains("Allows offline user authentication"));
    assert!(card.contains("Labels"));
    assert!(card.contains("auth, security"));
    assert!(card.contains("## Description"));
}

#[test]
fn test_render_ticket_card_no_color_strips_ansi() {
    let ticket = cccp::ai::DecomposedTicket {
        title: "fix(cli): resolve argument parsing error".into(),
        rationale: None,
        labels: vec![],
        body: "Fixes parsing bug.\n".into(),
    };
    let theme = cccp::cultural::Theme::init(true); // NO_COLOR

    let card = cccp::ai::render_ticket_card(&ticket, 1, 1, None, &theme);

    assert!(card.contains('┌'));
    assert!(card.contains('└'));
    assert!(card.contains("fix(cli): resolve argument parsing error"));
    assert!(
        !card.contains("\x1b["),
        "card must contain 0 ANSI escape sequences when no-color is active"
    );
}
