# Especificação Técnica — Ticket #178
# Painel Operacional Consolidado em `cccp status` (Zero Comandos Git Manuais)

- **Ticket:** #178
- **Tipo:** `feat(status)`
- **Escopo:** Enriquecimento do comando `cccp status` com visibilidade de arquivos alterados (staged, unstaged, untracked), paridade de tracking branch remoto (`ahead` / `behind`), hooks e ciclo DD-TDD.

---

## 1. Contexto & Motivação

O fluxo de engenharia do ecossistema Gosplan exige conformidade estrita com o ciclo DD-TDD (Fases 1 a 9) e delegação de operações ao CCCP.
Contudo, o desenvolvedor ainda precisava digitar comandos git de baixo nível (`git status`, `git diff --stat`, `git status -sb`) para inspecionar arquivos alterados antes de commitar ou checar commits pendentes de push.

---

## 2. Decisões Arquiteturais (Árvore de Decisão para a Natasha)

- **Por que consolidar tudo no comando `cccp status`?**
  - *Alternativa descartada:* Criar novos subcomandos ou aliases (`cccp diff`, `cccp branch`).
  - *Motivo do descarte:* O comando `cccp status` já é a entrada canônica de inspeção de estado. Expandir sua saída visual para ser o painel operacional definitivo elimina a proliferação de comandos e unifica a experiência.
- **Categorização de Arquivos Alterados:**
  - `Staged`: arquivos preparados no índice (`git add`) que entrarão no próximo commit DD-TDD.
  - `Unstaged`: modificações locais de trabalho ainda não indexadas.
  - `Untracked`: arquivos novos ainda não rastreados.
- **Detecção de Commits Pendentes de Envio (`ahead`):**
  - Checa a paridade contra o tracking branch remoto (`@{upstream}`).
  - Se houver commits à frente, alerta explicitamente que o envio deve ser feito via `cccp push`.
- **Integridade dos Hooks:**
  - Checa a existência e integridade dos executáveis em `.git/hooks/pre-commit` e `.git/hooks/pre-push`.

---

## 3. Modelo de Dados e Interfaces

### 3.1 `StatusDetails` em `src/git.rs`

```rust
#[derive(Debug, Clone, Default)]
pub struct StatusDetails {
    pub staged: Vec<String>,
    pub unstaged: Vec<String>,
    pub untracked: Vec<String>,
    pub ahead: usize,
    pub behind: usize,
    pub pre_commit_active: bool,
    pub pre_push_active: bool,
}
```

---

## 4. Critérios de Aceitação

1. `cccp status` deve exibir detalhadamente arquivos staged, unstaged e untracked.
2. `cccp status` deve exibir o status de sincronismo com o tracking branch remoto (`ahead`/`behind`).
3. Auditoria e conformidade estrita (100%) no `cccp check --strict`.
