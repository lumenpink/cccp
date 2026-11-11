# Especificação Técnica - Ticket #162: Flag --failures-only com Modo Strict Implícito

## 1. Contexto e Motivação
Em projetos com dezenas de regras declarativas, o comando `cccp check` gera uma listagem longa com dezenas de linhas `[PASS]`, forçando a desenvolvedora a rolar o terminal ou filtrar com `grep` para encontrar as violações.

Este ticket introduz a flag `--failures-only` no comando `cccp check`, que filtra a saída visual para expor unicamente falhas e warnings e força o modo de verificação estrita (`strict = true`) automaticamente.

## 2. Decisão Arquitetural e Rastreabilidade (Para a Natasha)

### Interface CLI (`AuditArgs`)
* Adicionar flag `--failures-only` com aliases amigáveis `--failed-only` e `--only-failures`.
* Campo: `pub failures_only: bool`.

### Regras de Negócio e Comportamento
1. **Modo Strict Compulsório**:
   * Quando `failures_only == true`, a auditoria opera compulsoriamente como `strict = true`.
   * Qualquer `FAIL` ou `WARN` aciona código de saída de falha (`ExitCode::FAILURE`), dispensando a necessidade de combinar `--strict --failures-only`.
2. **Filtragem Visual no Relatório**:
   * O relatório omite todos os checks com status `CheckStatus::Pass` e `CheckStatus::Skip`.
   * Renderiza exclusivamente violações (`CheckStatus::Fail` e `CheckStatus::Warn`).
   * **Caso de 100% de Sucesso**: Se nenhuma regra falhar nem emitir warning, renderiza uma mensagem concisa de sucesso:
     `✓ All {total} checks passed (100% compliance)` sem poluir o terminal.

## 3. Plano de Fases (GOST DD-TDD)
- **Fase 3**: Especificação viva registrada neste documento.
- **Fase 4**: Teste vermelho em `tests/audit_test.rs` validando que `failures_only` suprime passes e aciona exit code de falha.
- **Fase 5**: Implementação em `AuditArgs`, `render_audit_report` e `main.rs`.
- **Fase 7**: Testes de mutação cobrindo repositórios 100% conformes e repositórios com apenas warnings.
- **Fase 8**: Linters (`cargo fmt`, `cargo clippy`), testes unitários e de integração.
- **Fase 9**: Fechamento do ticket #162, merge na `main` e push.

## 4. Laudo de Validação e Quality Gate (Fase 8)
- `cargo fmt --check`: 100% aprovado.
- `cargo clippy --all-targets -- -D warnings`: 0 warnings, 0 erros.
- `cargo test --test audit_test`: 22 testes aprovados, incluindo cenários nominais e mutantes de `failures_only`, 100% passing e warnings sob strict.

## 5. Conclusão do Ciclo (Fase 9)
- **Status**: Homologado e concluído.
- **Rastreabilidade**: Natasha possui opção de auditoria enxuta via `--failures-only` com estrito compulsório sem risco de aprovações falsas.


