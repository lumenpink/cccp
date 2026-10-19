# Especificação Técnica: Sincronização Direta da Contagem de Tickets no Catálogo via API Remota

**Ticket Associado:** #151  
**Branch:** `feat/151-catalog-sync-tickets`  
**Norma:** GOST 1.4.0  

## 1. Objetivo
Permitir a auditoria e sincronização em lote da contagem de tickets abertos de todos os repositórios cadastrados no Catálogo do Estado Gosplan (`catalog.json`), consultando diretamente as APIs REST das forjas remotas (Codeberg, Forgejo, Gitea, GitHub, GitLab) através do cliente HTTP nativo (`ForgeClient`), **sem nunca depender de utilitários locais** (`tea`, `gh`, `glab`) e independentemente de haver clone local dos repositórios no disco.

## 2. Requisitos Funcionais e Critérios de Aceitação

1. **Consulta Leve via API REST (`src/forges.rs`):**
   - Implementar método no `ForgeClient`:
     ```rust
     pub fn fetch_open_issues_count(&self, info: &RepoForgeInfo) -> Result<usize, String>;
     ```
   - Consultar o endpoint de metadados das forjas suportadas:
     - **Codeberg / Forgejo / Gitea**: `GET https://{host}/api/v1/repos/{owner}/{repo}` $\rightarrow$ campo `"open_issues_count"`.
     - **GitHub**: `GET https://api.github.com/repos/{owner}/{repo}` $\rightarrow$ campo `"open_issues_count"`.
     - **GitLab**: `GET https://{host}/api/v4/projects/{owner}%2F{repo}` $\rightarrow$ campo `"open_issues_count"`.
   - Adicionar autenticação com token resolvido via `resolve_token_for_host` (suporte a variáveis de ambiente e arquivos `.cccprc` / `.env`).
   - Fallback leve: caso o endpoint de metadados não retorne a contagem, consultar `/issues?state=open&limit=1` ou a listagem padrão de issues.

2. **Comando e Parâmetros CLI (`src/cli.rs`):**
   - No subcomando `CatalogSubcommands::UpdateTickets`:
     - Adicionar o alias `"sync"`, além do existente `"sync-tickets"`.
     - `cccp catalog sync` (sem argumentos) deve sincronizar todos os repositórios (`all: true`) por padrão quando invocado, ou quando `--all` / `-a` for explicitamente passado.
     - `cccp catalog sync [REPOS...]` permite sincronizar um ou mais repositórios especificados por nome, slug ou alias.
   - No comando geral `cccp sync`:
     - Suportar flag ou opção para incluir sincronização de tickets do catálogo quando solicitado ou quando sincronizando o catálogo.

3. **Atualização Atômica e Resiliência (`src/catalog.rs`):**
   - Atualizar a lógica de `update_catalog_tickets`:
     - Para cada repositório alvo, converter `CatalogEntry` em `RepoForgeInfo` via `to_forge_info()`.
     - Chamar `client.fetch_open_issues_count(&forge_info)`.
     - **Degradação graciosa**: se uma forja estiver offline, inacessível por rede ou sem permissão de acesso, registrar aviso de falha sem abortar a varredura dos demais repositórios. Se já existir contagem em cache (`open_tickets`), mantê-la. Se houver clone local com `.antigravity/tickets.md`, usar como fallback de contagem local.
     - Persistir o estado atualizado em `catalog.json` de forma atômica e segura via `save_catalog`.

4. **Feedback Visual Gosplan (`src/main.rs`):**
   - Exibir feedback claro e conciso informando:
     - Repositórios com contagem de tickets atualizada (`• <name> (<slug>) -> <count> open ticket(s)`).
     - Repositórios que falharam ou foram mantidos sem interrupção do processo.
     - Resumo final do lote.
