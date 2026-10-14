# Especificação Técnica: Suporte a Instância Local de IA no Divã

**Ticket Associado:** #149  
**Branch:** `feat/149-local-ai-provider`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Permitir que o Divã (`cccp divan` / `cccp diva`) utilize servidores locais de inferência de IA compatíveis com o padrão universal OpenAI HTTP API (`/v1/chat/completions`), como LocalAI, llama.cpp server, vLLM, LM Studio ou sidecars Gosplan soberanos. Além disso, padronizar o modelo padrão de inferência local para `llama3.2:3b`, otimizado para hardware com 4GB de VRAM e alta aderência à governança GOST.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Extensão do Enum `AiProvider`:**
   - Adicionar variante `AiProvider::Local`.
   - String de exibição canônica: `"local"`.
   - Parsing tolerante aceitando: `"local"`, `"openai"`, `"localai"`, `"vllm"`, `"custom"`.
2. **Configuração Declarativa e Variáveis de Ambiente (`src/config.rs`):**
   - Suporte aos campos:
     - `local_ai_host`: URL base do endpoint local (default: `"http://localhost:8080/v1"`).
     - `local_ai_model`: Nome do modelo (default: `"llama3.2:3b"`).
     - `local_ai_api_key`: Chave de autenticação opcional (Bearer token).
     - `local_ai_temperature`: Temperatura de amostragem (default: `0.2`).
   - Resolução hierárquica respeitando variáveis de ambiente: `LOCAL_AI_HOST`, `OPENAI_BASE_URL`, `LOCAL_AI_MODEL`, `OPENAI_MODEL`, `LOCAL_AI_KEY` e `OPENAI_API_KEY`.
   - Atualização do modelo padrão do Ollama para `"llama3.2:3b"`.
3. **CLI Arguments (`src/cli.rs`):**
   - Adicionar flag `--local` (alias `--pl`) em `DivanArgs`.
   - Preservar compatibilidade total com `--host` e `--model` para sobreposição em linha de comando.
4. **Verificação de Acesso e Fail-Fast:**
   - `AiClient::check_access()` para `AiProvider::Local` valida a conectividade HTTP antes de prosseguir.
   - Mensagens de erro claras instruindo o usuário caso a instância local esteja inativa ou a porta bloqueada.
5. **Motor de Triagem Local (`src/ai.rs`):**
   - `triage_local()` envia payload JSON padronizado para `<host>/chat/completions` (reconciliando se o host já termina em `/v1` ou `/v1/chat/completions`).
   - Inclui cabeçalho `Authorization: Bearer <token>` se configurado.
   - Extrai o conteúdo de `choices[0].message.content` e delega para o parser de tickets Gosplan.

## 3. Contrato da API (`src/ai.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    Gemini,
    Ollama,
    Local,
}

impl AiClient {
    fn triage_local(
        &self,
        user_text: &str,
        repo_context: Option<&str>,
    ) -> Result<Vec<DecomposedTicket>, String>;
}
```
