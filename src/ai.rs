//! CCCP AI & LLM Triage Client (O Divã)
//! Supports local offline inference via Ollama and remote inference via Google Gemini REST API.

use crate::config::Config;
use crate::cultural::Theme;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// Supported AI providers for issue triage and decomposition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    Gemini,
    Ollama,
}

impl fmt::Display for AiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiProvider::Gemini => write!(f, "gemini"),
            AiProvider::Ollama => write!(f, "ollama"),
        }
    }
}

impl FromStr for AiProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "gemini" | "google" => Ok(AiProvider::Gemini),
            "ollama" | "local" => Ok(AiProvider::Ollama),
            other => Err(format!(
                "Unknown AI provider '{}'. Valid providers are 'gemini' and 'ollama'.",
                other
            )),
        }
    }
}

/// A structured decomposed ticket produced by the AI triage engine
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecomposedTicket {
    /// Conventional commit title (e.g. feat(scope): title)
    pub title: String,
    /// Explanation of why this is a distinct atomic issue
    #[serde(default)]
    pub rationale: Option<String>,
    /// Issue labels (e.g. ["bug", "rust"])
    #[serde(default)]
    pub labels: Vec<String>,
    /// Markdown body of the issue
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TriageJsonWrapper {
    #[serde(default)]
    tickets: Vec<DecomposedTicket>,
}

/// Unified AI Client for issue triage
pub struct AiClient {
    pub provider: AiProvider,
    pub model: String,
    pub host: String,
    pub temperature: f32,
    pub api_key: Option<String>,
    agent: ureq::Agent,
}

impl AiClient {
    /// Initialize AI client from CCCP hierarchical configuration
    pub fn from_config(config: &Config) -> Self {
        let provider = config
            .ai_provider
            .parse::<AiProvider>()
            .unwrap_or(AiProvider::Gemini);

        let (model, temperature) = match provider {
            AiProvider::Ollama => (config.ollama_model.clone(), config.ollama_temperature),
            AiProvider::Gemini => (config.gemini_model.clone(), config.gemini_temperature),
        };

        let ureq_cfg = ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .build();
        let agent = ureq::Agent::new_with_config(ureq_cfg);

        let api_key = config.gemini_api_key.clone().or_else(|| {
            std::env::var("GEMINI_API_KEY")
                .or_else(|_| std::env::var("gemini_api_key"))
                .ok()
                .filter(|s| !s.trim().is_empty())
        });

        Self {
            provider,
            model,
            host: config.ollama_host.clone(),
            temperature,
            api_key,
            agent,
        }
    }

    /// Override the active provider
    pub fn with_provider(mut self, provider: AiProvider) -> Self {
        self.provider = provider;
        self
    }

    /// Override the model name
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Override the Ollama host URL
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Override temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Override Gemini API key
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Decompose user report and repository context into structured Conventional Commit tickets
    pub fn triage(
        &self,
        user_text: &str,
        repo_context: Option<&str>,
    ) -> Result<Vec<DecomposedTicket>, String> {
        match self.provider {
            AiProvider::Ollama => self.triage_ollama(user_text, repo_context),
            AiProvider::Gemini => self.triage_gemini(user_text, repo_context),
        }
    }

    fn system_instructions() -> &'static str {
        "You are a senior software engineering triage assistant (O Divã do Gosplan). \
Your task is to analyze user reports, bug descriptions, and feature requests, and decompose them \
into one or more discrete, atomic issue tickets following the Conventional Commits specification for titles \
(e.g. feat(scope): title, fix(scope): title, refactor(scope): title). \
Split multi-topic or unrelated problems into separate tickets. \
Return ONLY a valid JSON object matching this schema: \
{\"tickets\": [{\"title\": \"Conventional Commit title\", \"rationale\": \"Why this is a distinct issue\", \"labels\": [\"bug\"], \"body\": \"## Description\\n\\n## Steps to Reproduce\\n\\n## Expected vs Actual\\n\"}]} \
Write the ticket body and descriptions in the language used by the user report."
    }

    fn build_prompt(user_text: &str, repo_context: Option<&str>) -> String {
        if let Some(ctx) = repo_context {
            if !ctx.trim().is_empty() {
                return format!(
                    "Repository context:\n{}\n\nUser input to triage:\n{}",
                    ctx.trim(),
                    user_text.trim()
                );
            }
        }
        format!("User input to triage:\n{}", user_text.trim())
    }

    /// Perform triage using local Ollama daemon (/api/chat)
    fn triage_ollama(
        &self,
        user_text: &str,
        repo_context: Option<&str>,
    ) -> Result<Vec<DecomposedTicket>, String> {
        let endpoint = format!("{}/api/chat", self.host.trim_end_matches('/'));
        let prompt = Self::build_prompt(user_text, repo_context);

        let payload = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": Self::system_instructions()
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "stream": false,
            "format": "json",
            "options": {
                "temperature": self.temperature
            }
        });

        let res = match self.agent.post(&endpoint).send_json(payload) {
            Ok(r) => r,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("404") {
                    return Err(format!(
                        "Model '{}' not found on Ollama server at '{}'. Pull the model with: ollama pull {}",
                        self.model, self.host, self.model
                    ));
                }
                if err_str.contains("Connection refused") || err_str.contains("Connect") {
                    return Err(format!(
                        "Failed to connect to Ollama at '{}'. Ensure the Ollama daemon is active: run 'ollama serve' or check your OLLAMA_HOST.",
                        self.host
                    ));
                }
                return Err(format!("Ollama API request failed: {}", e));
            }
        };

        let status = res.status();
        let body_str = res
            .into_body()
            .read_to_string()
            .map_err(|e| format!("Failed to read Ollama response body: {}", e))?;

        if status.as_u16() == 404 {
            if body_str.contains("model") && body_str.contains("not found") {
                return Err(format!(
                    "Model '{}' not found on Ollama server at '{}'. Pull the model with: ollama pull {}",
                    self.model, self.host, self.model
                ));
            }
            return Err(format!(
                "Ollama endpoint '{}' returned 404: {}",
                endpoint, body_str
            ));
        }

        if !status.is_success() {
            return Err(format!(
                "Ollama returned HTTP {}: {}",
                status.as_u16(),
                body_str
            ));
        }

        let resp_json: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse Ollama JSON envelope: {}", e))?;

        let content_text = resp_json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .or_else(|| resp_json.get("response").and_then(|r| r.as_str()))
            .ok_or_else(|| {
                format!(
                    "Ollama response does not contain 'message.content': {}",
                    body_str
                )
            })?;

        Self::parse_tickets_json(content_text)
    }

    /// Perform triage using Google Gemini REST API (:generateContent)
    fn triage_gemini(
        &self,
        user_text: &str,
        repo_context: Option<&str>,
    ) -> Result<Vec<DecomposedTicket>, String> {
        let api_key = match &self.api_key {
            Some(k) if !k.trim().is_empty() => k.trim(),
            _ => {
                return Err(
                    "GEMINI_API_KEY is not configured. Export GEMINI_API_KEY, add it to ~/.env or .cccprc, or switch to Ollama via '--provider ollama'."
                        .to_string(),
                );
            }
        };

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, api_key
        );
        let prompt = Self::build_prompt(user_text, repo_context);

        let payload = serde_json::json!({
            "system_instruction": {
                "parts": [{ "text": Self::system_instructions() }]
            },
            "contents": [
                {
                    "parts": [{ "text": prompt }]
                }
            ],
            "generationConfig": {
                "responseMimeType": "application/json",
                "temperature": self.temperature
            }
        });

        let res = self
            .agent
            .post(&endpoint)
            .send_json(payload)
            .map_err(|e| format!("Gemini API request failed: {}", e))?;

        let status = res.status();
        let body_str = res
            .into_body()
            .read_to_string()
            .map_err(|e| format!("Failed to read Gemini response body: {}", e))?;

        if !status.is_success() {
            return Err(format!(
                "Gemini returned HTTP {}: {}",
                status.as_u16(),
                body_str
            ));
        }

        let resp_json: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse Gemini response envelope: {}", e))?;

        let content_text = resp_json
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|cand| cand.get("content"))
            .and_then(|cont| cont.get("parts"))
            .and_then(|parts| parts.as_array())
            .and_then(|arr| arr.first())
            .and_then(|part| part.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                format!(
                    "Gemini response does not contain candidate text parts: {}",
                    body_str
                )
            })?;

        Self::parse_tickets_json(content_text)
    }

    /// Extract and parse tickets JSON from raw LLM output
    pub fn parse_tickets_json(raw: &str) -> Result<Vec<DecomposedTicket>, String> {
        let cleaned = clean_json_markdown(raw);

        // Case 1: Structured envelope {"tickets": [...]}
        if let Ok(wrapper) = serde_json::from_str::<TriageJsonWrapper>(cleaned) {
            if !wrapper.tickets.is_empty() {
                return Ok(wrapper.tickets);
            }
        }

        // Case 2: Direct array of tickets [{...}, {...}]
        if let Ok(tickets) = serde_json::from_str::<Vec<DecomposedTicket>>(cleaned) {
            if !tickets.is_empty() {
                return Ok(tickets);
            }
        }

        // Case 3: Single ticket object {...}
        if let Ok(single) = serde_json::from_str::<DecomposedTicket>(cleaned) {
            return Ok(vec![single]);
        }

        Err(format!(
            "Failed to parse structured tickets from LLM output: '{}'",
            raw
        ))
    }
}

/// Clean markdown code fences from JSON text
pub fn clean_json_markdown(raw: &str) -> &str {
    let mut text = raw.trim();
    if text.starts_with("```json") {
        text = text.trim_start_matches("```json").trim();
    } else if text.starts_with("```") {
        text = text.trim_start_matches("```").trim();
    }
    if text.ends_with("```") {
        text = text.trim_end_matches("```").trim();
    }
    text
}

/// Display Soviet Gosplan triage results
pub fn display_triage_results(
    tickets: &[DecomposedTicket],
    provider: AiProvider,
    model: &str,
    theme: &Theme,
) {
    println!(
        "{}",
        theme.bold("★ [ГОСПЛАН] O Divã — Triagem Interativa de Tickets ★")
    );
    println!(
        "Provedor: {} (modelo: {})",
        theme.cyan(&provider.to_string()),
        theme.yellow(model)
    );
    println!();

    let total = tickets.len();
    for (idx, ticket) in tickets.iter().enumerate() {
        let num = idx + 1;
        let header = format!("[ДИВАН] Ticket {}/{}: {}", num, total, ticket.title);
        println!("{}", theme.gold_bold(&header));

        if let Some(ref rationale) = ticket.rationale {
            println!("  {}: {}", theme.cyan("Motivação"), rationale);
        }
        if !ticket.labels.is_empty() {
            println!("  {}: {}", theme.cyan("Labels"), ticket.labels.join(", "));
        }
        println!();
        println!("{}", ticket.body);
        if idx + 1 < total {
            println!(
                "{}",
                theme.dim("────────────────────────────────────────────────────────────────")
            );
        }
    }
}
