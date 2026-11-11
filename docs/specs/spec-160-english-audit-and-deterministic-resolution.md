# Especificação Técnica: Relatório em Inglês e Resolução Determinística Única (Ticket #160)

## 1. Contexto e Motivação
O comando `cccp check` exibia cabeçalhos, índices de conformidade e vereditos hardcoded em português, enquanto os catálogos normativos operam canonicamente em inglês (`en`). Além disso, o módulo `src/profile.rs` mantinha listas de URLs candidatas de fallback contendo nomes de perfis específicos (`lilya`, `kollontai`, `gosplan`, `gost`), rotas cruas de forjas e caminhos locais de sistema de arquivos.

Para garantir que o Maestro seja estritamente agnóstico, o relatório padrão de auditoria deve ser unificado em inglês (com suporte a `--lang` conectado ao motor `lunokhod`) e a resolução de camadas deve operar sob uma única convenção determinística: `{url}/{version}/RULES.md` e `{url}/{version}/RULES.toml`.

## 2. Decisões Arquiteturais e Alternativas Descartadas (Para a Natasha)
- **Convenção Única de Busca de Normas (`src/profile.rs`):**
  - *Decisão:* A resolução remota de qualquer camada normada deve consultar estritamente:
    1. Texto unificado: `{url}/{version}/RULES.md`
    2. Regras declarativas: `{url}/{version}/RULES.toml`
    3. Catálogo de versões estáveis: `{url}/versions.json`
  - *Alternativa descartada:* Manter fallback para caminhos legados (`profile.md`, rotas raw de git, caminhos locais `/home/lumen/...`). Descartada porque gerava caminhos zumbis, dependência de infraestrutura externa e ocultava erros de publicação em sites de normas.
- **Relatório de Auditoria Agnóstico em Inglês (`src/audit.rs`):**
  - *Decisão:* Substituir os literais em português no cabeçalho e sumário por inglês técnico:
    - `☭ [RULES] NORMATIVE CONFORMANCE AUDIT`
    - `Standard: {ver} | Profile: {prof} | Languages: {langs}`
    - `Total checks: {total} | Conforming: {passed} | Warnings: {warnings} | Failures: {failures}`
    - `Compliance Index: {compliance}%`
  - *Alternativa descartada:* Manter terminologia `[ГОСТ / GOSPLAN]` no cabeçalho. Descartada pois violava o desacoplamento universal do motor para normas externas.
- **Conexão da Flag `--lang` com o Lunokhod:**
  - *Decisão:* Repassar a opção de idioma de `cccp check --lang <locale>` (default `"en"`) para `lunokhod::evaluate_lua_rules_for_scope`.

## 3. Plano de Fases (DD-TDD)
- **Fase 4 (Red Test):** Testes em `tests/audit_test.rs` comprovando cabeçalho em inglês e resolução agnóstica sem fallbacks específicos.
- **Fase 5 (Green Implement):** Atualização de `src/audit.rs`, `src/lua_rules.rs`, `src/profile.rs` e atualização da dependência `lunokhod` no `Cargo.lock`.
- **Fase 7 (Mutation Test):** Garantir que requisições com URLs arbitrárias geram o caminho canônico `{url}/{version}/RULES.toml` sem disparar buscas em nomes de normas específicas.
- **Fase 8 (Quality Gate):** `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.
- **Fase 9 (Close):** Laudo de fechamento e merge na base principal.

## 4. Resultados da Execução
- **Testes Unitários e Mutantes:** 37 testes automatizados executados e aprovados nas suítes `audit_test`, `rules_domain_test`, `profile_test`, `lua_rules_test`, `lunokhod_integration_test` e `assemble_rules_cascade_test`.
- **Quality Gate:** `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem qualquer advertência ou erro.
- **Integração Externa:** Lunokhod v0.1.0-dev.45 homologado e integrado via gambi.
