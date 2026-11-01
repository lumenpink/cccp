# Active Tickets - cccp
> Synced at: 2026-11-01 01:37:11 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#115] fix(ci): sobrescrever linker musl amd64 para evitar sigsegv no container de cd
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/115

### Description
## Description
No container de CI gost-ci-cross, a variavel de ambiente CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc esta definida globalmente na imagem. O script musl-gcc do Debian forca arquivos CRT conflitantes sobre o runtime musl embutido do rustc, causando SIGSEGV (codigo 139) na execucao de binarios x86_64-unknown-linux-musl.

## Expected Behavior & Acceptance Criteria
- [ ] No job deploy-banca de .forgejo/workflows/ci.yml, definir explicitamente CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER: x86_64-linux-gnu-gcc
- [ ] O binario x86_64-unknown-linux-musl deve compilar de forma puramente estatica e executar cccp --version com codigo 0
- [ ] Atualizar testes em tests/ci_pipeline_test.rs para validar a configuracao do linker no workflow

---
