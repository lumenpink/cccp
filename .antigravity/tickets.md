# Active Tickets - cccp
> Synced at: 2026-10-29 18:49:14 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#84] fix(tests): parametrizar branch main nos remotos bare de git_locks_push_test
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/84

### Description
## Description
Garantir que a criacao de repositorios bare e clones peer nos testes de paridade de git_locks_push_test usem explicitamente a branch main (-b main), prevenindo falhas em ambientes de CI onde init.defaultBranch nao esta configurado.

## Acceptance Criteria
- [ ] Inicializar remotos bare com git init --bare -b main.
- [ ] Garantir que o clone peer respeite a branch main.
- [ ] Passar cargo test --test git_locks_push_test com sucesso tanto localmente quanto dentro do container gost-ci-cross.
- [ ] Passar quality gates rigorosos (cargo fmt, cargo clippy -D warnings).

---
