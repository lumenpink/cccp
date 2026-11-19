# Active Tickets - cccp
> Synced at: 2026-11-19 17:42:24 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#184] refactor(gost): purga de elementos obsoletos das normas no cccp
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/184

### Description
## Contexto & Motivação
Com a implementação da absorção de linguagens no GOST.md (Ticket #183), os artefatos legados (.rules/, rules.json, PROFILE.md, UPGRADE.md e languages/) foram eliminados do cccp.

## Requisitos
1. Efetivar a purga de .rules/ e arquivos obsoletos em .gost/.
2. Recompilar .gost/GOST.md auto-contido.
3. Validar conformidade com cccp check --strict (24/24 pass).

---
