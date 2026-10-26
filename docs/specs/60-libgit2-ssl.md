# Especificação Técnica: Migração para Libgit2 com Suporte Completo a SSL

**Ticket Associado:** #60  
**Branch:** `feat/60-libgit2-ssl`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Substituir o motor de inspeção Git baseado em `gix` (Gitoxide) pelo `git2` (bindings para `libgit2` com suporte integrado a SSL/HTTPS via OpenSSL) no módulo `src/git.rs` do `cccp`, garantindo compatibilidade estrita com a API pública de `GitRepo`, `TagInfo`, `CommitSummary` e `LocalRepoStatus`, além de assegurar que o binário compile e passe por todos os Quality Gates.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Dependências (`Cargo.toml`):**
   - Remover `gix = "0.88"`.
   - Adicionar `git2 = "0.20"` (ou `0.21`) com features padrão e SSL ativado.
2. **Implementação do Módulo Git (`src/git.rs`):**
   - Substituir chamadas a `gix::discover` por `git2::Repository::discover`.
   - Implementar `find_latest_semver_tag` via busca de tags e referências do `git2`.
   - Implementar `walk_commits_since` usando `git2::Revwalk`.
   - Implementar `check_is_dirty` inspecionando status do repositório através de `repo.statuses()`.
   - Manter a assinatura e comportamento idênticos dos métodos públicos: `GitRepo::open`, `get_remote_url` e `inspect_local_status`.
3. **Conformidade de Compilação e Testes:**
   - Testes unitários de `GitRepo` devem passar com sucesso.
   - Zero advertências do Clippy (`-D warnings`) e formatação estrita (`cargo fmt`).
