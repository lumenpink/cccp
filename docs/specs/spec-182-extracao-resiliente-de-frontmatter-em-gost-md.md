# Especificação — Ticket #182: Extração Resiliente de Frontmatter em GOST.md

## 1. Contexto e Motivação
No `cccp`, a extração de versão e especificação a partir de `.gost/GOST.md` (`extract_gost_frontmatter` em `src/audit.rs`) dependia da suposição de que o primeiro bloco delimitado por `---` no documento conteria as chaves `standard:` e `version:`.
Entretanto, o cabeçalho montado pelo `cccp assemble` / `cccp sync` inclui separadores horizontais markdown (`---`) antes do início da primeira camada. Isso causava o encerramento prematuro do parser antes de inspecionar os blocos YAML reais das camadas compiladas, retornando `(None, None)`.
Em repositórios onde o arquivo de fallback `.rules/profile.toml` não existe (como o `kombinat`), o CCCP exibia incorretamente `GOST Specification: none`.

## 2. Escopo Técnico
1. **Parser Resiliente de Blocos YAML (`extract_gost_frontmatter`):**
   - Iterar por todos os blocos delimitados por `---` no documento markdown até encontrar `version` e `standard`.
   - Ignorar separadores horizontais solitários ou blocos sem as chaves esperadas.
   - Extrair a versão e padrão assim que encontrados no primeiro bloco YAML válido.
2. **Fallback para `.gost/metadata.json`:**
   - Caso `.gost/GOST.md` não contenha versão explícita, consultar `.gost/metadata.json` como fonte secundária estruturada antes dos fallbacks legados.

## 3. Árvore de Decisão Técnica (Natasha)
- **Por que iterar sobre múltiplos blocos em vez de buscar apenas regex no arquivo inteiro?**
  Garante que apenas blocos estruturados delimitados por `---` (frontmatters e camadas de especificação) sejam considerados como metadados válidos, evitando capturar menções casuais a "version:" no corpo textual de regras.
- **Por que priorizar a primeira camada compilada?**
  No cabeçalho compilado de `.gost/GOST.md`, a primeira camada define o padrão tático/estratégico de autoridade (ex: Kollontai ou Lilya).

## 4. Critérios de Aceite
- Teste unitário cobrindo `.gost/GOST.md` gerado pelo `assemble` (com separador horizontal markdown antes do frontmatter da Camada 1).
- `cccp version` em repositórios sem `.rules/` deve reportar corretamente `GOST Specification: 2.1.0`.
- Nenhum falso positivo nem regressão nos testes existentes.

## 5. Laudo de Resolução (Fase 9)
- **Status:** Concluído com 100% de conformidade GOST.
- **Implementação:**
  - `src/audit.rs`: `extract_gost_frontmatter` aprimorado para descartar blocos espúrios abertos por réguas horizontais markdown (`---`) seguidas de marcação HTML (`<!--`), tratar comentários inline com `#` e capturar `standard` e `version` de qualquer bloco YAML válido.
- **Validação:** 8 testes unitários passando em `tests/gost_spec_resolution_spec.rs`, `cargo fmt --check` limpo, `cargo clippy --all-targets -- -D warnings` sem alertas e `cccp check --strict` aprovado com 24 regras conformes (100%).

