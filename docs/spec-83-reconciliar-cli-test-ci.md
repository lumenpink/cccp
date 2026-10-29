# Especificação Técnica — Ticket #83: Reconciliação de Testes de CLI com Subcomando CI

## 1. Visão Geral e Contexto
Os tickets #76 e #82 reintroduziram formalmente o subcomando canônico `cccp ci` na CLI do CCCP (com suporte aos subcomandos de esteira `publish-artifacts`, `deploy-site`, `status`, `run` e `log`). No entanto, o arquivo `tests/cli_test.rs` ainda continha uma função de teste legada (`test_ci_command_removed`) que asseverava que `cccp ci status`, `cccp ci run` e `cccp ci log` deveriam ser rejeitados como erro de parsing.

Essa discrepância provocou a falha do job `Checks, Tests & Quality Gates` (Run #1502) no Forgejo, impedindo a execução do job dependente `deploy-banca` e congelando a versão publicada na Banca (`banca.2lp.in`) na `3.2.0-dev.260`.

## 2. Requisitos e Critérios de Aceite
- **RF-01 (Reconciliação de parsing de `cccp ci` em `tests/cli_test.rs`)**:
  - Atualizar o teste em `tests/cli_test.rs` para asseverar que `cccp ci` e seus subcomandos (`status`, `publish-artifacts`, `deploy-site`) são analisados corretamente pela CLI.
  - Verificar se as estruturas e variantes de `Commands::Ci` correspondem aos argumentos esperados.
- **RF-02 (Validação de Integridade e Subcomandos Inválidos)**:
  - Assegurar que subcomandos desconhecidos ou inválidos (ex.: `cccp ci invalid-cmd`) continuem resultando em erro (`.is_err()`).
- **RF-03 (Quality Gate e Desbloqueio de CI)**:
  - `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test --all-targets` devem passar com 100% de sucesso.
  - Permitir a execução bem-sucedida do pipeline de CI e despacho multiarch para a Banca.
