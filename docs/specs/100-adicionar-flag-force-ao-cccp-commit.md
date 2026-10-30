# Especificação Técnica: Flag --force no CCCP Commit

**Ticket Associado:** #100  
**Branch:** `feat/100-adicionar-flag-force-ao-cccp-commit`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Adicionar a opção `-f / --force` ao comando `cccp commit` para permitir forçar a gravação de commits em situações administrativas ou excepcionais quando explicitamente solicitado pelo usuário, desativando travas impeditivas locais (como bloqueio de commit na main, travas de stage/fase DD-TDD e verificação de hooks).

## 2. Requisitos Técnicos
1. **Opção na CLI (`src/cli.rs`)**:
   - Adicionar `#[arg(short = 'f', long = "force")] pub force: bool` em `CommitArgs`.
2. **Opção em `CommitOptions` (`src/commit_exec.rs`)**:
   - Adicionar campo `pub force: bool` à struct `CommitOptions`.
   - Quando `opts.force == true`, injetar variável de ambiente `CCCP_BYPASS_LOCKS=1` e adicionar flag `--no-verify` ao invocar o git.
3. **Desativação de Barreiras em `handle_commit` (`src/ddtdd.rs`)**:
   - Quando `cz.force == true`, não bloquear execuções na branch `main`.
   - Não bloquear violações de phase guards se `cz.force == true`.
   - Permitir commit direto com o resumo fornecido caso a branch não possua ticket associado.

## 3. Critérios de Aceitação
- Testes unitários comprovando que `CommitArgs::force` é interpretada corretamente tanto via `-f` quanto via `--force`.
- Teste comprovando que `CommitOptions::force` repassa as diretivas de liberação.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem erros.
