# Especificação Técnica: Salvamento Manual de Rascunhos e Gerenciamento por Repositório no Divã

**Ticket Associado:** #135  
**Branch:** `feat/135-divan-manual-drafts`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Permitir que o usuário salve rascunhos intencionalmente durante a redação do prompt ou durante a revisão de tickets no Divã (`cccp divan` / `cccp diva`), possibilitando alternar de repositório sem perder o conteúdo digitado. O rascunho é persistido em cache isolado por repositório e acionado via atalho `Ctrl+R`. Ao reabrir o Divã no mesmo repositório, um menu de decisão semelhante aos arquivos de swap do Vim é apresentado para determinar se o rascunho deve ser continuado, mantido ou descartado.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Atalho `Ctrl+R` e Feedback Visual no Editor:**
   - O CCCP Editor (`src/editor.rs`) captura o atalho `Ctrl+R` durante a edição de prompts ou tickets.
   - Ao acionar `Ctrl+R`, o editor invoca o salvador de rascunho configurado e atualiza a barra de status com mensagem imediata:
     `"★ Rascunho salvo com sucesso [Ctrl+R]"`
   - O rodapé de ajuda exibe `Ctrl+R Rascunho` entre os atalhos disponíveis.
2. **Isolamento de Rascunhos por Repositório (`src/draft.rs`):**
   - O armazenamento de rascunhos é isolado por caminho canônico sanitizado do repositório em `$XDG_CACHE_HOME/cccp/drafts/<sanitized_repo>.json`.
   - Um rascunho salvo no repositório `cccp` não interfere em nenhum outro repositório (ex.: `fuxico`).
3. **Suporte a Múltiplos Estágios de Rascunho:**
   - Suporta salvar tanto o texto do prompt inicial (antes da chamada à IA) quanto os tickets já processados pelo modelo que estejam aguardando revisão e aprovação.
4. **Menu de Decisão estilo Vim Swap ao Reabrir:**
   - Ao iniciar o Divã em um repositório com rascunho salvo existente, apresenta as opções:
     - `[c]` **Continuar**: restaura o rascunho na sessão e remove o arquivo de rascunho após submissão bem-sucedida.
     - `[m]` **Manter**: preserva o rascunho em disco para depois e inicia uma nova sessão limpa.
     - `[d]` **Descartar**: exclui o rascunho do disco e inicia uma nova sessão limpa.

## 3. Contrato da API (`src/draft.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DraftStage {
    Prompt,
    TicketsReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DivanDraft {
    pub repo_path: String,
    pub stage: DraftStage,
    pub prompt: String,
    pub tickets: Vec<DecomposedTicket>,
    pub current_index: usize,
    pub updated_at: String,
}

impl DivanDraft {
    pub fn save_prompt(repo_path: &Path, prompt: &str) -> Result<(), String>;
    pub fn save_tickets(
        repo_path: &Path,
        prompt: &str,
        tickets: &[DecomposedTicket],
        current_index: usize,
    ) -> Result<(), String>;
    pub fn load(repo_path: &Path) -> Result<Option<Self>, String>;
    pub fn clear(repo_path: &Path) -> Result<(), String>;
}
```
