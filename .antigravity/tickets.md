# Active Tickets - cccp
> Synced at: 2026-10-28 15:28:44 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#81] feat(repo): subcomando cccp repo top para monitoramento interativo do ci
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/81

### Description
## Description
Implementar o subcomando cccp repo top no estilo top/TUI para monitorar interativamente em tempo real o estado de CI de todos os repositorios do catalogo/workspace.

## Expected Behavior & Acceptance Criteria
- [ ] Listar o estado de CI dos repositorios com suporte a filtros por estado: ok, falha, na fila (incluindo em execucao) e sem fila.
- [ ] Suportar ordenacao configuravel/interativa:
  - Ordem de processamento (padrao inicial, abrangendo todos os estados)
  - Por estado de CI
  - Alfabetica por nome de repositorio
- [ ] Limitar a visualizacao a altura do terminal com paginacao navegavel.
- [ ] Atualizacao automatica da listagem a cada 10 segundos.
- [ ] Bottom bar com instrucao de ajuda: Ctrl+H help.
- [ ] Comandos interativos de controle precedidos por Ctrl.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificacao (Zero codigo de producao)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementacao minima funcional com teste passando
- [ ] Fase 6 - Refactor: Codigo limpo, idiomatico e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes criticos eliminados com assercoes robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatacao validados
- [ ] Fase 9 - Close: Changelog e documentacao atualizados; commit com (closes #<id>)

---
## [#80] fix(repo): remover repositorios espelho da listagem em cccp repo list
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/80

### Description
## Description
Remover repositorios espelhos da listagem gerada por cccp repo list para exibir unicamente os repositorios canonicos e ativos do workspace/catalogo, evitando poluicao visual e redundancia de espelhos secundarios.

## Expected Behavior & Acceptance Criteria
- [ ] Ocultar ou filtrar espelhos secundarios (mirrors) por padrao na saida de cccp repo list.
- [ ] Exibir apenas os repositorios canonicos e primarios configurados no catalogo.
- [ ] Manter integridade de comandos dependentes do catalogo.

## Tasks / DD-TDD Checklist (9 Fases GOST)
- [ ] Fase 1 - Ticket: Issue formalizada e escopo delimitado
- [ ] Fase 2 - Branch: Branch `<tipo>/<id>-<slug>` criada a partir de develop/main
- [ ] Fase 3 - Doc/Spec: Contrato preliminar e especificacao (Zero codigo de producao)
- [ ] Fase 4 - Red: Teste automatizado falho comprovando o requisito
- [ ] Fase 5 - Green: Implementacao minima funcional com teste passando
- [ ] Fase 6 - Refactor: Codigo limpo, idiomatico e com Zero Warnings
- [ ] Fase 7 - Mutation: Mutantes criticos eliminados com assercoes robustas
- [ ] Fase 8 - Quality Gate: Cobertura >= 90%, linter -D warnings e formatacao validados
- [ ] Fase 9 - Close: Changelog e documentacao atualizados; commit com (closes #<id>)

---
## [#79] feat(ai): precedencia de CCCP_GEMINI_TOKEN sobre GEMINI_TOKEN
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/79

### Description
Permitir a configuracao de token especifico do Gemini para o CCCP via variavel de ambiente CCCP_GEMINI_TOKEN (e CCCP_GEMINI_API_KEY), garantindo precedencia sobre GEMINI_TOKEN, GEMINI_API_KEY e GOOGLE_API_KEY.

Motivacao:
- Isolar chaves de API e cotas entre aplicacoes sem colisao de variaveis globais.
- Facilitar a utilizacao de tokens dedicados para o bot Divan e geracao de tickets/triage.

Ordem de precedencia proposta:
1. CCCP_GEMINI_TOKEN / CCCP_GEMINI_API_KEY
2. GEMINI_TOKEN / GEMINI_API_KEY
3. GOOGLE_API_KEY

---
## [#76] feat(ci): subcomandos de esteira cccp ci publish-artifacts e cccp ci deploy-site
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/76

### Description
Implementar subcomandos de automação executados na esteira hermética (.forgejo/workflows/ci.yml) conforme Seção 10 da GOST 2.0 e Seção 5 de Kollontai 2.0.

Requisitos:
- cccp ci publish-artifacts: compilação e despacho determinístico de binários e manifestos para o Cloudflare R2 / Banca (banca.2lp.in).
- cccp ci deploy-site: empacotamento da vitrine estática e deploy em worktree órfã stateless pages para os espelhos secundários sem inflação de histórico.

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
