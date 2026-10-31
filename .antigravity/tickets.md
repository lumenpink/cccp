# Active Tickets - cccp
> Synced at: 2026-10-31 14:15:10 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#107] fix(build): compilar binario musl amd64 nativo sem musl-gcc e zigbuild
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/107

### Description
Corrigir segfault no binário x86_64-unknown-linux-musl removendo musl-gcc e cargo zigbuild da compilação de Linux AMD64, e adicionar validação de execução cccp --version antes do upload.

---
