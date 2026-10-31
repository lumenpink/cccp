# Especificação Técnica: Ticket #107

## 1. Contexto e Problema
O binário compilado para `x86_64-unknown-linux-musl` e publicado como `cccp-linux-amd64` falhava com SIGSEGV (código 139) antes de atingir a função `main()`.
A causa identificada foi a intervenção de `musl-gcc` e `cargo-zigbuild`, que forçavam arquivos CRT (`crt1.o`) conflitantes sobre a biblioteca padrão musl embutida no próprio `rustc`.

## 2. Requisitos Técnicos
1. Remover `CC_x86_64_unknown_linux_musl = "musl-gcc"` de `.cargo/config.toml`.
2. No workflow de CI (`.forgejo/workflows/ci.yml`), compilar `x86_64-unknown-linux-musl` nativamente via `cargo build --release --target x86_64-unknown-linux-musl -p cccp`.
3. Adicionar passo de fumaça executando `./target/x86_64-unknown-linux-musl/release/cccp --version` antes de publicar os artefatos.
4. Manter enlace estático limpo sem interpretador dinâmico (`readelf -l`).

## 3. Critérios de Aceite
- Pipeline e testes em `tests/ci_pipeline_test.rs` aprovados com asserções positivas e de mutação.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos.
- `cccp check --strict` aprovado com 100% de conformidade.

## 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**: Compilação musl x86_64 limpa e nativa, livre de conflito CRT por musl-gcc ou zigbuild, com teste de fumaça executado deterministicamente em CI.
