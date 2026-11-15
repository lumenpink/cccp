# Active Tickets - cccp
> Synced at: 2026-11-15 11:18:09 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#177] chore(profile): migrar cccp para gost v3 e perfil lilya 2.1.0
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/177

### Description
## Description
Migrar a governança do repositório cccp da GOST v2.0.0 para a Metanorma GOST v3.0.0 e Perfil Lilya v2.1.0, eliminando manifestos legados (GOSTVER), adotando .rules/profile.toml, compilando .gost/GOST.md e registrando o laudo em .gost/UPGRADE.md.

## Expected Behavior & Acceptance Criteria
- Execução do upgrade de perfil via comando canônico cccp upgrade 2.1.0.
- Criação de .rules/profile.toml apontando para Lilya v2.1.0 e .gost/UPGRADE.md.
- Purga completa do arquivo legado GOSTVER.
- Compilação do .gost/GOST.md com regras vigentes de Lilya 2.1.0.
- Auditoria estrita com cccp check --strict aprovada.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch criada a partir de main
- [ ] Fase 3 - Doc/Spec: Contrato de migração e documentação de transição
- [ ] Fase 4 - Red: Verificação falha comprovando ausência da baseline v3
- [ ] Fase 5 - Green: Execução do upgrade, remoção do GOSTVER e compilação do GOST.md
- [ ] Fase 6 - Refactor: Limpeza e alinhamento de scripts/testes locais
- [ ] Fase 7 - Mutation: Verificação de consistência e ausência de resíduos
- [ ] Fase 8 - Quality Gate: Execução de fmt, clippy e testes rápidos
- [ ] Fase 9 - Close: Atualização do CHANGELOG e laudo de fechamento

---
