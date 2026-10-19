# Active Tickets - cccp
> Synced at: 2026-10-19 12:08:16 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#151] feat(catalog): atualizar contagem de tickets de todos os repos no catalogo
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/151

### Description
## Contexto
Atualmente, o comando `cccp catalog list` exibe a contagem de tickets dos repositórios cadastrados a partir do cache `catalog.json`. Quando tickets são resolvidos ou criados nas forjas remotas, o catálogo permanece com valores desatualizados até que cada repositório seja sincronizado individualmente.

## Requisitos e Critérios de Aceite
- Implementar subcomando ou opção no `cccp catalog` (ex: `cccp catalog sync` ou flag em `cccp sync -c`) para auditar/sincronizar a contagem de tickets de todos os repositórios cadastrados no `catalog.json`.
- Executar a varredura com degradação graciosa em caso de forjas offline ou repositórios remotos não clonados.
- Atualizar `catalog.json` atomicamente refletindo o estado fiel do ecossistema.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch dedicada por ticket
- [ ] Fase 3 - Doc/Spec: Especificação do comando de atualização de tickets no catálogo
- [ ] Fase 4 - Red: Teste falho validando a sincronização em lote
- [ ] Fase 5 - Green: Implementação da atualização em lote no cccp catalog
- [ ] Fase 6 - Refactor: Eliminação de duplicações e código idiomático
- [ ] Fase 7 - Mutation: Testes de mutação e resiliência
- [ ] Fase 8 - Quality Gate: Validação estrita
- [ ] Fase 9 - Close: Finalização e fechamento com (closes #<id>)

---
