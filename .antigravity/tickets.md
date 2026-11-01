# Active Tickets - cccp
> Synced at: 2026-11-01 09:31:27 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#117] fix(build): desativar default-features do git2 eliminando openssl no musl
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/117

### Description
## Description
O git2 estava configurado com features = ["vendored-openssl"], trazendo dependencias de transporte de rede em C (openssl-sys e libssh2-sys). No alvo x86_64-unknown-linux-musl, a compilacao do OpenSSL C gerava erros de linkagem estatica (undefined reference to fopen64 e SIGSEGV codigo 139).
Como o cccp utiliza o git2 exclusivamente para manipulacao local de repositorio, commits e objetos (sem usar funcoes de rede libgit2), a ativacao de default-features = false no git2 preserva 100% da autonomia de inspecao Git local e elimina por completo a dependencia do OpenSSL C.

## Expected Behavior & Acceptance Criteria
- [ ] No Cargo.toml, configurar git2 = { version = "0.20", default-features = false }.
- [ ] Remover dependencia transitiva de openssl-sys da arvore de dependencias.
- [ ] O binario estatico x86_64-unknown-linux-musl deve compilar sem erros de linkagem e executar cccp --version com codigo 0.
- [ ] Adicionar assercoes em tests/ci_pipeline_test.rs para garantir que git2 permanecera com default-features = false.

---
