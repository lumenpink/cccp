# Especificação Técnica - Ticket #161: Consumir Catálogo Dinâmico do Gossnab com Cache Volátil

## 1. Contexto e Motivação
Historicamente, o `cccp` operava com um arquivo estático `catalog.json` replicado localmente na máquina do desenvolvedor ou na raiz de repositórios. Isso gerava problemas crônicos:
1. Concorrência e desatualização: múltiplos nós de desenvolvimento e pipelines de CI modificando ou sobrescrevendo o catálogo localmente.
2. Débito de sincronização: ausência de autoridade única sobre metadados, pipelines de CI e visibilidade de repositórios.

Com o advento do **Gossnab** como Single Source of Truth (SSOT) do ecossistema, o `cccp` passa a ser um cliente consumidor leve do endpoint dinâmico `GET /api/v1/catalog`.

## 2. Decisões Arquiteturais e Rastreabilidade (Para a Natasha)

### Por que Gossnab via HTTP em vez de git submodule ou arquivo estático?
* O Gossnab já recebe webhooks em tempo real e consolida estados de esteiras e registros de novos repositórios.
* Consultar via HTTP elimina o risco de colisões de escrita no arquivo local e isola o `cccp` de credenciais de administração da forja.

### Estratégia de Cache e Resiliência Offline
* Natasha precisa garantir que o comando `cccp` funcione perfeitamente em aviões, conexões instáveis ou ambientes isolados.
* Ordem de precedência:
  1. **Flag Offline**: Se a variável de ambiente `CCCP_OFFLINE=1` ou `CCCP_OFFLINE=true` estiver ativa, nenhuma chamada de rede é tentada; o sistema recorre imediatamente ao cache local.
  2. **Consulta HTTP ao Gossnab**:
     * URL resolvida através de `CCCP_GOSSNAB_URL` ou `GOSSNAB_URL`.
     * Requisição `GET {url}/api/v1/catalog` com timeout restrito (2 segundos) via `ureq`.
     * Se bem-sucedida, o payload JSON é salvo atomica/assincronamente no cache volátil do usuário: `$XDG_CACHE_HOME/cccp/catalog.json` (padrão: `~/.cache/cccp/catalog.json`).
  3. **Fallback Primário (Cache Local)**: Se a requisição falhar (timeout, DNS, 5xx), tenta ler de `$XDG_CACHE_HOME/cccp/catalog.json`.
  4. **Fallback Secundário (Legado)**: Se o cache não existir, recorre aos caminhos legados via `find_catalog_path` (`CATALOG_FILE`, `.config/cccp/catalog.json`, etc.).

## 3. Contrato de Interface e Tipos
* O payload do Gossnab é serializado em JSON com campos alinhados ao `CatalogEntry` do `cccp`.
* `load_catalog(config, start_dir)` mantém assinatura compatível, orquestrando a precedência descrita acima.
* Adicionada função pública `fetch_catalog_from_gossnab(gossnab_url: &str)` e `get_catalog_cache_path()`.

## 4. Plano de Fases (GOST DD-TDD)
- **Fase 3**: Especificação viva registrada neste documento.
- **Fase 4**: Teste vermelho em `tests/catalog_test.rs` simulando mock HTTP server com `std::net::TcpListener`.
- **Fase 5**: Implementação de `fetch_catalog_from_gossnab`, cache em `$XDG_CACHE_HOME/cccp/catalog.json` e fallback integrado em `load_catalog`.
- **Fase 7**: Testes de mutação cobrindo timeouts, respostas 500, JSON corrompido e flag `CCCP_OFFLINE`.
- **Fase 8**: Linters (`cargo fmt`, `cargo clippy`), testes de unidade e integração.
- **Fase 9**: Fechamento do ticket #161 com laudo canônico e integração na branch `main`.
