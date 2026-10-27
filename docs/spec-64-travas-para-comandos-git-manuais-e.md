# Especificação Técnica — Ticket #64: Travas para Comandos Git Manuais e Paridade de Push

## 1. Visão Geral e Motivação
A especificação GOST v2.0.0 e o perfil Kollontai v2.0.0 definem o Maestro CCCP como o orquestrador autoritativo do ciclo de desenvolvimento. Invocações manuais e diretas da CLI do `git` (como `git commit` ou `git push`) contornam as travas de fase do ciclo DD-TDD, as validações de esteira e a geração determinística do Laudo de Fechamento.

Adicionalmente, rejeições de `git push` causadas por commits concorrentes no upstream criam atrito manual desnecessário. O comando `cccp push` deve inspecionar automaticamente a paridade com a branch remota (`origin`), reconciliando divergências simples (fast-forward) ou bloqueando o envio antes de rejeições remotas ruidosas.

---

## 2. Requisitos Técnicos

### 2.1 Travas de Proteção contra Execução Manual de `git`
1. **Hook `pre-commit`**:
   - Inspeciona a variável de ambiente `CCCP_MAESTRO_ACTIVE`.
   - Se for igual a `"1"`, permite a execução normal do hook (sincronização do `VERSION` e checagens).
   - Se estiver ausente ou diferente de `"1"`, verifica condições de bypass:
     - Variável `CCCP_BYPASS_LOCKS=1`.
     - Ambiente de CI (`CI=true`, `GITHUB_ACTIONS=true`, `FORGEJO_ACTIONS=true`).
   - Caso nenhuma exceção seja atendida, rejeita o commit com código de saída `1` e exibe mensagem informativa orientando o uso de `cccp commit` ou `cccp close`.

2. **Hook `pre-push`**:
   - Inspeciona `CCCP_MAESTRO_ACTIVE`.
   - Bloqueia execuções manuais diretas de `git push` caso não esteja sob o Maestro CCCP ou variáveis de bypass.
   - Orienta a utilização do subcomando canônico `cccp push`.

3. **Injeção do Contexto Maestro no CCCP**:
   - Todos os pontos de despacho git do CCCP (`src/commit_exec.rs`, `src/ddtdd.rs`, `src/main.rs`, `src/release.rs`) definem `CCCP_MAESTRO_ACTIVE=1` nos processos filhos do Git.

### 2.2 Checagem de Paridade e Reconciliação no `cccp push`
1. **Detecção do Upstream**:
   - Obtém a branch local atual e seu tracking branch remoto (`@{u}`).
   - Se não houver tracking branch (ex.: branch nova ainda não publicada), prossegue diretamente configurando `-u <remote> <branch>` no primeiro push.
2. **Fetch Silencioso e Cálculo de Divergência**:
   - Executa `git fetch <remote> <branch>` para atualizar a visão local do repositório remoto.
   - Calcula a contagem de commits divergentes (`git rev-list --left-right --count HEAD...<remote>/<branch>`):
     - `ahead`: commits locais ainda não enviados ao remote.
     - `behind`: commits no remote ainda não incorporados localmente.
3. **Ações Determinísticas**:
   - **`behind == 0`**: Paridade satisfeita. Prossegue com `git push`.
   - **`behind > 0 && ahead == 0`**: Repositório local está puramente atrás do remote. Executa `git merge --ff-only` automaticamente e informa a sincronização transparente.
   - **`behind > 0 && ahead > 0`**: Repositório possui divergência simultânea (commits locais e remotos não lineares). Bloqueia o push preventivamente, instruindo a reconciliação (rebase/merge).

---

## 3. Matriz de Testes (Fase 4 & 7)
- `tests/git_locks_push_test.rs`:
  1. Teste de bloqueio de `git commit` direto quando `CCCP_MAESTRO_ACTIVE` não está definido.
  2. Teste de autorização de commit quando `CCCP_MAESTRO_ACTIVE=1` ou `CCCP_BYPASS_LOCKS=1`.
  3. Teste de bloqueio de `git push` direto sem Maestro.
  4. Teste de `cccp push` com tracking branch em paridade (`ahead >= 0, behind == 0`).
  5. Teste de `cccp push` com avanço automático fast-forward quando local está atrasado (`behind > 0, ahead == 0`).
  6. Teste de bloqueio preventivo de `cccp push` em caso de histórico divergente (`ahead > 0, behind > 0`).
