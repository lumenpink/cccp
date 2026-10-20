# Active Tickets - cccp
> Synced at: 2026-10-20 00:46:48 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#156] refactor(cli): padronizar comandos, remover aliases e unificar fluxo de tickets e sync
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/156

### Description
## Description
Revisar e consolidar todos os comandos da CLI (`cccp`), eliminando inconsistências na experiência do usuário.
- Remover aliases desnecessários ou concorrentes (ex: `audit|gost`).
- Resolver ambiguidade entre `tickets` e `sync`: atualmente `tickets` apenas sincroniza e não gera tickets, criando redundância confusa com o comando `sync`.
- Uniformizar a nomenclatura, documentação embutida (`--help`) e assinaturas dos subcomandos.

## Expected Behavior & Acceptance Criteria
- [ ] Todos os aliases secundários (ex: `audit|gost`) são removidos, mantendo apenas comandos canônicos bem definidos.
- [ ] O conflito entre `tickets` e `sync` é resolvido (definição clara de responsabilidade de cada um ou unificação/depreciação de um deles).
- [ ] O output de `cccp --help` e os subcomandos refletem uma estrutura limpa e sem duplicidades.
- [ ] Testes de CLI cobrem todos os comandos canônicos e garantem a remoção dos aliases obsoletos.

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
## [#155] feat(ui): exibir status da última execução de CI no catálogo
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/155

### Description
## Description
Exibir no catálogo o status da última execução das esteiras de CI sincronizadas para cada projeto/repositório.

## Expected Behavior & Acceptance Criteria
- [ ] Os cards ou itens do catálogo devem conter indicadores visuais de status (ex: sucesso, falha, pendente, em execução) para as esteiras de CI.
- [ ] Possibilidade de visualizar detalhes ou tooltip com a data/hora e o nome de cada esteira avaliada.
- [ ] Manter consistência visual com o tema e padrões de layout existentes.

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
## [#154] feat(core): sincronizar status da última execução de CI de todas as esteiras
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/154

### Description
## Description
Implementar a coleta e sincronização da última execução de CI ( abrangendo todas as esteiras configuradas no repositório/forge) durante a rotina de sync de tickets.

## Expected Behavior & Acceptance Criteria
- [ ] A rotina de sync de tickets deve acionar a verificação das execuções de CI na API do Codeberg/forge.
- [ ] Coletar e persistir o status mais recente de cada esteira/pipeline de CI associada.
- [ ] Tratar cenários onde não há execuções de CI ou a API retorne erros transitórios sem interromper o sync dos tickets.

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
