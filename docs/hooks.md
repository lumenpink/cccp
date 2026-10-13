# Git Hooks & Automation

> **«НИ ШАГУ НАЗАД ОТ СТАНДАРТОВ!»**  
> *(Not a single step back from standards! — Order No. 227)*

CCCP acts as an automated commissar for your Git repository, installing and orchestrating native hooks that prevent non-compliant commits and broken releases.

---

## 1. The Hook Squad

| Hook | Role & Trigger | Behavior |
| :--- | :--- | :--- |
| `commit-msg` | Fires during `git commit` | Rejects non-conventional commit messages; provides immediate corrective hints. |
| `post-commit` | Fires after successful commit | Updates `CHANGELOG.md` and generates fresh predictive `VERSION`. |
| `pre-push` | Fires during `git push` | Verifies that release tags match committed `VERSION` files and working trees are clean. |
| `reference-transaction` | Fires on tag creation | Blocks creating release tags with dirty working trees or mismatched versions. |
| `post-merge` | Fires after successful merge | Inspects merged commits (`ORIG_HEAD..HEAD`), parses canonical `(closes #<id>)` tags, and automatically closes corresponding tickets via Forge API. |

---

## 2. Versionamento e Auto-Atualização Segura de Hooks

Os hooks instalados pelo CCCP contêm metadados padronizados:
* `cccp-hook-version`: Versão do pacote CCCP instalador.
* `cccp-hook-template-version`: Versão lógica do script do template (ex: `1`).
* `cccp-hook-checksum`: Hash determinístico do conteúdo do script.

### Mecanismo de Integridade Gosplan
* **Hooks não modificados:** Quando uma nova versão do CCCP ou template é executada, o CCCP auto-atualiza os hooks de forma segura e transparente, eliminando falso-positivos e ruídos no terminal.
* **Hooks customizados localmente:** Se o desenvolvedor inseriu personalizações locais no hook, o CCCP detecta a divergência do checksum, preserva intactas as alterações locais e exibe um alerta de advertência amigável.

---

## 3. Inspecting & Auditing Hooks (`cccp hooks`)

CCCP includes a built-in inspectorate to audit your installed hooks against canonical standards:

```bash
# Audit installed hooks
cccp hooks audit

# Display unified diff showing local customizations or third-party hooks (e.g. Husky)
cccp hooks diff
```

---

## 4. Post-Push Automation & Custom Scripts

CCCP allows running custom automation immediately after successful pushes:

```bash
# Push to upstream and trigger post-push lifecycle:
cccp push origin main
```

### Hook Discovery Order
1. Path defined in `POST_PUSH_HOOK` environment variable or `.cccprc`.
2. Shared project hook in `.cccp/hooks/post-push`.
3. Local developer hook in `.git/hooks/post-push.local`.

### Built-in Automated Syncing
You can configure CCCP to run automatic post-push tasks:

```bash
# Enable ticket and agent synchronization after every push:
cccp config post_push_sync "tickets,agent"
```
