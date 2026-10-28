# Especificação Técnica — Ticket #82: Reconciliação de Testes de Domínio da CLI com Subcomando CI

## 1. Visão Geral e Contexto
O ticket #76 reintroduziu formalmente o subcomando canônico `cccp ci` (com suporte a `publish-artifacts`, `deploy-site`, `status`, `run` e `log`). No entanto, o arquivo de teste `tests/cli_domains_consolidation_test.rs`, remanescente da consolidação de domínios prévia, ainda continha asserções exigindo a rejeição de `cccp ci` e sua ausência no menu de ajuda root.

## 2. Requisitos e Critérios de Aceite
- **RF-01 (Remoção de 'ci' da lista negra de subcomandos obsoletos)**:
  - Em `test_removed_root_commands_are_rejected`, remover `assert!(Cli::try_parse_from(["cccp", "ci"]).is_err());`.
  - Garantir que `cccp ci` e `cccp ci status` sejam aceitos na CLI.
- **RF-02 (Remoção de 'ci' da lista de comandos proibidos no help)**:
  - Em `test_mutation_root_help_slim_and_canonical`, remover `"ci"` do loop de comandos proibidos na listagem do help.
  - Adicionar `assert!(help.contains("ci"));` na lista de comandos canônicos exibidos.
- **RF-03 (Quality Gate)**:
  - `cargo test --all-targets` deve passar com 100% de sucesso.
