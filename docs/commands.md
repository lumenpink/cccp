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
