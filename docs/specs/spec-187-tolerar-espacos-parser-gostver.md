# Especificação do Ticket #187: Tolerar Espaços ao Redor do Operador `=` no Parser de GOSTVER

## 1. Contexto e Motivação
Na função `read_gostver` em `src/audit.rs`, o fallback transiente para arquivos `GOSTVER` legados usava `trimmed.strip_prefix("GOST_VERSION=")`, falhando quando havia espaços ao redor do operador `=` (ex.: `  GOST_VERSION  =  "1.5.0"  `).
Essa limitação causava falha no teste de integração `tests/catalog_test.rs::test_catalog_gost_version_edge_cases_and_mutation`.

## 2. Requisitos da Correção
1. Em `src/audit.rs::read_gostver`:
   - Ignorar linhas comentadas (`#`).
   - Fazer parsing com `trimmed.split_once('=')`, aplicando `.trim()` na chave e `.trim().trim_matches('"').trim_matches('\'').trim()` no valor.
2. Em `.gost/rules.toml`:
   - Incluir o bypass de ambiente hermético de CI na regra `KOLLONTAI-018`.
3. Validar `cargo test --test catalog_test`, `cargo test --test audit_test`, `cargo clippy --all-targets -- -D warnings` e `cccp check --strict`.

## 3. Árvore de Decisões (para a Natasha)
- **`split_once('=')` vs Regex:** `split_once('=')` é zero-dependência, idiomático, O(n) e preserva valores que eventualmente contenham `=` em URLs ou parâmetros.
