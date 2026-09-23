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
| `gh` | **Optional** | GitHub CLI integration for issue dispatch and token discovery | Falls back to environment variables (`GITHUB_TOKEN`), `.cccprc`, or direct REST API |
| `jq` | **Optional** | Performance accelerator for JSON payload extraction in `diva` | Automatic fallback to 100% pure POSIX `sed` / `grep` / parameter expansion |

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
- `cat`, `cut`, `date`, `diff`, `dirname`, `find`, `grep`, `head`, `mkdir`, `mktemp`, `mv`, `printf`, `rm`, `sed`, `sleep`, `sort`, `tail`, `tr`, `uname`, `uniq`, `wc`, `xargs`.

No non-standard flags (GNU extensions) may be used with these tools.
