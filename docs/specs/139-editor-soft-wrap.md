# Especificação Técnica: Soft Wrap no CCCP Editor

**Ticket Associado:** #139  
**Branch:** `feat/139-editor-soft-wrap`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Implementar o motor de quebra suave de linha (*soft wrap*) no editor de texto minimalista embutido no CCCP (`src/editor.rs`), permitindo que textos com linhas que excedem a largura visível do terminal sejam automaticamente quebrados e renderizados visualmente em múltiplas linhas físicas da tela, sem alterar os dados do buffer em memória nem persistir caracteres `\n` artificiais em disco.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Quebra Suave Visual:**
   - Quando `soft_wrap` estiver ativo (padrão: `true`), linhas lógicas com tamanho superior a `screen_cols` são divididas em chunks visuais consecutivos de largura máxima `screen_cols`.
   - Se `soft_wrap` for falso, preserva o comportamento anterior de corte com rolagem horizontal (`col_offset`).
2. **Preservação Estrita do Buffer Real:**
   - O buffer `rows: Vec<String>` e a saída de `to_content()` permanecem inalterados. Nenhuma quebra artificial é injetada.
3. **Mapeamento e Navegação do Cursor:**
   - O cursor navega de forma natural através dos chunks visuais.
   - Pressionar a seta para `Down` dentro de uma linha quebrada transiciona o cursor para o próximo chunk visual da mesma linha lógica (na coluna correspondente).
   - Pressionar a seta para `Up` dentro de um chunk secundário transiciona o cursor para o chunk visual anterior da mesma linha lógica.
   - Pressionar `Left` / `Right` percorre os caracteres da linha e, ao cruzar o limite visual de uma linha física, reposiciona o cursor na próxima linha física da tela.
4. **Rolagem Vertical Suave:**
   - A rolagem vertical opera sobre as linhas visuais renderizadas (`visual_row_offset`), garantindo visibilidade contínua de blocos longos sem saltos bruscos.
5. **Barra de Status:**
   - Mantém as métricas do buffer real: número da linha real (`Ln {}`), coluna real (`Col {}`) e total de linhas lógicas (`{} lines`).
   - Exibe o indicador `[WRAP]` quando o modo de quebra suave estiver ativado.

## 3. Modelo de Dados e Funções
- Struct `VisualLine`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct VisualLine {
      pub logical_row: usize,
      pub char_start: usize,
      pub char_len: usize,
  }
  ```
- Campos adicionados a `Editor`:
  - `pub soft_wrap: bool` (default: `true`)
  - `pub visual_row_offset: usize`
- Métodos auxiliares:
  - `compute_visual_lines(&self) -> Vec<VisualLine>`
  - `cursor_to_visual_pos(&self, visual_lines: &[VisualLine]) -> (usize, usize)`
  - `move_cursor_vertical(&mut self, up: bool, visual_lines: &[VisualLine])`
