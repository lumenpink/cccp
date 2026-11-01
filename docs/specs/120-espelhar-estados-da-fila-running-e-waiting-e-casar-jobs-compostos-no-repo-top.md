# Especificação Técnica: Ticket #120
## feat(ci): espelhar estados da fila running e waiting e casar jobs compostos no repo top

### 1. Contexto e Problema
Na interface de monitoramento do CCCP (`cccp repo top` e `cccp repo list`), foram identificadas divergências na representação dos estados de CI em relação à interface web do Forgejo Actions:
1. **Casamento de Nomes Compostos de Jobs**:
   - Os status de commit gerados pelo Forgejo registram o nome com o contexto completo da esteira:
     `"<Workflow> / <Job Name> (<event>)"` (exemplo: `"CI, Quality & Release / Rust — Checks, Tests & Quality Gates (push)"`).
   - O endpoint consolidado de runners (`/api/v1/user/actions/runners/jobs`) fornece apenas o nome puro do job (`"Rust — Checks, Tests & Quality Gates"`).
   - A função `merge_live_runner_jobs` realizava comparação estrita de igualdade (`p.name == job.name`), impedindo a atualização de pipelines existentes e inserindo entradas duplicadas.
2. **Precedência e Distinção de Estados da Fila**:
   - As funções `format_ci_summary` e `repo_ci_category` priorizavam `failed > 0` antes de verificar jobs em andamento ou em fila.
   - Quando um repositório possuía falhas de uma execução anterior e um novo job entrava na fila (como no caso do `joaju-pdf`), ele continuava sendo exibido como `✗ fail`, mascarando a nova execução.
   - Não havia diferenciação entre jobs ativamente executando (`running`) e jobs aguardando disponibilidade de runner (`waiting`), agrupando tudo sob o termo genérico `~ pending`.

### 2. Requisitos Técnicos
1. **Casamento Inteligente de Jobs (`src/forges.rs`)**:
   - Implementar a função utilitária `pipeline_matches_job_name(pipeline_name: &str, job_name: &str) -> bool`.
   - A função deve suportar tanto correspondência exata quanto extração do `<Job Name>` a partir do formato canônico `<Workflow> / <Job Name> (<event>)`.
   - Em `merge_live_runner_jobs` e `apply_repo_ci_update`, utilizar `pipeline_matches_job_name` para atualizar a pipeline existente correspondente.
2. **Espelhamento Fiel dos Estados da Fila (`src/catalog.rs`)**:
   - Em `format_ci_summary`:
     - Se `running > 0`: exibir `~ running` (ou `~ running (x/y)`) em amarelo (`yellow`).
     - Se `waiting > 0` (e `running == 0`): exibir `~ waiting` (ou `~ waiting (x/y)`) em amarelo (`yellow`).
     - Se `blocked > 0` (e `running == 0 && waiting == 0`): exibir `~ blocked` em amarelo (`yellow`).
     - Se `failed > 0` (e nenhum job ativo na fila): exibir `✗ fail` em vermelho (`red`).
     - Se todos concluídos: exibir `✓ pass` em verde (`green`).
   - Em `format_catalog_ci_details`: exibir os ícones correspondentes (`~` em amarelo para `running` e `waiting`).
3. **Categorização e Ordenação no Repo Top (`src/repo_top.rs`)**:
   - Em `repo_ci_category`: verificar se há jobs em fila/execução (`running`, `waiting`, `pending`, `progress`, `blocked`) com prioridade sobre falhas passadas, retornando `"queued"`.
   - Repositórios com novas execuções ativas sobem imediatamente para o topo da listagem interativa (prioridade 0).

### 3. Critérios de Aceitação
- Repositórios com jobs aguardando runner exibem `~ waiting` em amarelo e são categorizados como `queued`.
- Repositórios com jobs em execução exibem `~ running` em amarelo.
- O casamento de nomes compostos reconhece corretamente jobs do Forgejo e atualiza a pipeline existente sem duplicação.
- O repositório `joaju-pdf` (e similares) reflete o estado `~ waiting` na fila em vez de `✗ fail`.
- Todos os testes unitários e de integração são aprovados.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**:
  - **[Resumo]**: Espelhamento fiel dos estados da fila do Forgejo (running, waiting, blocked, pending) e eliminação de duplicidades via casamento de jobs compostos no repo top.
  - **[Solução]**: Implementação de `pipeline_matches_job_name`, espelhamento dos estados com ícones mono-largura em amarelo (`format_ci_summary`) e precedência estrita de jobs ativos sobre falhas históricas em `repo_ci_category` e `apply_repo_ci_update`.
  - **[Validação]**: Suíte completa de testes (`catalog_ci_ui_test`, `repo_top_test`), validação de mutação com cenário real `joaju-pdf`, `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` 100% aprovados.
