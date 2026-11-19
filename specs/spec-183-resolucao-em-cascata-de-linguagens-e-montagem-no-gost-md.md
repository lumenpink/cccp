# Especificação Técnica: Resolução em Cascata de Linguagens e Montagem no GOST.md (Ticket #183)

## 1. Contexto e Motivação
Na Metanorma Fundamental GOST v3.0.0, o arquivo `.gost/GOST.md` é o SSOT único, integral e auto-contido da norma do repositório.
Em conformidade com o desacoplamento formal de competências (Ticket #45 do `gost` e Ticket #26 do `gost-profile-kollontai`), as especificações técnicas de linguagem pertencem à Camada Tática (`tier: tactical`) e devem ser resolvidas deterministicamente na mesma cadeia de herança dos perfis operacionais e táticos, sendo embutidas diretamente dentro de `.gost/GOST.md`.

## 2. Requisitos Mandatórios

1. **Resolução de Linguagens em Cascata:**
   Para cada linguagem declarada no repositório (via frontmatter de `.gost/GOST.md` ou detecção), a especificação técnica é buscada na seguinte ordem de precedência:
   - **Camada Operacional:** No perfil operacional ativo (ex.: `gost-profile-lilya/languages/<lang>.md` ou URL do perfil).
   - **Camada Tática (Teto Máximo):** No perfil tático base (ex.: `gost-profile-kollontai/languages/<lang>.md` ou `https://kollontai.2lp.in/<ver>/languages/<lang>.md`).
   - **Fallback de Transição:** Em `/home/lumen/src/gost/specs/languages/<lang>.md` ou cópia residual local.

2. **Montagem Auto-Contida em `.gost/GOST.md`:**
   - Durante `assemble_gost_specs`, as especificações de linguagens resolvidas são sanitizadas contra termos proibidos e apensadas ao final de `.gost/GOST.md` sob seção técnica canônica (ex.: `## Especificação Técnica da Stack: <Stack>`).
   - O frontmatter YAML individual da linguagem é removido na interpolação para preservar a unicidade do frontmatter raiz do `GOST.md`.

3. **Interrupção de Geração Obsoleta:**
   - `assemble_gost_specs` e `newrepo::sync_gost_specs` deixam de criar `.rules/` e deixam de baixar `.gost/rules.json`, `.gost/PROFILE.md` e `.gost/languages/`.
   - O arquivo de regras declarativas `.gost/rules.toml` permanece como catálogo canônico de regras sem duplicatas em `.rules/`.

4. **Verificação de Conformidade:**
   - Testes unitários dedicados em Rust validando a concatenação e a integridade do `.gost/GOST.md` gerado.

## 3. Resolução
Implementada resolução em cascata e compilação de linguagens no GOST.md aprovada em 9 fases DD-TDD.
