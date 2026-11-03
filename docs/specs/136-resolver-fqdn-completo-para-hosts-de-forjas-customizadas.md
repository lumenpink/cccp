# Especificação Técnica: Ticket #136
## fix(catalog): resolver fqdn completo para hosts de forjas customizadas

### 1. Contexto e Problema
Após a migração para a exibição de nomes de forja (`gambi` em vez de `forgejo`), identificou-se uma quebra na resolução de hosts da API:
1. **Omissão de Host no Registro**:
   - `scan_and_add_from_src` obtinha `info.host` (`gambi.2lp.in`), mas gravava no catálogo apenas `forge: "gambi"`, deixando `server_host` e `canonical_url` vazios.
2. **Síntese Incorreta de URLs**:
   - `enrich_catalog_entry` sintetizava `canonical_url` como `https://gambi/<slug>` e `server_host` como `"gambi"`.
3. **Falha na Descoberta de Hosts**:
   - `to_forge_info()` e `discover_forgejo_hosts()` utilizavam o host não-qualificado `"gambi"`.
   - Como consequência, o monitor (`repo top`) tentava consultar `https://gambi/api/v1/user/actions/runners/jobs`, falhando na resolução de DNS e impossibilitando a exibição em tempo real de jobs `running` e `waiting`.

### 2. Requisitos Técnicos
1. **Preservação de FQDN em `scan_and_add_from_src` (`src/catalog.rs`)**:
   - Preencher explicitamente `server_host: Some(info.host.clone())` e `canonical_url: Some(format!("https://{}/{}", info.host, slug))` no momento da criação da entrada do catálogo.
2. **Resolução Robusta em `enrich_catalog_entry` (`src/catalog.rs`)**:
   - Mapear `"gambi"` e hosts curtos sem ponto para seu domínio FQDN correspondente (`"gambi.2lp.in"`).
   - Se `canonical_url` ou `server_host` contiverem apenas `"gambi"` ou um host sem ponto, normalizar para o FQDN.
3. **Garantia de FQDN em `to_forge_info` (`src/catalog.rs`)**:
   - Assegurar que `to_forge_info` retorne `info.host` sempre qualificado (ex.: `"gambi.2lp.in"`).
4. **Descoberta Confiável em `discover_forgejo_hosts` (`src/forges.rs`)**:
   - Garantir que todos os hosts retornados por `discover_forgejo_hosts` contenham domínios válidos para chamadas HTTP.

### 3. Critérios de Aceitação
- Entradas adicionadas via `scan_and_add_from_src` possuem `server_host == Some("gambi.2lp.in")`.
- `to_forge_info` em entradas com `forge: "gambi"` retorna `info.host == "gambi.2lp.in"`.
- `discover_forgejo_hosts` retorna `gambi.2lp.in` e nunca `gambi`.
- Testes unitários herméticos garantem todas as regras de resolução.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` aprovados.

### 4. Laudo de Implementação e Verificação
- **Fase 4 (Red)**: Testes `test_discover_forgejo_hosts_with_short_forge_name_gambi` e `test_enrich_catalog_entry_resolves_gambi_fqdn` criados em `tests/ci_consolidated_runner_jobs_test.rs`, falhando comprovando a ausência de qualificação FQDN.
- **Fase 5 (Green)**: Implementada normalização de FQDN em `enrich_catalog_entry`, preservação em `scan_and_add_from_src` e fallback robusto em `to_forge_info` em `src/catalog.rs`. Testes passando.
- **Fase 6 (Refactor)**: Simplificação de fluxos e validação de clippy com `is_none_or`.
- **Fase 7 (Mutation)**: Adicionado teste `test_mutation_catalog_fqdn_resolution_edge_cases` cobrindo cenários com URLs HTTP, SSH e forjas com e sem domínio explícito.
- **Fase 8 (Quality Gate)**: `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` executados com sucesso total (0 erros, 0 advertências). Testes direcionados com 100% de aprovação.

