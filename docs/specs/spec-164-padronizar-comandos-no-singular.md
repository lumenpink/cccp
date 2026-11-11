# Especificação Técnica: Padronização de Comandos no Singular e Purgatório de Aliases (Ticket #164)

## 1. Contexto e Motivação
A convivência de variantes no plural e singular em comandos de linha de comando (`cccp ticket` vs `cccp tickets`) gerava ambiguidade, duplicação e fragilidade nas esteiras de integração e automação.

Adicionalmente, este ticket consolida a resiliência dos testes contra variáveis de ambiente de desenvolvedores (`CCCP_GEMINI_API_KEY`, etc.), assegura determinismo na sincronização de regras compiladas (`.rules/RULES.md` e `.rules/RULES.toml`) e previne contaminação de working tree no `assemble_repo`.

## 2. Decisões Arquiteturais e Alternativas Descartadas (Para a Natasha)
- **Eliminação de Aliases Plurais na CLI (`src/cli.rs`, `tests/divan_cli_test.rs`):**
  - *Decisão:* Todos os comandos são estritamente no singular (`ticket`, `hook`, `repo`, `spool`, `tag`, etc.). Sem aliases plurais tolerantes.
  - *Alternativa descartada:* Manter `tickets` como alias oculto. Descartada pois gerava dispersão sintática e dependências ambíguas em scripts.
- **Isolamento de Variáveis de Credenciais nos Testes (`tests/ai_test.rs`, `tests/config_test.rs`):**
  - *Decisão:* Iterar a constante normativa `GEMINI_ENV_KEYS` limpando e restaurando todas as variáveis de credenciais durante a execução dos testes de chave ausente.
  - *Alternativa descartada:* Limpar apenas `GEMINI_API_KEY`. Descartada porque vazava `CCCP_GEMINI_API_KEY` e causava falsos positivos quando executado em máquinas configuradas.
- **Idempotência no Auto-Commit de `assemble_repo` (`src/catalog.rs`, `src/profile.rs`):**
  - *Decisão:* Adicionar o diretório `.rules` ao `git add` no auto-commit se existir, e assegurar que `sync_gost_specs` execute antes da gravação de `output` em `.gost/GOST.md`.
  - *Alternativa descartada:* Deixar `.rules/` desacompanhado no working tree. Descartada porque quebrava a checagem de working tree limpa na segunda execução de assemble.
- **Isolamento Concorrente de I18n em Testes (`src/i18n.rs`, `tests/i18n_test.rs`):**
  - *Decisão:* Uso de mutex estático de teste para testes que manipulam `CURRENT_LOCALE` e proteção contra locks envenenados com `into_inner()`.

## 3. Resultados da Execução
- **Testes Unitários e Integrados:** 100% de aprovação em todos os alvos com `cargo test --all-targets`.
- **Quality Gate:** `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem qualquer erro ou advertência.
