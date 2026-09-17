# cccp.sh 
## Conventional Commits Compliance Program

A comprehensive, dependency-free tool for enforcing and managing Conventional Commits in your Git workflow. This program maintains consistent commit messages, provides portable Git hook wrappers, supports hierarchical configuration, and automates predictive semantic versioning.

## What is it?

`cccp.sh` is a POSIX-compliant solution that:
- Installs directly into your system `PATH` (`cccp`) or manages local Git hooks (`.git/hooks/`)
- Enforces Conventional Commits formatting locally before commits are pushed
- Employs portable wrapper hook scripts with version stamps and synchronization warnings
- Features a full hierarchical configuration system (`Defaults < Global ~/.config < Local .cccprc < ENV < CLI flags`)
- Automates predictive semantic versioning and changelog synchronization
- Supports intelligent self-updating across `stable` and `nightly` release channels

## Prerequisites

Your system must have the following tools installed and available in your `PATH`:
- `git` (https://git-scm.com/)
- Standard POSIX utilities: `sed`, `grep`, `date`, `cut`, `tr` (no `awk` required; fully compatible with minimal BusyBox and diverse POSIX shells)
- `curl` or `wget` (optional, needed only for self-updating via `cccp update`)

> [!NOTE]
> **Pure POSIX & BusyBox Compatibility**: `cccp` contains zero dependency on `awk`, eliminating dialect incompatibilities across GNU `gawk`, `mawk`, BSD `awk`, and BusyBox. All configuration and stream parsing run natively in pure POSIX shell.

> [!IMPORTANT]
> If you are using Windows as your operating system, you must run all commands through the "Git Bash" terminal application.

---

## Installation

### 1. Global Installation (System PATH)
You can install `cccp` into your system `PATH` so that it is globally accessible from any terminal and any repository:

```bash
# Download and install globally into ~/.local/bin/cccp (or /usr/local/bin/cccp if root)
curl -fsSL https://github.com/lumenpink/cccp.sh/releases/latest/download/cccp.sh -o cccp.sh
chmod +x cccp.sh
./cccp.sh install --global
```

If `~/.local/bin` is not yet in your `$PATH`, the installer automatically adds it to your `~/.bashrc` (and `~/.zshrc` if present) with instructions.

### 2. Local Repository Hooks Installation
Inside any Git repository, register portable Git hook wrappers:

```bash
cccp install
```

This installs standalone POSIX shell wrappers in `.git/hooks/commit-msg` and `.git/hooks/post-commit` tagged with the active cccp version (`# cccp-hook-version: <ver>`). Any existing non-cccp hooks are safely backed up with incrementing suffixes (`.old`, `.old.1`, etc.).

---

## Hierarchical Configuration System

`cccp` provides a unified configuration hierarchy where settings cascade cleanly:
```text
Built-in Defaults  <  Global (~/.config/cccp/config)  <  Local Repo (.cccprc)  <  Environment Variables  <  CLI Flags
```

### The `cccp config` CLI
Configure settings easily without manually editing configuration files:

```bash
# View configuration
cccp config strict_scopes                   # Reads effective value (local > global > default)
cccp config --global update_channel         # Reads specific global value
cccp config --list                          # Lists all active global and local settings

# Set configuration
cccp config strict_scopes 1                 # Sets strict_scopes in local repository (.cccprc)
cccp config --global update_channel nightly # Sets global update channel to nightly
cccp config --global update_interval_days 7 # Check for updates every 7 days

# Unset configuration
cccp config --unset strict_scopes           # Removes override, reverting to global or default
```

### Supported Configuration Keys

| Key | Description | Default | Allowed Values |
| :--- | :--- | :--- | :--- |
| `types` | Space-separated list of allowed commit types | Standard 13 types | String of types |
| `scopes` | Space-separated list of allowed commit scopes | Standard 16 scopes | String of scopes |
| `subscopes` | Space-separated list of allowed subscopes | Standard 9 subscopes | String of subscopes |
| `strict_types` | Require commit type to match allowed `types` | `1` | `1` (strict known types), `0` (allow any alphanumeric type) |
| `strict_scopes` | Require scope to match allowed `COMMIT_SCOPES` | `0` | `0` (any scope), `1` (strict) |
| `strict_subscopes` | Require subscope to match allowed `COMMIT_SUBSCOPES` | `0` | `0` (any subscope), `1` (strict) |
| `disable_subscopes` | Disallow slash-delimited subscopes (e.g. `api/auth`) | `0` | `0` (allowed), `1` (prohibited) |
| `disable_multiple_scopes` | Disallow comma-separated scopes (e.g. `ui, api`) | `0` | `0` (allowed), `1` (prohibited) |
| `default_base_version` | Fallback SemVer when no Git tags exist | `0.0.1` | Valid SemVer (e.g. `0.0.1`, `0.1.0`) |
| `no_v` | Create release tags without `v` prefix | `0` | `0` (`v1.0.0`), `1` (`1.0.0`) |
| `update_channel` | Release channel for updates | `stable` | `stable`, `nightly` |
| `update_interval_days` | Number of days between automated update checks | `30` | Integer |
| `check_updates` | Enable automated background update notifications | `1` | `1` (enabled), `0` (disabled) |
| `pinned_version` | Freeze tool updates to a specific version | None | Valid version string (e.g. `2.0.0`) |
| `type_desc_<type>` | Custom descriptor/help message for a commit type | Built-in dictionary | String |
| `scope_desc_<scope>` | Custom descriptor/help message for a commit scope | Built-in dictionary | String |

> [!TIP]
> **Corporate & Gosplan Customization**: Enterprise teams with custom conventions can define restricted vocabularies and custom descriptors directly in their repository `.cccprc`:
> ```ini
> types = feat fix chore docs security
> strict_types = 1
> scopes = core api auth billing infrastructure
> strict_scopes = 1
> type_desc_security = Security patches and vulnerability remediations
> ```

> [!NOTE]
> **Backward Compatibility**: Legacy flags `ALLOW_ANY_SCOPE=0` and `ALLOW_ANY_SUBSCOPE=0` remain supported in environment variables and config files, mapping automatically to `STRICT_SCOPES=1` and `STRICT_SUBSCOPES=1`.

---

## Predictive Semantic Versioning

Rather than simply reflecting historical release tags, `cccp` implements **Predictive Semantic Versioning**. It scans commit messages following the Conventional Commits specification since the last valid SemVer tag (or fallback base) to anticipate and compute the **next target release version**.

### Format

- **Development Mode** (commits ahead of base tag):
  ```text
  <target_major>.<target_minor>.<target_patch>-dev.<commit_count>+<date>.<commit_hash>
  ```
  *Example:* `0.3.0-dev.6+20260914.790f5b4`

- **Tagged Release** (exact checkout on a clean tag):
  ```text
  <major>.<minor>.<patch>
  ```
  *Example:* `0.3.0`

### Version Bump Triggers

| Conventional Commit Trigger | Inferred Bump | Target Version Calculation |
| :--- | :--- | :--- |
| `BREAKING CHANGE:` in body/footer, or `!` before `:` (e.g. `feat!:`, `refactor(auth)!:`) | **MAJOR** | `(major + 1).0.0` |
| `feat:` or `feat(<scope>):` | **MINOR** | `major.(minor + 1).0` |
| `fix:`, `perf:`, `refactor:`, `chore:`, etc. | **PATCH** | `major.minor.(patch + 1)` |
| Active pre-release tag base (e.g. `v2.0.0-dev`, `v2.0.0-alpha`) | **TARGET BASE** | Exact target version (`major.minor.patch`) |
| Clean tag checkout (0 commits ahead) | **NONE** | Exact tag version |

---

## Intelligent Updates, Telemetry & Version Pinning

### Upstream Verification & Telemetry (`check-update`)
Query release telemetry to inspect installed version, active channel, remote version, and Gosplan pin status:

```bash
cccp check-update
```

Example output:
```text
★ CCCP Update Verification Bureau ★
Installed Version : 2.0.0
Release Channel   : stable
Pinned Version    : 2.0.0 (Gosplan Directive Active)
Remote Version    : 2.0.0
Status            : Pinned to 2.0.0. Updates are frozen by Gosplan decree.
```

### Self-Updating & Version Pinning (`update`)
Update `cccp` to the latest release in your active channel (`stable` or `nightly`), or pin to a specific release tag:

```bash
cccp update                      # Updates from default configured channel (stable)
cccp update --channel nightly    # Updates directly to the latest rolling nightly build
cccp update --pin                # Enacts Gosplan directive: pins to currently installed version
cccp update --pin 2.0.0          # Pins and updates to explicit release tag 2.0.0
cccp update --unpin              # Lifts Gosplan pin directive and tracks latest channel releases
```

When pinned, background update notices are automatically suppressed, and unpinning or targeting a new pin is required to perform upgrades.

### Automated Update Notifications
Every `update_interval_days` (default: 30 days), interactive commands check if a newer release is available and print a non-blocking diagnostic notice to `stderr`. When version pinning is active, automated update checks are suppressed. Automated Git hooks remain completely offline and non-blocking for speed.

### Hook Version Synchronization
When you commit, `cccp` verifies that the Git hook wrapper in `.git/hooks/` was stamped with the same version as your active `cccp` binary. If your system `cccp` was updated, it prints a friendly reminder:
```text
[cccp] Warning: Git hook 'commit-msg' was installed with cccp v1.0.0 (current: v2.0.0).
[cccp] Run 'cccp install' to synchronize git hooks with your current cccp version.
```

### Git Hooks Inspectorate & Diffing (`hooks`)

Perform precise audits of repository Git hooks, detect third-party managers (Husky, Lefthook, pre-commit), discover historical backups, and generate unified diffs against canonical CCCP wrappers:

```bash
cccp hooks                # Full inspection and actionable intervention advice
cccp hooks audit          # Explicit hook audit
cccp hooks diff           # Unified diff of all differing hooks
cccp hooks diff commit-msg # Detailed unified diff for commit-msg hook
```

Example audit report:
```text
========================================================
 ★ CCCP Git Hooks Inspectorate (Komissariat Audit) ★
========================================================
 Repository    : /home/comrade/project
 Hooks Dir     : /home/comrade/project/.git/hooks
 CCCP Version  : 2.0.0

 Hook: commit-msg
   Status      : Custom / Non-CCCP
   Framework   : Husky
   Chains CCCP : No
   Backups     : commit-msg.old
   Intervention:
     - To replace with CCCP: Run 'cccp install' (current hook will be backed up).
     - To chain CCCP inside this hook: Add 'cccp commit-msg "$@"' to .git/hooks/commit-msg.
     - To inspect differences: Run 'cccp hooks diff commit-msg'.

 Hook: post-commit
   Status      : Synchronized (v2.0.0)
   Details     : Up to date with active CCCP version.
========================================================
```

---

## Usage & Commands

```bash
cccp [command] [options]
```

| Command | Description |
| :--- | :--- |
| `status` | Inspect repository health, working tree, and Gosplan diagnostics |
| `cz, commit -i` | Assemble a commit using the interactive terminal wizard |
| `lint [range]` | Lint commit messages across a git revision range |
| `completion [shell]` | Generate shell tab autocompletion (bash, zsh, fish) |
| `commit [options] <msg>` | Validate formatting and create a commit |
| `install [--global]` | Install to PATH (`--global`) or configure Git hooks |
| `hooks [audit\|diff]` | Deep audit and unified diffing of Git hooks provenance |
| `config [options] [k] [v]` | Manage hierarchical configuration (`--global`, `--local`, `--list`, `--unset`) |
| `version` | Generate predictive SemVer metadata to `VERSION` |
| `tag [version] [options]` | Create annotated tag, commit `VERSION` and `CHANGELOG.md` |
| `changelog` | Generate or update `CHANGELOG.md` |
| `update [options]` | Update `cccp` executable (`--channel`, `--pin`, `--unpin`) |
| `check-update` | Inspect remote version, release channel, and Gosplan pin status |
| `help [command]` | Display deep-dive documentation for any command |

### Shell Autocompletion (`completion`)

Generate autocompletion scripts with commands, flags, and configuration keys:

#### Bash
```bash
# In your ~/.bashrc:
eval "$(cccp completion bash)"

# Or persist statically:
mkdir -p ~/.bash_completion.d
cccp completion bash > ~/.bash_completion.d/cccp
```

#### Zsh
```zsh
# In your ~/.zshrc (before compinit):
fpath=(~/.zsh/completion $fpath)
autoload -Uz compinit && compinit

# Generate completion script:
mkdir -p ~/.zsh/completion
cccp completion zsh > ~/.zsh/completion/_cccp
```

#### Fish
```fish
mkdir -p ~/.config/fish/completions
cccp completion fish > ~/.config/fish/completions/cccp.fish
```

### Interactive Commit Wizard (`cz` / `commit -i`)

Construct Conventional Commits effortlessly with a pure POSIX shell interactive terminal wizard (inspired by Commitizen, built for comrades):

```bash
cccp cz
# or
cccp commit -i
```

The wizard guides you through:
1. **Type selection**: Dynamically displays configured types with descriptors.
2. **Scope selection**: Quick hints for common scopes or enter custom scope.
3. **Subject line**: Imperative summary with non-empty validation.
4. **Breaking change flag**: Prompts if change introduces breaking updates (`!`).
5. **Extended body**: Optional multi-line body description.
6. **Confirmation**: Formatted preview before committing into Git history.

### Repository State Inspection (`status`)

Inspect your repository health, working tree cleanliness, Git hooks synchronization, and predictive SemVer targeting at a glance:

```bash
cccp status
```

Example output:
```text
========================================================
 ★ CCCP State Inspection (Gosplan Quality Control) ★
========================================================
 Repository:       /home/comrade/project
 Active Branch:    main (up to date with origin/main)
 Working Tree:     clean (all state directives satisfied)
 Current Version:  1.3.0
 Target Version:   1.4.0 (3 commits ahead of tag)
 Git Hooks:        installed (v1.3.0)
 System Binary:    /home/comrade/.local/bin/cccp
 Local Config:     /home/comrade/project/.cccprc
 Global Config:    /home/comrade/.config/cccp/config
 Types Policy:     strict (curated)
 Scopes Policy:    permissive (any scope)
 Update Channel:   stable (every 30 days)
========================================================
```

### Commit History & Pull Request Linting (`lint`)

Verify that all commits in a branch, pull request, or history adhere to the Conventional Commits specification:

```bash
cccp lint                          # Auto-detects range (upstream or last commit)
cccp lint origin/main..HEAD        # Lint all commits in current feature branch
cccp lint HEAD~5..HEAD             # Lint the last 5 commits
```

#### GitHub Actions Pull Request Workflow

Ensure zero non-compliant commits reach your main branch:

```yaml
name: Lint Commits
on:
  pull_request:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Run CCCP Commit Lint
        run: |
          curl -sSL https://raw.githubusercontent.com/lumenpink/cccp.sh/main/cccp.sh -o /usr/local/bin/cccp
          chmod +x /usr/local/bin/cccp
          cccp lint origin/${{ github.base_ref }}..HEAD
```

### Release Tagging (`tag`)

The `tag` command verifies a clean working tree, updates `VERSION` and `CHANGELOG.md`, creates a release commit (`chore(release): <tag> - <title>`), and creates an annotated Git tag:

```bash
cccp tag -t "Sputnik Protocol"               # Auto-tags with predicted version and title
cccp tag 2.0.0 -t "Major Overhaul"           # Explicit version with release title
cccp tag 2.1.0 --no-v                        # Tagged as 2.1.0 (without 'v')
cccp tag 1.0.0 -m "Custom tag annotation"    # Custom raw annotation
```

Release titles are automatically formatted into `CHANGELOG.md` (`### [v1.2.0] - Sputnik Protocol`) and published to GitHub Release titles.

### Commit Message Format

```text
<type>(<scope>): <subject>
<type>(<scope>/<subscope>): <subject>
<type>(<scope>)!: <subject>
<type>!: <subject>
```

#### Supported Types
- `feat`     - New feature (Minor bump)
- `fix`      - Bug fix (Patch bump)
- `perf`     - Performance improvement (Patch bump)
- `refactor` - Code refactoring (Patch bump)
- `revert`   - Revert changes
- `chore`    - Maintenance tasks
- `build`    - Build system changes
- `ci`       - CI configuration changes
- `docs`     - Documentation changes
- `ops`      - Operational changes
- `style`    - Code style changes
- `test`     - Test related changes
- `merge`    - Merge commits

### Help Documentation (`help`)

Display comprehensive documentation, command guides, and conventional commit specifications stamped with the active tool version:

```bash
cccp help            # General documentation with active CCCP version banner
cccp help <command>  # Command-specific detailed manual (e.g. cccp help tag, cccp help config)
cccp -h, --help      # Short help flags
```

### State Directive & Cultural Archives (`soviet`)

For true comrades and enthusiasts of Soviet heritage, `cccp` includes cultural commands and historic state directives:

```bash
cccp soviet      # Displays the state emblem, Order No. 227 ("Not a step back!"), and directives
cccp sputnik     # Alias for state inspection and cultural archives
cccp anthem      # Alias for the cultural archives
cccp gosplan     # Alias for state inspection
```

---

## Development & Dual-Test Architecture

`cccp` follows a rigorous dual-matrix testing standard inspired by Gosplan quality assurance. The codebase is maintained modularly during development and compiled into a standalone, single-file POSIX distribution script.

```text
       ┌────────────────────────┐
       │   src/ (Modular Code)  │
       └───────────┬────────────┘
                   │
         ┌─────────┴─────────┐
         ▼                   ▼
  Development         Distribution
   (bin/cccp)         (combine.sh)
         │                   │
         │                   ▼
         │             ┌───────────┐
         │             │  cccp.sh  │
         │             └─────┬─────┘
         ▼                   ▼
    Matrix Run 1        Matrix Run 2
  [Modular Tests]     [Bundled Tests]
   (shellspec)      (CCCP_TEST_TARGET=bundled)
```

### Running Tests

Run the complete test suite across both matrices:

```bash
# 1. Run tests against modular development source code
shellspec

# 2. Compile standalone distribution bundle
./combine.sh

# 3. Run tests against compiled standalone bundle
CCCP_TEST_TARGET=bundled shellspec
```

Both test matrices run automatically in CI via GitHub Actions on every pull request and push to `main`.

---

## Benefits

- **Zero Runtime Dependencies**: Works anywhere POSIX shell and Git are available.
- **Pure POSIX (Zero AWK)**: 100% independent of `awk`, running identically on minimal BusyBox (`ash`), Debian (`dash`), macOS, Linux, and Windows Git Bash.
- **Dual-Matrix Verified**: 100% verified across both modular source code and compiled standalone bundle.
- **Strict or Flexible**: Defaults to permissive flags while allowing fine-grained enforcement via `.cccprc` or CLI flags.
- **Predictive Versioning**: Know what version will be published before making the release tag.
- **Safe & Auditable**: Git hooks are standalone scripts, not fragile symlinks, stamped with version provenance.

## Support

If you encounter any issues or have questions, please open an issue in the project repository.

