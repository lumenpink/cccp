# Especificação Técnica: Exibição do Status da Última Execução de CI no Catálogo

**Ticket Associado:** #155  
**Branch:** `feat/155-catalog-ui-ci-status`  
**Norma:** GOST 1.4.0  

## 1. Objetivo
Apresentar de forma visual, padronizada e informativa no Catálogo Gosplan (`cccp catalog list`) o status da execução mais recente das esteiras de CI sincronizadas para cada projeto/repositório, permitindo filtragem por estado de CI, ordenação e exibição detalhada com data/hora e nome de cada esteira avaliada.

## 2. Requisitos Funcionais e Critérios de Aceitação

1. **Estrutura de Resumo do Catálogo (`CatalogRepoSummary` em `src/catalog.rs`):**
   - Adicionar o campo:
     ```rust
     pub ci_status: Vec<CiPipelineStatus>,
     ```
   - Em `summarize_catalog`:
     - Preencher a partir de `entry.ci_status`.
     - Caso `entry.ci_status` esteja vazio e o repositório possua clone local, verificar e ler `.antigravity/ci_status.json` se existente como fallback local.

2. **Apresentação na Tabela Gosplan (`format_catalog_table` em `src/catalog.rs`):**
   - Adicionar a coluna `CI` na tabela principal:
     `│ REPOSITORY │ FORGE │ GOST │ CI │ TICKETS │ STATUS │ LOCAL PATH │`
   - Indicadores visuais com convenção semântica e suporte a cores:
     - Todas as esteiras com sucesso: `✓ pass` (ou `✓ pass (n)` se `n > 1`) em verde (`green`).
     - Falha em pelo menos uma esteira: `✗ fail (f/t)` em vermelho (`red`).
     - Em execução ou pendente: `⏳ running` ou `⏳ pending` em amarelo (`yellow`).
     - Sem esteiras ou status desconhecido: `-` em cor atenuada (`dim`).

3. **Parâmetros de Filtragem e Detalhamento da CLI (`CatalogFilterArgs` em `src/cli.rs`):**
   - Flag `--ci <status>`:
     - Permite filtrar repositórios por status de CI: `pass` / `success`, `fail` / `failure`, `pending` / `running`, ou `none`.
   - Flag `--ci-details`:
     - Exibe bloco expandido abaixo da tabela listando individualmente cada esteira por repositório com nome, status, data/hora (`updated_at`) e target_url (se presente).
   - Suporte a `--sort ci`:
     - Permite ordenar os repositórios pelo estado de CI (falhas no topo, seguidas de pendentes, sucessos e sem CI).

4. **Consistência Visual e Erros:**
   - Manter compatibilidade com temas visuais do Gosplan (`Theme`), cálculo estrito de larguras de colunas e caracteres UTF-8.
