# Especificação do Ticket #191: Completar Scaffolding Pós-Gossnab e Alinhar Testes Legados ao GOST v3

## 1. Contexto e Motivação
A execução completa da suíte `cargo test --all-targets --no-fail-fast` revelou 3 pontos de quebra:
1. **`src/newrepo.rs` (`tests/newrepo_test.rs` — 4 falhas):** Quando o binário `gossnab` está presente no `PATH`, `cmd_newrepo` invocava `gossnab new` e executava `return Ok(())` imediatamente após escrever `GOST_VERSION="1.1.0"`, pulando os blocos `write_file_if_missing` responsáveis por garantir `GOSTVER` completo, `CHANGELOG.md`, `site/` e `.forgejo/workflows/ci.yml`.
2. **`tests/repo_assemble_test.rs:226`:** Exigia a criação do diretório legado `.gost/languages/shell.md`, que passou a ser embutido diretamente em `.gost/GOST.md` (Ticket #183).
3. **`tests/repo_upgrade_governance_test.rs:9`:** Exigia os artefatos legados `.rules/profile.toml` e `.gost/UPGRADE.md`, purgados na consolidação GOST v3 (Ticket #184).

## 2. Requisitos da Correção
1. Em `src/newrepo.rs`:
   - Após delegação bem-sucedida a `gossnab new`, **não** retornar precocemente nem escrever `GOST_VERSION="1.1.0"`; prosseguir para os blocos idempotentes `write_file_if_missing` para assegurar todos os arquivos canônicos sem sobrescrever artefatos gerados pelo Gossnab.
2. Em `tests/repo_assemble_test.rs`:
   - Validar que `.gost/GOST.md` existe e não está vazio em vez de exigir `.gost/languages/shell.md`.
3. Em `tests/repo_upgrade_governance_test.rs`:
   - Validar presença de `.gost/GOST.md`, `.gost/rules.toml` e `.gost/delivery.toml`, e ausência definitiva de `GOSTVER` e `.rules/`.
4. Validar **100% da suíte** (`cargo test --all-targets`), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cccp check --strict`.

## 3. Árvore de Decisões (para a Natasha)
- **Idempotência pós-Gossnab:** Como todos os passos subsequentes de `cmd_newrepo` usam `write_file_if_missing`, continuar o fluxo após `gossnab new` garante paridade total entre ambientes com e sem `gossnab` no `PATH` sem sobrescrever o scaffolding do Gossnab.
