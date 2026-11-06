# Especificação Técnica — Ticket #141: Normalizar Esquemas HTTP e Portas em `enrich_catalog_entry`

## 1. Escopo e Motivação
A esteira de integração contínua (CI) falhou na execução de `tests/catalog_metadata_aliases_test.rs` após a introdução de testes para extração de domínios e aliases automáticos. Quando um repositório no catálogo possui o campo `forge` preenchido com URL completa (ex.: `"https://gambi.2lp.in"` ou `"http://git.local.internal:8080"`), a função `enrich_catalog_entry`:
1. Concatenava `https://` em cima de `base_host`, resultando em `"https://https://gambi.2lp.in/..."`.
2. Não limpava a porta ou o esquema antes de derivar o `server_host`, gerando nomes de host inválidos (ex.: `"http.2lp.in"`) e aliases automáticos errôneos (ex.: `["https:cccp", "https/cccp"]`).

O objetivo desta especificação é definir a extração canônica de host para forjas e garantir que `enrich_catalog_entry` produza URLs e aliases válidos para qualquer variante de entrada.

---

## 2. Requisitos e Contratos

### 2.1 Extração e Normalização de Host (`clean_forge_host`)
- Qualquer entrada em `entry.forge` ou `entry.canonical_url` deve ter seus esquemas removidos (`https://`, `http://`, `git@`).
- Caminhos residuais (`/slug...`) e portas (`:8080`) devem ser descartados ao determinar o host.
- Espaços em branco devem ser removidos (`trim`).

### 2.2 Resolução de FQDN
- Aliases curtos conhecidos continuam mapeados:
  - `github` -> `github.com`
  - `gitlab` -> `gitlab.com`
  - `codeberg` / `codeberg.org` -> `codeberg.org`
  - `tildegit` / `tildegit.org` -> `tildegit.org`
  - `gambi` -> `gambi.2lp.in`
- Se o host limpo contiver ponto (`.`), é considerado FQDN direto (ex.: `gambi.2lp.in`, `git.subversao.su`, `git.local.internal`).
- Se não contiver ponto e não estiver vazio, o sufixo padrão `.2lp.in` é aplicado.
- Se vazio, assume o padrão `gambi.2lp.in`.

### 2.3 `canonical_url`
- Deve ser formatado sempre como `https://{base_host}/{entry.slug}` sem protocolos repetidos.

### 2.4 `server_host` e Aliases
- `entry.server_host` deve ser exatamente o FQDN resolvido sem porta e sem esquema.
- A derivação de aliases automáticos via `extract_domain_prefix(host)` deve operar sobre o FQDN limpo (ex.: `gambi.2lp.in` -> prefixo `gambi` -> `gambi:cccp`).

---

## 3. Plano de Testes (TDD)
1. **Red Test**:
   - Executar `tests/catalog_metadata_aliases_test.rs` e registrar as falhas existentes.
2. **Green Implement**:
   - Implementar a normalização e certificar passagem de 100% dos testes.
3. **Refactor & Mutation**:
   - Adicionar testes de mutação cobrindo portas, esquemas misturados e múltiplos casos de borda.
4. **Quality Gate**:
   - `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, e suite direcionada de testes.

---

## 4. Relatório de Conclusão (Fase 9)
*(A ser preenchido ao término da implementação com [Resumo], [Solução] e [Validação])*
