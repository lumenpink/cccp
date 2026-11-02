# Active Tickets - cccp
> Synced at: 2026-11-02 15:50:17 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#125] fix(ci): casar jobs compostos com CI/CD e extrair tempo resiliente
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/125

### Description
### Descrição
Corrigir divergências entre o monitor repo top e a interface web do Forgejo:
1. Em src/forges.rs, ajustar pipeline_matches_job_name / strip_composite para usar s.rfind(" / ") ou s.rfind('/'), permitindo casar corretamente contextos contendo CI/CD.
2. Em src/repo_top.rs, tornar format_ci_elapsed_time resiliente para buscar qualquer job ativo com updated_at válido, evitando retornar '-' prematuramente.
3. Adicionar testes unitários para nomes de jobs com CI/CD e formatação de tempo resiliente.

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
