# Active Tickets - cccp
> Synced at: 2026-10-19 18:44:26 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

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
## [#156] feat(ci): compilação soberana na esteira, pre-tag act e distribuição via banca
- **Status**: `open`
- **Labels**: `ci`, `gost`, `distribution`
- **Link**: https://codeberg.org/lumenpink/cccp/issues/156

### Description
## Description
Implementar o Princípio da Compilação Soberana Exclusiva na Esteira, vinculada às versões atômicas por commit. Adequar a especificação GOST (`.gost/GOST.md`), remover a execução do `act` por padrão no hook `pre-push` (tornando o git push instantâneo e leve), migrar o gate de validação com `act` exclusivamente para o comando `cccp tag` (pré-tag release), e reconfigurar o módulo de auto-atualização (`cccp update`) e o workflow de release para publicar e consumir binários da Banca (`https://banca.2lp.in/downloads/cccp/`), eliminando qualquer dependência do Codeberg para hospedagem de artefatos.

## Expected Behavior & Acceptance Criteria
- [ ] Atualização normativa no `.gost/GOST.md` estipulando a proibição de compilação pesada local e delegação integral de releases para a esteira.
- [ ] Hook `pre-push` (`hook_runner.rs`) atualizado para não disparar o `act` por padrão, mantendo apenas verificações leves de tags e VERSION.
- [ ] Comando `cccp tag` (`release.rs`) atualizado para executar o gate de CI com `act` antes de selar a tag estável oficial.
- [ ] Módulo `update.rs` reconfigurado para consultar `https://banca.2lp.in/downloads/cccp/index.json` (ou `/version`) e baixar artefatos de `stable-latest` / `nightly-latest`, sem URLs de fallback do Codeberg.
- [ ] Workflow `.forgejo/workflows/release.yaml` ajustado para entregar pacotes no storage da Banca em vez de `action-gh-release`.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `feat/156-ci-banca-distribution` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #156)

---
