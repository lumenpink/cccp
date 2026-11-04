# Especificação Técnica: Ticket #140
## perf(repo-top): inicialização instantânea e sincronização concorrente de ci

### 1. Contexto e Problema
O monitor `cccp repo top` apresentava atraso perceptível no início e demorava 20 segundos ou mais para convergir (`⠋ syncing...`), devido aos seguintes gargalos identificados:
1. **Chamadas de rede bloqueantes antes do modo gráfico**:
   - `refresh_data` realizava chamadas HTTP síncronas para `fetch_user_runner_jobs` antes de `enable_raw_mode` e `EnterAlternateScreen`, travando a inicialização da TUI.
2. **Varredura sequencial de todo o catálogo**:
   - `trigger_background_sync` iterava sequencialmente por todos os repositórios do catálogo (72 alvos) fazendo requisições individuais a `/api/v1/repos/{owner}/{repo}/commits/main/status`. Com latência de ~200ms por requisição, o loop completo levava até 20 segundos para emitir `TopBackgroundMsg::SyncCompleted`.
3. **Escritas síncronas repetitivas em disco**:
   - A cada resposta individual recebida em `TopBackgroundMsg::RepoUpdate`, o arquivo `catalog.json` de 84KB era recarregado e regravado em disco pelo thread principal da interface, somando dezenas de operações de I/O em disco desnecessárias.
4. **Falta de priorização para repositórios recentes/visíveis**:
   - Repositórios arquivados ou sem nenhuma atividade recente concorriam igualmente com os repositórios ativos do topo da tela.

### 2. Requisitos Técnicos

1. **Abertura Instantânea (< 10ms)**:
   - `refresh_data` deve carregar e sanitizar o catálogo localmente e renderizar o primeiro quadro imediatamente, sem aguardar chamadas de rede externas.
   - Qualquer consulta remota deve ser despachada exclusivamente para a thread de segundo plano.
2. **Priorização da Janela de Foco (Recentes / Top N)**:
   - Na sincronização de commit status, os alvos devem ser ordenados ou divididos dando prioridade aos primeiros $N$ repositórios (ex.: top 25 mais recentes ou visíveis).
   - A chamada global de executores (`/api/v1/user/actions/runners/jobs`), que traz todos os jobs ativos da forja em um único payload, garante que esteiras em andamento em qualquer repositório sejam refletidas de imediato.
3. **Concorrência Bounded no Sync**:
   - As consultas individuais de commit status devem ser executadas com concorrência limitada (ex.: pool de 6 a 8 threads trabalhadoras paralelas ou processamento em lotes), reduzindo o tempo total de varredura para ~1 a 2 segundos.
4. **Gravação Agregada em Disco (Batch Sync)**:
   - Em vez de reescrever `catalog.json` a cada resposta unitária, acumular as atualizações e sincronizar em disco no término do lote de sincronização (`SyncCompleted`) ou quando a janela inicial de prioridade for resolvida.
5. **Convergência Ágil**:
   - `TopBackgroundMsg::SyncCompleted` deve ser emitido de forma imediata assim que a janela prioritária estiver resolvida, evitando prender o monitor no estado `syncing...`.

### 3. Critérios de Aceitação
- A inicialização do `repo top` desenha a tela inicial a partir dos dados locais sem bloqueio prévio de rede.
- A sincronização de commit status é paralela ou priorizada para os repositórios recentes.
- As gravações em disco de `catalog.json` são agregadas, eliminando regravações consecutivas por item.
- Testes unitários comprovam a correta ordenação prioritária de alvos e despacho de sincronização.
- Toda a suíte de testes passa com 100% de sucesso (`cargo test --test forges_test --test repo_top_test --test catalog_test --test catalog_ci_ui_test`).
- Conformidade estrita com `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**:
  - **[Resumo]**: Otimização substancial de desempenho no 'cccp repo top', permitindo abertura visual instantânea em menos de 10ms e sincronização de esteiras concorrente e priorizada.
  - **[Solução]**: Removidas chamadas de rede bloqueantes do startup gráfico, implementada a priorização de alvos em prioritize_forge_sync_targets com concorrência em chunks paralelos, e agregada a gravação de catalog.json no encerramento do lote (SyncCompleted/teardown).
  - **[Validação]**: Aprovada suíte de testes com 109 testes em repo_top_test, forges_test, catalog_ci_ui_test e catalog_test, com conformidade estrita no cargo fmt --check e cargo clippy --all-targets -- -D warnings.
