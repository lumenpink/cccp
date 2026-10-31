# Especificação Técnica: Compilação Estática Musl Sem Interpretador Dinâmico

**Ticket Associado:** #106  
**Branch:** `fix/106-compilar-binarios-musl-estaticamente-sem-interpretador-dinamico`  
**Norma:** GOST 1.5.0

## 1. Objetivo
Garantir que a compilação multiarch dos binários Linux baseados em Musl (`x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`) produza executáveis ELF 100% estáticos (`statically linked`), desprovidos de cabeçalho `INTERP` (`.interp` / `/lib/ld-musl-x86_64.so.1`).
Isso previne falhas de execução (`exit status 127` / `command not found` ou `exit status 139` / `segmentation fault`) ao executar os artefatos em contêineres e sistemas baseados em Glibc (Debian, Ubuntu) ou contêineres mínimos.

## 2. Requisitos Técnicos
1. **Configuração de Linker no `.cargo/config.toml`**:
   - Para o alvo `[target.x86_64-unknown-linux-musl]`, declarar explicitamente:
     `rustflags = ["-C", "target-feature=+crt-static", "-C", "link-arg=-static"]`.
   - Para o alvo `[target.aarch64-unknown-linux-musl]`, declarar explicitamente:
     `rustflags = ["-C", "target-feature=+crt-static", "-C", "link-arg=-static"]`.
   - Assegurar na seção `[env]` que variáveis estáticas essenciais (`OPENSSL_STATIC`, `LIBZ_SYS_STATIC`, `PKG_CONFIG_ALL_STATIC`) permaneçam ativas.

2. **Workflow do Forgejo CI (`.forgejo/workflows/ci.yml`)**:
   - No passo `Compile CCCP Multiarch Binaries`, garantir que a compilação com `cargo zigbuild` e fallback `cargo build` invoque o linker com `-C link-arg=-static`.
   - Adicionar asserção determinística contra cabeçalhos de interpolação dinâmica antes do empacotamento e publicação:
     ```bash
     readelf -l target/x86_64-unknown-linux-musl/release/cccp | grep -q "program interpreter" && exit 1 || true
     ```

3. **Contratos e Testes de Pipeline (`tests/ci_pipeline_test.rs`)**:
   - Estender `tests/ci_pipeline_test.rs` para auditar a presença obrigatória de `link-arg=-static` em `.cargo/config.toml` e no workflow `.forgejo/workflows/ci.yml`.
   - Teste de mutação verificando sensibilidade à ausência de `link-arg=-static`.

## 3. Critérios de Aceitação
- `tests/ci_pipeline_test.rs` aprovado com asserções estritas de `link-arg=-static`.
- `.cargo/config.toml` e `.forgejo/workflows/ci.yml` configurados com garantia de enlace puramente estático.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos.
- `cccp check --strict` com conformidade aprovada.
