# Especificação Técnica: Layout Responsivo, Corte UTF-8 e Padding ANSI no CCCP Repo Top

**Ticket Associado:** #102  
**Branch:** `fix/102-ajustar-layout-responsivo-80-colunas-corte-utf8-e-padding-ansi`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Corrigir a renderização do monitor interativo `cccp repo top` para que caiba perfeitamente em terminais padrão de 80 colunas sem line wrapping (quebra de linha involuntária), alinhar rigorosamente as colunas compensando códigos de escape ANSI e garantir truncamento seguro de strings UTF-8.

## 2. Requisitos Técnicos
1. **Ajuste para Terminais de 80 Colunas**:
   - Redesenhar a tabela para que a largura total em modo base (80 colunas) não ultrapasse 79 caracteres:
     - Marcador: 2 chars (`▶ ` ou `  `)
     - `REPOSITÓRIO`: 22 chars (expansível dinamicamente se o terminal for mais largo)
     - `FORJA`: 9 chars
     - `ESTADO CI`: 18 chars
     - `TICKETS`: 8 chars
     - `LOCAL`: 16 chars
   - Linhas divisórias e cabeçalhos devem ser limitados exatamente à largura do terminal (`cols`), evitando estourar a tela.
2. **Padding Visual Imune a ANSI**:
   - O alinhamento da coluna `ESTADO CI` deve calcular o preenchimento de espaços (*padding*) usando o comprimento visível de caracteres (`ci_text.chars().count()`), e não a quantidade de bytes da string formatada com ANSI escapes (`colored_ci`).
3. **Truncamento UTF-8 Seguro com Reticências**:
   - Função utilitária `truncate_utf8_ellipsis(s: &str, max_chars: usize) -> String` que itera sobre caracteres UTF-8 e adiciona `…` quando exceder o limite, evitando panics de fatiamento de bytes.

## 3. Critérios de Aceitação
- Testes unitários comprovando que nenhuma linha gerada por `render_top_view` para terminal de largura 80 excede 80 colunas visuais.
- Teste comprovando que strings com cores ANSI mantêm o alinhamento correto das colunas subsequentes.
- Teste comprovando truncamento de caracteres acentuados em UTF-8 sem erros.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem erros.
