# Especificação Técnica: Sincronização Live da Forja, Filtro Fail+Pending e Ordenação por Fila no CCCP Repo Top

**Ticket Associado:** #105  
**Branch:** `fix/105-sincronizacao-live-da-forja-filtro-fail-pending-e-ordenacao-por-fila`  
**Norma:** GOST 2.0.0

## 1. Objetivo
Ajustar o monitor interativo `cccp repo top` para refletir o status real e em tempo real dos pipelines de CI a partir da API da forja, incluir o filtro combinado de falhas e esteiras pendentes/em execução (`fail+pending`) e definir a ordenação padrão priorizando jobs pendentes (por ordem na fila do runner), seguidos de falhas e sucessos.

## 2. Requisitos Técnicos

1. **Resolução de Tokens do CLI `tea` (`src/forges.rs`)**:
   - `resolve_token_for_host` deve ler credenciais de `~/.config/tea/config.yml` (ou `$XDG_CONFIG_HOME/tea/config.yml`) para instâncias Forgejo/Gitea (ex.: `gambi.2lp.in`).
   - Isso permite que `fetch_user_runner_jobs` e consultas autenticadas de CI funcionem de forma transparente sem exigir variáveis de ambiente manuais.

2. **Filtro Conjunto `fail+pending` (`CiTopFilter`)**:
   - Adicionar a variante `FailOrPending` ao enum `CiTopFilter`.
   - Suportar parsing de strings: `"fail+pending"`, `"pending+fail"`, `"fail-pending"`, `"issues"`, `"alertas"`, `"falha+pendente"`, `"pendente+falha"`.
   - No ciclo de filtros (`next()`), incluir a opção após `All`.
   - O filtro seleciona repositórios cujo status de CI seja `fail` ou `queued` (pending/running/waiting).

3. **Ordenação Padrão Prioritária com Desempate por Fila (`CiTopSort` e `apply_top_filters_and_sorting`)**:
   - Alterar a ordenação padrão em `RepoTopState::new` para a ordenação por status de CI prioritária.
   - A ordem de precedência de status de CI deve ser:
     1. `queued` / `pending` (jobs em execução ou aguardando na fila)
     2. `fail` (falhas)
     3. `ok` (sucessos)
     4. `none` (sem esteira)
   - Dentro de repositórios em `pending`, a ordenação deve respeitar a ordem na fila do runner (jobs com menor ID ou posição de entrada anterior na lista de runner jobs vêm primeiro).

4. **Sincronização Live dos Status de Repositórios**:
   - No `refresh_data` do `repo_top`, além de consultar `fetch_user_runner_jobs`, atualizar os status dos repositórios a partir da API da forja para que conclusões de esteiras (como sucessos recentes de `lumenlohn` e `kokorosim`) reflitam no monitor sem congelar no estado estático antigo.

## 3. Critérios de Aceitação
- Testes unitários para resolução de token via configuração do `tea`.
- Testes unitários para o filtro `CiTopFilter::FailOrPending`.
- Testes unitários para a ordenação `pending -> fail -> pass` e ordenação por fila do runner.
- Suite de testes passando (`cargo test`) e conformidade estrita (`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`).

## 4. Conclusão e Laudo
- [Resumo]: Implementação de sincronização live da forja, filtro `fail+pending` e ordenação padrão `pending/fail/pass` ordenada por fila no `repo top`.
- [Solução]: Token discovery via tea config, variante `FailOrPending` no `CiTopFilter`, reordenação prioritária no `CiTopSort` com queue index, e atualização de CI status em tempo real.
- [Validação]: Testes automatizados cobrindo os novos cenários, `cargo test` e `cargo clippy` 100% aprovados.
