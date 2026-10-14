use cccp::ai::{AiClient, AiProvider};
use cccp::config::Config;
use std::str::FromStr;

#[test]
fn test_ai_provider_local_parsing() {
    assert_eq!(AiProvider::from_str("local").unwrap(), AiProvider::Local);
    assert_eq!(AiProvider::from_str("openai").unwrap(), AiProvider::Local);
    assert_eq!(AiProvider::from_str("vllm").unwrap(), AiProvider::Local);
    assert_eq!(AiProvider::from_str("localai").unwrap(), AiProvider::Local);
    assert_eq!(AiProvider::from_str("custom").unwrap(), AiProvider::Local);
    assert_eq!(AiProvider::Local.to_string(), "local");
}

#[test]
fn test_config_local_ai_defaults() {
    let config = Config::default();
    assert_eq!(config.local_ai_host, "http://localhost:8080/v1");
    assert_eq!(config.local_ai_model, "llama3.2:3b");
    assert_eq!(config.local_ai_temperature, 0.2);
    assert!(config.local_ai_api_key.is_none());
    assert_eq!(config.ollama_model, "llama3.2:3b");
}

#[test]
fn test_ai_client_from_config_with_local_provider() {
    let config = Config {
        ai_provider: "local".to_string(),
        local_ai_host: "http://127.0.0.1:5000/v1".to_string(),
        local_ai_model: "my-custom-model".to_string(),
        ..Default::default()
    };

    let client = AiClient::from_config(&config);
    assert_eq!(client.provider, AiProvider::Local);
    assert_eq!(client.host, "http://127.0.0.1:5000/v1");
    assert_eq!(client.model, "my-custom-model");
}

#[test]
fn test_parse_openai_completion_response() {
    let payload = serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "{\"tickets\": [{\"title\": \"feat(core): suporte local\", \"rationale\": \"Necessario para offline\", \"labels\": [\"enhancement\"], \"body\": \"## Description\\nTeste local\"}]}"
                },
                "finish_reason": "stop"
            }
        ]
    });

    let raw_openai_response = payload.to_string();
    let tickets = AiClient::parse_openai_chat_response(&raw_openai_response).unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].title, "feat(core): suporte local");
    assert_eq!(tickets[0].labels, vec!["enhancement".to_string()]);
}
