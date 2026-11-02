# Especificação Técnica: Ticket #126
## fix(ci): migrar para endpoint nativo de status consolidado e preservar tempo na fila

### 1. Contexto e Problema
No `cccp`, a sincronização de status de CI/CD para repositórios remotos (`fetch_ci_status` em `src/forges.rs`) consultava o endpoint de eventos brutos de commit:
`https://{host}/api/v1/repos/{owner}/{repo}/commits/{ref}/statuses` (no plural).
Esse endpoint é um append-only log de eventos históricos. Nele, cada job registra eventos parciais intermediários como `"Has started running"`.
Isso causava dois problemas críticos:
1. **Status Fantasma de "Running"**:
   Workflows que já haviam sido finalizados (ou que tiveram jobs pulados) mantinham entradas antigas com a descrição `"Has started running"`. Como o leitor de JSON não garantia a ordenação pelo evento final e aplicava deduplicação ingênua, repositórios finalizados ou com jobs pulados (como `lacraia`) continuavam sendo exibidos como `running`.
2. **Desalinhamento com a Interface Web do Forgejo**:
   A interface web do Forgejo utiliza o endpoint consolidado oficial:
   `https://{host}/api/v1/repos/{owner}/{repo}/commits/{ref}/status` (no singular).
   Esse endpoint já consolida o estado global do commit (`state: "failure" | "success" | "pending"`) e entrega no campo `statuses` a lista única de jobs no seu estado final consolidado, sem duplicatas intermediárias e com timestamps precisos (`created_at`, `updated_at`).

Além disso, ao sincronizar jobs ativos de executores no monitor interativo (`repo_top`), é fundamental preservar os timestamps `created_at` e `updated_at` para que a coluna de tempo na fila (`CI Time`) reflita com precisão o tempo real de espera ou execução.

### 2. Requisitos Técnicos
1. **Migrar URL para Endpoint Nativo Consolidado (`src/forges.rs`)**:
   - Para provedores `Forgejo`, `Codeberg`, `Tildegit` e `GitHub`, utilizar a rota singular:
     `https://{host}/api/v1/repos/{owner}/{repo}/commits/{ref}/status` (Forgejo/Gitea)
     `https://api.github.com/repos/{owner}/{repo}/commits/{ref}/status` (GitHub).
2. **Suportar o Payload Consolidado de Status (`src/forges.rs`)**:
   - O payload retornado é um objeto JSON que contém:
     - `state`: o estado consolidado da esteira.
     - `statuses`: array contendo os jobs no estado final consolidado.
   - O leitor deve suportar extrair os itens tanto de `body["statuses"]` (quando for um objeto) quanto de `body` diretamente (caso seja um array, garantindo retrocompatibilidade).
   - Extrair `updated_at` ou `created_at` e preservá-lo em `CiPipelineStatus`.
3. **Preservação de Timestamp no Merge de Jobs Ativos (`src/forges.rs` e `src/repo_top.rs`)**:
   - Em `merge_live_runner_jobs`, garantir que ao associar jobs ativos do executor (`running` ou `waiting`), caso o job da API do executor não contenha timestamp direto, o timestamp `updated_at` já existente na esteira do repositório seja preservado.

### 3. Critérios de Aceitação
- Testes unitários validam o parsing do objeto de status consolidado (`/commits/{ref}/status`) contendo objeto com `state` e array `statuses`.
- Testes unitários comprovam que estados finais (`failure`, `skipped`, `success`) sobrepõem corretamente e não geram duplicatas com status `running`.
- Testes unitários validam que `updated_at` é extraído e utilizado por `format_ci_elapsed_time` para o cálculo correto do tempo na fila.
- `cargo fmt --check` e `cargo clippy --all-targets -- -D warnings` sem erros.

### 4. Status de Resolução (Fase 9)
- **Status**: Concluído e Aprovado.
- **Laudo**:
  - **[Resumo]**: Migração da consulta de status de commit para o endpoint consolidado oficial do Forgejo/GitHub e preservação integral dos timestamps de fila.
  - **[Solução]**:
    1. Alteração da rota de status de `/commits/{ref}/statuses` para `/commits/{ref}/status` em `src/forges.rs`.
    2. Implementação da função `parse_ci_status_json` que extrai os jobs finais consolidados e seus respectivos `updated_at` e `created_at`.
    3. Preservação do timestamp em `merge_live_runner_jobs` durante atualizações ao vivo dos executores.
  - **[Validação]**:
    - Suíte de 25 testes em `tests/forges_test.rs` e 23 testes em `tests/repo_top_test.rs` (total de 48 testes) aprovados com 100% de sucesso.
    - `cargo fmt --check` e `cargo clippy -j 1 --all-targets -- -D warnings` sem nenhum aviso.

