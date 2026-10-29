# Especificação Técnica — Ticket #85: Atualização de Tracking Branch no Fetch de Paridade

## 1. Visão Geral e Contexto
Durante a execução da suíte de testes no ambiente containerizado de CI do Forgejo, os testes de reconciliação de paridade em `tests/git_locks_push_test.rs` falharam:
- `test_push_auto_fast_forward_when_behind`: retornou `AlreadyUpToDate` em vez de `FastForwarded`.
- `test_push_blocked_on_divergent_history`: não detectou a divergência com o upstream.

A causa raiz reside na etapa 4 de `src/push.rs`:
```rust
Command::new("git").args(["fetch", remote_name, remote_branch])
```
No Git padrão, passar um nome de branch sem refspec de destino grava os objetos em `FETCH_HEAD`, mas não atualiza a branch de rastreamento remoto (`refs/remotes/<remote>/<branch>`). Quando o passo subsequente executa:
```rust
git rev-list --left-right --count HEAD...<upstream>
```
onde `<upstream>` é tipicamente `origin/main`, o tracking branch ainda reflete o estado anterior ao fetch, resultando em 0 commits atrás/à frente no ambiente limpo do CI.

## 2. Requisitos e Critérios de Aceite
- **RF-01 (Atualização Explícita do Tracking Branch Remoto)**:
  - Em `src/push.rs`, garantir que o comando de fetch atualize o tracking branch remoto de destino (`refs/remotes/<remote_name>/<remote_branch>`).
  - Utilizar o refspec canônico de sincronização: `format!("+refs/heads/{}:refs/remotes/{}/{}", remote_branch, remote_name, remote_branch)`.
  - Como fallback seguro ou alternativo, suportar `git fetch <remote_name>`.
- **RF-02 (Detecção Fiel de Divergência e Fast-Forward)**:
  - Garantir que `check_and_reconcile_parity` detecte fielmente estados `FastForwarded`, `AlreadyUpToDate` e erros de histórico divergente.
- **RF-03 (Quality Gate e Desbloqueio)**:
  - `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test --test git_locks_push_test` devem passar com 100% de sucesso.
