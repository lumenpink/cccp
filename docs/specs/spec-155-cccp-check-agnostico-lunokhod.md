# Especificação do Ticket #155: cccp check 100% Agnóstico via Lunokhod

## Objetivo
Desacoplar o Maestro `cccp` de qualquer regra de conformidade ou decreto normativo hardcoded em código Rust compilado. O comando `cccp check` passa a atuar estritamente como orquestrador, delegando toda a avaliação de conformidade para o motor dinâmico `lunokhod` a partir do catálogo declarativo `rules.toml` (ou `.gost/rules.toml`).

## Requisitos de Engenharia
1. **Purga de Regras Rígidas em Rust (`src/audit.rs`):**
   - Remover as checagens legadas hardcoded (`GOST-16`, `GOST-01..07`, `GOST-17`, `GOST-20`, `GOST-24`, `GOST-26`, `GOST-27`).
   - Remover as funções internas auxiliares de inspeção estática (`check_gost20_docs`, `check_gost24_binaries`, `check_gost26_metadata`, `check_gost27_domain_purity`).
2. **Orquestração Declarativa:**
   - Na função `run_gost_audit_with_scope`:
     - Obter metadados do cabeçalho de relatório via `read_gostver`.
     - Carregar regras desativadas via `read_disabled_rules`.
     - Se `.gost/rules.toml` e `rules.toml` não existirem, mas houver `GOSTVER` ou `profile.md`, acionar `assemble_gost_specs` para compilar o catálogo em cascata.
     - Se nenhum catálogo for localizado ou gerado, reportar falha determinística com código `NO-RULES-FOUND`.
     - Havendo catálogo, delegar a avaliação das regras diretamente para `crate::lua_rules::evaluate_lua_rules_for_scope`.
3. **Hermeticidade e Desacoplamento:**
   - O `cccp` não deve impor ou assumir nomes ou regras além do contrato padrão `rules.toml` e do runtime `lunokhod`.
4. **Atualização dos Testes de Auditoria:**
   - Ajustar `tests/audit_test.rs` para atestar a natureza agnóstica do Maestro: falha clara sob ausência de catálogo, sucesso na presença de `rules.toml` declarativo e aplicação de `--strict`.
