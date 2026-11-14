# Especificação Técnica — Ticket #175: Alinhar Mensagem de Detalhe para Regras Skipped

- **Ticket:** #175
- **Branch:** `fix/175-alinhar-mensagem-de-detalhe-para-regras-skipped-por-configuracao`
- **Autoridade Normativa:** GOST v3.0.0 / Lilya v2.1.0

---

## 1. Contexto & Motivação
No Ticket #174, corrigiu-se o mapeamento de regras desabilitadas para emitir `CheckStatus::Skip`. Contudo, a mensagem de detalhe emitida foi `Rule <name> disabled in configuration`. A suíte canônica de testes de integração `tests/lua_rules_test.rs:99` esperava a ocorrência de `"Rule skipped by configuration"`, causando falha na esteira de CI do Forgejo.

## 2. Decisão Arquitetural
- Unificar o formato da mensagem de detalhe em `src/lua_rules.rs`:
  ```rust
  detail: format!(
      "Rule {} skipped by configuration (disabled in configuration)",
      rule_name
  )
  ```
- Este formato atende simultaneamente às asserções de `tests/lua_rules_test.rs` (`contains("Rule skipped by configuration")`) e de `tests/config_disable_test.rs` (`contains("disabled in configuration")`).

## 3. Critérios de Aceite e Validação
- `cargo test --test lua_rules_test` passa com 100% de sucesso.
- `cargo test --test config_disable_test` passa com 100% de sucesso.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` limpos com zero advertências.
