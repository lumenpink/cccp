# Especificação Técnica: Ticket #121
## feat(ci): normalizar status pending via description e cor ciano para running

### 1. Contexto e Problema
Na integração com o Forgejo / Gitea Actions:
1. O endpoint canônico de commit statuses (`GET /api/v1/repos/{owner}/{repo}/commits/{ref}/statuses`) retorna `status: "pending"` para qualquer esteira ou job que ainda não esteja em estado terminal (sucesso ou falha).
2. O detalhamento do estado real da execução é incluído pelo Forgejo exclusivamente no campo `description`:
   - `"Waiting to run"` -> tarefa pronta na fila aguardando runner.
   - `"Has started running"` -> tarefa em execução ativa no runner.
   - `"Blocked by required conditions"` -> tarefa bloqueada aguardando dependências (`needs`).
3. A função `fetch_ci_status` em `src/forges.rs` lia unicamente o valor `"pending"`, fazendo com que comandos como `cccp repo list`, o cache do catálogo e o estado inicial do `repo top` exibissem `~ pending` mesmo quando a tarefa já estava em `waiting` ou `running`.
4. Além disso, em `format_ci_summary` (`src/catalog.rs`) e na renderização do `repo top` (`src/repo_top.rs`), tanto `running` quanto `waiting` e `blocked` utilizavam a mesma cor amarela (`"yellow"`), dificultando a identificação imediata de qual repositório está ativamente compilando no runner vs apenas aguardando na fila.

### 2. Requisitos Técnicos
1. **Normalização em `fetch_ci_status` (`src/forges.rs`)**:
   - Quando o `status` (ou `state`) retornado for `"pending"`:
     - Se `description` contiver ou iniciar por `"Waiting to run"` (case-insensitive): normalizar `status` para `"waiting"`.
     - Se `description` contiver ou iniciar por `"Has started running"`: normalizar `status` para `"running"`.
     - Se `description` contiver ou iniciar por `"Blocked by required conditions"`: normalizar `status` para `"blocked"`.
     - Caso contrário, preservar `"pending"`.
   - Criar função utilitária `normalize_commit_status(status: &str, description: Option<&str>) -> String` para testabilidade unitária estrita.
2. **Destaque Visual de `running` com Cor Ciano (`src/catalog.rs` e `src/repo_top.rs`)**:
   - Em `format_ci_summary`:
     - Se `running > 0`: retornar a cor `"cyan"` (em vez de `"yellow"`).
     - Se `waiting > 0`: manter `"yellow"`.
     - Se `blocked > 0`: manter `"yellow"`.
     - Se `pending > 0`: manter `"yellow"`.
   - Em `format_catalog_ci_details`:
     - Para pipelines com status `running`: renderizar com glifo `~` em ciano (`theme.cyan`).
   - Em `repo_top.rs`:
     - No match de cores da coluna CI, mapear `"cyan"` para `theme.cyan(&ci_text)`.

### 3. Critérios de Aceitação
- Commits com descrições do Forgejo `"Waiting to run"`, `"Has started running"` e `"Blocked by required conditions"` são normalizados para `"waiting"`, `"running"` e `"blocked"`, respectivamente.
- Repositórios com jobs executando ativamente (`running > 0`) têm seu resumo formatado com cor `"cyan"`.
- O `repo top` renderiza o resumo em ciano para jobs em `running`.
- Repositórios em `waiting` ou `blocked` mantêm a cor amarela.
- Suíte de testes unitários e de integração aprovada sem regressões.

### 4. Status de Resolução
- **Status**: Em implementação (Fase 3).
