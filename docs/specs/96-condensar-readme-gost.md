# Especificação Técnica: Condensação do README para Conformidade GOST v2.0.0

**Ticket Associado:** #96  
**Branch:** `docs/96-condensar-readme-gost`  
**Norma:** GOST 2.0.0 (Regra GOST-20-SITE-TYPE-AND-SSOT-DOCS)

## 1. Objetivo
Adequar o arquivo `README.md` da raiz do repositório `cccp` ao teto normativo estrito de no máximo 50 linhas determinado pela regra `GOST-20-SITE-TYPE-AND-SSOT-DOCS` da norma GOST v2.0.0, eliminando a única inconformidade apontada pelo comando `cccp check --strict` e atingindo 100% de conformidade.

## 2. Requisitos Técnicos
1. **Contagem Estrita de Linhas**:
   - O `README.md` deve possuir estritamente no máximo 50 linhas (incluindo quebras em branco).
2. **Preservação de Informações Essenciais**:
   - Manter a identidade visual com logo (`site/assets/logo.svg`), lema em russo e badges principais.
   - Quickstart conciso com instalação e primeiro comando.
   - Apontamento explícito para a documentação abrangente em `docs/` e o portal web oficial em `site/` (`https://cccp.2lp.in`).
   - Apontamentos de licenças e governança.
3. **Validação Automatizada**:
   - Criar teste de contrato em `tests/readme_gost_test.rs` validando que `line_count <= 50`.
   - Assegurar aprovação de 100% no `cccp check --strict`.

## 3. Critérios de Aceitação
- `README.md` com `<= 50` linhas.
- `cccp check --strict` retornando 100% de aprovação (13/13 itens conformes).
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou erros.

## 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**: `README.md` condensado para 42 linhas, preservando identidade visual, quickstart e links de governança. Testes unitários, testes de mutação, clippy e auditoria `cccp check --strict` 100% aprovados (13/13 itens conformes).


