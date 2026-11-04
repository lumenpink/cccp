# Especificação Técnica: Ticket #138
## feat(repo-top): espelhar lista de repositórios recentes e status real de ci da forja

### 1. Contexto e Problema
O comando interativo `cccp repo top` foi concebido para fornecer uma visão operacional do estado dos repositórios e de suas esteiras de integração contínua (CI).
Contudo, o comportamento atual divergia severamente da interface web do Forgejo (Gambi):
1. **Ordenação Incompatível com o Painel**: O Forgejo lista na página inicial os repositórios em ordem decrescente de atividade/atualização recente (`sort=updated&order=desc`). O `repo top` ordenava por `CiStatus`, priorizando qualquer esteira com status `pending`.
2. **Poluição por Estados Pendentes Fantasmas**: Repositórios com execuções concluídas no passado permaneceram com o campo `status: "pending"` no catálogo em disco (mesmo com descrições como `"Successful in 2m"`). Com a priorização por `CiStatus`, esses repositórios ficavam eternamente presos no topo do monitor como pendentes há semanas.
3. **Ausência da Resolução Real de Commit Status**: O monitor consultava apenas a fila dos executores ativos (`/api/v1/user/actions/runners/jobs`), sem consultar o endpoint canônico de commit status da forja (`/api/v1/repos/{owner}/{repo}/commits/{branch}/status`), impedindo a visualização do resultado final consolidado das esteiras (sucesso/falha/em execução).

### 2. Requisitos Técnicos

1. **Ordenação por Atividade Recente (`CiTopSort::RecentUpdate`)**:
   - Adicionar variante de ordenação `CiTopSort::RecentUpdate` ("updated" / "recente") e torná-la a ordenação padrão ou primária no `repo top`.
   - Ordenar os repositórios pelo carimbo de data/hora mais recente de atualização (seja obtido da forja ou do timestamp local/catálogo).
2. **Resolução de Commit Status na Forja (`src/forges.rs`)**:
   - Implementar método no `ForgeClient` para consultar o commit status de um repositório na forja: `fetch_repo_commit_status(host, owner, repo, git_ref, token)`.
   - Mapear a resposta JSON de `/api/v1/repos/{owner}/{repo}/commits/{ref}/status`:
     - `"success"` -> `status: "success"` (renderizado como `✓ pass` verde)
     - `"failure"` / `"error"` -> `status: "failure"` (renderizado como `✗ fail` vermelho)
     - `"pending"` -> `status: "pending"` / `"running"` (renderizado como `~ running` amarelo/ciano)
     - Ausente ou vazio -> sem esteira (`-`)
3. **Integração no Ciclo de Background do `repo_top.rs`**:
   - No ciclo de atualização em segundo plano (`trigger_background_sync` e `refresh_data`), sincronizar o commit status real das forjas para os repositórios do catálogo.
   - Saneamento de status fantasmas: ao consolidar resumos, descrições contendo termos de sucesso (`"Successful in..."`) com status residual `"pending"` sem job ativo nos executores devem ser normalizadas para `"success"`.
4. **Renderização Visual no `render_top_view`**:
   - A coluna de ESTADO CI deve refletir o status canônico obtido da forja de forma límpida:
     - `✓ pass` (verde)
     - `✗ fail` (vermelho)
     - `~ running` (amarelo/ciano)
     - `-` (cinza/dim)

### 3. Critérios de Aceitação
- Testes unitários em `tests/forges_test.rs` comprovam o parsing e mapeamento correto de respostas do commit status da forja.
- Testes unitários em `tests/repo_top_test.rs` validam a nova opção de ordenação `RecentUpdate` e o saneamento de status pendentes fantasmas.
- A suíte de testes de `repo_top` e `forges` passa com 100% de sucesso.
- Conformidade estrita com `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**:
  - **[Resumo]**: Espelhamento da lista de repositórios atualizados recentemente e do status canônico da esteira (CI) da forja no monitor `repo top`.
  - **[Solução]**:
    1. Implementada a variante `CiTopSort::RecentUpdate` ("recent" / "updated") e definida como ordenação padrão no `repo top`.
    2. Adicionado o método `fetch_repo_commit_status` e `parse_forgejo_commit_status_json` em `src/forges.rs` para resolução direta do commit status na forja (`/api/v1/repos/{owner}/{repo}/commits/{ref}/status`).
    3. Implementada a função `sanitize_pipeline_statuses` em `src/repo_top.rs` para expurgar status residuais "pending" de esteiras finalizadas no catálogo.
    4. Atualizado o ciclo em segundo plano do monitor para sincronizar os commit statuses reais da forja.
  - **[Validação]**:
    - Suíte com 31 testes em `tests/forges_test.rs` e 29 testes em `tests/repo_top_test.rs` (60 testes no total) aprovada com 100% de sucesso.
    - `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` 100% conformes com zero avisos.
