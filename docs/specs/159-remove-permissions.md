# Especificação Técnica — Remoção de Campo Permissions no Workflow do CI (#58)

- **ID do Ticket:** #58
- **Status:** Aprovado
- **Autora:** Lumen
- **Alinhamento:** GOST 1.5.0 / Forgejo Actions / baquara-infra#36

## 1. Contexto & Motivação
A diretiva top-level `permissions:` (`contents: write`) não é suportada nativamente pelo Forgejo Actions runner e gera poluição visual e advertências nos logs de execução do pipeline de integração contínua do CCCP.

## 2. Requisitos Técnicos
1. **Remoção de `permissions:` em `.forgejo/workflows/ci.yml`:**
   - Eliminar a diretiva global `permissions:` e sub-chaves declaradas.
2. **Preservação de Integridade:**
   - Assegurar que os jobs de CI (`test`, `bundle`, `deploy-banca`, `sync-mirrors`) permaneçam inalterados e sintaticamente válidos.
3. **Auditoria em Teste Automatizado:**
   - Adicionar verificação em `tests/audit_test.rs` que assegure que nenhum workflow em `.forgejo/workflows/` declare `permissions:`.
   - Adicionar teste de mutação garantindo que a injeção acidental de `permissions:` cause a reprovação do teste.
