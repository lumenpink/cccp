# Especificação Técnica: Recuperação de Estado de Sessão (Checkpoints) no Divan

**Ticket Associado:** #134  
**Branch:** `feat/134-divan-session-recovery`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Implementar um mecanismo de checkpoint automático e recuperação de sessão no Divã do CCCP (`cccp divan` / `cccp diva`), indexado por repositório alvo. O mecanismo assegura que falhas de rede (como timeout da API do modelo generativo ou falha de conexão com a forge remota), quedas de terminal ou interrupções abruptas não resultem na perda do prompt do usuário nem dos tickets já decompostos, permitindo a retomada determinística a partir do último estado bem-sucedido.

## 2. Requisitos Funcionais e Critérios de Aceitação
1. **Isolamento e Persistência Não-Invasiva:**
   - Os checkpoints devem ser armazenados de forma isolada por repositório no diretório de cache do usuário (`$XDG_CACHE_HOME/cccp/checkpoints/` com fallback para `~/.cache/cccp/checkpoints/`), indexados pela chave canônica do caminho do repositório.
   - O armazenamento não polui nem cria arquivos não rastreados no repositório de trabalho.
2. **Estágios de Checkpoint:**
   - **`PromptEntered`**: Salvo imediatamente após o usuário fornecer o prompt de triagem e antes do envio ao modelo generativo.
   - **`TicketsPendingReview`**: Salvo após a recepção bem-sucedida dos tickets estruturados retornados pela IA, contendo a lista completa de `DecomposedTicket` e o índice atual de revisão (`current_index`).
3. **Detecção e Diálogo de Restauração:**
   - Ao iniciar o Divã para um repositório que possua um checkpoint ativo válido, o sistema deve apresentar um prompt informativo perguntando se o usuário deseja restaurar a sessão anterior.
   - Se aceito (`Y` / Enter):
     - Se o estágio for `TicketsPendingReview`, o sistema pula nova chamada ao modelo e retoma diretamente no loop de revisão/submissão a partir de `current_index`.
     - Se o estágio for `PromptEntered`, o sistema restaura o prompt original sem exigir redigitação.
   - Se rejeitado (`n`): O checkpoint existente é descartado e uma nova sessão limpa é iniciada.
4. **Atualização Contínua e Limpeza:**
   - Conforme cada ticket é submetido ou avançado no loop de revisão, o `current_index` do checkpoint é atualizado.
   - Ao concluir a revisão e submissão de todos os tickets (ou cancelamento explícito da sessão com descarte), o checkpoint é automaticamente removido (`clear`).

## 3. Modelo de Dados e Funções
- Structs e enums em `src/checkpoint.rs`:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
  pub enum CheckpointStage {
      PromptEntered,
      TicketsPendingReview,
  }

  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
  pub struct DivanCheckpoint {
      pub repo_path: String,
      pub stage: CheckpointStage,
      pub prompt: String,
      pub tickets: Vec<DecomposedTicket>,
      pub current_index: usize,
      pub updated_at: String,
  }

  impl DivanCheckpoint {
      pub fn save_prompt(repo_path: &Path, prompt: &str) -> Result<(), String>;
      pub fn save_tickets(repo_path: &Path, prompt: &str, tickets: &[DecomposedTicket], current_index: usize) -> Result<(), String>;
      pub fn load(repo_path: &Path) -> Result<Option<Self>, String>;
      pub fn clear(repo_path: &Path) -> Result<(), String>;
  }
  ```
