# Installation & Setup Guide

> **«ДАТЬ РОДИНЕ КАЧЕСТВЕННЫЙ КОД!»**  
> *(Deliver quality code to the Motherland!)*

CCCP can be deployed in two modes:
1. **Global Installation**: Placed in your system `PATH` to manage conventional commits across any repository.
2. **Local Repository Hooks**: Configured in `.git/hooks` within a single repository with automatic fallback wrappers.

---

## 1. Global Installation (System PATH)

The fastest way to equip your developer workstation is via the automated installer:

```bash
# Download and install globally into ~/.local/bin/cccp (or /usr/local/bin/cccp if run as root)
curl -fsSL https://codeberg.org/lumenpink/cccp/raw/branch/main/cccp.sh | sh -s -- install -g
```

Alternatively, from within a cloned repository:

```bash
./cccp.sh install --global
```

Verify your installation:

```bash
cccp version
```

---

## 2. Local Repository Hooks Installation

To enforce compliance on a specific Git project without requiring global system privileges:

```bash
# In your repository root:
cccp install
```

This installs lightweight wrapper hooks inside `.git/hooks/`:
- `commit-msg`: Validates commit messages in real-time according to Conventional Commits standards.
- `post-commit`: Automatically updates `CHANGELOG.md` and bumps the local predictive development version in `VERSION`.
- `pre-push`: Guards your remote branches against non-compliant version tags and uncommitted working states.
- `reference-transaction`: Validates local tag creation before commits are committed to Git refs.

### Fallback Mechanism

The installed Git hooks are portable:
1. They search for `cccp` in system `$PATH`.
2. If absent, they look for the local script path recorded during install.
3. If moved, they check for `./cccp.sh` in the current repository root.

---

## 3. Uninstallation

To safely remove Git hooks from a repository:

```bash
cccp uninstall
```

To remove the global binary:

```bash
rm -f ~/.local/bin/cccp
# or if installed as root:
sudo rm -f /usr/local/bin/cccp
```
