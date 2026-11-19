# Active Tickets - cccp
> Synced at: 2026-11-19 23:34:18 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#185] fix(audit): quebra de recursao infinita em repositorios grundnorm ou sem perfil
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/185

### Description
Evitar stack overflow por recursao mutua entre read_gostver e read_profile_config quando GOST_PROFILE_URL for vazio ou nao declarado (ex: repositorio gost).

---
