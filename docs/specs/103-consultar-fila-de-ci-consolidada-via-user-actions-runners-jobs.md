# Especificação Técnica: Consulta Consolidada da Fila de CI via User Actions Runners Jobs

**Ticket Associado:** #103  
**Branch:** `feat/103-consultar-fila-de-ci-consolidada-via-user-actions-runners-jobs`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Substituir as consultas sequenciais e repetitivas de CI repo por repo pela consulta consolidada ao endpoint `GET /api/v1/user/actions/runners/jobs` do Forgejo, obtendo toda a fila de execução (`waiting`, `running`) de todos os repositórios em uma única chamada de alta performance (< 400ms).

## 2. Requisitos Técnicos
1. **Estrutura de Dados `ForgeActionRunnerJob`**:
   - Mapear os campos essenciais retornados pela API:
     - `id`: identificador numérico do job
     - `run_id`: identificador do workflow run
     - `attempt`: número da tentativa
     - `name`: nome do job/esteira
     - `html_url`: URL canônica para navegação
     - `status`: estado de execução (`waiting`, `running`, etc.)
     - `repo_id`: ID do repositório no Forgejo
2. **Método `fetch_user_runner_jobs` no `ForgeClient`**:
   - Requisitar `GET /api/v1/user/actions/runners/jobs` usando autenticação por token do host especificado.
   - Tratar respostas de erro e desserializar em `Vec<ForgeActionRunnerJob>`.
3. **Extração de Slug a partir da URL do Job**:
   - Função utilitária `extract_repo_slug_from_action_url(html_url: &str) -> Option<String>` para associar jobs ao repositório correspondente (ex: `https://gambi.2lp.in/lumen/cccp/actions/runs/26/jobs/0/attempt/1` -> `lumen/cccp`).
4. **Integração no Monitor Interativo `repo_top`**:
   - Função `merge_live_runner_jobs(summaries: &mut [CatalogRepoSummary], jobs: &[ForgeActionRunnerJob])` que atualiza os pipelines com status `waiting` e `running`.
   - Na inicialização e nos ciclos de refresh do `run_repo_top`, efetuar a consulta consolidada se houver forja Forgejo configurada, garantindo visibilidade em tempo real sem lentidão.

## 3. Critérios de Aceitação
- Testes unitários para desserialização e parse de `ForgeActionRunnerJob`.
- Testes unitários para extração correta de slugs a partir de URLs do runner.
- Testes unitários para mesclagem (`merge_live_runner_jobs`) atualizando ou adicionando pipelines ativos aos resumos de repositório.
- Bateria de testes passando e `cccp check --strict` sem avisos.

## 4. Conclusão e Laudo
- [Resumo]: Integração da consulta consolidada da fila de CI de todos os repositórios através de chamada única ao endpoint `GET /api/v1/user/actions/runners/jobs` do Forgejo.
- [Solução]: Adicionados o modelo `ForgeActionRunnerJob`, utilitários `extract_repo_slug_from_action_url`, `merge_live_runner_jobs`, `discover_forgejo_hosts` e o método `fetch_user_runner_jobs` no `ForgeClient`, integrando a atualização em tempo real no monitor interativo `run_repo_top`.
- [Validação]: Testes automatizados em `tests/ci_consolidated_runner_jobs_test.rs`, `tests/repo_top_layout_test.rs` e `tests/repo_top_test.rs` 100% aprovados; `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem falhas.

