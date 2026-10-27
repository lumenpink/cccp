# Especificação Normativa: Expansão da Auditoria GOST 2.0 e Ciclo DD-TDD

- **Ticket de Referência:** `#70`
- **Norma Base:** GOST v2.0.0 (Seções 3, 5, 6, 9, 10, 12, 15)
- **Perfil:** Kollontai v2.0.0 / Lilya v2.0.0
- **Fase DD-TDD:** Fase 3 — Doc/Spec (Zero código de produção)
- **Status:** Aprovado para Implementação

---

## 1. Escopo e Propósito

Esta especificação define as regras normativas, critérios de severidade e testes estáticos avaliados pelo comando `cccp check [--strict]` para conformidade determinística com a norma GOST v2.0.0 e validação do fluxo DD-TDD de 9 fases.

---

## 2. Matriz de Novas Regras Auditadas no `cccp check`

### 2.1 Regra `GOST-04-COMMIT-PHASE-TAG` (Seção 5)
* **Objetivo:** Garantir rastreabilidade de tickets e fases DD-TDD nos commits recentes.
* **Critério:** Todo commit intermediário deve conter `(ref #<id> - ph[3-8])` ou fechar com `(closes #<id> - ph9)`.
* **Severidade:** `MUST` (Falha em modo estrito se commits recentes não contiverem tags de fase ou ticket).

### 2.2 Regra `GOST-04-A-CLOSE-RESOLUTION-REPORT` (Seção 5)
* **Objetivo:** Exigir Laudo Textual de Resolução nos commits de fechamento de ticket.
* **Critério:** Commits com tag `(closes #<id> - ph9)` devem possuir corpo com $\ge 80$ caracteres contendo obrigatoriamente os três cabeçalhos:
  - `[Resumo]:`
  - `[Solução]:`
  - `[Validação]:`
* **Severidade:** `MUST`.

### 2.3 Regra `GOST-05-BRANCH-AND-LOCAL-FIRST` (Seção 5)
* **Objetivo:** Impedir desvios no padrão de branches.
* **Critério:** A branch ativa deve casar com o padrão de trabalho `^(feat|fix|docs|refactor|test|chore)/[0-9]+-[a-z0-9-]+$` ou com branches persistentes autorizadas (`main`, `v[0-9]+`, `epic/[a-z0-9-]+`, `pages`). Commits diretos na `main` geram advertência ou falha estrita.
* **Severidade:** `MUST`.

### 2.4 Regra `GOST-20-SITE-TYPE-AND-SSOT-DOCS` (Seção 9)
* **Objetivo:** Manter README enxuto como portal e documentação estática centralizada.
* **Critério:**
  - O arquivo `README.md` na raiz deve ter no máximo **50 linhas** (`lines.count() <= 50`).
  - O diretório `site/` e o arquivo `site/index.html` devem existir.
* **Severidade:** `MUST`.

### 2.5 Regra `GOST-24-ZERO-BINARIES-IN-GIT` (Seção 10)
* **Objetivo:** Proibir artefatos compilados no repositório.
* **Critério:** O repositório não pode conter arquivos com extensões: `.exe`, `.so`, `.dll`, `.dylib`, `.tar.gz`, `.zip`, `.deb`, `.rpm`, `.apk`, `.AppImage`.
* **Severidade:** `MUST`.

### 2.6 Regra `GOST-25-REMOTES-AND-INBOX-QUARANTINE` (Seção 15)
* **Objetivo:** Garantir soberania do remote `origin` e proteção de espelhos passivos.
* **Critério:** Se existirem remotes adicionais além do `origin` (como `codeberg`, `github`), estes devem ter `pushurl = no_push` configurado no Git.
* **Severidade:** `WARN` / `MUST` em modo estrito.

### 2.7 Regra `GOST-26-METADATA-MANIFEST-SSOT` (Seção 3)
* **Objetivo:** Integridade do manifesto `.gost/metadata.json` e filtro anti-jargão.
* **Critério:**
  - Se `.gost/metadata.json` estiver presente, campos obrigatórios devem estar definidos (`slug`, `title`, `site_type`, `tagline`, `summary`, `abstract`, `tags`, `icon`, `theme`).
  - Prosa não deve conter jargões vazios: `"soberano"`, `"soberana"`, `"hexagonal"`, `"ssot"`, `"rust puro"`, `"em rust"`, `"em php"`, `"wasm"`.
* **Severidade:** `MUST`.

### 2.8 Regra `GOST-27-DDD-HEXAGONAL-PURITY` (Seção 12)
* **Objetivo:** Pureza arquitetural do core de domínio.
* **Critério:** Se o diretório `src/domain/` existir, nenhum arquivo sob ele pode importar `std::fs`, `std::net`, `std::process`, `tokio::fs` ou bibliotecas de I/O de rede.
* **Severidade:** `MUST`.

---

## 3. Retorno do Laudo de Auditoria

O comando `cccp check` computa o total de verificações, a contagem de conformes (`PASS`), avisos (`WARN`) e falhas (`FAIL`), emitindo o **Índice de Conformidade Gosplan** percentual e falhando com status não-zero se houver falhas críticas.
