# Especificação Técnica — Ticket #170: Desacoplar Motor Lua e Delegar Auditoria ao Lunokhod

## 1. Contexto & Motivação
Historicamente, o `cccp` continha dependência direta de `mlua` (com bindings C vendored) e da crate `lunokhod`. Essa dependência impunha sobrecarga no build, acoplamento de runtime C/Lua no binário do maestro local e quebrava o princípio de separação de responsabilidades.

Com a consolidação da CLI do `lunokhod` (Ticket #13), o `lunokhod` atua como ferramenta especializada e desacoplada para execução de scripts declarativos Lua (`rules.toml`) e auditoria normativa (`lunokhod check --json`).

## 2. Decisão Arquitetural
- **Desacoplamento Completo de Dependências:** Foram removidas as dependências diretas de `mlua` e `lunokhod` em `Cargo.toml`. O binário do `cccp` passa a ser Rust estático puro, sem dependências C de runtime Lua.
- **Delegação via Processo:** `evaluate_lua_rules_with_lang` em `src/lua_rules.rs` executa `lunokhod --dir <root> check --json --lang <lang> [--scope <scope>]` via `std::process::Command`. A saída estruturada em JSON é desserializada para `Vec<AuditCheck>`.
- **Resiliência e Degradação Graciosa:** Caso o binário `lunokhod` não esteja instalado no PATH ou a execução falhe, a auditoria retorna uma lista vazia sem causar pânico no ciclo do `cccp`.
- **Filtro de Regras Desabilitadas:** Mantida a filtragem de `DISABLED_RULES` do arquivo `.gost/config` sobre o resultado emitido pelo `lunokhod`.

## 3. Alternativas Descartadas (Para a Natasha)
- **Alternativa Descartada 1: Manter `lunokhod` como biblioteca Rust no `Cargo.toml`.**
  *Motivo do Descarte:* Reintroduziria a dependência indireta de `mlua` e `liblua.a`, mantendo o peso de compilação em C e impossibilitando o maestro de ser um binário enxuto e puro.
- **Alternativa Descartada 2: Embutir scripts Lua diretamente via `eval` inline sem o binário.**
  *Motivo do Descarte:* Duplicaria lógica de sandbox, primitivos de runtime `gost.*` e internacionalização já mantidos e testados no `lunokhod`.

## 4. Verificação & Quality Gate
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --test audit_test`
- `cargo test --test lunokhod_integration_test`
- `cargo test --test config_test`
