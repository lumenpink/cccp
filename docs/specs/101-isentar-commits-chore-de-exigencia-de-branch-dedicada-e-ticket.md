# Especificação Técnica: Isenção de Commits do Tipo Chore de Exigência de Branch e Ticket

**Ticket Associado:** #101  
**Branch:** `feat/101-isentar-commits-chore-de-exigencia-de-branch-dedicada-e-ticket`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Isentar integralmente commits do tipo `chore` da exigência de branch dedicada de ticket, de amarração a tickets do Forgejo/GitHub e de travas de commit direto na branch `main`.
Commits `chore` destinam-se a manutenção de rotina, automações e tarefas de infraestrutura simples que não devem ser travadas pela burocracia de ciclo DD-TDD completo.

## 2. Requisitos Técnicos
1. **Identificação de Commits `chore`**:
   - `is_chore_commit(cz: &CommitArgs) -> bool`:
     - Retorna `true` se `cz.commit_type == Some("chore")`.
     - Retorna `true` se `summary` ou `message` iniciarem com `chore:` ou `chore(`.
2. **Permissão na Branch `main` e em Qualquer Branch**:
   - Se for identificado como `chore`, suprimir o bloqueio `Bloqueio de Governança GOST: Commits diretos na branch 'main' são estritamente proibidos`.
   - Permitir gravação imediata do commit sem inferência obrigatória de ticket ID.
3. **Formatação Padronizada da Mensagem**:
   - Se a mensagem já contiver o prefixo `chore: ` ou `chore(...)`, preservá-la.
   - Caso contrário, compor como `chore(<scope>): <summary>` ou `chore: <summary>`.
4. **Isenção de Phase Guards**:
   - Desativar a checagem de stage/phase guard (`validate_phase_guards`) para commits `chore`.

## 3. Critérios de Aceitação
- Testes unitários comprovando que `is_chore_commit` detecta corretamente commits declarados como chore.
- Teste unitário comprovando que commits `chore` são permitidos sem ticket mesmo na branch `main`.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem erros.

## 4. Status de Resolução e Validação
- **Fase 5 (Green):** Implementada função `is_chore_commit` e integração em `handle_commit`, permitindo commits do tipo chore diretamente na `main` e em qualquer branch sem necessidade de ticket ou phase guards.
- **Fase 7 (Mutation):** Testes de mutação cobrindo variações de prefixo, argumentos explícitos e rejeição estrita de tipos não-chore.
- **Fase 8 (Quality Gate):** Validado com `cargo fmt --check`, `cargo test --test chore_commit_exemption_test` (2 testes aprovados) e `cccp check --strict` com conformidade Gosplan de 100%.
- **Fase 9 (Close):** Ciclo finalizado com laudo de encerramento GOST.
- **Integração:** Pronto para merge na main.
