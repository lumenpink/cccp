# Active Tickets - cccp
> Synced at: 2026-09-23 16:55:45 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

## [#27] feat(catalog): suporte a diretório padrão via config global, env e flags
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/27

### Description
## Descrição  A função `catalog` atualmente não possui um diretório padrão definido. É necessário implementar a resolução do diretório alvo utilizando a seguinte ordem de precedência:  1. Flag passada pela linha de comando 2. Variável de ambiente 3. Configuração global do `cccp` 4. Valor fallback: `~/src`  ## Passos para Reproduzir  1. Executar a função/comando `catalog` sem especificar um diretório por parâmetro/flag. 2. Verificar a ausência de um fallback ou busca na configuração global.  ## Esperado vs Atual  - **Esperado**: A função `catalog` deve determinar o diretório utilizando a hierarquia (Flag > Env > Config Global > `~/src`). - **Atual**: A função não possui suporte a diretório padrão.

---
## [#26] fix(generator): prevent truncation of long/tall tickets in output
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/26

### Description
## Description  Ao gerar tickets com grande volume de texto ou muitas linhas, o resultado final está sendo exibido apenas parcialmente (truncado).  ## Steps to Reproduce  1. Criar ou gerar um ticket com conteúdo extenso (altura/linhas elevadas). 2. Exibir a saída do gerador. 3. Observar que o ticket é cortado e apenas uma parte é apresentada.  ## Expected vs Actual  - **Esperado:** O ticket deve ser exibido integralmente, independentemente da altura/quantidade de linhas. - **Atual:** Apenas uma parte do ticket é exibida quando ele excede uma determinada altura.

---
## [#25] fix(generator): fix ASCII table right border and text wrapping in columns
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/25

### Description
## Description  A representação em tabela ASCII do gerador de tickets não está fechando a borda da margem direita e não aplica quebra de linha (text wrapping) no conteúdo da coluna X, prejudicando a apresentação visual.  ## Steps to Reproduce  1. Gerar um ticket que contenha texto longo na coluna X. 2. Visualizar a tabela ASCII gerada. 3. Observar que a borda lateral direita está ausente/desalinhada e o texto não quebra linha.  ## Expected vs Actual  - **Esperado:** A tabela ASCII deve ser exibida com a borda direita devidamente fechada e com quebra de linha automática nas colunas para manter a largura consistente. - **Atual:** A borda direita fica aberta e o texto da coluna X é impresso sem quebras de linha.

---
## [#24] refactor(core): auditar e reutilizar trechos de código e funções duplicadas na base de código
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/24

### Description
## Description  Mapear e refatorar a base de código para identificar outras funções idênticas ou similares, bem como blocos de código duplicados, promovendo a reutilização através de utilitários e módulos comuns.  ## Steps to Reproduce  N/A (Auditoria de código/refatoração geral)  ## Expected vs Actual  - **Esperado**: Base de código enxuta, sem blocos ou funções redundantes. - **Atual**: Presença de código duplicado e lógica redundante espalhada no projeto.

---
## [#23] refactor(repo): consolidar funções de identificação de repositório e usuário
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/23

### Description
## Description  Existe duplicação de funções encarregadas de obter dados sobre o repositório atual, como o nome do repositório e o usuário/owner. Esta lógica deve ser extraída e consolidada em uma única função/módulo reutilizável.  ## Steps to Reproduce  N/A (Identificação de débito técnico/refatoração)  ## Expected vs Actual  - **Esperado**: Uma única função centralizada, reutilizável e com testes unitários para identificar o repositório e usuário atual. - **Atual**: Funções e trechos de código repetidos executando a mesma verificação em múltiplos locais.

---
## [#22] ci(deps): automate external binary dependency detection and track in DEPENDENCIES.md
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/22

### Description
### Problem Statement
The application aims to remain strictly POSIX-compliant with minimal external dependencies. However, there is no automated auditing mechanism to detect when non-POSIX utilities or third-party binaries (e.g., `jq`, `curl`, `wget`, `git`) are introduced into the scripts.

### Proposed Solution
Implement an automated static analysis routine to inspect executable invocations across all scripts, distinguish standard POSIX utilities from external binaries, and maintain an up-to-date `DEPENDENCIES.md` at the repository root.

1. **Detection Strategy**:
   - Run a static analysis step (e.g., parsing command tokens via `shfmt -tojson` or AST traversal) to extract all invoked command names.
   - Cross-reference invoked commands against the standard IEEE Std 1003.1 (POSIX.1-2017 / POSIX.1-2024) utilities list.
   - Filter out shell built-ins, internal functions, and core utilities to isolate third-party dependencies.
2. **Artifact Generation (`DEPENDENCIES.md`)**:
   - Automatically generate or verify `DEPENDENCIES.md` documenting:
     - Binary name.
     - Mandatory vs. optional usage.
     - Fallback implementations (if any).
3. **Verification**:
   - Add a CI check that fails if an unlisted external dependency is introduced or if `DEPENDENCIES.md` is out of sync with the codebase.

### Deliverables
- [ ] Create an audit script (e.g., `scripts/check-deps.sh`) using POSIX-compliant tooling or a dedicated parser.
- [ ] Commit initial baseline `DEPENDENCIES.md` detailing current requirements.
- [ ] Add CI check to prevent merging code that introduces untracked external binaries.

---
## [#21] feat(tooling): establish POSIX shell formatting standards and linting automation
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/21

### Description
### Problem Statement
The codebase currently lacks an enforced code style, indentation rules, and automated syntax validation. As a pure POSIX shell project, maintaining consistency across scripts manually is error-prone and risks introducing non-POSIX extensions (bashisms) into the codebase.

### Proposed Solution
Adopt a standardized formatting and linting pipeline tailored specifically for POSIX shell scripts:

1. **Formatter (`shfmt`)**:
   - Standardize on `shfmt` configured for POSIX dialect (`-ln posix` / `-p`).
   - Define formatting rules (e.g., 2 or 4 spaces indentation, binary operator placement).
2. **Linter (`shellcheck`)**:
   - Enforce static analysis targeting POSIX (`shellcheck -s sh`) to flag potential bugs, quoting pitfalls, and accidental bashisms.
3. **Automation & CI**:
   - Provide a local check script/Makefile target (`make lint` or `make fmt-check`).
   - Add a GitHub Actions workflow to validate that all PRs adhere to the formatted style without changes.

### Deliverables
- [ ] Configure `shfmt` flags (`-i 2 -ci -bn -ln posix` or repository standard).
- [ ] Add `.editorconfig` reflecting indentation and line endings.
- [ ] Set up local formatting/linting scripts for contributors.
- [ ] Integrate formatting and linting checks into CI to fail on violations.

---
## [#20] refactor(agents): purge legacy awk usage in agents.sh for pure posix compliance
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/20

### Description
### Summary
As defined in ARCHITECTURE.md (Section 2.1), awk is strictly forbidden across all CCCP modules to ensure portability across minimal POSIX environments (BusyBox ash, dash, BSD sh, macOS, Termux).

Two legacy occurrences of awk remain in `src/utils/agents.sh`:

1. **Line 284**:
```sh
clean_val=$(printf '%s' "$var_val" | awk '{ if (NR > 1) printf "\\n"; printf "%s", $0 }')
```
*Purpose:* Normalization / newline escaping for single-line config values.
*Approved alternative:* Pure POSIX parameter expansion or `tr` / `sed`.

2. **Lines 494-510**:
```sh
result=$(printf '%s\n' "$result" | awk -v vname="$v" '...')
```
*Purpose:* Interpolation of `{{var}}` and `${var}` placeholders within agent template files.
*Approved alternative:* Pure POSIX parameter expansion loop or `sed`.

### Acceptance Criteria
- [ ] Purgar todas as chamadas a `awk` em `src/utils/agents.sh`.
- [ ] Garantir que `! grep -rn "awk" src/` retorna 0 ocorrências de código executável.
- [ ] Passar na suíte de testes do ShellSpec (`shellspec`).
- [ ] Passar no ShellCheck (`shellcheck -s sh src/*.sh`).

---
