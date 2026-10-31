# Especificação Técnica: Ticket #113
## fix(ui): substituir ampulheta por caractere de largura 1 no repo top

### 1. Contexto e Problema
Na interface textual do CCCP (`cccp repo top` e `cccp catalog list`), o status de pipelines de CI em execução ou pendentes era representado pelo caractere/emoji de ampulheta `⏳` (`U+23F3`).
Embora `⏳` seja contabilizado como 1 caractere via `str.chars().count()`, sua largura visual em emuladores de terminal monospaçados modernos (padrão Unicode East Asian Width / `wcwidth`) é de 2 colunas.
Isso gerava um desalinhamento sistemático das colunas nas tabelas:
1. Em `cccp repo top`, o cálculo `pad_visual(&colored_ci, ci_text.chars().count(), 18)` alocava 9 espaços para `"⏳ pending"` (comprimento 9), resultando em 19 colunas visuais em vez de 18, empurrando as colunas subsequentes 1 coluna para a direita nas linhas com pipelines pendentes.
2. Em `cccp catalog list`, as bordas e separadores verticais de tabela `│` ficavam deslocados em 1 coluna para a direita nas linhas com pipelines pendentes.
3. Na visualização detalhada de CI (`cccp catalog list --ci-details`), o ícone `⏳` causava desalinhamento em relação aos ícones `✓`, `✗` e `•`.

### 2. Requisitos Técnicos
1. **Substituição do Símbolo de CI Pendente (`src/catalog.rs`)**:
   - Substituir o emoji `⏳` por um caractere ASCII padrão de 1 coluna fixa (`~`).
   - `"⏳ pending"` passa a ser `"~ pending"` (com contagem de jobs: `"~ pending ({}/{})"`).
   - Na lista detalhada de jobs (`format_catalog_ci_details`), o ícone para pipelines com status pendente/em execução passa a ser `"~"`.
2. **Garantia de Largura e Alinhamento**:
   - Assegurar que `"~ pending"` possua contagem de caracteres (`chars().count()`) estritamente igual à sua largura visual de renderização em terminal (9 colunas para `"~ pending"`).
   - O padding computado em `pad_visual` e na tabela de `catalog list` deve manter alinhamento tabular perfeito e determinístico em qualquer terminal.

### 3. Critérios de Aceitação
- `format_ci_summary` retorna `"~ pending"` ou `"~ pending (x/y)"` para pipelines pendentes, com cor `"yellow"`.
- Nenhum caractere de largura dupla (`wcwidth = 2`) é utilizado na representação de status de CI em tabelas do sistema.
- A visualização expandida de pipelines em `format_catalog_ci_details` utiliza o ícone `"~"` para status pendente/executando.
- Todos os testes unitários e de integração validam a nova formatação e a ausência de `⏳`.
