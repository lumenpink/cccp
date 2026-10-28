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
    Local,
}

impl fmt::Display for AiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiProvider::Gemini => write!(f, "gemini"),
            AiProvider::Ollama => write!(f, "ollama"),
            AiProvider::Local => write!(f, "local"),
        }
    }
}

impl FromStr for AiProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "gemini" | "google" | "g" | "pg" => Ok(AiProvider::Gemini),
            "ollama" | "o" | "po" => Ok(AiProvider::Ollama),
            "local" | "openai" | "localai" | "vllm" | "custom" | "pl" => Ok(AiProvider::Local),
            other => Err(format!(
                "Unknown AI provider '{}'. Valid providers are 'gemini', 'ollama', and 'local'.",
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

/// Resolves Gemini API key with strict precedence:
/// 1. config.gemini_api_key or config.extra["gemini_api_key"]
/// 2. CCCP_GEMINI_TOKEN / CCCP_GEMINI_API_KEY
/// 3. GEMINI_TOKEN / GEMINI_API_KEY / gemini_api_key
/// 4. GOOGLE_API_KEY / google_api_key
pub fn resolve_gemini_api_key(config: &Config) -> Option<String> {
    config
        .gemini_api_key
        .clone()
        .or_else(|| config.extra.get("gemini_api_key").cloned())
        .or_else(|| {
            std::env::var("CCCP_GEMINI_TOKEN")
                .or_else(|_| std::env::var("CCCP_GEMINI_API_KEY"))
                .or_else(|_| std::env::var("GEMINI_TOKEN"))
                .or_else(|_| std::env::var("GEMINI_API_KEY"))
                .or_else(|_| std::env::var("gemini_api_key"))
                .or_else(|_| std::env::var("GOOGLE_API_KEY"))
                .or_else(|_| std::env::var("google_api_key"))
                .ok()
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

impl AiClient {
    /// Initialize AI client from CCCP hierarchical configuration
    pub fn from_config(config: &Config) -> Self {
        let provider = config
            .ai_provider
            .parse::<AiProvider>()
            .unwrap_or(AiProvider::Gemini);

        let (model, host, temperature, api_key) = match provider {
            AiProvider::Ollama => (
                config.ollama_model.clone(),
                config.ollama_host.clone(),
                config.ollama_temperature,
                None,
            ),
            AiProvider::Gemini => {
                let key = resolve_gemini_api_key(config);
                (
                    config.gemini_model.clone(),
                    config.ollama_host.clone(),
                    config.gemini_temperature,
                    key,
                )
            }
            AiProvider::Local => {
                let key = config
                    .local_ai_api_key
                    .clone()
                    .or_else(|| config.extra.get("local_ai_api_key").cloned())
                    .or_else(|| std::env::var("LOCAL_AI_KEY").ok())
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .filter(|s| !s.trim().is_empty());
                (
                    config.local_ai_model.clone(),
                    config.local_ai_host.clone(),
                    config.local_ai_temperature,
                    key,
                )
            }
        };

        let ureq_cfg = ureq::config::Config::builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .build();
        let agent = ureq::Agent::new_with_config(ureq_cfg);

        Self {
            provider,
            model,
            host,
            temperature,
            api_key,
            agent,
        }
    }

    /// Fail-fast verification of AI provider credentials and connectivity
    pub fn check_access(&self) -> Result<(), String> {
        match self.provider {
            AiProvider::Gemini => match &self.api_key {
                Some(k) if !k.trim().is_empty() => Ok(()),
                _ => Err(
                    "GEMINI_API_KEY is not configured. Export CCCP_GEMINI_TOKEN (or GEMINI_API_KEY / GOOGLE_API_KEY) in ~/.env or environment, or switch to Ollama (-po / --ollama)."
                        .to_string(),
                ),
            },
            AiProvider::Ollama => {
                let ping_url = format!("{}/api/tags", self.host.trim_end_matches('/'));
                let ping_agent = ureq::Agent::new_with_config(
                    ureq::config::Config::builder()
                        .timeout_global(Some(Duration::from_secs(2)))
                        .build(),
                );
                match ping_agent.get(&ping_url).call() {
                    Ok(resp) if resp.status().is_success() => Ok(()),
                    Ok(resp) => Err(format!(
                        "Ollama server at {} returned HTTP status {}.",
                        self.host,
                        resp.status()
                    )),
                    Err(e) => Err(format!(
                        "Cannot connect to Ollama at {}: {}. Make sure the Ollama daemon is running ('ollama serve') or switch to Gemini (-pg / --gemini).",
                        self.host, e
                    )),
                }
            }
            AiProvider::Local => {
                let base = self.host.trim_end_matches('/');
                let ping_url = if base.ends_with("/chat/completions") {
                    base.to_string()
                } else if base.ends_with("/v1") {
                    format!("{}/models", base)
                } else {
                    format!("{}/v1/models", base)
                };
                let ping_agent = ureq::Agent::new_with_config(
                    ureq::config::Config::builder()
                        .timeout_global(Some(Duration::from_secs(3)))
                        .build(),
                );
                let mut req = ping_agent.get(&ping_url);
                if let Some(ref key) = self.api_key {
                    req = req.header("Authorization", &format!("Bearer {}", key));
                }
                match req.call() {
                    Ok(resp) if resp.status().is_success() => Ok(()),
                    Ok(resp) => {
                        if resp.status().as_u16() == 404 || resp.status().as_u16() == 405 {
                            Ok(())
                        } else {
                            Err(format!(
                                "Local AI server at {} returned HTTP status {}.",
                                self.host,
                                resp.status()
                            ))
                        }
                    }
                    Err(e) => Err(format!(
                        "Cannot connect to local AI server at {}: {}. Ensure your local AI instance (LocalAI, llama.cpp, vLLM, etc.) is running or check your host configuration.",
                        self.host, e
                    )),
                }
            }
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

    /// Override Gemini API key with optional string
    pub fn with_api_key_opt(mut self, key: Option<String>) -> Self {
        self.api_key = key;
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
            AiProvider::Local => self.triage_local(user_text, repo_context),
        }
    }

    fn system_instructions() -> &'static str {
        "You are a senior software engineering triage assistant (O Divã do Gosplan). \
Your task is to analyze user reports, bug descriptions, and feature requests, and decompose them \
into one or more discrete, atomic issue tickets following the Conventional Commits specification for titles \
(e.g. feat(scope): title, fix(scope): title, refactor(scope): title). \
Split multi-topic or unrelated problems into separate tickets. \
Every ticket body MUST strictly include the canonical GOST 9-phase DD-TDD checklist under '## Tasks / DD-TDD Checklist (9 Fases GOST)': \
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado \
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main \
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção) \
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito \
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando \
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings \
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas \
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados \
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>) \
Return ONLY a valid JSON object matching this schema: \
{\"tickets\": [{\"title\": \"Conventional Commit title\", \"rationale\": \"Why this is a distinct issue\", \"labels\": [\"bug\"], \"body\": \"## Description\\n\\n## Expected Behavior & Acceptance Criteria\\n- [ ] ...\\n\\n## Tasks / DD-TDD Checklist (9 Fases GOST)\\n- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado\\n- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main\\n- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)\\n- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito\\n- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando\\n- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings\\n- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas\\n- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados\\n- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)\\n\"}]} \
Write the ticket body and descriptions in the language used by the user report, adapting checklist labels to that language if necessary while strictly preserving the 9 phases in order."
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

    /// Query Ollama /api/tags to list installed models
    pub fn list_ollama_models(&self) -> Result<Vec<String>, String> {
        let endpoint = format!("{}/api/tags", self.host.trim_end_matches('/'));
        let res = self.agent.get(&endpoint).call().map_err(|e| {
            format!(
                "Failed to connect to Ollama at '{}'. Ensure the Ollama daemon is active: run 'ollama serve' or check your OLLAMA_HOST. Details: {}",
                self.host, e
            )
        })?;
        let body_str = res
            .into_body()
            .read_to_string()
            .map_err(|e| format!("Failed to read Ollama /api/tags response: {}", e))?;
        let json: serde_json::Value = serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse Ollama /api/tags JSON: {}", e))?;
        let mut models = Vec::new();
        if let Some(list) = json.get("models").and_then(|m| m.as_array()) {
            for item in list {
                if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                    models.push(name.to_string());
                }
            }
        }
        Ok(models)
    }

    /// Check whether a model name indicates a text/chat generative model (excluding embeddings)
    pub fn is_generative_model(name: &str) -> bool {
        let lower = name.to_ascii_lowercase();
        !lower.contains("embed") && !lower.contains("bge") && !lower.contains("bert")
    }

    /// Resolve requested Ollama model with auto-detection if not specified explicitly
    pub fn resolve_ollama_model(&self, is_explicit: bool) -> Result<String, String> {
        let installed = self.list_ollama_models()?;
        if installed.is_empty() {
            return Err(format!(
                "No models found on Ollama server at '{}'. Pull a model with: ollama pull {}",
                self.host, self.model
            ));
        }

        let target = self.model.to_ascii_lowercase();
        // 1. Exact match
        if let Some(exact) = installed.iter().find(|m| m.to_ascii_lowercase() == target) {
            return Ok(exact.clone());
        }

        // 2. Prefix or base name match (e.g. 'llama3.2' matches 'llama3.2:1b' or 'llama3.2:latest')
        if let Some(matching) = installed.iter().find(|m| {
            let m_low = m.to_ascii_lowercase();
            m_low.starts_with(&format!("{}:", target))
                || m_low.split(':').next() == target.split(':').next()
        }) {
            return Ok(matching.clone());
        }

        // If user explicitly requested a specific model via --model, do not silently switch
        if is_explicit {
            return Err(format!(
                "Model '{}' not found on Ollama server at '{}'. Available models: {}. Pull it with: ollama pull {}",
                self.model,
                self.host,
                installed.join(", "),
                self.model
            ));
        }

        // 3. Fallback to first available generative model
        if let Some(first_gen) = installed.iter().find(|m| Self::is_generative_model(m)) {
            return Ok(first_gen.clone());
        }

        // 4. Fallback to first installed model
        Ok(installed[0].clone())
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
                    "GEMINI_API_KEY is not configured. Export CCCP_GEMINI_TOKEN (or GEMINI_API_KEY / GOOGLE_API_KEY), add it to ~/.env or .cccprc, or switch to Ollama via '--provider ollama'."
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

    /// Parse OpenAI-compatible chat completion JSON response
    pub fn parse_openai_chat_response(raw_json: &str) -> Result<Vec<DecomposedTicket>, String> {
        let resp_json: serde_json::Value = serde_json::from_str(raw_json)
            .map_err(|e| format!("Failed to parse OpenAI JSON envelope: {}", e))?;

        let content_text = resp_json
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                format!(
                    "OpenAI chat completion response does not contain 'choices[0].message.content': {}",
                    raw_json
                )
            })?;

        Self::parse_tickets_json(content_text)
    }

    /// Perform triage using local OpenAI-compatible HTTP endpoint (/v1/chat/completions)
    fn triage_local(
        &self,
        user_text: &str,
        repo_context: Option<&str>,
    ) -> Result<Vec<DecomposedTicket>, String> {
        let base = self.host.trim_end_matches('/');
        let endpoint = if base.ends_with("/chat/completions") {
            base.to_string()
        } else if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else {
            format!("{}/v1/chat/completions", base)
        };

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
            "temperature": self.temperature,
            "response_format": { "type": "json_object" }
        });

        let mut req = self.agent.post(&endpoint);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", &format!("Bearer {}", key));
        }

        let res = match req.send_json(payload) {
            Ok(r) => r,
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("Connection refused") || err_str.contains("Connect") {
                    return Err(format!(
                        "Failed to connect to local AI server at '{}'. Ensure the server daemon is active or verify your host configuration.",
                        self.host
                    ));
                }
                return Err(format!("Local AI request failed: {}", e));
            }
        };

        let status = res.status();
        let body_str = res
            .into_body()
            .read_to_string()
            .map_err(|e| format!("Failed to read local AI response body: {}", e))?;

        if !status.is_success() {
            return Err(format!(
                "Local AI server returned HTTP {}: {}",
                status.as_u16(),
                body_str
            ));
        }

        Self::parse_openai_chat_response(&body_str)
    }

    /// Extract and parse tickets JSON from raw LLM output
    pub fn parse_tickets_json(raw: &str) -> Result<Vec<DecomposedTicket>, String> {
        let cleaned = clean_json_markdown(raw);

        // Try direct parse first
        if let Some(tickets) = Self::try_parse_json(cleaned) {
            return Ok(tickets);
        }

        // Fallback: sanitize unescaped literal newlines inside JSON string literals
        let sanitized = sanitize_json_newlines(cleaned);
        if let Some(tickets) = Self::try_parse_json(&sanitized) {
            return Ok(tickets);
        }

        Err(format!(
            "Failed to parse structured tickets from LLM output: '{}'",
            raw
        ))
    }

    fn try_parse_json(text: &str) -> Option<Vec<DecomposedTicket>> {
        // Case 1: Structured envelope {"tickets": [...]}
        if let Ok(wrapper) = serde_json::from_str::<TriageJsonWrapper>(text) {
            if !wrapper.tickets.is_empty() {
                return Some(wrapper.tickets);
            }
        }

        // Case 2: Direct array of tickets [{...}, {...}]
        if let Ok(tickets) = serde_json::from_str::<Vec<DecomposedTicket>>(text) {
            if !tickets.is_empty() {
                return Some(tickets);
            }
        }

        // Case 3: Single ticket object {...}
        if let Ok(single) = serde_json::from_str::<DecomposedTicket>(text) {
            return Some(vec![single]);
        }

        // Case 4: Loose JSON Value extraction (resilient to minor LLM variations / duplicate keys)
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(text) {
            if let Some(arr) = val.get("tickets").and_then(|t| t.as_array()) {
                if arr.is_empty() {
                    return Some(vec![]);
                }
                let parsed: Vec<DecomposedTicket> = arr
                    .iter()
                    .filter_map(|item| {
                        let title = item.get("title").and_then(|t| t.as_str())?.to_string();
                        let rationale = item
                            .get("rationale")
                            .and_then(|r| r.as_str())
                            .map(ToString::to_string);
                        let labels = item
                            .get("labels")
                            .and_then(|l| l.as_array())
                            .map(|a| {
                                a.iter()
                                    .filter_map(|x| x.as_str().map(ToString::to_string))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let body = item
                            .get("body")
                            .and_then(|b| b.as_str())
                            .unwrap_or("")
                            .to_string();
                        Some(DecomposedTicket {
                            title,
                            rationale,
                            labels,
                            body,
                        })
                    })
                    .collect();
                if !parsed.is_empty() {
                    return Some(parsed);
                }
            }
        }

        None
    }
}

/// Extract summary of primary package manifest (Cargo.toml, package.json, go.mod, pyproject.toml)
pub fn extract_project_manifest(root: &std::path::Path) -> Option<String> {
    // 1. Rust Cargo.toml
    let cargo_path = root.join("Cargo.toml");
    if cargo_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            let mut name = None;
            let mut version = None;
            let mut desc = None;
            let mut deps = Vec::new();
            let mut in_package = false;
            let mut in_deps = false;

            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') {
                    in_package = trimmed == "[package]";
                    in_deps = trimmed == "[dependencies]" || trimmed == "[dev-dependencies]";
                    continue;
                }
                if in_package {
                    if let Some(rest) = trimmed.strip_prefix("name =") {
                        name = Some(rest.trim().trim_matches('"').to_string());
                    } else if let Some(rest) = trimmed.strip_prefix("version =") {
                        version = Some(rest.trim().trim_matches('"').to_string());
                    } else if let Some(rest) = trimmed.strip_prefix("description =") {
                        desc = Some(rest.trim().trim_matches('"').to_string());
                    }
                } else if in_deps && !trimmed.is_empty() && !trimmed.starts_with('#') {
                    if let Some(dep_name) = trimmed.split('=').next().map(str::trim) {
                        if !dep_name.is_empty() && deps.len() < 8 {
                            deps.push(dep_name.to_string());
                        }
                    }
                }
            }

            if let Some(n) = name {
                let v = version.unwrap_or_else(|| "0.1.0".to_string());
                let d = desc.map(|s| format!(" - {}", s)).unwrap_or_default();
                let dep_str = if deps.is_empty() {
                    String::new()
                } else {
                    format!(" [deps: {}]", deps.join(", "))
                };
                return Some(format!("Rust crate: {} v{}{}{}", n, v, d, dep_str));
            }
        }
    }

    // 2. Node / JS package.json
    let pkg_path = root.join("package.json");
    if pkg_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let name = json
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("project");
                let version = json
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1.0.0");
                let desc = json.get("description").and_then(|v| v.as_str());
                let mut deps = Vec::new();
                if let Some(deps_obj) = json.get("dependencies").and_then(|v| v.as_object()) {
                    for k in deps_obj.keys().take(6) {
                        deps.push(k.as_str());
                    }
                }
                let dep_str = if deps.is_empty() {
                    String::new()
                } else {
                    format!(" [deps: {}]", deps.join(", "))
                };
                let d = desc.map(|s| format!(" - {}", s)).unwrap_or_default();
                return Some(format!(
                    "Node package: {} v{}{}{}",
                    name, version, d, dep_str
                ));
            }
        }
    }

    // 3. Go go.mod
    let go_path = root.join("go.mod");
    if go_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&go_path) {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("module ") {
                    return Some(format!("Go module: {}", rest.trim()));
                }
            }
        }
    }

    // 4. Python pyproject.toml
    let py_path = root.join("pyproject.toml");
    if py_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&py_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("name =") {
                    return Some(format!("Python package: {}", rest.trim().trim_matches('"')));
                }
            }
        }
    }

    None
}

/// Extract top modules and function/type signatures from source directory
pub fn extract_codebase_symbols(
    root: &std::path::Path,
    max_files: usize,
    max_symbols_per_file: usize,
) -> Vec<String> {
    let mut results = Vec::new();
    let src_dirs = [
        root.join("src"),
        root.join("lib"),
        root.join("app"),
        root.join("pkg"),
    ];

    let mut files_to_scan = Vec::new();
    for dir in &src_dirs {
        if dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if matches!(ext, "rs" | "ts" | "js" | "py" | "go") {
                                files_to_scan.push(path);
                                if files_to_scan.len() >= max_files {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        if files_to_scan.len() >= max_files {
            break;
        }
    }

    files_to_scan.sort();

    for path in files_to_scan {
        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        if let Ok(content) = std::fs::read_to_string(&path) {
            let mut symbols = Vec::new();
            for line in content.lines().take(400) {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("pub fn ") {
                    let name = rest.split(['(', '<']).next().unwrap_or("").trim();
                    if !name.is_empty() {
                        symbols.push(format!("{}()", name));
                    }
                } else if let Some(rest) = trimmed.strip_prefix("pub struct ") {
                    let name = rest.split(['{', '<', ';', ' ']).next().unwrap_or("").trim();
                    if !name.is_empty() {
                        symbols.push(name.to_string());
                    }
                } else if let Some(rest) = trimmed.strip_prefix("pub enum ") {
                    let name = rest.split(['{', '<', ' ']).next().unwrap_or("").trim();
                    if !name.is_empty() {
                        symbols.push(name.to_string());
                    }
                } else if let Some(rest) = trimmed.strip_prefix("pub trait ") {
                    let name = rest.split(['{', '<', ' ']).next().unwrap_or("").trim();
                    if !name.is_empty() {
                        symbols.push(name.to_string());
                    }
                } else if let Some(rest) = trimmed.strip_prefix("export function ") {
                    let name = rest.split(['(', '<']).next().unwrap_or("").trim();
                    if !name.is_empty() {
                        symbols.push(format!("{}()", name));
                    }
                } else if let Some(rest) = trimmed.strip_prefix("export class ") {
                    let name = rest.split(['{', '<', ' ']).next().unwrap_or("").trim();
                    if !name.is_empty() {
                        symbols.push(name.to_string());
                    }
                } else if let Some(rest) = trimmed.strip_prefix("export interface ") {
                    let name = rest.split(['{', '<', ' ']).next().unwrap_or("").trim();
                    if !name.is_empty() {
                        symbols.push(name.to_string());
                    }
                } else if line.starts_with("def ") {
                    let name = line
                        .trim_start_matches("def ")
                        .split(['(', ':'])
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !name.is_empty() && !name.starts_with('_') {
                        symbols.push(format!("{}()", name));
                    }
                } else if line.starts_with("class ") {
                    let name = line
                        .trim_start_matches("class ")
                        .split(['(', ':'])
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !name.is_empty() {
                        symbols.push(name.to_string());
                    }
                } else if let Some(rest) = trimmed.strip_prefix("func ") {
                    let name = rest.split('(').next().unwrap_or("").trim();
                    if !name.is_empty() && name.chars().next().is_some_and(|c| c.is_uppercase()) {
                        symbols.push(format!("{}()", name));
                    }
                }

                if symbols.len() >= max_symbols_per_file {
                    break;
                }
            }

            if !symbols.is_empty() {
                results.push(format!("{}: {}", rel_path, symbols.join(", ")));
            }
        }
    }

    results
}

/// Extract project outline with caching in .git/cccp-context.cache
pub fn extract_project_summary(
    root: &std::path::Path,
    head_commit: Option<&str>,
    is_ollama: bool,
) -> String {
    let cache_file = root.join(".git").join("cccp-context.cache");
    if let (Some(commit), true) = (head_commit, cache_file.is_file()) {
        if let Ok(cached) = std::fs::read_to_string(&cache_file) {
            let mut lines = cached.lines();
            if let Some(first) = lines.next() {
                if first.trim() == format!("HEAD:{}", commit) {
                    return lines.collect::<Vec<_>>().join("\n");
                }
            }
        }
    }

    let (max_files, max_syms) = if is_ollama {
        (8, 4) // Compact Gosplan austerity budget (~200 tokens)
    } else {
        (15, 6) // Richer budget for Gemini
    };

    let mut sections = Vec::new();

    if let Some(manifest_info) = extract_project_manifest(root) {
        sections.push(format!("Project Identity: {}", manifest_info));
    }

    let symbols = extract_codebase_symbols(root, max_files, max_syms);
    if !symbols.is_empty() {
        sections.push(format!(
            "Key Modules & Symbols:\n• {}",
            symbols.join("\n• ")
        ));
    }

    let summary = sections.join("\n\n");

    if let Some(commit) = head_commit {
        if root.join(".git").is_dir() {
            let cache_content = format!("HEAD:{}\n{}", commit, summary);
            let _ = std::fs::write(&cache_file, cache_content);
        }
    }

    summary
}

/// Generate rich repository and project context for LLM triage
pub fn build_enriched_context(
    repo: Option<&crate::git::GitRepo>,
    forge: Option<&crate::forges::RepoForgeInfo>,
    config: &Config,
) -> String {
    build_enriched_context_with_provider(repo, forge, config, None)
}

/// Generate rich repository and project context for LLM triage with provider awareness
pub fn build_enriched_context_with_provider(
    repo: Option<&crate::git::GitRepo>,
    forge: Option<&crate::forges::RepoForgeInfo>,
    config: &Config,
    provider: Option<AiProvider>,
) -> String {
    let mut parts = Vec::new();
    let is_ollama = matches!(provider, Some(AiProvider::Ollama));

    if let Some(f) = forge {
        parts.push(format!(
            "Target Forge: {} ({}/{}) at {}",
            f.provider, f.owner, f.repo, f.host
        ));
    }

    if let Some(r) = repo {
        parts.push(format!(
            "Branch: {} | Commit: {} | Working tree clean: {}",
            r.branch.as_deref().unwrap_or("main"),
            r.head_short_id,
            !r.is_dirty
        ));

        if let Some(ref tag) = r.latest_tag {
            parts.push(format!("Latest release tag: {}", tag.name));
        }

        if !r.commits_since_tag.is_empty() {
            let limit = if is_ollama { 5 } else { 10 };
            let recent: Vec<String> = r
                .commits_since_tag
                .iter()
                .take(limit)
                .map(|c| format!("- {}", c.summary))
                .collect();
            parts.push(format!("Recent commits:\n{}", recent.join("\n")));
        }

        let summary =
            extract_project_summary(&r.root_dir, Some(r.head_short_id.as_str()), is_ollama);
        if !summary.trim().is_empty() {
            parts.push(summary);
        }
    }

    if !config.commit_types.is_empty() {
        parts.push(format!(
            "Configured commit types: {}",
            config.commit_types.join(", ")
        ));
    }
    if !config.commit_scopes.is_empty() {
        parts.push(format!(
            "Configured commit scopes: {}",
            config.commit_scopes.join(", ")
        ));
    }

    parts.join("\n\n")
}

/// Replace literal unescaped newlines inside JSON string literals with \n
pub fn sanitize_json_newlines(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            output.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' {
            escape = true;
            output.push(ch);
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            output.push(ch);
            continue;
        }

        if in_string && ch == '\n' {
            output.push_str("\\n");
            continue;
        }

        if in_string && ch == '\r' {
            continue;
        }

        output.push(ch);
    }

    output
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

fn visible_width(s: &str) -> usize {
    let mut in_escape = false;
    let mut len = 0;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

fn wrap_box_text(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            out.push(String::new());
            continue;
        }
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut cur = String::new();
        for word in words {
            if cur.is_empty() {
                cur.push_str(word);
            } else if cur.chars().count() + 1 + word.chars().count() <= width {
                cur.push(' ');
                cur.push_str(word);
            } else {
                out.push(cur);
                cur = word.to_string();
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
    }
    out
}

fn render_box_line(content: &str, width: usize, theme: &Theme) -> String {
    let v_width = visible_width(content);
    let pad = width.saturating_sub(v_width);
    let border = theme.dim("│");
    format!("{} {}{} {}\n", border, content, " ".repeat(pad), border)
}

/// Render a single ticket inside a Soviet Gosplan styled ticket card with borders
pub fn render_ticket_card(
    ticket: &DecomposedTicket,
    idx: usize,
    total: usize,
    destination: Option<&str>,
    theme: &Theme,
) -> String {
    let width = 72;
    let bar = "─".repeat(width + 2);
    let top_border = format!("{}{}{}\n", theme.dim("┌"), theme.dim(&bar), theme.dim("┐"));
    let mid_border = format!("{}{}{}\n", theme.dim("├"), theme.dim(&bar), theme.dim("┤"));
    let bot_border = format!("{}{}{}\n", theme.dim("└"), theme.dim(&bar), theme.dim("┘"));

    let mut out = String::new();
    out.push('\n');
    out.push_str(&top_border);

    // Title line
    let title_prefix = format!("★ [TICKET {}/{}] ", idx + 1, total);
    let title_text = format!(
        "{}{}",
        theme.gold_bold(&title_prefix),
        theme.bold(&ticket.title)
    );
    out.push_str(&render_box_line(&title_text, width, theme));

    out.push_str(&mid_border);

    // Metadata lines
    if let Some(dest) = destination {
        let dest_line = format!("{}: {}", theme.cyan("Destination"), theme.yellow(dest));
        out.push_str(&render_box_line(&dest_line, width, theme));
    }
    if let Some(ref rationale) = ticket.rationale {
        let rat_line = format!("{}: {}", theme.cyan("Rationale  "), rationale);
        for line in wrap_box_text(&rat_line, width) {
            out.push_str(&render_box_line(&line, width, theme));
        }
    }
    if !ticket.labels.is_empty() {
        let lbl_line = format!(
            "{}: {}",
            theme.cyan("Labels     "),
            ticket.labels.join(", ")
        );
        out.push_str(&render_box_line(&lbl_line, width, theme));
    }

    out.push_str(&mid_border);

    // Body lines
    let wrapped_body = wrap_box_text(&ticket.body, width);
    for line in wrapped_body {
        if line.starts_with('#') {
            out.push_str(&render_box_line(&theme.bold(&line), width, theme));
        } else {
            out.push_str(&render_box_line(&line, width, theme));
        }
    }

    out.push_str(&bot_border);
    out
}

/// Display single ticket details inside a framed ticket card
pub fn display_single_ticket(ticket: &DecomposedTicket, idx: usize, total: usize, theme: &Theme) {
    print!("{}", render_ticket_card(ticket, idx, total, None, theme));
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
        theme.bold("★ [GOSPLAN] The Divan — Interactive Ticket Triage ★")
    );
    println!(
        "Provider: {} (model: {})",
        theme.cyan(&provider.to_string()),
        theme.yellow(model)
    );
    println!();

    let total = tickets.len();
    for (idx, ticket) in tickets.iter().enumerate() {
        display_single_ticket(ticket, idx, total, theme);
        if idx + 1 < total {
            println!(
                "{}",
                theme.dim("────────────────────────────────────────────────────────────────")
            );
        }
    }
}

/// Read a single keypress immediately from terminal without requiring Enter
pub fn read_single_key() -> std::io::Result<char> {
    use crossterm::event::{self, Event, KeyCode, KeyEvent};
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
    use std::io::IsTerminal;

    if !std::io::stdin().is_terminal() {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        return Ok(line.chars().next().unwrap_or('n'));
    }

    enable_raw_mode()?;
    let key = loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char(c) => break c.to_ascii_lowercase(),
                KeyCode::Esc => break 'q',
                KeyCode::Enter => break 'y',
                _ => continue,
            }
        }
    };
    disable_raw_mode()?;
    Ok(key)
}

/// Open an external or comrade editor preloaded with initial text, returning edited content
/// Open the internal minimalist terminal editor preloaded with initial text, returning edited content
pub fn open_text_editor(initial_content: &str, title: &str) -> Result<String, String> {
    match crate::editor::edit_text(title, initial_content) {
        Ok(Some(edited)) => Ok(edited),
        Ok(None) => Ok(initial_content.to_string()),
        Err(e) => Err(e),
    }
}
