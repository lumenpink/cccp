# Active Tickets - cccp
> Synced at: 2026-10-26 15:06:38 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#62] fix(deps): habilitar vendored-openssl no git2 para build musl
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/62

### Description
Habilitar o feature vendored-openssl na dependencia git2 para permitir a compilacao estatica para x86_64-unknown-linux-musl na esteira de release.

---
## [#57] refactor(update): descontinuar auto-update interno em favor da jornaleira
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/57

### Description
Remover rotina interna de verificacao periodica de auto-update (check_auto_update) no CCCP, transferindo a gestao de ciclo de vida e atualizacoes dos binarios para o gestor central Jornaleira.

---
## [#54] refactor(core): remove hardcoded codeberg references in favor of git.2lp.in forge
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/54

### Description
## Description
O projeto migrou seu forge primário do Codeberg para o endereço local `git.2lp.in` (Forgejo). Existem referências residuais hardcoded a `codeberg.org` no código-fonte, templates e defaults de resolução de forge que precisam ser mapeadas e substituídas pela configuração dinâmica da forge ou pelo novo host padrão `git.2lp.in`.

## Expected Behavior & Acceptance Criteria
- [ ] Nenhuma URL ou string fixa apontando para `codeberg.org` permanece como fallback rígido no código-fonte.
- [ ] A resolução de forges respeita a configuração do usuário ou utiliza `git.2lp.in` como destino Forgejo padrão.
- [ ] Testes de regressão garantem que detecção e montagem de URLs de clone, API e release funcionem corretamente com `git.2lp.in`.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `refactor/<id>-remove-codeberg-references` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#53] fix(core): ensure profile assemble updates gost and gostver across repositories
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/53

### Description
## Description
Durante a execução do comando de montagem (`profile assemble`), os repositórios gerenciados não estão tendo os artefatos GOST ou o valor `gostver` atualizados conforme esperado. O fluxo deve verificar, baixar/sincronizar os recursos GOST e garantir a gravação e consistência da versão GOST em todos os repositórios processados.

## Expected Behavior & Acceptance Criteria
- [ ] O comando `profile assemble` itera sobre todos os repositórios configurados e atualiza a especificação GOST.
- [ ] A versão canônica do GOST (`gostver`) é atualizada e registrada nos arquivos de metadados de cada repositório.
- [ ] A sincronização trata falhas parciais informando erros sem corromper repositórios íntegros.
- [ ] Testes de unidade e integração validam a atualização de `gost` e `gostver` em múltiplos diretórios.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `fix/<id>-assemble-gost-update` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificação (Zero código de produção)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementação mínima funcional com teste passando
- [ ] Fase 6 - Refactor: Código limpo, idiomático e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes críticos eliminados com asserções robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatação validados
- [ ] Fase 9 - Close: Changelog e documentação atualizados; commit com (closes #<id>)

---
## [#1] feat(ticket): Radar multi-forge para sincronização de tickets e PRs (GitHub, Codeberg, Forgejo)
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/1

### Description
### Proposta e Contexto

Com a migração do ecossistema para a instância soberana `git.2lp.in` como upstream canônico e os repositórios públicos operando com **Push Mirrors** para o **GitHub** e **Codeberg**, surge a necessidade de gerenciar o fluxo de contribuições externas (Issues e Pull Requests).

### Requisitos / Escopo
1. Estender o comando `cccp ticket sync` e `cccp forge` para inspecionar múltiplos remotes (`origin`, `github`, `codeberg`).
2. Sincronizar issues e PRs abertos nos espelhos externos diretamente no arquivo local `.antigravity/tickets.md`.
3. Oferecer comandos auxiliares para revisão ágil de PRs externos (ex.: checkout de branches de PRs do GitHub/Codeberg para testes locais no padrão GOST).
4. Suportar credenciais automáticas a partir do `.env` e variáveis de ambiente canônicas (`GITHUB_COM_TOKEN`, `CODEBERG_ORG_TOKEN`, `FORGEJO_TOKEN`).

### Alinhamento
- Norma GOST DD-TDD
- Perfil Kollontai / Lumen

---
