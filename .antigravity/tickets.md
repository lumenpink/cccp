# Active Tickets - cccp
> Synced at: 2026-10-15 01:30:05 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#129] feat(cli): adicionar filtros na listagem para repositórios dirty e com push pendente
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/129

### Description
## Description
Adicionar opções de filtragem na CLI para inspecionar repositórios de acordo com seu estado local de git, permitindo filtrar exclusivamente os repositórios dirty (com alterações locais) ou exclusivamente os limpos com commits pendentes de envio ao remote.

## Expected Behavior & Acceptance Criteria
- [ ] Suportar flag (ex: `--dirty`) para listar unicamente repositórios que possuem modificações staged ou unstaged no working tree.
- [ ] Suportar flag (ex: `--unpushed` ou `--pending-push`) para listar unicamente repositórios limpos com commits locais pendentes de push.
- [ ] Exibir saída estruturada e legível informando contagem de arquivos modificados ou commits ahead correspondentes.
- [ ] Suportar combinação coerente ou validação de exclusão mútua das flags de filtro.

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
## [#127] feat(cli): atualizar contagem de tickets abertos de repositórios no catálogo
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/127

### Description
## Description
Necessidade de atualizar a contagem de tickets abertos armazenada nos metadados dos repositórios do catálogo. O comando deve permitir atualizar todos os repositórios ou apenas itens selecionados, integrando com o comando/fluxo existente de tickets quando aplicável.

## Expected Behavior & Acceptance Criteria
- [ ] Permitir atualizar a contagem de tickets de todos os repositórios cadastrados via comando CLI (ex: `catalog update-tickets --all` ou extensão do comando `ticket`).
- [ ] Permitir especificar repositórios individuais como argumentos para atualização pontual.
- [ ] Tratar falhas de conectividade ou autenticação na API do forge de maneira graciosa sem interromper todo o lote.
- [ ] Refletir e persistir a contagem atualizada no catálogo.

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
## [#126] feat(cli): implementar remoção de itens do catálogo
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/126

### Description
## Description
Atualmente não há comando ou mecanismo direto na CLI para remover repositórios ou entradas já cadastradas no catálogo. É necessário implementar um comando (ex: `catalog remove` ou `catalog rm`) que desvincule/remova itens por identificador, caminho ou nome.

## Expected Behavior & Acceptance Criteria
- [ ] Disponibilizar comando na CLI para remover itens do catálogo passando identificador ou caminho do repositório.
- [ ] Suportar flag ou confirmação interativa antes da exclusão permanente da entrada no catálogo.
- [ ] Retornar erro explicativo caso o item solicitado para remoção não exista no catálogo.
- [ ] Atualizar o arquivo/base de dados do catálogo persistindo as alterações.

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
