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
