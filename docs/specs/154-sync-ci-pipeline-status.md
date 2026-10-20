# Especificação Técnica: Sincronização do Status da Última Execução de CI de Todas as Esteiras

**Ticket Associado:** #154  
**Branch:** `feat/154-sync-ci-pipeline-status`  
**Norma:** GOST 1.4.0  

## 1. Objetivo
Implementar a coleta e sincronização do status da execução mais recente de todas as esteiras e fluxos de CI configurados no repositório/forja durante a rotina de sincronização (`sync` / `tickets`), persistindo os metadados de forma atômica no modelo do catálogo (`CatalogEntry`) e no cache local do projeto (`.antigravity/ci_status.json`), sem interromper o fluxo em caso de forjas sem CI ou falhas transitórias.

## 2. Requisitos Funcionais e Critérios de Aceitação

1. **Modelo de Dados de CI (`src/catalog.rs`):**
   - Estrutura `CiPipelineStatus`:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
     pub struct CiPipelineStatus {
         pub name: String,
         pub status: String,
         pub description: Option<String>,
         pub target_url: Option<String>,
         pub updated_at: Option<String>,
     }
     ```
   - Adicionar o campo `ci_status: Vec<CiPipelineStatus>` em `CatalogEntry` e `RawCatalogEntry` (com serde skip se vazio para manter JSON limpo).

2. **Consulta na API de Forges (`src/forges.rs`):**
   - Método no `ForgeClient`:
     ```rust
     pub fn fetch_ci_status(&self, info: &RepoForgeInfo, git_ref: Option<&str>) -> Result<Vec<CiPipelineStatus>, String>;
     ```
   - **Codeberg / Forgejo / Gitea**: Consulta `GET /api/v1/repos/{owner}/{repo}/commits/{ref}/statuses` (padrão ref: `"main"`).
     - Desduplica e preserva o status mais recente para cada `context` (nome da esteira).
   - **GitHub**: Consulta `GET /repos/{owner}/{repo}/commits/{ref}/statuses`.
   - **GitLab**: Consulta correspondente de pipelines/jobs.
   - Autenticação transparente com token resolvido via `resolve_token_for_host`.

3. **Integração com Rotina de Sync (`src/tickets.rs` e `src/catalog.rs`):**
   - Durante `cccp ticket` ou `cccp sync`:
     - Consulta os status de CI das esteiras na forja.
     - Persiste em `.antigravity/ci_status.json` no repositório local.
     - Atualiza o registro correspondente no `catalog.json` caso o repositório esteja registrado.
   - Em `update_catalog_tickets` (`cccp catalog sync`):
     - Atualiza o campo `ci_status` de cada repositório sincronizado.

4. **Resiliência e Degradação Graciosa:**
   - Se a forja não tiver esteiras configuradas, retornar lista vazia `[]`.
   - Se houver falha de rede ou timeout na consulta de CI, manter os status previamente cacheados sem interromper a sincronização de tickets nem o restante da rotina.
