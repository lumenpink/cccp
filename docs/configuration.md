# Configuration & Hierarchy

> **«ПЛАН — ЭТО ЗАКОН, ВЫПОЛНЕНИЕ — ДОЛГ!»**  
> *(The plan is law, fulfillment is duty! — Gosplan)*

CCCP features a 4-tier hierarchical configuration cascade modeled after the precision of the State Planning Committee (Gosplan):

```text
Priority 1: Environment Variables (highest - temporary overrides)
Priority 2: Local Repository Config (.cccprc in repository root)
Priority 3: Global User Config (~/.cccprc in user home directory)
Priority 4: Built-in Gosplan Defaults (deterministic fallback)
```

---

## The `cccp config` CLI

You can inspect and update your settings directly through the command line.

### View Configuration

```bash
# View active resolved configuration across all tiers
cccp config

# Show values from global config (~/.cccprc)
cccp config --global

# Show values from local repository config (.cccprc)
cccp config --local
```

### Set Configuration

```bash
# Set a local repository key
cccp config tickets_provider codeberg

# Set a global key across all repositories
cccp config --global cz_prompt_breaking true
```

### Unset Configuration

```bash
# Remove key from local config
cccp config --unset tickets_provider

# Remove key from global config
cccp config --global --unset cz_prompt_breaking
```

---

## Supported Configuration Keys

| Key | Description | Default | Valid Options / Example |
| :--- | :--- | :--- | :--- |
| `types` | Allowed conventional commit types | Standard 11 types | `feat,fix,docs,style,refactor,perf,test,build,ci,chore,revert` |
| `strict_types` | Enforce strict type validation | `1` | `1` (strict), `0` (permissive) |
| `strict_scopes` | Enforce configured scopes list | `0` | `1` (strict), `0` (allow any scope) |
| `scopes` | Allowed commit scopes list | Empty (all allowed) | `api,ui,db,core,cli` |
| `strict_subscopes` | Enforce configured subscopes | `0` | `1` (strict), `0` (allow any) |
| `subscopes` | Allowed commit subscopes | Empty | `auth,billing,users` |
| `disable_scopes` | Disallow any scope usage | `0` | `1` (disabled), `0` (allowed) |
| `disable_subscopes`| Disallow nested subscopes | `0` | `1` (disabled), `0` (allowed) |
| `disable_multiple_scopes` | Disallow comma-separated scopes | `0` | `1` (disabled), `0` (allowed) |
| `default_base_version` | Starting version for new repos | `0.0.1` | Any valid SemVer (e.g. `0.1.0`) |
| `cz_prompt_breaking` | Interactive wizard prompts breaking changes | `false` | `true`, `false` |
| `check_updates` | Check upstream for updates | `1` | `1` (enabled), `0` (disabled) |
| `pinned_version` | Pin specific version (locks updates) | Empty | E.g. `2.3.0` |
| `tickets_provider`| Forge tickets provider | Auto-detected | `forgejo`, `github`, `gitlab`, `sourcehut` |
| `tickets_limit` | Hourly sync request limit ceiling | `55` | Integer up to 60 |
| `post_push_hook` | Path to custom script run after push | Empty | `.cccp/hooks/post-push`, etc. |
| `post_push_sync` | Built-in automated post-push sync actions | Empty | `tickets`, `agent`, or `tickets,agent` |
