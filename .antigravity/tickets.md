# Active Tickets - cccp
> Synced at: 2026-10-28 09:50:33 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

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
