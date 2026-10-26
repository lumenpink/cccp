# Especificação Técnica: Atualização de Asserções de CLI para Comando Top-level Divan

**Ticket Associado:** #61  
**Branch:** `fix/61-atualizar-assercao-cli-divan`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Alinhar os testes de integração de CLI em `tests/cli_test.rs` com a funcionalidade implementada no ticket #52, onde `cccp divan` passou a ser aceito diretamente como comando de topo no enum `Commands::Divan`.

## 2. Requisitos Técnicos
1. Em `tests/cli_test.rs`, substituir a asserção obsoleta `assert!(Cli::try_parse_from(["cccp", "divan"]).is_err());` por teste que valida que `["cccp", "divan"]` analisa com sucesso em `Commands::Divan(_)`.
2. Validar que subcomandos ou opções inválidas continuam sendo rejeitados com erro pelo Clap.

## 3. Critérios de Aceitação
- `cargo test --test cli_test` passa integralmente com 40/40 testes aprovados.
- Conformidade total com `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.
