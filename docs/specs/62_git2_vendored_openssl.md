# Especificação Técnica: Vendored OpenSSL no git2 para Suporte Multiarch Musl

**Ticket Associado:** #62  
**Branch:** `fix/62-git2-vendored-openssl`  
**Norma:** GOST 1.5.0

## 1. Objetivo
Permitir que o binário CCCP seja compilado estaticamente para o target `x86_64-unknown-linux-musl` sem depender de headers de OpenSSL do sistema host, resolvendo a falha do job de release e publicação na Banca.

## 2. Requisitos Técnicos
1. No manifesto `Cargo.toml`, atualizar a declaração da crate `git2`:
   ```toml
   git2 = { version = "0.20", features = ["vendored-openssl"] }
   ```
2. Garantir que a compilação local e a execução dos testes e checagens de clippy continuem aprovadas com zero advertências.

## 3. Critérios de Aceitação
- `cargo fmt --check` aprovado.
- `cargo clippy --all-targets -- -D warnings` aprovado com 0 erros/avisos.
- `cargo test --test cli_test` aprovado.
