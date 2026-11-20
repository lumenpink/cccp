# Especificação do Ticket #188: Cessar Injeção em `composer.json` e Migrar CI do CCCP para Generic Packages

## 1. Contexto e Motivação
1. **Injeção indevida em `composer.json`:** `sync_project_manifests` em `src/semver_calc.rs` estava injetando `"version"` em `composer.json` com `inject_if_missing = true`. O Composer v2 rejeita versões `-dev.<N>+<meta>` na validação de esquema (`composer validate`) e recomenda oficialmente omitir `"version"` em projetos versionados por Git.
2. **Orquestração via Kombinat:** O workflow `.forgejo/workflows/ci.yml` do próprio `cccp` ainda executava uploads diretos para Cloudflare R2 (`wrangler r2`), commits diretos no repositório da Banca e deploy direto no Cloudflare Pages (`wrangler pages deploy`). Com o Kombinat operando a partir dos hooks da Gambi, a CI deve publicar apenas nos Generic Packages da forja.

## 2. Requisitos da Correção
1. Em `src/semver_calc.rs`:
   - Remover `composer.json` da sincronização automática de versão (`scan_manifests_recursive` e `sync_project_manifests`).
2. Em `tests/multi_manifest_sync_test.rs`:
   - Atualizar os testes para comprovar que `composer.json` nunca é modificado nem recebe injeção de `"version"`.
3. Em `.forgejo/workflows/ci.yml` e `.gost/delivery.toml`:
   - Remover steps diretos de Cloudflare R2, rotação manual no repo da Banca e Cloudflare Pages.
   - Publicar os artefatos empacotados (`dist/packages/*` e `site.tar.gz`) no Generic Package Registry da Gambi (`/api/packages/lumen/generic/cccp/${VERSION}/...`), mantendo o alias `nightly/cccp-linux-amd64` para as imagens de `gost-ci`.
   - Declarar `[binaries]` e `[sites]` em `.gost/delivery.toml`.

## 3. Árvore de Decisões (para a Natasha)
- **Por que nunca sincronizar `composer.json`:** A especificação do Composer v2 impõe uma regex incompatível com SemVer de canal dev (`-dev.N`) e instrui que projetos sob VCS derivem a versão de tags/branches. Tentar traduzir ou injetar `"version"` cria drift entre `VERSION` e `composer.lock`, quebrando `composer validate`.
