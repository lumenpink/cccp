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

