# CCCP Architecture & Engineering Constraints

> **«В СВЕТЛОЕ БУДУЩЕЕ С ЧИСТЫМ GIT!»**  
> *(Into a bright future with clean Git!)*

This document defines the architectural principles, strict engineering constraints, module structure, and coding standards governing the **Conventional Commits Compliance Program (CCCP)**.

---

## 1. Core Philosophy

CCCP is engineered as a **zero-dependency, universally portable, deterministic POSIX shell utility**. It must run reliably on any Unix-like system without requiring external interpreters (Python, Node.js, Ruby, Perl) or modern non-standard runtimes.

### Target Environments
* Minimalist containers (Alpine Linux `/bin/sh` / BusyBox `ash`)
* Debian / Ubuntu base systems (`/bin/dash`)
* BSD systems (FreeBSD, OpenBSD `/bin/sh`)
* macOS (`/bin/sh` / POSIX mode)
* Mobile environments (Android via Termux `/data/data/com.termux/files/usr/bin/sh`)

---

## 2. Strict Technical Constraints

All code contributing to CCCP must strictly comply with IEEE Std 1003.1 (POSIX). The following constraints are non-negotiable:

### 2.1. The Absolute "No AWK" Rule (⚠️ STRICTLY ENFORCED)
**`awk` is strictly forbidden across all CCCP modules.**

#### Rationale:
1. **Dialect Divergence:** `gawk` (GNU), `mawk` (Debian default), `nawk` (BSD/macOS), and BusyBox `awk` exhibit subtle, breaking differences in regex handling, RS/FS parsing, multibyte UTF-8 characters, and buffer flushing.
2. **Deterministic Portability:** To guarantee 100% reproducible execution across Alpine, Debian, OpenBSD, and Termux, CCCP standardizes on pure shell builtins and unified POSIX utilities.

#### Approved Replacements:
* **String Manipulation:** POSIX parameter expansion (`${var#*prefix}`, `${var%suffix*}`, `${#var}`).
* **Stream Transformations:** `sed` (using only standard POSIX commands: `s///`, `d`, `p`, `y///`, avoiding GNU extensions).
* **Delimited Field Parsing:** `cut -d... -f...` or `IFS` loop parsing (`while IFS= read -r line; do ...; done`).
* **Character Translation:** `tr -d`, `tr '[:upper:]' '[:lower:]'`.
* **Pattern Matching:** `grep -E`, `grep -q`, `case ... in ... esac`.

---

### 2.2. Prohibition of Bashisms
The codebase must execute cleanly under standard `/bin/sh`. The following Bash/Zsh constructs must never be used:

| Forbidden Bashism | Approved POSIX Alternative |
| :--- | :--- |
| `[[ expression ]]` | `[ expression ]` or `test expression` |
| `(( expression ))` | `[ "$((expression))" -eq 0 ]` |
| `function foo() { ... }` | `foo() { ... }` |
| `source file.sh` | `. file.sh` |
| `arr=(a b c)` / `${arr[@]}` | Positional parameters (`set -- ...`), newline-delimited strings, or quoted lists |
| `echo -e` / `echo -n` | `printf '%s\n'` / `printf '%s'` |
| `${var//pattern/replace}` | `printf '%s\n' "$var" \| sed "s|pattern|replace|g"` |
| `<<< "string"` | `printf '%s\n' "string" \| ...` |
| `local var` (non-standard) | Subshell isolation `( ... )` or prefixed variable naming `_foo_var` |

---

### 2.3. Zero External Runtime Dependencies
* **Permitted Binaries:** Standard POSIX core utilities (`sed`, `grep`, `tr`, `cut`, `cat`, `mkdir`, `cp`, `mv`, `rm`, `chmod`, `date`, `uname`, `find`), `git`, and `curl` (for network I/O).
* **Forbidden Dependencies:** Python, Node.js, Ruby, Perl, Go, Rust binaries, or non-standard CLI helpers (`jq` may be used only with pure-shell fallbacks where applicable).

---

## 3. Modular Architecture

```
src/
├── cccp-base.sh           # Core entrypoint, CLI dispatcher, Gosplan rate-limiter
├── config.sh              # Hierarchical configuration parser (.cccprc)
├── semver.sh              # SemVer 2.0.0 math, parsing and predictive release calculation
├── hooks.sh               # Git hook generator and lifecycle management
├── tickets.sh             # Multi-forge ticket fetcher and markdown synchronizer
├── catalog.sh             # (Issue #18) Local workspace scanner & forge metadata engine
└── diva.sh                # (Issue #19) Interactive Gemini-powered ticket triage bot
```

### 3.1. Compilation via `combine.sh`
* During development, features are isolated in discrete modules under `src/`.
* Running `./combine.sh` amalgamates all source components into a single standalone, executable script: [`cccp.sh`](file:///home/lumen/src/cccp/cccp.sh).
* Distribution is delivered as a zero-install single file (`curl -fsSL https://cccp.2lp.in | sh`).

---

## 4. Multi-Forge Integration & Wire Protocol Boundary

### 4.1. The Git Protocol Boundary
The native Git wire protocol (`git push`, `git fetch`) strictly synchronizes:
* Commits, Trees, Blobs, Tags, and Branch References.

Native Git **does not** synchronize repository descriptions (`.git/description`), issues, pull requests, labels, or platform metadata.

### 4.2. CCCP's Bridge Architecture
CCCP bridges the local Git environment with remote forge databases:

```
┌──────────────────────────────────────────────────────────────┐
│                    CCCP Multi-Forge Engine                   │
├──────────────────────────────┬───────────────────────────────┤
│         Local Machine        │         Remote Forges         │
│                              │                               │
│  .git/description            │  Codeberg (Forgejo API)       │
│  .antigravity/tickets.md <───┼─ TildeGit (Gitea API)         │
│  catalog.json                │  GitHub (REST API / gh)       │
│                              │  GitLab / SourceHut           │
└──────────────────────────────┴───────────────────────────────┘
```

1. **Local State:** Writes directly to standard `.git/description` and `.antigravity/tickets.md`.
2. **Remote State:** Communicates over HTTPS REST using `curl` with bearer/personal access tokens to update descriptions, topics, and issues in the forge's platform database.

---

## 5. Upcoming Subsystems

### 5.1. `cccp catalog` (Issue #18)
* Discovers and indexes repositories across `~/src` or defined workspaces.
* Extracts one-line descriptions, licenses, and topics using pure POSIX `sed` / `grep`.
* Synchronizes metadata bi-directionally between `.git/description` and remote forges.

### 5.2. `cccp diva` (Issue #19)
* Interactive natural language diagnostic session ("O Divã").
* Calls Google Gemini REST API (`gemini-2.5-flash`) via `curl` with structured prompt templates.
* Decomposes raw bug reports and feature ideas into discrete, atomic Conventional Commits issues.
* Interactive preview and one-click dispatch to the active repository's forge.

---

## 6. Verification & Quality Assurance

Every change must pass:
1. **ShellCheck:** Strict POSIX linting:
   ```bash
   shellcheck -s sh src/*.sh
   ```
2. **ShellSpec:** Automated behavioral test suite:
   ```bash
   shellspec
   ```
3. **No-AWK Verification:**
   ```bash
   ! grep -rnw "awk" src/
   ```
