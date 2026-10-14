# Especificação Técnica: Posicionamento Inicial do Cursor Abaixo do Cabeçalho no Editor do Divã

**Ticket Associado:** #131  
**Branch:** `feat/131-divan-editor-cursor-pos`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Ao abrir a sessão do CCCP Editor no Divã (`cccp divan` / `cccp diva`) para redação do prompt de triagem, o template inicial injeta comentários informativos no topo (ex.: instruindo sobre formatação e atalhos). O cursor atualmente inicializa em `(0, 0)`, forçando o usuário a pressionar a seta para baixo várias vezes antes de iniciar a digitação.
O objetivo é fornecer posicionamento automático do cursor logo após o bloco inicial de comentários de cabeçalho, posicionando-o na primeira linha livre disponível.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Posicionamento Automático após Comentários:**
   - O `Editor` deve dispor de um método `position_cursor_after_comments(&mut self)`.
   - O método inspeciona as linhas do buffer a partir do início (linha 0):
     - Enquanto as linhas iniciais forem comentários (iniciando por `#` ignorando espaços) ou linhas em branco que separam o bloco de cabeçalho, o cursor avança.
     - O cursor é posicionado no início da primeira linha útil (não comentada) disponível.
     - Se o cabeçalho for seguido por uma linha em branco (como no template padrão do Divan), o cursor é posicionado nessa linha livre.
2. **Resiliência e Casos de Borda:**
   - Se o buffer não contiver nenhum comentário, o cursor permanece em `(0, 0)`.
   - Se o buffer for composto exclusivamente por comentários, uma nova linha em branco é garantida ao final e o cursor é posicionado nela.
   - O método chama internamente `clamp_cursor()` para assegurar que `cursor_y` e `cursor_x` permaneçam em limites válidos e sincronizados com a viewport.
3. **Helper Dedicado para Prompts:**
   - Disponibilizar `edit_prompt(title: &str, initial_content: &str) -> Result<Option<String>, String>` em `src/editor.rs`, que instancia o editor, chama `position_cursor_after_comments()` e executa o loop do editor.
4. **Integração no Fluxo do Divã:**
   - Em `src/main.rs`, na etapa de obtenção interativa do prompt do Divan, utilizar `edit_prompt` em vez de `edit_text`.

## 3. Contrato da API (`src/editor.rs`)
```rust
impl Editor {
    /// Posiciona o cursor na primeira linha livre após comentários de cabeçalho ('#').
    pub fn position_cursor_after_comments(&mut self);
}

/// Edita prompt do Divã interativamente, posicionando o cursor após comentários.
pub fn edit_prompt(title: &str, initial_content: &str) -> Result<Option<String>, String>;
```
