# Active Tickets - cccp
> Synced at: 2026-11-02 08:35:47 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#123] feat(ci): adicionar coluna dedicada de tempo decorrido na fila do monitor
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/123

### Description
### Descrição
Adicionar uma coluna dedicada TEMPO (com largura de 7-8 caracteres) na tabela do monitor repo top (e opcionalmente nas listagens detalhadas do catálogo), calculando o tempo decorrido relativo ao momento atual a partir de updated_at/created_at das pipelines de CI.

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
## [#122] feat(ci): priorizar repositorios em running no topo da ordenacao do monitor
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/122

### Description
### Descrição
Refatorar a ordenação do monitor do catálogo para que repositórios com jobs em execução ativa (running > 0) fiquem no topo absoluto (prioridade 0), seguidos por waiting (prioridade 1), blocked (prioridade 2), falhas (prioridade 3) e sucesso (prioridade 4).

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
## [#121] feat(ci): normalizar status pending via description e cor ciano para running
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/121

### Description
### Descrição
Normalizar o status de commit retornado pelo Forgejo quando for pending utilizando o campo description (Waiting to run -> waiting, Has started running -> running, Blocked by required conditions -> blocked). Aplicar a cor ciano (cyan) para running no resumo de CI e na renderização do monitor top, mantendo amarelo para waiting e blocked.

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
