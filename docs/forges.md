# Multi-Forge & Ticket Synchronization

> **«ДРУЖБА И СОТРУДНИЧЕСТВО ВСЕХ СИСТЕМ!»**  
> *(Friendship and cooperation among all systems!)*

CCCP features an automated, multi-forge ticket synchronization engine that compiles active upstream issues into local markdown files ([`.antigravity/tickets.md`](file:///home/lumen/src/cccp/.antigravity/tickets.md)) for AI agents and developers.

---

## 1. Supported Forge Providers

CCCP automatically detects your remote Git provider from your push or fetch URLs:

| Forge Provider | Detection Rules & Fast-Paths | Active Probing Capabilities |
| :--- | :--- | :--- |
| **Forgejo / Gitea** | `codeberg.org`, self-hosted domains | Probes `/api/v1/version` |
| **GitHub** | `github.com`, enterprise instances | Uses GitHub REST API v3 (filters PRs) |
| **GitLab** | `framagit.org`, `gitlab.com`, subgroups | Probes `/api/v4/version` |
| **SourceHut** | `git.sr.ht`, `*.sr.ht` | Uses GraphQL issue tracker |

---

## 2. Command Usage (`cccp tickets`)

```bash
# Synchronize tickets from auto-detected remote:
cccp tickets

# Explicitly select a provider:
cccp tickets --provider forgejo

# Override repository credentials:
TICKETS_OWNER=myorg TICKETS_REPO=myproject cccp tickets
```

---

## 3. Gosplan Hourly Quota Protection

To prevent exceeding public API rate limits (such as GitHub's 60 requests/hour unauthenticated limit), CCCP implements a sliding 1-hour window rate limiter:
- **Default Ceiling**: 55 requests per hour.
- **Behavior**: When the ceiling is reached, CCCP exits cleanly with code `0` and displays a Gosplan state quota notice without interrupting Git push workflows or AI agent loops.

---

## 4. Continuous Integration & Automated Site Publishing

CCCP includes an automated Forgejo Actions pipeline (`.forgejo/workflows/ci.yml`) that validates and publishes the static documentation website (`site/`) upon pushing to the `main` branch:

- **Primary Target — Cloudflare Pages**:
  - Domain: [`https://cccp.2lp.in`](https://cccp.2lp.in)
  - Configuration: Set `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` in repository secrets.
  - Project Name: Configurable via repository variable `CLOUDFLARE_PROJECT_NAME` (defaults to `cccp`).
  - Security & Caching: Configured via [`site/_headers`](../site/_headers).
  - Graceful Fallback: If Cloudflare credentials are not configured, the pipeline logs an informational notice without failing the CI run.
- **Redundancy Target — Codeberg Pages**:
  - Domain: `https://<user>.codeberg.page/<repo>/`
  - Automated deployment directly pushes the verified static site to the `pages` branch using the action's built-in token.
- **Multi-Forge Redundancy — Netlify**:
  - Native support via committed [`netlify.toml`](../netlify.toml).
- **Integrity Validation**:
  - Run [`bin/verify-site.sh`](../bin/verify-site.sh) locally or during CI to guarantee zero broken relative links or missing assets across all language versions.

