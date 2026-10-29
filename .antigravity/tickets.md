# Active Tickets - cccp
> Synced at: 2026-10-29 20:59:16 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#87] fix(ai): normalizar tratamento de erro de conexao com ollama e proxy
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/87

### Description
## Description
Normalizar analise de erros de rede HTTP em src/ai.rs para case-insensitive cobrindo CONNECT proxy failed e connection refused, eliminando falha em tests/ai_test.rs no CI.

## Expected Behavior & Acceptance Criteria
- [ ] Tratar erros de conexao e proxy de forma case-insensitive em src/ai.rs.
- [ ] Retornar mensagem amigavel 'Failed to connect to Ollama' mesmo sob proxy HTTP (502).
- [ ] Resguardar provedor local contra falhas similares.
- [ ] Validar suite de testes em tests/ai_test.rs.
- [ ] Desbloquear a esteira de CI e despacho multiarch para a Banca.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch fix/86-ai-connect-proxy-error criada
- [ ] Fase 3 - Doc/Spec: Especificacao tecnica detalhada
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementacao minima funcional com teste passando
- [ ] Fase 6 - Refactor: Codigo limpo, idiomatico e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes criticos eliminados com assercoes robustas
- [ ] Fase 8 - Quality Gate: Linter -D warnings e testes validados
- [ ] Fase 9 - Close: Changelog e documentacao atualizados; commit com (closes #86)

---
