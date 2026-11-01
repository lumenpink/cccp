# Especificação Técnica: Ticket #118
## fix(ui): corrigir altura e eliminar flickering no repo top

### 1. Contexto e Problema
Na interface textual do monitor do CCCP (`cccp repo top`), o cabeçalho superior contendo informações do monitor e o estado de convergência da sincronização (`✓ convergente` / animação de sincronização) não estava visível, e a tela apresentava cintilação constante (*flickering*):

1. **Estouro de Altura e Desaparecimento do Cabeçalho**:
   - A visualização do `repo top` possui 5 linhas fixas de controle e moldura:
     - Linha 1: Cabeçalho com título, filtro, ordenação, total e indicador de convergência (`header_line`).
     - Linha 2: Nomes das colunas da tabela (`col_header`).
     - Linha 3: Separador de traço superior (`sep`).
     - Linha N-1: Separador de traço inferior (`sep`).
     - Linha N: Linha inferior de atalhos e ajuda (`bottom_display`).
   - O cálculo das linhas de dados disponíveis (`visible_lines`) em `render_top_view` e o controle de rolagem em `run_repo_top` utilizavam `rows.saturating_sub(4).max(1)`.
   - Em um terminal com altura de 24 linhas, eram alocadas 20 linhas para a tabela. Somando as 5 linhas fixas, a tela gerava 25 linhas.
   - A impressão da 25ª linha provocava o rolamento (*scroll*) automático do buffer do terminal em 1 linha, empurrando o cabeçalho (linha 1) para fora da janela visível.

2. **Cintilação Contínua (Flickering)**:
   - Durante a sincronização em segundo plano, o frame era repintado a cada 100 ms.
   - O loop executava `crossterm::terminal::Clear(ClearType::All)` seguido de `MoveTo(0, 0)` a cada ciclo.
   - O ato de limpar a tela inteira com `\x1b[2J` a cada 100 ms e imediatamente reenviar 25 linhas provocava o piscar da tela e novo scroll, tornando impossível a visualização estável do cabeçalho.

### 2. Requisitos Técnicos
1. **Orçamento Fixo de Linhas (`src/repo_top.rs`)**:
   - Definir a constante `pub const FIXED_NON_DATA_ROWS: usize = 5;`.
   - O cálculo de linhas de repositórios disponíveis no corpo da tabela em `render_top_view` deve ser `rows.saturating_sub(FIXED_NON_DATA_ROWS).max(1)`.
   - O cálculo de linhas visíveis no controle de scroll em `run_repo_top` deve utilizar a mesma constante `FIXED_NON_DATA_ROWS`.
   - O número total de linhas geradas por `render_top_view` nunca deve ultrapassar a altura `rows` especificada para o terminal.

2. **Redesenho In-Place Livre de Flickering**:
   - Limpar a tela apenas uma vez ao entrar na Alternate Screen.
   - No loop contínuo de renderização, posicionar o cursor em `(0, 0)` via `MoveTo(0, 0)` sem invocar `Clear(ClearType::All)`.
   - Adicionar o escape de limpeza do cursor até o final da tela (`\x1b[J`) após a escrita do frame para limpar eventuais resíduos caso a lista seja reduzida por filtros.
   - Cada linha já encerra com `\x1b[K\r\n` (ou `\x1b[K` na última linha), garantindo a substituição atômica sem flicker.

### 3. Critérios de Aceitação
- Em qualquer altura de terminal informada (`height >= 10`), a contagem total de linhas produzidas por `render_top_view` satisfaz estritamente `rendered.lines().count() <= height as usize`.
- O cabeçalho superior permanece perfeitamente visível na primeira linha da tela sem sofrer rolamento.
- O loop de renderização do `repo top` não executa `Clear(ClearType::All)` a cada tick, eliminando o efeito estroboscópico/flickering.
- Todos os testes unitários e de mutação em `tests/repo_top_layout_test.rs` passam com sucesso.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado nos testes unitários e de qualidade.
- **Laudo**: Definida a constante `FIXED_NON_DATA_ROWS = 5`, garantindo que o orçamento de linhas para a tabela considere todas as molduras e cabeçalhos sem estourar a altura do terminal nem rolar a tela. Eliminada a invocação de `Clear(All)` a cada tick de 100 ms em `run_repo_top`, adotando redesenho in-place com `MoveTo(0, 0)` e `\x1b[J`, eliminando por completo o flickering e mantendo o cabeçalho de convergência perfeitamente estável.
