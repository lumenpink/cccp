# Especificação Técnica: Revisão e Submissão de Tickets no CCCP Editor

**Ticket Associado:** #138  
**Branch:** `feat/138-editor-ticket-submission`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Migrar o fluxo de revisão e submissão de tickets do Divã (`cccp divan` / `cccp diva`) para dentro do CCCP Editor (`src/editor.rs`), eliminando a exibição prévia excessiva de blocos repetitivos (como o checklist completo de 9 fases GOST) no terminal e unificando a experiência de ajuste interativo de título e corpo da issue com validação sintática estrita antes do envio.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Supressão do Eyecandy no Terminal:**
   - Suprimir o cartão prolixo com todas as fases GOST renderizado antes da postagem no terminal.
   - O Divã abre diretamente a sessão do CCCP Editor com os dados do ticket.
2. **Layout do Buffer no Editor:**
   - A primeira linha não vazia do buffer representa o título da issue.
   - Uma linha em branco imediatamente após o título é tratada como delimitador opcional e desconsiderada no início do corpo.
   - O restante do buffer compõe o corpo da issue (`body`).
3. **Atalho e Rodapé Contextual:**
   - O editor exibe barra de título e rodapé adaptados ao contexto de submissão de ticket:
     - Título: `★ [CCCP TICKET EDITOR] | <contexto>`
     - Rodapé de ajuda: `Ctrl+S Submit  Ctrl+Q / Esc Cancel  Alt+W Wrap  Arrows Navigate  Ctrl+K Cut Line`
4. **Validação Estrita Pré-Envio:**
   - Título não vazio.
   - Título estritamente compatível com a especificação Conventional Commits (`<type>(<scope>): <subject>` ou `<type>: <subject>`).
   - Corpo não vazio.
   - Se inválido, o erro é reportado com clareza impedindo submissões malformadas.
5. **Cancelamento Seguro:**
   - Caso o usuário cancele a edição (`Ctrl+Q`, `Esc` ou `Ctrl+C`), a operação é abortada sem publicar nada na forge remota e sem efeitos colaterais.

## 3. Modelo de Dados e Funções
- Função de parsing e validação de buffer:
  ```rust
  pub fn parse_ticket_buffer(buffer: &str) -> Result<(String, String), String>;
  pub fn validate_ticket_submission(title: &str, body: &str, config: &Config) -> Result<(), String>;
  ```
- Suporte a cabeçalho e ação contextual no `Editor`:
  ```rust
  impl Editor {
      pub fn set_save_label(&mut self, label: &str);
      pub fn set_header_prefix(&mut self, prefix: &str);
  }
  pub fn edit_ticket(title: &str, initial_content: &str) -> Result<Option<String>, String>;
  ```
