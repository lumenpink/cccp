# Especificação Técnica: Ticket #127
## feat(catalog): exibir nome da forja ao inves do software na fila e listagem

### 1. Contexto e Problema
Nas visualizações do catálogo (`cccp repo list` e `cccp repo top`), a coluna de identificação de forja (`FORGE`) exibia o nome genérico do software (`forgejo`) para repositórios hospedados em instâncias Forgejo (como `gambi.2lp.in`), em vez de exibir o nome ou host da forja correspondente (ex.: `gambi`).
Isso gerava confusão visual: enquanto repositórios no Codeberg, GitHub e Tildegit exibiam seus respectivos nomes (`codeberg`, `github`, `tildegit`), repositórios na forja interna `gambi` exibiam o software (`forgejo`).

### 2. Requisitos Técnicos
1. **Identificação de Nome de Forja Dinâmico (`src/forges.rs`)**:
   - Atualizar `RepoForgeInfo::forge_name(&self)` para que, no caso de instâncias `ForgeType::Forgejo`, extraia o nome/prefixo da forja a partir do host (ex.: `extract_domain_prefix(&self.host)` resultando em `"gambi"` para `gambi.2lp.in`).
   - Manter compatibilidade com `codeberg`, `github`, `gitlab` e `tildegit`.
2. **Resolução de Forja em Catálogos Existentes (`src/catalog.rs`)**:
   - Em `summarize_catalog`, quando o campo `entry.forge` for `"forgejo"`, resolver o nome real da forja a partir de `entry.server_host` ou `entry.canonical_url` utilizando `extract_domain_prefix`.
   - Assegurar que `cccp repo list` e `cccp repo top` exibam `gambi` para repositórios do host `gambi.2lp.in`.

### 3. Critérios de Aceitação
- Testes unitários validam que `RepoForgeInfo::forge_name` retorna `"gambi"` para `gambi.2lp.in`.
- Testes unitários validam que `summarize_catalog` converte entradas legadas com `"forge": "forgejo"` para o nome da forja real derivada de `server_host`.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou erros.
