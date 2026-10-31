# Especificação Técnica: Ticket #110
## fix(ui): implementar quebra de linha por palavra no editor de texto integrado

### 1. Contexto e Problema
No editor interativo integrado (`src/editor.rs`), quando o modo de quebra de linha suave (`soft_wrap`) está ativado, a rotina `compute_visual_lines` realizava o particionamento de linhas longas dividindo o texto estritamente por blocos de tamanho fixo `cols` (`(num_chars - start).min(cols)`).
Isso provocava o corte abrupto de palavras no meio ao atingir a margem direita da tela (ex.: a palavra "configuração" sendo quebrada em "configu" na linha superior e "ração" na linha inferior), prejudicando sensivelmente a legibilidade e a experiência de edição interativa de commits e tickets.

### 2. Requisitos Técnicos
1. **Quebra por Limite de Palavra (Word-Wrap) em `compute_visual_lines` (`src/editor.rs`)**:
   - Para linhas lógicas cujo comprimento exceda a largura útil da tela (`cols`):
     - Inspecionar a janela candidata de caracteres `start .. start + cols`.
     - Caso o caractere subsequente (`start + cols`) seja um espaço/delimitador de whitespace, a linha visual recebe exatamente `cols` caracteres e o cursor/avanço prossegue a partir da próxima palavra.
     - Caso a janela corte uma palavra e haja pelo menos um espaço em branco precedente na janela (`start + 1 .. start + cols`), quebrar a linha visual no último limite de palavra, mantendo a palavra inteira para a linha visual seguinte.
     - Caso uma palavra individual seja maior do que a largura inteira da coluna `cols` (sem espaços disponíveis na janela), realizar fallback gracioso dividindo pelo tamanho da coluna (`cols`), garantindo que o editor nunca entre em loop infinito nem trave.
2. **Preservação de Navegação e Invariantes do Editor**:
   - A estrutura `self.rows` permanece como SSOT intacta e imutável pelo wrapping.
   - O mapeamento bidirecional entre coordenadas do cursor lógico `(cursor_y, cursor_x)` e coordenadas visuais na tela `(v_row, v_col)` via `cursor_to_visual_pos` e `move_cursor` deve continuar determinístico e contínuo.
   - Compatibilidade total com o modo `soft_wrap = false` (1 linha visual por linha lógica).

### 3. Critérios de Aceitação
- Frases longas com palavras separadas por espaços são quebradas nos limites de palavras em vez de cortar palavras ao meio.
- Palavras individuais com comprimento maior que `cols` são quebradas graciosamente sem travar ou entrar em loop infinito.
- Todas as linhas visuais geradas respeitam estritamente `char_len <= cols`.
- Testes unitários cobrindo quebra por palavra, palavras longas, navegação de cursor e preservação do buffer.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**: Implementada quebra de linha suave respeitando limites de palavras (word-wrap) no editor interativo integrado, com fallback gracioso para termos individuais longos, garantindo legibilidade e estabilidade do buffer de edição.

