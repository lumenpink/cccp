# Especificação Técnica: Ticket #139
## fix(ci): respeitar estado consolidado da forja e expurgar falsos pendentes

### 1. Contexto e Problema
Na interface web do Forgejo (Gambi), repositórios exibem com precisão o estado consolidado da esteira de integração contínua (CI), como `✓ pass` (verde) ou `✗ fail` (vermelho). No entanto, o `cccp repo top` vinha exibindo diversos repositórios como `~ waiting` ou `~ pending`, mesmo com a forja completamente ociosa (zero runners ativos).

As causas raízes identificadas são:
1. **Descarte do estado raiz consolidado (`state`) no parsing de commit status**:
   - O payload do endpoint canônico `/api/v1/repos/{owner}/{repo}/commits/{ref}/status` do Forgejo contém um atributo raiz `"state"` (ex.: `"success"`, `"failure"`, `"pending"`) que representa a conclusão combinada de todas as checagens e fluxos de trabalho.
   - Em `src/forges.rs:parse_forgejo_commit_status_json`, se o array `"statuses"` contiver itens individuais, esses sub-jobs eram processados cegamente.
2. **Sub-jobs pulados ou bloqueados classificados como pendentes ativos**:
   - Em esteiras com múltiplos jobs ou dependências condicionais (`needs:`), quando um job falha ou termina, os jobs subsequentes recebem da forja descrições como `"Blocked by required conditions"` ou `"Has been skipped"`, com estados brutos como `"waiting"`, `"blocked"` ou `"pending"`.
   - Esses sub-jobs eram importados como `status: "waiting"` ou `status: "pending"`, tratando jobs pulados/bloqueados como se estivessem aguardando execução na fila.
3. **Inversão de precedência em `format_ci_summary` e `repo_ci_priority`**:
   - Em `src/catalog.rs:format_ci_summary`, a checagem de `waiting > 0`, `blocked > 0` e `pending > 0` era realizada *antes* de checar `failed > 0` ou avaliar o sucesso geral.
   - Consequentemente, se uma esteira falhasse e os 3 jobs seguintes fossem pulados (`"Blocked by required conditions"`), o monitor ignorava a falha e rotulava o repositório como `~ waiting`.
   - Da mesma forma, se a forja reportou `"state": "success"`, a existência de qualquer sub-job residual pendente/pulado estragava o status consolidado.

### 2. Requisitos Técnicos

1. **Precedência do Estado Canônico Raiz em `parse_forgejo_commit_status_json` (`src/forges.rs`)**:
   - O campo raiz `"state"` do payload do Forgejo deve ser respeitado como a verdade canônica da esteira consolidada.
   - Se `root_state == "success"`, nenhum sub-job secundário pode reverter o estado geral para `waiting` ou `pending`.
   - Se `root_state == "failure"`, o estado geral consolidado deve refletir `failure` (`✗ fail`).
2. **Expurgo de Sub-jobs Pulados ou Bloqueados (`src/forges.rs` e `src/repo_top.rs`)**:
   - Itens individuais cujo status seja `waiting`, `blocked` ou `pending`, mas cuja descrição contenha `"Blocked by required conditions"`, `"Has been skipped"`, `"skipped"` ou `"cancelled"`, não devem ser computados como fila ativa de espera.
   - Eles devem ser mapeados para `"skipped"` ou ignorados no cômputo da fila.
3. **Correção de Precedência em `src/catalog.rs:format_ci_summary` e `src/repo_top.rs:repo_ci_priority`**:
   - Falha consolidada (`failed > 0`) deve ter precedência sobre `waiting` decorrente de sub-jobs bloqueados/pulados.
   - Se todos os executores da forja estiverem ociosos (`running_jobs == 0`), pipelines em `waiting`/`pending` sem job correspondente na fila do runner não devem manter o repositório indefinidamente em `~ waiting`.
4. **Saneamento e Resiliência do Catálogo**:
   - `sanitize_pipeline_statuses` em `src/repo_top.rs` deve normalizar pipelines marcadas como `waiting`/`pending` com descrições de bloqueio/pulo ou quando o status raiz da forja for conclusivo (`success`/`failure`).

### 3. Critérios de Aceitação
- Testes unitários comprovam que um commit status com `"state": "success"` e sub-job `"Blocked by required conditions"` resulta em `✓ pass` verde.
- Testes unitários comprovam que um commit status com `"state": "failure"` e sub-job bloqueado resulta em `✗ fail` vermelho.
- `format_ci_summary` reflete corretamente falha ou sucesso sem ser mascarado por sub-jobs pulados.
- Testes passam com 100% de sucesso em `cargo test --test forges_test --test repo_top_test --test catalog_test`.
- Conformidade estrita com `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings`.

### 4. Status de Resolução (Fase 9)
- **Status**: Pendente de implementação (Fases 4 a 8).
