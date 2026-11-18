# Especificação — Ticket #180: Leitura de Spec via .gost e Subcomando gost setup

## 1. Contexto e Motivação
No padrão GOST v3.0, o arquivo plano `GOSTVER` na raiz é um artefato legado. A fonte da verdade das diretrizes reside exclusivamente dentro do diretório `.gost/` (`.gost/GOST.md`, `.gost/metadata.json`, `.gost/rules.toml` e `.gost/delivery.toml`).
Atualmente, o motor de auditoria (`src/audit.rs`) e o comando de versão (`src/main.rs`) ainda procuram compulsoriamente por `GOSTVER` na raiz. Na ausência deste arquivo, o CCCP reporta falsamente `GOST Specification: none` e assume `Languages: None`.
Adicionalmente, falta um comando nativo no CCCP para inicializar ou atualizar repositórios para a v3.0 do padrão de forma hermética e atômica.

## 2. Escopo Técnico
1. **Resolução de Versão e Metadados via `.gost/`:**
   - `read_gostver(root)` passa a buscar primeiro o frontmatter YAML de `.gost/GOST.md` (campos `version`, `standard`, `inherits_from`).
   - Se não encontrar em `.gost/GOST.md`, consulta `.gost/metadata.json`.
   - Detecção inteligente de linguagens (`autodetect_languages(root)`): caso não haja declaração explícita, identifica a stack com base nos manifestos do projeto (`Cargo.toml` -> `rust`, `package.json` -> `javascript`, `composer.json` -> `php`, `go.mod` -> `go`, etc.).
   - Mantém leitura de `GOSTVER` apenas como fallback secundário e transiente.
2. **Subcomando `cccp gost setup`:**
   - Sintaxe: `cccp gost setup [--profile <url>] [--force]`
   - Executa a montagem completa da árvore `.gost/`:
     - Compilação em cascata de `.gost/GOST.md`.
     - Sincronização de `.gost/rules.toml`.
     - Inicialização de `.gost/metadata.json` com `schema_version = 1`.
     - Inicialização de `.gost/delivery.toml` declarativo com presença ativa.
     - Remoção compulsória de `GOSTVER` legado na raiz se presente.

## 3. Árvore de Decisão Técnica (Natasha)
- **Por que autodetectar linguagens nos manifestos de projeto?**
  Evita que repositórios sem configuração manual rodem checks com `Languages: None`, garantindo a aplicação imediata das regras do toolchain correspondente.
- **Por que `cccp gost setup` em vez de apenas `cccp profile assemble`?**
  `assemble` monta apenas o documento markdown (`GOST.md`). O `setup` orquestra a anatomia completa do diretório `.gost/` (especificação, regras, metadados e delivery) e purga o passado legado de uma só vez.

## 4. Laudo de Resolução (Fase 9)
- **Status:** Concluído com 100% de conformidade GOST.
- **Implementação:**
  - `src/audit.rs`: Extração de versão/padrão do frontmatter de `.gost/GOST.md`, autodetecção de stack via `autodetect_languages` e fallback resiliente.
  - `src/catalog.rs` & `src/main.rs`: `extract_gost_version` e `cccp version` integrados a `.gost/GOST.md`.
  - `src/profile.rs` & `src/cli.rs`: Subcomando `cccp gost setup` (com alias `profile setup`) para montagem da anatomia completa de `.gost/` e purga do arquivo legado `GOSTVER`.
- **Validação:** 6 cenários de testes automatizados em `tests/gost_spec_resolution_spec.rs`, `cargo clippy --all-targets -- -D warnings` limpo e `cccp check --strict` aprovado com 24 regras conformes.
