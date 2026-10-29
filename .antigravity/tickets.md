# Active Tickets - cccp
> Synced at: 2026-10-29 21:31:18 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#88] fix(profile): atualizar endpoint canonico da spec gost para gambi.2lp.in
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/88

### Description
## Description
Atualizar os endpoints de download da especificacao GOST (SPEC.md) e recursos normativos em src/profile.rs e src/newrepo.rs de git.2lp.in para o dominio canonico ativo gambi.2lp.in, eliminando falhas de resolucao de DNS e montagem de perfil na esteira de CI.

## Acceptance Criteria
- [ ] Priorizar https://gambi.2lp.in/lumen/gost/raw/branch/main/SPEC.md em src/profile.rs.
- [ ] Atualizar as URLs candidatas em src/newrepo.rs para incluir gambi.2lp.in.
- [ ] Garantir que tests/profile_test.rs passe 100% tanto localmente quanto dentro do container gost-ci-cross.
- [ ] Passar quality gates rigorosos (cargo fmt, cargo clippy -D warnings).

---
