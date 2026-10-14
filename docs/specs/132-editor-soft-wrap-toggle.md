# Especificação Técnica: Quebra Automática Padrão e Atalho de Alternância no Editor

**Ticket Associado:** #132  
**Branch:** `feat/132-editor-soft-wrap-toggle`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Garantir e formalizar que o editor embutido do CCCP (`src/editor.rs`) abra com quebra automática de linha (*soft wrap*) ativada por padrão, forneça um atalho dedicado (`Alt+W`) para alternar o modo sob demanda, informe o estado atual na barra de status (`[WRAP]`) e no rodapé, e reconcilie deterministicamente os offsets de rolagem horizontal e vertical na transição entre os modos ligado e desligado.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Ativação Padrão:**
   - Ao ser instanciado via `Editor::new`, o editor deve iniciar com `soft_wrap = true`.
2. **Método e Atalho de Alternância:**
   - O método `toggle_soft_wrap(&mut self) -> bool` deve alternar o estado booleano de `soft_wrap`.
   - Ao ativar `soft_wrap` (`true`), o offset horizontal (`col_offset`) é zerado para evitar distorções de renderização visual.
   - Ao desativar `soft_wrap` (`false`), o offset visual (`visual_row_offset`) é zerado e o `row_offset` lógico é alinhado à posição do cursor.
   - O atalho `Alt+W` no loop de eventos deve invocar `toggle_soft_wrap`.
3. **Sinalização Visual na Interface:**
   - A barra de status deve exibir o marcador `[WRAP]` quando `soft_wrap` estiver ativo e ocultá-lo quando desativo.
   - O rodapé de ajuda deve indicar persistentemente `Alt+W Wrap`.
4. **Navegação do Cursor Conforme o Modo:**
   - Com `soft_wrap = true`, as setas verticais navegam entre os chunks visuais da mesma linha lógica ou entre linhas adjacentes.
   - Com `soft_wrap = false`, a navegação vertical opera diretamente entre linhas lógicas com rolagem horizontal (`col_offset`) ativada ao ultrapassar a largura da tela.

## 3. Modelo de Funções
- Método público adicionado à `struct Editor`:
  ```rust
  impl Editor {
      pub fn toggle_soft_wrap(&mut self) -> bool;
  }
  ```
