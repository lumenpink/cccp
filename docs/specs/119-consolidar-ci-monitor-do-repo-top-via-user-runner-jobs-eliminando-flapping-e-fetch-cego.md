# Especificação Técnica: Ticket #119
## fix(ui): consolidar ci monitor do repo top via user runner jobs eliminando flapping e fetch cego

### 1. Contexto e Problema
Na interface interativa de monitoramento do CCCP (`cccp repo top`), foram identificados comportamentos conflitantes e ineficientes:
1. **Conflito de Estados (*Flapping*) entre "fail" e "fila"**:
   - A cada ciclo, os jobs ativos do runner (`/api/v1/user/actions/runners/jobs`) marcavam o repositório como `running` ou `waiting`.
   - Concomitantemente, uma thread em segundo plano consultava os status do commit (`/commits/main/statuses`) para cada repositório, recebendo o status histórico da execução anterior (`failure`).
   - Ao receber o resultado do commit, o código descartava o runner ativo e rebaixava o status para `failure`. No ciclo seguinte, a fila de runners sobrescrevia novamente para `running`. Isso gerava alternância caótica e constante reordenação da lista.
2. **Duplicação de Esforços e Lentidão Extrema**:
   - Uma varredura cega executava `git fetch` e chamadas HTTP a `/commits/main/statuses` para todos os 74 repositórios do catálogo sequencialmente. O processo levava mais de 30 segundos.
   - Como o intervalo padrão do monitor é de 10 segundos, novas threads eram disparadas antes do término da anterior, tornando o estado de convergência (`✓ convergente`) inalcançável.
3. **Desacoplamento dos Filtros**:
   - Mesmo com filtros aplicados (ex.: exibindo apenas 2 repositórios em falha ou fila), o processo continuava tentando sincronizar os 74 repositórios do catálogo em segundo plano.

### 2. Requisitos Técnicos
1. **Monitoramento Consolidado via Runner Jobs (`src/repo_top.rs`)**:
   - Substituir a varredura sequencial de 74 `git fetch` e 74 chamadas de commit por uma única chamada consolidada e ultrarrápida a `/api/v1/user/actions/runners/jobs` por host de forja.
   - Mesclar o estado em tempo real dos runners sobre o catálogo base em memória (`merge_live_runner_jobs`).
   - Se uma pipeline possuir status ativo (`running`, `waiting`, `pending`), este estado possui precedência absoluta e não pode ser sobrescrito por status históricos de commits.
2. **Ciclo de Atualização e Convergência Limpa**:
   - A sincronização em segundo plano deve consultar a fila consolidada de runners e atualizar apenas repositórios que sofreram transição de estado.
   - Ao concluir a consulta consolidada (< 200 ms), o monitor atinge o estado de convergência (`has_converged = true`, `is_syncing = false`).
   - O temporizador de intervalo (`state.interval_secs`) só reinicia após a conclusão da sincronização, garantindo que o indicador `✓ convergente` permaneça visível e estável durante todo o intervalo configurado.
3. **Respeito aos Filtros e Atualização Sob Demanda**:
   - O filtro de tela (`Ctrl+F`) opera diretamente na projeção em memória dos repositórios, de forma imediata e sem disparar requisições em massa desnecessárias.
   - O atalho de atualização manual (`Ctrl+R`) força nova consulta imediata aos runners e recarrega o catálogo.

### 3. Critérios de Aceitação
- Ausência de alternância errática (*flapping*) entre status de runner ativo e falha histórica em repositórios monitorados.
- O estado de convergência (`✓ convergente`) é alcançado de forma imediata e permanece ativo durante o intervalo estabelecido.
- A sincronização não executa comandos `git fetch` sequenciais em segundo plano para todo o catálogo durante a exibição do monitor.
- Testes unitários comprovam a estabilidade do estado de convergência e a integridade da mesclagem de status.

### 4. Status de Resolução (Fase 9)
- **Status**: Em andamento.
- **Laudo**: Pendente de implementação e validação das 9 fases de GOST DD-TDD.
