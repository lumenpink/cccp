# Especificação Técnica: Exibição da Coluna de Versão GOST no Catálogo

**Ticket Associado:** #146  
**Branch:** `feat/146-catalog-gost-version-column`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Adicionar uma coluna dedicada `GOST` na tabela de listagem de repositórios do catálogo Gosplan (`cccp catalog list`), permitindo identificar visualmente a versão ativa da governança GOST em cada repositório local a partir do seu arquivo `GOSTVER`, além de possibilitar a ordenação do catálogo pela versão GOST via `--sort gost`.

## 2. Interface CLI
- Subcomando afetado: `cccp catalog list [OPTIONS]`
- Novo valor para `--sort`: `gost` (com alias `gostver`)
  - Exemplo: `cccp catalog list --sort gost`
  - Ordenação ascendente por versão semântica/alfanumérica da norma GOST (ex.: repositórios sem versão ou remotos agrupados/ordenados previsivelmente).

## 3. Comportamento e Apresentação da Tabela
1. **Cabeçalho da Tabela:**
   - Adicionar a coluna `GOST` entre `FORGE` e `TICKETS`:
     `┌─ REPOSITORY ─┬─ FORGE ─┬─ GOST ─┬─ TICKETS ─┬─ STATUS ─┬─ LOCAL PATH ─┐`
2. **Extração da Versão GOST:**
   - Para cada repositório local, inspecionar o arquivo `GOSTVER` na raiz do repositório local.
   - Extrair o valor da variável `GOST_VERSION` (aceitando formatos como `GOST_VERSION="1.4.0"` ou `GOST_VERSION='1.4.0'` ou `GOST_VERSION=1.4.0`).
   - Se o repositório for apenas remoto ou se `GOSTVER` não existir ou não contiver `GOST_VERSION`, o valor é tratado como ausente (`None`).
3. **Renderização de Valores:**
   - Quando presente: exibe a versão formatada (ex.: `1.4.0`).
   - Quando ausente: exibe `-` (dimmed/cinza quando cores estão ativas).
4. **Largura da Coluna:**
   - A coluna `GOST` possui cabeçalho `GOST` (4 caracteres) e comporta versões típicas (ex.: `1.4.0`, largura mínima 6 ou adaptada ao conteúdo).

## 4. Estrutura de Dados
- Na struct `CatalogRepoSummary`:
  - Adicionar o campo `pub gost_version: Option<String>`.
- No enum `CatalogSort`:
  - Adicionar a variante `Gost` (com atributo `#[value(alias = "gostver")]`).
- Na função `extract_gost_version(repo_dir: &Path) -> Option<String>`:
  - Função auxiliar para parsing de `GOSTVER`.
