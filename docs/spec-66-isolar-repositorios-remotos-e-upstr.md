# Especificação Técnica — Ticket #66: Isolar Repositórios Remotos e Upstream contra Mutações Locais

## 1. Visão Geral
No ecossistema do Gosplan State Catalog, certos repositórios são mantidos como referências remotas ou upstreams corporativos/oficiais (`is_upstream = true`). Nesses repositórios, o desenvolvedor local frequentemente não possui permissões de push, ou o workspace serve estritamente como fonte de leitura e sincronização de tickets/CI.

Qualquer intervenção acidental por comandos em lote (`assemble`, `sync`, `hook`) ou invocações seletivas que tentem escrever arquivos, recompilar especificações locais ou executar commits nesses repositórios deve ser estritamente bloqueada.

---

## 2. Requisitos Técnicos

### 2.1 Isolamento em Operações Seletivas
- Em `assemble_catalog_repo` (e comandos similares de mutação):
  - Inspecionar a propriedade `is_upstream` da entrada do catálogo.
  - Se `entry.is_upstream == true`:
    - Abortar imediatamente sem efetuar qualquer escrita em disco.
    - Retornar erro explicativo:
      `"Repositório '<slug>' está configurado como upstream e é isolado contra mutações locais."`

### 2.2 Isolamento em Operações em Lote
- Em `batch_assemble_gost` e `batch_assemble_gost_repos`:
  - Repositórios com `is_upstream == true` devem ser automaticamente ignorados e contabilizados em `skipped_remote` / `skipped_upstream`.
  - Nenhuma tentativa de leitura de working tree ou escrita de arquivos em repositórios marcados como upstream/remote sem diretório local deve ser executada.

---

## 3. Matriz de Testes (Fases 4 & 7)
- `tests/repo_isolation_test.rs`:
  1. Teste de rejeição de `assemble_catalog_repo` quando o repositório está marcado como `is_upstream: true`.
  2. Teste de rejeição quando o repositório não possui workspace local clonado (`find_local_repo_path` retorna `None`).
  3. Teste de isolamento em lote (`batch_assemble_gost`) pulando silenciosamente e com segurança entradas `is_upstream`.
  4. Teste de permissão normal para repositórios locais padrão (`is_upstream: false`).
