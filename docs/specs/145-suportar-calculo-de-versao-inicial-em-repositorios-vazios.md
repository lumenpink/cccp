# Especificação Técnica — Ticket #145
## Suporte a Cálculo de Versão Inicial SemVer em Repositórios Vazios

### 1. Escopo e Propósito
Corrigir o cálculo de versão de desenvolvimento (`dev_version`) no `semver_calc` e nos hooks do Git quando executados em um repositório recém-inicializado com zero commits (Protocolo do Dia Zero / Gênese).

### 2. Causa Raiz
Quando `repo.head_short_id` é uma string vazia (nenhum commit existente), a formatação `{head_date}.{head_short_id}` gera metadados SemVer inválidos (ex.: `20261108.`), disparando erro `empty identifier segment in build metadata`.

### 3. Solução
1. Em `git.rs`, inicializar `head_short_id` com `"0000000"` e `head_id` com 40 zeros em repositórios sem commits.
2. Em `semver_calc.rs` e `hook_runner.rs`, aplicar fallback seguro garantindo que identificadores vazios nunca sejam injetados nos metadados de build do SemVer 2.0.0.
