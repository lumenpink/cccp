# Active Tickets - cccp
> Synced at: 2026-09-23 12:54:00 UTC
> Ceiling: 10240 bytes (strictly uncut tickets)

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
- [x] Create an audit script (e.g., `scripts/check-deps.sh`) using POSIX-compliant tooling or a dedicated parser.
- [x] Commit initial baseline `DEPENDENCIES.md` detailing current requirements.
- [x] Add CI check to prevent merging code that introduces untracked external binaries.

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
- [x] Configure `shfmt` flags (`-i 2 -ci -bn -ln posix` or repository standard).
- [x] Add `.editorconfig` reflecting indentation and line endings.
- [x] Set up local formatting/linting scripts for contributors.
- [x] Integrate formatting and linting checks into CI to fail on violations.

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
- [x] Purgar todas as chamadas a `awk` em `src/utils/agents.sh`.
- [x] Garantir que `! grep -rn "awk" src/` retorna 0 ocorrências de código executável.
- [x] Passar na suíte de testes do ShellSpec (`shellspec`).
- [x] Passar no ShellCheck (`shellcheck -s sh src/*.sh`).

---
## [#19] feat(diva): integrate gemini-powered interactive ticket triage bot into posix shell
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/19

### Description
### Context & Rationale
Integrate the capabilities of `diva` (`ticket_bot.py`) into CCCP as a native interactive subcommand (`cccp diva` or `cccp divan`).

### Scope & Architecture
1. **Interactive Conversational Triage (O Divã):**
   - Accept freeform problem descriptions, complaints, or brainstorms in natural language (Portuguese, English, etc.) via interactive prompt or stdin.
2. **Gemini REST API Integration:**
   - Pure POSIX `curl` integration with Google Gemini REST API (`v1beta/models/gemini-2.5-flash:generateContent`).
   - Use structured JSON response schemas or system instructions to split raw input into discrete issues with:
     - `title` (Conventional Commits formatted or concise summary)
     - `body` (reproduction steps, expected behavior, context)
     - `labels` / `tags`
3. **User Confirmation & Review:**
   - Display proposed tickets cleanly in terminal with preview styling.
   - Interactive prompt allowing the user to approve (`[Y/n]`), edit, or cancel before opening.
4. **Automated Multi-Forge Dispatch:**
   - Dispatch approved issues to the active project's forge (Codeberg/Forgejo API or GitHub via REST / `tea` / `gh`).

### Strict POSIX & Tooling Constraints
- ⚠️ **NO AWK ALLOWED**: Do NOT use `awk`. JSON formatting, extraction and text processing must be handled via pure shell string manipulation, `sed`, `grep`, `tr`, and `cut`.
- Zero external runtime dependencies (no Python, no `requests`, no `dotenv`).
- Reads `GEMINI_API_KEY` from environment or `.cccprc`.
- Must run cleanly under `dash`, `ash`, `busybox sh`, and standard POSIX `/bin/sh`.

### Acceptance Criteria
- [x] 100% POSIX-compliant shell code verified via ShellCheck and ShellSpec.
- [x] Absolutely zero `awk` invocations across the codebase.
- [x] Integrates seamlessly with CCCP's existing ticket synchronization pipeline (`.antigravity/tickets.md`).

---
## [#18] feat(catalog): port catalog metadata engine and forge synchronizer to posix shell
- **Status**: `open`
- **Labels**: none
- **Link**: https://codeberg.org/lumenpink/cccp/issues/18

### Description
### Context & Rationale
Integrate the capabilities of `catalog.py` into CCCP as a zero-dependency POSIX shell module (`cccp catalog`).

### Scope & Architecture
1. **Local Cataloging:**
   - Scan workspace/source directories for Git repositories.
   - Read and write standard `.git/description` per repository.
   - Generate structured `catalog.json` and `README.md` project tables.
2. **Metadata Extraction:**
   - Heuristically extract 1-line descriptions, licenses, and tags from `Cargo.toml`, `package.json`, `composer.json`, and `README.md`.
3. **Multi-Forge Sync:**
   - Direct HTTP REST integration via `curl` with Forgejo (Codeberg, TildeGit) and GitHub APIs.
   - Synchronize descriptions, topics/tags, and website URLs.

### Strict POSIX & Tooling Constraints
- ⚠️ **NO AWK ALLOWED**: Do NOT use `awk` (avoid dialect incompatibilities between gawk, mawk, nawk, and busybox awk).
- Use strictly POSIX-compliant standard shell utilities: pure parameter expansion (`${var#...}`, `${var%...}`), `sed`, `grep`, `tr`, `cut`, and shell builtins.
- Zero external runtime dependencies (no Python, no Node, no Perl).
- Must run cleanly under `dash`, `ash`, `busybox sh`, and standard POSIX `/bin/sh`.

### Acceptance Criteria
- [x] 100% POSIX-compliant shell code verified via ShellCheck and ShellSpec.
- [x] Absolutely zero `awk` invocations across the codebase.
- [x] Subcommands: `cccp catalog list`, `cccp catalog sync-local`, `cccp catalog sync-remote`.
- [x] Unit and integration tests under `spec/`.

---
