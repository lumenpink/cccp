# Especificação Técnica: Instrução de Comandos CCCP nas Mensagens de Bloqueio dos Hooks

**Ticket Associado:** #92  
**Branch:** `fix/92-instruir-comandos-cccp-nos-hooks`  
**Norma:** GOST 1.5.0

## 1. Objetivo
Padronizar todas as mensagens de barreira, validação e bloqueio emitidas pelos hooks do Git gerenciados pelo CCCP (`pre-commit`, `commit-msg`, `pre-push`, `post-merge`), garantindo que tanto desenvolvedores humanos quanto agentes autônomos de IA recebam comandos canônicos, explícitos e copiáveis do CCCP (`cccp commit`, `cccp push`, `cccp close`, `cccp tag`, `cccp sync`), além de alertas severos e expressos contra contornos manuais (como flags `--no-verify`, manipulação de variáveis como `CCCP_MAESTRO_ACTIVE` ou commits diretos).

## 2. Requisitos Técnicos
1. **Hook `pre-commit`**:
   - Bloquear execuções diretas de `git commit` sem autorização do Maestro.
   - Apresentar com destaque os comandos canônicos copiáveis:
     - `cccp commit "<resumo>"` (Fases 3 a 8 do DD-TDD)
     - `cccp close "<resumo>" -b "<laudo>"` (Fase 9 do DD-TDD)
   - Adicionar aviso explícito contra contornos com comandos git manuais ou variáveis de bypass.
2. **Hook `commit-msg`**:
   - Ao rejeitar mensagem de commit fora da convenção Conventional Commits, instruir o uso de `cccp commit` e `cccp close` em vez de retentativas diretas com `git commit`.
   - Alertar contra o uso de `--no-verify`.
3. **Hook `pre-push`**:
   - Bloquear execuções diretas de `git push` sem autorização do Maestro.
   - Apresentar o comando canônico `cccp push`.
   - Alertar explicitamente que a esteira CI roda automaticamente no push autorizado e proibir comandos git manuais e bypasses.
   - Quando tags de release forem rejeitadas (ausência de VERSION ou incompatibilidade), orientar expressamente o uso de `cccp tag`.
4. **Hook `post-merge` e `cccp merge`**:
   - Instruir o fechamento com `cccp close` antes do merge caso o commit final não contenha a tag de fechamento canônica `(closes #<id>)`.

## 3. Critérios de Aceitação
- Testes unitários validando a presença dos comandos sugeridos e alertas anti-bypass nas saídas dos hooks.
- Testes de mutação cobrindo os textos e garantindo ausência de regressão.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` 100% aprovados.

## 4. Status de Resolução
- **Status**: Validado no Quality Gate (Fase 8).
- **Laudo**: Implementadas instruções explícitas de comandos canônicos copiáveis (`cccp commit`, `cccp close`, `cccp push`, `cccp tag`) e alertas anti-bypass contra flags `--no-verify` e variáveis manuais (`CCCP_MAESTRO_ACTIVE`, `CCCP_BYPASS_LOCKS`). Testes unitários, mutações, rustfmt e clippy 100% aprovados sem avisos.

