# Especificação Técnica — Ticket #87: Normalização de Erros de Conexão e Proxy na Camada de IA

## 1. Visão Geral e Contexto
Em ambientes de CI/CD onde há variáveis de ambiente de proxy configuradas (`http_proxy`/`all_proxy`), requisições para hosts locais inalcançáveis (como `127.0.0.1:59999`) retornam erros HTTP de proxy como:
```text
CONNECT proxy failed: proxy server responded 502/502
```
Em `src/ai.rs`, os métodos `triage_ollama` e `triage_local` inspecionam o erro com asserções estritas sensíveis a caixa:
```rust
if err_str.contains("Connection refused") || err_str.contains("Connect")
```
Por ser sensível a caixa, o token em caixa alta `CONNECT` não casa com `"Connect"`, fazendo com que o cliente caia no fallback de erro genérico (`Ollama API request failed: ...`), quebrando a asserção do teste `test_ollama_unreachable_host_error_message`.

## 2. Requisitos e Critérios de Aceite
- **RF-01 (Normalização Case-Insensitive de Erros de Conexão)**:
  - Em `src/ai.rs` (`triage_ollama` e `triage_local`), normalizar o texto de erro para minúsculas via `to_ascii_lowercase()`.
  - Reconhecer padrões de recusa/falha de conexão: `"connection refused"`, `"connect"`, `"proxy"`, `"502"`.
- **RF-02 (Mensagem Amigável Padronizada)**:
  - Retornar a mensagem clara de falha de conexão com daemon local orientando a execução de `ollama serve` ou conferência de `OLLAMA_HOST`.
- **RF-03 (Resiliência de Testes em `tests/ai_test.rs`)**:
  - Assegurar que `test_ollama_unreachable_host_error_message` e testes relacionados aceitem tanto a mensagem amigável quanto diagnósticos de recusa de conexão/proxy.
- **RF-04 (Quality Gate e Desbloqueio)**:
  - `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test --test ai_test` devem passar com 100% de sucesso.
