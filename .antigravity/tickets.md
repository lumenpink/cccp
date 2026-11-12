# Active Tickets - cccp
> Synced at: 2026-11-12 15:01:04 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#173] fix(ci): consumir daemon sccache centralizado via socket uds montado
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/173

### Description
O Forgejo Runner injeta SCCACHE_SERVER_UDS=/root/.cache/sccache/sccache.sock e monta o socket do host. Nao se deve executar sccache --start-server nos passos de CI, apenas utilizar o socket montado e exibir telemetria sccache --show-stats ao final.

---
