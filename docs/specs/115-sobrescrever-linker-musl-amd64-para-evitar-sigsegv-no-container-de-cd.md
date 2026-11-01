# Especificação Técnica: Ticket #115

## 1. Contexto e Problema
O binário compilado para `x86_64-unknown-linux-musl` no pipeline de CD (`deploy-banca` no arquivo `.forgejo/workflows/ci.yml`) falhava com SIGSEGV (código 139) durante o teste de fumaça `./target/x86_64-unknown-linux-musl/release/cccp --version`.
A investigação e testes empíricos comprovaram que a imagem de contêiner utilizada (`gambi.2lp.in/lumen/gost-ci-cross:latest`) define globalmente a variável de ambiente `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc`.
Por precedência do Cargo, variáveis de ambiente sobrepõem qualquer configuração em `.cargo/config.toml`. O script `musl-gcc` do Debian impõe especificações CRT que conflitam com o runtime musl empacotado no compilador `rustc`, causando corrupção e falha de segmentação ao inicializar o executável.

## 2. Requisitos Técnicos
1. No arquivo `.forgejo/workflows/ci.yml`, na etapa `Compile CCCP Multiarch Binaries` do job `deploy-banca`, definir explicitamente a variável de ambiente:
   `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER: "x86_64-linux-gnu-gcc"`.
2. Assegurar que o teste de fumaça `./target/x86_64-unknown-linux-musl/release/cccp --version` execute com código 0 e que o binário gerado seja 100% estático sem `program interpreter`.
3. Atualizar a suíte de testes em `tests/ci_pipeline_test.rs` para asserir a presença da sobreposição da variável `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER` com `x86_64-linux-gnu-gcc` no workflow, prevenindo regressões.

## 3. Critérios de Aceite
- `tests/ci_pipeline_test.rs` validando que `.forgejo/workflows/ci.yml` contém a variável sobrescrita com `x86_64-linux-gnu-gcc`.
- Testes de mutação verificando a detecção de ausência do linker ou reintrodução de `musl-gcc`.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou falhas.
- Conformidade estrita com as 9 fases do GOST DD-TDD.

## 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**: Variável de ambiente `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER: "x86_64-linux-gnu-gcc"` adicionada à etapa `Compile CCCP Multiarch Binaries` do workflow `deploy-banca`. Isso impede que o Cargo no contêiner `gost-ci-cross` herde o script `musl-gcc` do Debian que provocava SIGSEGV devido a conflitos de CRT. O binário musl AMD64 é puramente estático sem interp e executa o smoke test `cccp --version` com sucesso.
