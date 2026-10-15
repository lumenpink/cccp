# Spec #129: Filtros na Listagem do Catálogo para Repositórios Dirty e Push Pendente

## 1. Contexto e Motivação
O Catálogo Gosplan (`cccp catalog list` / `cccp catalog`) permite ao camarada inspecionar os repositórios gerenciados e seu status git. Em sessões de desenvolvimento com múltiplos repositórios, é frequente a necessidade de filtrar rapidamente apenas repositórios com alterações pendentes de commit (`dirty`) para revisão, ou apenas repositórios limpos com commits pendentes de envio (`unpushed`) para sincronização upstream.
O Ticket #129 adiciona flags diretas de filtragem na CLI (`--dirty` e `--unpushed` / `--pending-push`), com validação de exclusão mútua e informação estruturada na saída.

## 2. Requisitos & Aceite
1. **Flags na CLI (`CatalogFilterArgs`):**
   - Flag `--dirty`: filtra exclusivamente repositórios com modificações unstaged ou staged no working tree (`is_clean == Some(false)`).
   - Flag `--unpushed` (com alias `--pending-push`): filtra exclusivamente repositórios limpos com commits locais à frente do upstream (`is_clean == Some(true) && unpushed > 0`).
   - Validação de exclusão mútua via clap (`conflicts_with`), impedindo uso simultâneo de `--dirty` e `--unpushed`.
2. **Propagação e Mesclagem de Filtros (`CatalogFilterArgs::merge`):**
   - Propagar `dirty` e `unpushed` ao mesclar opções globais e subcomandos do catálogo (`cccp catalog --dirty list`, `cccp catalog list --unpushed`).
3. **Mecanismo de Filtragem (`filter_and_sort_summaries`):**
   - Reter repositórios em conformidade com as flags booleanas.
4. **Detecção e Exibição de Status Detalhado:**
   - Inspecionar contagem de arquivos modificados (`modified_files`) em `inspect_local_status`.
   - Exibir na coluna `STATUS` a contagem de arquivos modificados (ex: `dirty [1*]` ou `dirty [2* ↑1]`) e commits ahead (ex: `clean [↑2]`).
