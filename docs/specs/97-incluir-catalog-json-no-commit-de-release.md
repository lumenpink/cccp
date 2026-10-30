# Especificação Técnica: Inclusão do catalog.json no Commit de Rotação de Release na Banca

**Ticket Associado:** #97  
**Branch:** `fix/97-ci-catalog-sync`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Garantir que a pipeline de CI, ao efetuar a rotação de versão e manifestos na Banca via `scripts/rotate-downloads.py`, inclua o arquivo consolidado `site/downloads/catalog.json` no commit e push para o repositório da Banca.

## 2. Requisitos Técnicos
1. **Inclusão no Stage (`git add`)**:
   - No workflow `.forgejo/workflows/ci.yml`, na etapa `Rotate Manifest & Commit to Banca`, o comando `git add` deve incluir expressamente o catálogo consolidado:
     `git add site/downloads/cccp/index.json site/downloads/catalog.json`
2. **Contratos e Testes de Pipeline (`tests/ci_pipeline_test.rs`)**:
   - Validar estritamente que o arquivo de workflow contém a inclusão conjunta de `site/downloads/cccp/index.json` e `site/downloads/catalog.json` no step de commit.

## 3. Critérios de Aceitação
- Teste de contrato em `tests/ci_pipeline_test.rs` validando que `site/downloads/catalog.json` é adicionado no step de commit na Banca.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou erros.
