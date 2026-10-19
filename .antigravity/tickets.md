# Active Tickets - cccp
> Synced at: 2026-10-19 10:14:22 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#153] feat(core): suporte para remover itens do catálogo com remoção opcional de diretório limpo
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/153

### Description
## Description
Implementar suporte à remoção de itens registrados no catálogo. A operação deve permitir opcionalmente remover o diretório correspondente do sistema de arquivos, contanto que o diretório esteja em estado limpo (clean / sem alterações não commitadas ou arquivos não rastreados), prevenindo perda acidental de dados.

## Expected Behavior & Acceptance Criteria
- [ ] Possibilidade de remover um item específico do catálogo.
- [ ] Opção (flag/parâmetro) para solicitar a deleção do diretório físico correspondente.
- [ ] Validação estrita de segurança: o diretório só deve ser deletado se estiver estritamente limpo (sem modificações locais não salvas/pendentes).
- [ ] Lançamento de erro claro e interrupção segura caso a deleção do diretório seja requisitada mas o estado não esteja limpo (abortando sem apagar o diretório).
- [ ] Manter consistência do catálogo caso ocorra falha durante o processo.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#152] feat(core): suporte para remover itens do catálogo com remoção opcional de diretório limpo
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/152

### Description
## Description
Implementar suporte à remoção de itens registrados no catálogo. A operação deve permitir opcionalmente remover o diretório correspondente do sistema de arquivos, contanto que o diretório esteja em estado limpo (clean / sem alterações não commitadas ou arquivos não rastreados), prevenindo perda acidental de dados.

## Expected Behavior & Acceptance Criteria
- [ ] Possibilidade de remover um item específico do catálogo.
- [ ] Opção (flag/parâmetro) para solicitar a deleção do diretório físico correspondente.
- [ ] Validação estrita de segurança: o diretório só deve ser deletado se estiver estritamente limpo (sem modificações locais não salvas/pendentes).
- [ ] Lançamento de erro claro e interrupção segura caso a deleção do diretório seja requisitada mas o estado não esteja limpo (abortando sem apagar o diretório).
- [ ] Manter consistência do catálogo caso ocorra falha durante o processo.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
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
