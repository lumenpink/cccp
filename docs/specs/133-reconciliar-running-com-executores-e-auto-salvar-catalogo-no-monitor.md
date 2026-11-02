# Especificação Técnica: Ticket #133
## fix(ci): reconciliar running com executores e auto salvar catalogo no monitor

### 1. Contexto e Problema
No `cccp repo top`, as esteiras de CI dos repositórios são carregadas a partir de `catalog.json` e mescladas em tempo real com a lista consolidada de jobs ativos retornada pelos executores da forja (`fetch_user_runner_jobs`).
Entretanto, foram observadas duas fragilidades:
1. **Running Fantasma por Herança de Disco**:
   Se o arquivo `catalog.json` em disco contiver registros gravados no passado com `status: "running"` para esteiras que já foram concluídas ou canceladas, e essas esteiras não estiverem mais na lista de jobs ativos do executor, a função `merge_live_runner_jobs` não as alterava. Isso fazia com que o monitor continuasse exibindo repositórios com `~ running` indefinidamente.
   A lista de jobs ativos do executor (`/api/v1/user/actions/runners/jobs`) deve ser a **autoridade máxima de verdade**: se um repositório não possui nenhum job sendo executado no momento nos executores, ele não pode ser exibido como `running`.
2. **Dessincronização do Catálogo em Disco**:
   O monitor atualizava os resumos apenas em memória (`raw_summaries`), deixando o arquivo `catalog.json` em disco desatualizado até que comandos manuais como `cccp repo update-tickets` fossem executados. Ao persistir automaticamente as esteiras atualizadas em disco no ciclo de background do `repo top`, o catálogo permanece sempre quente e sincronizado para todos os demais comandos da CLI (`repo list`, etc.).

### 2. Requisitos Técnicos
1. **Reconciliação Estrita no `merge_live_runner_jobs` (`src/forges.rs`)**:
   - Mapear todos os jobs que estão **genuinamente** no estado `"running"` reportados pela forja.
   - Para qualquer repositório no catálogo que possua pipelines marcadas como `"running"`, se nenhuma dessas pipelines coincidir com um job ativo em execução real no executor, despromover/reconciliar esse status fantasma (ex.: para `"pending"` caso haja jobs aguardando na fila, ou ajustando para o estado consolidado real).
   - Garantir que a quantidade de repositórios em `running` no monitor nunca exceda o número de jobs em execução real nos executores.
2. **Auto-Persistência do Catálogo no Monitor (`src/repo_top.rs` e `src/catalog.rs`)**:
   - No ciclo de atualização em segundo plano do `repo top` (ao receber `RunnerJobsUpdated` ou ao sincronizar esteiras), salvar automaticamente as esteiras consolidadas no arquivo `catalog.json` em disco de forma não bloqueante.

### 3. Critérios de Aceitação
- Testes unitários comprovam que `merge_live_runner_jobs` expurga status `running` fantasma herdado de registros prévios quando o executor não reporta nenhum job em execução para aquele repositório.
- Testes unitários comprovam que quando um job legítimo está em `running`, apenas ele e seu respectivo repositório são mantidos como `running`.
- Testes unitários validam a atualização persistente do catálogo no monitor.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem avisos ou erros.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**:
  - **[Resumo]**: Reconciliação autoritativa do status running com os executores e auto-salvamento do catálogo em disco no monitor.
  - **[Solução]**:
    1. Implementada a reconciliação em `merge_live_runner_jobs` (`src/forges.rs`), onde apenas repositórios com jobs realmente em execução pelos executores mantêm o estado `running`; qualquer status anterior herdado do disco é expurgado/reconciliado.
    2. Implementada a função `auto_sync_catalog_ci_status` em `src/repo_top.rs`, que persiste automaticamente o estado reconciliado de CI no arquivo `catalog.json` sempre que há alterações.
  - **[Validação]**:
    - Suíte de 29 testes em `tests/forges_test.rs` e 24 testes em `tests/repo_top_test.rs` (53 testes no total) aprovados com 100% de sucesso.
    - `cargo fmt --check` e `cargo clippy -j 1 --all-targets -- -D warnings` 100% limpos.

