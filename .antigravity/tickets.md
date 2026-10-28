# Active Tickets - cccp
> Synced at: 2026-10-28 01:06:54 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#78] fix(dx): gplv3 e ci no init, merge sem emenda ff e chores isentos no check
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/78

### Description
Ajustar cccp init para licenca padrao GPL-3.0 e CI ubuntu-latest limpo, eliminar emenda de chore desnecessaria em merges fast-forward e isentar commits chore da exigencia de ticket em cccp check --strict.

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
## [#75] feat(site): gerador deterministico cccp site banner a partir de gost metadata
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/75

### Description
Implementar a geração automatizada do banner minimalista de documentação conforme Regras GOST-20, GOST-26 e Seção 6.2 de Kollontai 2.0.

Requisitos:
- Subcomando cccp site banner para gerar deterministicamente site/assets/og_preview.png (1200x630, Safe Zone 4:3 centralizada) a partir de .gost/metadata.json.
- Renderizar fundo com theme.bg_color, acentos com theme.accent_color, ícone oficial e texto {title} — {tagline} (<= 65 caracteres), sem dependências externas de browser headless.

---
## [#74] feat(governance): subcomandos cccp epic para branches persistentes e cccp inbox para quarentena de prs
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/74

### Description
Implementar a governança de branches persistentes e quarentena de fronteira conforme Seção 15 (Regra GOST-25) e Seção 1.2 de Kollontai 2.0.

Requisitos:
- cccp epic start|finish <slug>: ciclo de vida e checkout de branches persistentes de integração epic/<slug>.
- cccp inbox list|review|accept: quarentena de PRs externos em refs/heads/inbox/* isolados contra push acidental nos espelhos secundários (pushurl = no_push).

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
