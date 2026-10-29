# Active Tickets - cccp
> Synced at: 2026-10-29 18:52:47 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#85] fix(push): atualizar tracking branch remota no fetch de paridade
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/85

### Description
## Description
Ajustar a reconciliacao de paridade em src/push.rs para atualizar explicitamente a tracking branch remota durante o fetch silencioso, eliminando falhas em git_locks_push_test no CI.

## Expected Behavior & Acceptance Criteria
- [ ] Executar fetch atualizando o tracking branch remoto de destino em src/push.rs.
- [ ] Garantir que HEAD...origin/branch detecte corretamente commits remotos pendentes e divergencias.
- [ ] Validar testes em tests/git_locks_push_test.rs.
- [ ] Desbloquear a esteira de CI e publicacao multiarch na Banca.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch fix/84-push-parity-fetch criada
- [ ] Fase 3 - Doc/Spec: Especificacao tecnica detalhada
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementacao minima funcional com teste passando
- [ ] Fase 6 - Refactor: Codigo limpo, idiomatico e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes criticos eliminados com assercoes robustas
- [ ] Fase 8 - Quality Gate: Linter -D warnings e testes validados
- [ ] Fase 9 - Close: Changelog e documentacao atualizados; commit com (closes #84)

---
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
