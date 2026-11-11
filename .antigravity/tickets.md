# Active Tickets - cccp
> Synced at: 2026-11-11 21:30:38 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#165] fix(ci): unificar compilacao estatica musl amd64 com cargo zigbuild
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/165

### Description
## Problema
O job 'deploy-banca' na esteira de CI falha ao linkar 'x86_64-unknown-linux-musl' com 'cargo build' usando o linker glibc host 'x86_64-linux-gnu-gcc'. Dependências em C (como Lua no mlua) acionam extensões de 64-bit da glibc (fopen64, fseeko64) ausentes na musl libc estática.

## Solução
Substituir a compilação de x86_64-unknown-linux-musl para 'cargo zigbuild', padronizando a toolchain multiarch com zigbuild já usada com sucesso para ARM64 e Windows.

---
