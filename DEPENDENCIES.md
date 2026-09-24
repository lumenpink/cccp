# External Binary Dependencies

> Conventional Commits Compliance Program (CCCP)
> Specification: IEEE Std 1003.1 (POSIX) Compliance & External Dependency Matrix

This document tracks all external (non-POSIX) binary requirements, their usage scope, and fallback mechanisms. CCCP is architected to operate in constrained POSIX environments (BusyBox ash, dash, BSD sh, macOS, Linux) with zero runtime dependencies beyond standard shell tools and Git.

## 1. External Binaries Matrix

| Binary | Category | Purpose | Fallback / Behavior when absent |
| :--- | :--- | :--- | :--- |
| `git` | **Mandatory** | Core VCS operations, hook installation, commit linting, tag & version generation | Fatal error: CCCP requires Git to function |
| `cccp` | **Self** | PATH resolution check in hooks and update scripts — verifies the installed binary is reachable | Graceful fallback to script-relative invocation |
| `curl` | **Optional** | HTTP client for remote updates, tickets sync, catalog forge sync, and diva bot | Falls back to `wget` where available, or returns graceful warning |
| `wget` | **Optional** | Alternative HTTP client for updates and hook downloads | Used if `curl` is missing; graceful error if both are absent |
| `tea` | **Optional** | Gitea/Codeberg CLI integration and token discovery | Falls back to environment variables (`CODEBERG_TOKEN`), `.cccprc`, or direct config parsing |
| `gh` | **Optional** | GitHub CLI integration for issue dispatch and token discovery | Falls back to environment variables (`GITHUB_TOKEN`, `GH_TOKEN`), `.cccprc`, direct `~/.config/gh/hosts.yml` parsing, or direct REST API |
| `jq` | **Optional** | Performance accelerator for JSON payload extraction in `diva` | Automatic fallback to 100% pure POSIX `sed` / `grep` / parameter expansion |
| `kv` | **Optional** | Micro terminal text editor in pure Rust with full keyboard navigation (Kvant) for ticket editing in `diva` | Falls back to `kvant`, `$VISUAL`, `$EDITOR`, or POSIX `vi` |
| `kvant` | **Optional** | Micro terminal text editor in pure Rust with full keyboard navigation for ticket editing in `diva` | Falls back to `$VISUAL`, `$EDITOR`, or POSIX `vi` |

## 2. Banned Utilities (Gosplan Directives)

| Binary | Status | Rationale |
| :--- | :--- | :--- |
| `awk` | **BANNED** | Dialect fragmentation across BusyBox awk, mawk, GNU gawk, and BSD/macOS awk. All `awk` variants are explicitly prohibited. |
| `gawk` | **BANNED** | GNU awk dialect — prohibited as per project POSIX portability policy. |
| `mawk` | **BANNED** | mawk dialect — prohibited as per project POSIX portability policy. |
| `nawk` | **BANNED** | nawk (POSIX awk) — prohibited to eliminate all awk dialect risk. |

All `awk` variants must be replaced by pure POSIX parameter expansions, `sed`, `tr`, and `cut`.

## 3. Standard POSIX Baseline (IEEE Std 1003.1)

The following utilities are assumed to be present in any standard POSIX-compliant environment:
- `cat`, `cut`, `date`, `diff`, `dirname`, `find`, `fold`, `grep`, `head`, `mkdir`, `mktemp`, `mv`, `printf`, `rm`, `sed`, `sleep`, `sort`, `tail`, `tr`, `uname`, `uniq`, `wc`, `xargs`.

No non-standard flags (GNU extensions) may be used with these tools.

## 4. BusyBox and Minimalist POSIX Environments (Ticket #33 Assessment)

In strict POSIX and BusyBox environments (e.g., Alpine Linux, Docker containers, embedded devices):

1. **`git`**:
   - BusyBox does not include Git. Git remains the sole mandatory external dependency because CCCP's core mission is commit linting, hook enforcement, and version management.
   - For minimal environments, `git-core` or static git builds provide the smallest footprint (~15MB).

2. **Network Clients (`curl` / `wget`)**:
   - BusyBox provides a lightweight `wget` applet (often compiled with or without TLS).
   - CCCP update scripts and hook download mechanisms support both `curl` and `wget`.
   - REST API interactions with Gemini and remote forges (GitHub, GitLab, Codeberg, SourceHut) require HTTPS/TLS support.

3. **Data Parsing (`jq` vs. Pure POSIX)**:
   - BusyBox does not include `jq`.
   - CCCP provides 100% pure POSIX fallbacks for all JSON extractions (`_diva_extract_tickets`, `_diva_extract_html_url`, ticket field parsing) using standard `sed`, `grep`, and parameter expansions. `jq` is strictly an optional acceleration layer when present.

4. **Forge CLI Decoupling (`gh` / `tea`)**:
   - Neither `gh` nor `tea` is required.
   - CCCP parses `~/.config/gh/hosts.yml` and `~/.config/tea/config.yml` directly in pure POSIX sh to discover authentication tokens. Issue dispatch and repo metadata synchronization utilize direct REST API requests via `curl` with pure POSIX payload generation.

5. **Terminal Editors (`kv` / `kvant` / `$EDITOR`)**:
   - BusyBox includes a standard `vi` applet.
   - CCCP's editor resolver (`_diva_resolve_editor`) checks: `DIVA_EDITOR` -> `.cccprc` `diva_editor` -> `$VISUAL` / `$EDITOR` -> `kv` / `kvant` -> POSIX `vi`.
   - For users seeking a lightweight, modern micro-editor with full keyboard navigation and zero complex dependencies, `kv` (Kvant) in pure Rust serves as an optimal alternative.
