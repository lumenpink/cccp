# Especificação Técnica — Ticket #54: Migração de Referências Codeberg para Forge Soberana git.2lp.in

## 1. Visão Geral
O ecossistema migrou sua infraestrutura primária de forjas Git para a instância soberana `git.2lp.in` (Forgejo).
No entanto, diversas partes da base de código mantinham fallbacks estáticos para `codeberg.org`, incluindo a resolução padrão de `ForgeType::Forgejo`, fallbacks de slug simplificado (`owner/repo`), templates de `AGENTS.md`, especificação canônica do GOST e detecção de token.

Esta especificação define a substituição dos fallbacks rígidos de `codeberg.org` pela forja padrão `git.2lp.in` (e suporte dinâmico via configuração), preservando a compatibilidade com o Codeberg apenas como provedor explícito.

---

## 2. Requisitos Técnicos

### 2.1 Resolução de Host Padrão para Forjas (`src/forges.rs`)
- O host padrão para `ForgeType::Forgejo` deve ser `git.2lp.in` (em vez de `codeberg.org`):
  ```rust
  ForgeType::Forgejo => "git.2lp.in"
  ```
- Detecção e resolução de tokens:
  - Adicionar suporte nativo à variável `GIT_2LP_IN_TOKEN` e `FORGEJO_TOKEN` na lista prioritária de tokens para `git.2lp.in` e `ForgeType::Forgejo`.

### 2.2 Fallbacks de Slugs e Catálogo (`src/catalog.rs`)
- Ao resolver ou normalizar slugs no formato curto `owner/repo` sem host explícito:
  - O fallback deve utilizar `ForgeType::Forgejo` e host `git.2lp.in` como padrão do ecossistema soberano.
- Em `extract_domain_prefix`: manter suporte a hosts soberanos (`git.2lp.in`, `gambi.2lp.in`, etc.).

### 2.3 Templates e Documentos Canônicos
- Em `src/agents.rs` e `src/config.rs`:
  - Atualizar o template padrão de AGENTS.md para `https://git.2lp.in/lumen/cccp/raw/branch/main/src/base/AGENTS_template.md`.
- Em `src/profile.rs` e `src/newrepo.rs`:
  - Atualizar os fallbacks da norma GOST para `https://gost.2lp.in/SPEC.md` ou `https://git.2lp.in/lumen/gost`.

---

## 3. Critérios de Aceitação
1. `ForgeType::Forgejo.default_host()` retorna `git.2lp.in`.
2. Inferência de slug curto `owner/repo` no catálogo resulta em `ForgeType::Forgejo` apontando para `git.2lp.in`.
3. Bateria de testes em `tests/sovereign_forge_test.rs` comprovando a nova resolução e URLs geradas.
4. Conformidade total com `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.
