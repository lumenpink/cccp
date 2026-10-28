# Especificação Técnica: Ocultação de Repositórios Espelho em `cccp repo list`

**Ticket:** #80  
**Componente:** `src/catalog.rs` / `src/cli.rs`  
**Status:** Aprovado para Implementação  
**Padrão:** GOST v2.0 / Norma 1.5.0  

---

## 1. Contexto e Motivação

O catálogo do Gosplan (`catalog.json`) centraliza o inventário de repositórios do ecossistema, incluindo projetos canônicos e, eventualmente, espelhos secundários (*pull mirrors* ou migrações de bibliotecas de upstream com flag `mirror = true` ou `is_mirror = true`).

Atualmente, o comando `cccp repo list` (e `cccp catalog list`) renderiza indistintamente todos os repositórios cadastrados na saída tabular, causando poluição visual e redundância informativa para o operador.

---

## 2. Requisitos e Regras de Negócio

1. **Ocultação por Padrão:**
   - Todo repositório marcado como espelho (`is_mirror = true`) DEVE ser filtrado e omitido por padrão na listagem tabular de `cccp repo list` e `cccp catalog list`.
2. **Flag de Inclusão (`--include-mirrors` / `--mirrors`):**
   - A estrutura `CatalogFilterArgs` deve disponibilizar o argumento opcional `--include-mirrors` (alias `--mirrors`).
   - Quando ativado, os espelhos são incluídos na tabela normalmente.
3. **Estrutura de Dados:**
   - `CatalogEntry` e `RawCatalogEntry` devem suportar o campo booleano `is_mirror` (desserializando de `is_mirror` ou `mirror`, padrão `false`).
   - `CatalogRepoSummary` deve carregar `pub is_mirror: bool`.
   - `summarize_catalog` deve propagar `entry.is_mirror` e também reconhecer se o repositório local possui remote configurado como mirror.
4. **Preservação de Integridade:**
   - Comandos diretos por identificador ou slug (como `cccp repo path <slug>`, `cccp repo assemble <slug>`) continuam operando normalmente sem bloqueio por espelhamento.
