# Especificação do Ticket #190: Hermetizar `language_cascade_assemble_spec` na CI

## 1. Contexto e Motivação
O teste de integração `tests/language_cascade_assemble_spec.rs` invocava `assemble_gost_specs` em um `tempdir` sem fixture local de linguagem. Na máquina de desenvolvimento, `fetch_language_spec_content` encontrava `/home/lumen/src/gost-profile-kollontai/languages/rust.md`; na esteira hermética de CI, esse diretório não existe e a tentativa de download HTTP falhava, quebrando o teste.

## 2. Requisitos da Correção
1. Em `tests/language_cascade_assemble_spec.rs`:
   - Provisionar a fixture `.gost/languages/rust.md` (contendo `"GOST Language Specification: Rust"`) e artefatos legados (`.rules/`, `.gost/rules.json`, `.gost/PROFILE.md`) dentro do `tempdir` antes de chamar `assemble_gost_specs`.
   - Validar que `assemble_gost_specs` embute o conteúdo da especificação de Rust no `.gost/GOST.md` e purga completamente `.gost/languages/`, `.rules/`, `.gost/rules.json` e `.gost/PROFILE.md`.
2. Validar `cargo test --test language_cascade_assemble_spec`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cccp check --strict`.

## 3. Árvore de Decisões (para a Natasha)
- **Fixture Local vs Mock HTTP:** `fetch_language_spec_content` já implementa precedência de autonomia local (`.gost/languages/<lang>.md`) seguida de purga automática em GOST v3. Usar a fixture local valida simultaneamente a leitura, o embutimento no `GOST.md` e a purga do diretório legado de forma 100% offline e determinística.
