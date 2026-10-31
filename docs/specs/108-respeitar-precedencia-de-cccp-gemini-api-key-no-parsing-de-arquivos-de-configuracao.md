# Especificação Técnica: Ticket #108
## fix(ai): respeitar precedência de CCCP_GEMINI_API_KEY no parsing de arquivos de configuração

### 1. Contexto e Problema
O CCCP provê suporte à triagem de tickets assistida por IA (`cccp ticket divan`).
Na especificação normativa do Ticket #79 (`docs/spec-79-precedencia-cccp-gemini-token.md`), definiu-se a precedência estrita para credenciais do Google Gemini:
1. `CCCP_GEMINI_TOKEN` / `CCCP_GEMINI_API_KEY` (Prioridade Máxima)
2. `GEMINI_TOKEN` / `GEMINI_API_KEY` / `gemini_api_key` (Prioridade Secundária)
3. `GOOGLE_API_KEY` / `google_api_key` (Prioridade Terciária)

Entretanto, ao carregar a configuração a partir de arquivos `.env`, `~/.env` ou `.cccprc`:
- `set_key_value()` em `src/config.rs` apenas mapeava a chave `gemini_api_key`.
- Declarações de `CCCP_GEMINI_TOKEN` e `CCCP_GEMINI_API_KEY` eram armazenadas no mapa `config.extra`, mas nunca consultadas por `resolve_gemini_api_key()`.
- Se o ambiente ou arquivo de configuração contivesse uma chave desativada em `GEMINI_API_KEY` e uma chave válida em `CCCP_GEMINI_API_KEY`, a chave desativada prevalecia, causando erro HTTP 401 na comunicação com a API Gemini.

### 2. Requisitos Técnicos
1. Em `src/config.rs`:
   - Atualizar `set_key_value()` para tratar as chaves `cccp_gemini_token` e `cccp_gemini_api_key`, além de `gemini_token` e `google_api_key`, armazenando ou respeitando a precedência normativa durante a mesclagem de arquivos de configuração (`merge_from_str`).
   - Se uma chave de maior prioridade já tiver sido definida, uma chave de menor prioridade presente no mesmo arquivo ou em arquivos anteriores de menor escopo não deve sobrescrevê-la.
2. Em `src/ai.rs`:
   - Atualizar `resolve_gemini_api_key()` para verificar a precedência completa tanto no objeto `Config` (incluindo chaves em `extra` se aplicável) quanto no ambiente do processo (`resolve_gemini_env_key`).
   - Assegurar que valores em branco ou preenchidos apenas com espaços sejam descartados (`trim()`).

### 3. Critérios de Aceitação
- Quando `CCCP_GEMINI_API_KEY` ou `CCCP_GEMINI_TOKEN` estiver presente em `~/.env`, `.env` ou `.cccprc`, essa chave deve ter precedência sobre `GEMINI_API_KEY` e `GOOGLE_API_KEY`.
- Se `GEMINI_API_KEY` estiver definida no ambiente ou no arquivo `.env`, mas `CCCP_GEMINI_API_KEY` estiver definida em `~/.env` ou `.cccprc`, a chave específica do CCCP deve ser a resolvida.
- Testes automatizados de integração cobrindo a precedência completa ao carregar via `Config::load_for_dir()` e `resolve_gemini_api_key()`.
