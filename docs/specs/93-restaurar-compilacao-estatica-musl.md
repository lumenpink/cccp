# Especificação Técnica: Restauração da Compilação Estática Musl para Linux AMD64

**Ticket Associado:** #93  
**Branch:** `fix/93-restaurar-compilacao-estatica-musl`  
**Norma:** GOST 1.5.0

## 1. Objetivo
Restabelecer o target `x86_64-unknown-linux-musl` como alvo primário e exclusivo do binário Linux AMD64 do CCCP, revertendo o desvio temporário do Ticket #89 para `gnu.2.28` e garantindo um binário 100% autocontido (`statically linked`), livre de dependências dinâmicas de glibc.

## 2. Requisitos Técnicos
1. **Configuração de Ambiente no `.cargo/config.toml`**:
   - Manter o target `[target.x86_64-unknown-linux-musl]` com `rustflags = ["-C", "target-feature=+crt-static"]`.
   - Na seção `[env]`, substituir `CC_x86_64_unknown_linux_musl = "gcc"` por `CC_x86_64_unknown_linux_musl = "musl-gcc"`.
   - Adicionar diretivas de enlace estático obrigatório:
     - `OPENSSL_STATIC = "1"`
     - `LIBZ_SYS_STATIC = "1"`
     - `PKG_CONFIG_ALL_STATIC = "1"`
2. **Workflow do Forgejo CI (`.forgejo/workflows/ci.yml`)**:
   - Ajustar o passo `Compile CCCP Multiarch Binaries` para compilar diretamente `x86_64-unknown-linux-musl`.
   - Empacotar e publicar `target/x86_64-unknown-linux-musl/release/cccp` como `cccp-linux-amd64` no Forgejo Generic Packages.
3. **Contratos e Testes de Pipeline (`tests/ci_pipeline_test.rs`)**:
   - Modificar o teste de compilação para assegurar que o workflow exige o target `x86_64-unknown-linux-musl` e o caminho `target/x86_64-unknown-linux-musl/release/cccp`.

## 3. Critérios de Aceitação
- `tests/ci_pipeline_test.rs` validando estritamente `x86_64-unknown-linux-musl`.
- Compilação local de `x86_64-unknown-linux-musl` gerando binário estático puro (`statically linked`).
- Teste de execução em contêiner Alpine limpo bem-sucedido.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` 100% aprovados.

## 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**: Binário `x86_64-unknown-linux-musl` estático puro (`statically linked`). Validação em Alpine e pipeline de CI 100% aprovados.

