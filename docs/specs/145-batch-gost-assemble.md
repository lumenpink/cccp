# Especificação Técnica: Montagem de Especificações GOST em Lote para o Catálogo

**Ticket Associado:** #145  
**Branch:** `feat/145-batch-gost-assemble`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Implementar a funcionalidade de montagem e consolidação em lote das especificações canônicas `.gost/GOST.md` para todos os repositórios locais gerenciados no catálogo do CCCP, respeitando as camadas de herança em cascata (`extends`), perfis (`GOSTVER`) e personalizações locais (`CUSTOM.md`).

## 2. Interface CLI
- Subcomando primário: `cccp catalog assemble [--dry-run|-n]` (com alias `cccp catalog gost`).
- Opção estendida em profile: `cccp profile assemble [--all|-a] [--dry-run|-n]`.
- Parâmetros:
  - `-n, --dry-run`: Pré-visualiza os repositórios locais que possuem `GOSTVER` elegíveis para montagem, sem realizar escritas em disco.
  - `-a, --all`: Executa a montagem para todos os repositórios do catálogo a partir do subcomando `profile assemble`.

## 3. Critérios de Elegibilidade e Processamento
Para cada entrada `CatalogEntry` registrada no catálogo Gosplan:
1. **Clone Local Existente:** O caminho local do repositório (`local_path`) deve existir e ser acessível no disco. Se ausente, o repositório é classificado como `skipped_remote`.
2. **Definição de Versão GOST (`GOSTVER`):** O arquivo `GOSTVER` deve existir na raiz do repositório local. Se ausente, é classificado como `skipped_no_gostver`.
3. **Modo Dry-Run:** Se a flag `dry_run` estiver ativada, o repositório é registrado em `dry_run` e nenhuma operação de I/O é realizada.
4. **Execução Real:** Se elegível e `dry_run` for falso:
   - Invoca `cccp::profile::assemble_gost_specs(path)`.
   - Se retornar `Ok(_)`, registra em `assembled`.
   - Se retornar `Err(e)`, registra em `failed` com a mensagem do erro.

## 4. Estrutura de Dados e Relatório
- Struct: `BatchAssembleReport`:
  - `assembled: Vec<String>`
  - `dry_run: Vec<String>`
  - `skipped_no_gostver: Vec<String>`
  - `skipped_remote: Vec<String>`
  - `failed: Vec<(String, String)>`
- Exibição de sumário formatado construtivista (`format_batch_assemble_report`), com ícones ANSI e totalizadores.
