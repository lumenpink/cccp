# Especificação Técnica: Simplificação da Opção --phase do CCCP Commit

**Ticket Associado:** #99  
**Branch:** `feat/99-simplificar-option-phase-do-cccp-commit-para-aceitar-apenas-numeros-3-a-8`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Simplificar o parâmetro `-p / --phase` do subcomando `cccp commit` para que aceite diretamente os números inteiros correspondentes às fases de desenvolvimento (`3` a `8`), eliminando a necessidade de digitar o prefixo `ph`.
Adicionalmente, implementar travas explícitas que impeçam o uso de fases sem commit (`1` e `2`) e impeçam o uso da fase `9` (que exige obrigatoriamente o subcomando `cccp close`).

## 2. Requisitos Técnicos
1. **Entradas Aceitas para `--phase` / `-p`**:
   - `"3"` (ou `"ph3"` para compatibilidade retroativa) -> Fase 3: Doc/Spec.
   - `"4"` (ou `"ph4"`) -> Fase 4: Red Test.
   - `"5"` (ou `"ph5"`) -> Fase 5: Green Implement.
   - `"6"` (ou `"ph6"`) -> Fase 6: Refactor.
   - `"7"` (ou `"ph7"`) -> Fase 7: Mutation Test.
   - `"8"` (ou `"ph8"`) -> Fase 8: Quality Gate.
2. **Rejeição das Fases 1 e 2**:
   - Se `--phase` receber `"1"` ou `"2"` (ou `"ph1"`, `"ph2"`):
     Interromper com erro:
     `Erro GOST: Fases 1 (Ticket) e 2 (Branch) não possuem commits. Inicie o ciclo com 'cccp ticket new' e 'cccp start <id>'.`
3. **Rejeição da Fase 9**:
   - Se `--phase` receber `"9"` (ou `"ph9"`):
     Interromper com erro:
     `Erro GOST: A Fase 9 (Finish & Close) exige o laudo técnico de fechamento e deve ser executada obrigatoriamente através de 'cccp close "<resumo>"'.`
4. **Mensagem de Erro para Fases Inválidas**:
   - Qualquer outro valor deve instruir claramente que as fases de commit válidas são de 3 a 8.

## 3. Critérios de Aceitação
- Testes unitários comprovando que `parse_phase("3")` até `parse_phase("8")` mapeiam para `Ph3` a `Ph8`.
- Testes unitários comprovando que `parse_phase("1")` e `parse_phase("2")` retornam erro explicativo indicando que fases 1 e 2 não geram commits.
- Teste unitário comprovando que `parse_phase("9")` retorna erro indicando a obrigatoriedade do comando `cccp close`.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou erros.

## 4. Status de Resolução e Validação
- **Fase 5 (Green):** Implementada função `parse_phase_arg` aceitando diretamente números `3` a `8`, rejeitando `1` e `2` com explicação e rejeitando `9` com orientação expressa de `cccp close`.
- **Fase 7 (Mutation):** Testes de mutação cobrindo espaços em branco, maiúsculas, valores fora da faixa (`0`, `10`, etc.).
- **Fase 8 (Quality Gate):** Validado com `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --test ddtdd_phase_parsing_test` (4 testes aprovados) e `cccp check --strict` com conformidade de 100%.
- **Fase 9 (Close):** Ciclo finalizado com laudo de encerramento GOST.
