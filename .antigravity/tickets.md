# Active Tickets - cccp
> Synced at: 2026-11-02 10:11:00 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#124] feat(ci): configurar cancelamento de esteiras redundantes cancel-in-progress
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/124

### Description
### Descrição
Configurar cancelamento de workflows concorrentes redundantes na mesma branch (concurrency.cancel-in-progress: true), evitando acúmulo de filas obsoletas.
1. Ajustar .forgejo/workflows/ci.yml do CCCP com cancel-in-progress: true.
2. Atualizar molde gerado em src/newrepo.rs (cccp init) para incluir concurrency com cancel-in-progress: true.
3. Adicionar asserções em tests/newrepo_test.rs.

### Fases GOST DD-TDD
- [ ] Phase 1: Ticket criado
- [ ] Phase 2: Branch criada
- [ ] Phase 3: Especificação técnica documentada
- [ ] Phase 4: Teste Red implementado
- [ ] Phase 5: Implementação Green
- [ ] Phase 6: Refatoração limpa
- [ ] Phase 7: Testes de mutação aprovados
- [ ] Phase 8: Quality Gate aprovado
- [ ] Phase 9: Conclusão e fechamento

---
