# Active Tickets - cccp
> Synced at: 2026-10-29 17:12:41 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#83] fix(ci): reconciliar teste de cli ci e desbloquear deploy na banca
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/83

### Description
## Description
Reconciliar teste legado em tests/cli_test.rs com o subcomando canonico cccp ci, eliminando a falha que bloqueia o job deploy-banca na esteira de CI.

## Expected Behavior & Acceptance Criteria
- [ ] Atualizar tests/cli_test.rs para remover assercoes que rejeitam cccp ci.
- [ ] Validar contrato e subcomandos de cccp ci (status, publish-artifacts, deploy-site).
- [ ] Garantir que cargo test --all-targets passe com 100% de sucesso.
- [ ] Desbloquear a esteira e o despacho de novos binarios para a Banca (banca.2lp.in).

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch fix/83-reconciliar-cli-test-ci criada
- [ ] Fase 3 - Doc/Spec: Especificacao tecnica detalhada
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementacao minima funcional com teste passando
- [ ] Fase 6 - Refactor: Codigo limpo, idiomatico e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes criticos eliminados com assercoes robustas
- [ ] Fase 8 - Quality Gate: Linter -D warnings e testes validados
- [ ] Fase 9 - Close: Changelog e documentacao atualizados; commit com (closes #83)

---
