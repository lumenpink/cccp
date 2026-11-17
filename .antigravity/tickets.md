# Active Tickets - cccp
> Synced at: 2026-11-17 12:26:48 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#179] feat(package): upload de artefatos para staging na forja
- **Status**: `open`
- **Labels**: none
- **Link**: https://gambi.2lp.in/lumen/cccp/issues/179

### Description
### 1. Contexto e Motivação
Para evitar scripts manuais com curl e tokens expostos nos workflows de CI da Gambi, o CCCP deve fornecer um subcomando nativo para publicar arquivos ou diretórios compactados no Generic Package Registry da forja remota.

### 2. Escopo
- Subcomando cccp package upload <caminho> [--app <nome>] [--version <ver>].
- Autodetecção de owner, app e forja a partir do git remote origin.
- Autodetecção de versão via VERSION / manifesto.
- Caso o alvo seja um diretório (ex: site/), compacta automaticamente em site.tar.gz.
- Upload via PUT autenticado para /api/packages/{owner}/generic/{app}/{version}/{filename}.

### 3. Critérios de Aceite
- Upload de arquivos avulsos e diretórios compactados com sucesso.
- Resolução limpa de tokens do ambiente (FORGEJO_TOKEN, GAMBI_2LP_IN_TOKEN).
- Testes automatizados e conformidade 100% com GOST v3 e Lilya v2.1.0.

---
