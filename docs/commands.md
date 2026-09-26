# CLI Command Reference

> **«БУДЬ НАЧЕКУ И СОБЛЮДАЙ ДИСЦИПЛИНУ!»**  
> *(Be vigilant and maintain discipline!)*

Complete reference guide for all CCCP commands and subcommands:

---

## 1. Primary Commands

### `cccp commit -i` (or `cccp cz`)
Interactive commit wizard that prompts for type, optional scope, subject line, body, and breaking changes. Enforces standard character limits and formatting.

### `cccp status`
Displays the full health inspectorate of your Git repository:
- Current branch and commit status.
- Predictive Semantic Version.
- Installed hooks health and version synchronization.
- Active configuration sources (`.cccprc` / environment).

### `cccp lint [<revision-range>]`
Validates commit messages across branches or pull request ranges:
```bash
# Lint current branch commits against origin/main:
cccp lint origin/main..HEAD

# Lint the last 5 commits:
cccp lint HEAD~5..HEAD
```

### `cccp tag [<version>]`
Calculates next SemVer based on commit signals, finalizes changelog, and creates an annotated Git tag.

### `cccp push [<args>...]`
Runs standard `git push`, triggers the `pre-push` verification, and automatically launches the configured post-push lifecycle.

### `cccp web` (or `cccp diva --web`)
Launches the interactive Gosplan Web Cockpit powered by Comrade Mayak (`mk`) command bridge:
- Visual interactive ticket triage with Diva.
- Repository inspectorate and active tickets table.
- Local authenticated bridge with token protection.

### `cccp check` (or `cccp gost check`)
Executes canonical GOST compliance check against the active repository:
- Validates declared `GOSTVER`, license, SemVer in `VERSION`, root file anatomy, ticket commits traceability, dedicated branch names, and security headers.
- Supports `--strict` to treat advisory checks (`SHOULD`) as failures.
- Colorized Gosplan audit report with `NO_COLOR` support.

### `cccp gost <subcommand>`
Full lifecycle management for GOST technical software standards:
- `cccp gost sync`: Downloads canonical `GOST.md`, `PROFILE.md`, `rules.json`, and selectively language specs listed in `GOST_LANGUAGES` to `.gost/`.
- `cccp gost check`: Audits project conformity against GOST rules.
- `cccp gost upgrade <version>`: Displays migration guide, updates `GOSTVER`, and synchronizes cache.
- `cccp gost init`: Initializes a standard `GOSTVER` file in the project root.

### `cccp serve [dir] [options]`
Unified HTTP server for static sites, dev watcher, and VDNKh demonstration:
- Static mode: autodetects `./site`, `./dist`, `./public` or `./`; prioritizes comrade Mayak (`mk`) with fallback to `python3` and `busybox`.
- Dev mode (`--dev`): hot-reloads builders (`combine.sh`, `cargo build`, `make`), serves dynamic PHP (`php -S`), or launches Node development scripts.
- Demo mode (`--demo`): executes setup/fixtures (`scripts/demo.sh`), healthcheck polling, VDNKh exhibition banner, and auto-launches the browser.
- Network: binds to `0.0.0.0` by default (accessible via LAN/Tailscale) or `--localhost`/`-l` for 127.0.0.1; port auto-increments if occupied.

---

## 2. Shell Autocompletion (`cccp completion`)

CCCP provides native tab-completion for Bash, Zsh, and Fish:

### Bash
```bash
# Add to ~/.bashrc:
eval "$(cccp completion bash)"
```

### Zsh
```bash
# Add to ~/.zshrc (before compinit):
eval "$(cccp completion zsh)"
```

### Fish
```bash
# Save completion script:
cccp completion fish > ~/.config/fish/completions/cccp.fish
```

---

## 3. Cultural Directives (`cccp soviet`)

Displays the legendary Gosplan directive, Order No. 227 banner, and Stakhanovite labor guidelines:

```bash
cccp soviet
```

---

## 4. Multi-Repository Management (`cccp repo`)

Provides unified status overview and inspection across multiple workspaces and repositories, identifying dirty working trees, unpushed commits, and open forge tickets.

```bash
# View general overview across repositories (auto-discovers ~/src or catalog.json)
cccp repo

# Filter only repositories with uncommitted/unstaged changes (dirty)
cccp repo --dirty

# Filter only repositories with unpushed commits
cccp repo --unpushed

# Filter only repositories with open tickets/issues
cccp repo --issues

# Combine filters (e.g. dirty or unpushed work)
cccp repo --dirty --unpushed

# Machine-readable structured JSON output
cccp repo --json

# Specify custom scan directory or catalog file
cccp repo -d /path/to/workspaces
cccp repo -c /path/to/catalog.json
```
