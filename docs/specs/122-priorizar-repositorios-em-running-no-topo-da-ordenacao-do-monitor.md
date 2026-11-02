# Especificação Técnica: Ticket #122
## feat(ci): priorizar repositorios em running no topo da ordenacao do monitor

### 1. Contexto e Problema
Na interface interativa do monitor (`cccp repo top`), a ordenação por status de CI (`CiTopSort::CiStatus`) agrupava todos os repositórios com pipelines ativas (`running`, `waiting`, `blocked`, `pending`) na mesma categoria `"queued"` com prioridade 0.
Quando múltiplos repositórios estavam na fila, o critério de desempate utilizava unicamente o ID numérico do runner job (`runner_job_id`), sem diferenciar se o job estava ativamente compilando no runner ou apenas aguardando sua vez na fila de espera.
Com isso, repositórios que já estavam em execução ativa (`running`) podiam ser exibidos abaixo de repositórios que estavam apenas em espera (`waiting`), caso estes possuíssem IDs anteriores.

### 2. Requisitos Técnicos
1. **Hierarquia Granular de Prioridade de Ordenação (`src/repo_top.rs`)**:
   - Em `apply_top_filters_and_sorting`, estruturar a função de prioridade de CI:
     - **Prioridade 0 (Topo Absoluto)**: Repositórios com pelo menos um job com status `"running"` ou `"progress"`.
     - **Prioridade 1**: Repositórios com pelo menos um job com status `"waiting"` ou `"queued"` (e sem jobs em `running`).
     - **Prioridade 2**: Repositórios com jobs com status `"blocked"` ou `"pending"` (e sem jobs em `running`/`waiting`).
     - **Prioridade 3**: Repositórios com falhas (`"fail"` / `"failure"` / `"error"`).
     - **Prioridade 4**: Repositórios com todos os jobs aprovados (`"ok"` / `"success"`).
     - **Prioridade 5**: Repositórios sem esteiras registradas (`"none"`).
2. **Critério de Desempate Intra-Nível**:
   - Entre repositórios no mesmo nível de prioridade (ex.: dois repositórios em `running`, ou múltiplos em `waiting`):
     - Ordenar pelo menor `runner_job_id` (ordem de entrada na fila do runner).
     - Em caso de empate ou ausência de ID, ordenar alfabeticamente pelo nome do repositório (`name.to_lowercase()`).
3. **Compatibilidade com Filtros de CI**:
   - Os filtros `CiTopFilter::Queued` e `CiTopFilter::FailOrPending` continuam selecionando repositórios em `running`, `waiting`, `blocked` e `pending`.

### 3. Critérios de Aceitação
- Em `cccp repo top` ordenado por CI, qualquer repositório em `running` fica posicionado estritamente acima de repositórios em `waiting`, `blocked`, `fail` ou `ok`.
- Repositórios em `waiting` ficam posicionados acima de repositórios em `blocked`, `fail` ou `ok`.
- Testes unitários comprovam que mesmo se um repositório em `waiting` tiver um ID de job menor (ex.: job #10 em waiting vs job #20 em running), o repositório em `running` é ordenado primeiro.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` aprovados com 100% de sucesso.

### 4. Status de Resolução
- **Status**: Em implementação (Fase 3).
