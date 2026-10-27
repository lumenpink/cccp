# Especificação Técnica — Ticket #63: Permitir Argumento Posicional ou Flag de Repo no Comando Divan

## 1. Visão Geral
O subcomando `cccp divan` (e seu alias `cccp diva`, bem como `cccp ticket divan`) atua como assistente interativo de triagem e consulta com IA (Ollama, Gemini, Local).
Para máxima ergonomia e integração com o ecossistema Gosplan, o comando deve permitir especificar o repositório-alvo tanto através do argumento posicional `<repo>` quanto pela opção `--repo <repo>` (ou `-r <repo>`).
Além do parsing de CLI, o contexto completo do repositório selecionado (forja remota, commits recentes, branch, AGENTS.md e workspace local) deve ser repassado fielmente ao fluxo do Divan, evitando contaminação pelo repositório do diretório corrente.

---

## 2. Requisitos Técnicos

### 2.1 Parser CLI e Resolução de Argumentos
- Suportar tanto no comando de topo `cccp divan` quanto em `cccp ticket divan`:
  - `cccp divan <repo>`: argumento posicional identificando o repositório-alvo.
  - `cccp divan <repo> <prompt>`: argumento posicional de repositório seguido de instrução de triagem.
  - `cccp divan --repo <repo>` (ou `-r <repo>`): opção explícita de repositório.
  - `cccp divan --repo <repo> <prompt>`: repositório via flag e prompt posicional.
  - `cccp divan "<prompt com espacos>"`: inferência automática de prompt quando o argumento contém espaços e nenhuma flag de repositório foi fornecida.
- Garantir que tanto `Commands::Divan(DivanArgs)` quanto `TicketAction::Divan(DivanArgs)` processem a resolução uniformemente via `DivanArgs::resolve_repo_and_prompt()`.

### 2.2 Resolução de Contexto do Repositório Alvo
- Quando um repositório for especificado (via posicional ou `--repo`) ou selecionado interativamente no catálogo:
  1. Resolver as informações de forja (`RepoForgeInfo`) via `resolve_target_repo`.
  2. Localizar o caminho do workspace local correspondente ao repositório alvo (através do catálogo via `find_local_repo_path_with_config`, ou caminhos convencionais em `~/src/<repo>`, ou diretório informado).
  3. Se o workspace local do repositório alvo existir:
     - Abrir o `GitRepo` associado a esse diretório específico (e não ao diretório de trabalho de onde o comando foi invocado).
     - Definir `target_dir` para o diretório do repositório alvo, garantindo que rascunhos (`DivanDraft`) e checkpoints (`DivanCheckpoint`) fiquem associados ao repositório correto.
  4. Se o repositório alvo for puramente remoto (sem clone local):
     - Não reutilizar o `GitRepo` do diretório corrente (evitando passar commits ou arquivos irrelevantes no `repo_ctx`).
     - Definir `git_repo = None`, enriquecendo o contexto apenas com os metadados da forja remota.

### 2.3 Exibição Visual no Editor do Divan
- O cabeçalho do editor interativo do Divan deve exibir claramente o alvo resolvido:
  `# The Divan — Target: <Provider> (<owner>/<repo>)` ou o nome qualificado do repositório.

---

## 3. Critérios de Aceitação
1. Testes unitários de parsing cobrindo:
   - `cccp divan <repo>`
   - `cccp divan -r <repo>`
   - `cccp divan --repo <repo> <prompt>`
   - `cccp ticket divan --repo <repo>`
2. Testes de integração validando a resolução de diretório e contexto do repositório alvo quando diferente do diretório atual.
3. Conformidade total com `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.
