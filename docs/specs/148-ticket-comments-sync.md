# Especificação Técnica: Suporte a Comentários em Tickets no CLI, Divã e Sincronização

**Ticket Associado:** #148  
**Branch:** `feat/148-ticket-comments-sync`  
**Norma:** GOST 1.4.0

## 1. Objetivo
Possibilitar o ciclo de vida completo de comentários em tickets nas forges suportadas (Codeberg/Forgejo, GitHub e GitLab) através da CLI do CCCP (`cccp ticket comment`), permitir ao Divã interagir sobre um ticket existente com contexto completo dos comentários já postados (`cccp divan -c <ID>`), e incluir os comentários na sincronização local de `.antigravity/tickets.md` respeitando o teto de segurança de 10 KB.

## 2. Requisitos Funcionais e Critérios de Aceitação

1. **Camada de Modelos e Clientes de Forge (`src/forges.rs`):**
   - Definir struct `ForgeComment`:
     ```rust
     #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
     pub struct ForgeComment {
         pub id: u64,
         pub author: String,
         pub body: String,
         pub created_at: String,
     }
     ```
   - Estender `ForgeIssue` para conter `pub comments: Vec<ForgeComment>` (com `#[serde(default)]`).
   - Implementar na `ForgeClient`:
     - `fetch_issue_comments(&self, info: &RepoForgeInfo, issue_id: u64) -> Result<Vec<ForgeComment>, String>`
     - `create_comment(&self, info: &RepoForgeInfo, issue_id: u64, body: &str) -> Result<ForgeComment, String>`
   - Suporte unificado de endpoints:
     - **Codeberg / Forgejo / Tildegit**: `GET/POST /api/v1/repos/{owner}/{repo}/issues/{id}/comments`
     - **GitHub**: `GET/POST /repos/{owner}/{repo}/issues/{id}/comments`
     - **GitLab**: `GET/POST /projects/{id}/issues/{issue_iid}/notes`

2. **CLI de Tickets (`src/cli.rs` e `src/main.rs`):**
   - Adicionar subcomando em `TicketAction`:
     ```rust
     Comment {
         #[arg(required = true)]
         id: u64,
         #[arg(short = 'm', long = "message")]
         message: Option<String>,
     }
     ```
   - Invocação: `cccp ticket comment <id> [-m "texto"]`.
   - Se `-m` for omitido em terminal interativo, abre o editor de texto (`edit_text`) com cabeçalho `CCCP COMMENT EDITOR` e salva com `Ctrl+S`.

3. **Modo Comentário no Divã (`src/cli.rs`, `src/ai.rs`, `src/main.rs`):**
   - Suportar flag opcional `--comment <ID>` / `-c <ID>` em `DivanArgs`.
   - Ao executar `cccp divan -c <ID>`:
     - Baixa a issue alvo e seus comentários existentes.
     - Monta o contexto da IA contendo o título, corpo original e o histórico cronológico de comentários.
     - A IA elabora uma sugestão de comentário técnico de acordo com a solicitação do usuário.
     - Permite revisão no editor e publica na forge via `create_comment`.

4. **Sincronização Markdown (`src/tickets.rs`):**
   - Na formatação individual do ticket (`format_ticket_markdown`), incluir a seção de comentários caso existam:
     ```markdown
     ### Comments (N)
     - **@autor** (YYYY-MM-DD HH:MM:SS UTC):
       Corpo do comentário
     ```
   - A função `generate_tickets_markdown` mantém o respeito estrito ao limite configurado de bytes (`max_bytes`, padrão de 10240 bytes / 10 KB), interrompendo a inclusão antes de estourar a cota.
