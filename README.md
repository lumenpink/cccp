# cccp.sh 
## Conventional Commits Compliance Program

A comprehensive tool for enforcing and managing conventional commits in your Git workflow. This program helps maintain consistent commit messages and automate version management through semantic versioning.

## What is it?

cccp.sh is a Git hook-based solution that:
- Enforces conventional commit message formatting
- Validates commit messages locally before they're pushed
- Automates semantic version generation based on commit history
- Helps maintain clean and meaningful commit history
- Simplifies version management in your projects

## Prerequisites

Your system must have the following tools installed and available in your `PATH`:
- `git` (https://git-scm.com/)
- Standard POSIX utilities: `sed`, `grep`, `date`, `cut`, `tr`

`cccp.sh` automatically verifies the availability of these prerequisites upon execution. If any requirement is missing or if the command is executed outside of a Git repository, it immediately terminates with a clear and explanatory error message.

> [!IMPORTANT]
> If you are using Windows as your operating system, you must run all commands through the "Git Bash" terminal application.

## Installation

To install the Git hooks in your repository, run:
```bash
wget https://github.com/lumenpink/cccp.sh/raw/refs/heads/main/cccp.sh
chmod +x cccp.sh
./cccp.sh install
```

This registers the `commit-msg` and `post-commit` hooks in `.git/hooks/`.

## Predictive Semantic Versioning

Rather than simply reflecting historical release tags, `cccp.sh` implements **Predictive Semantic Versioning**. It scans commit messages following the Conventional Commits specification since the last valid SemVer tag (or fallback base) to anticipate and compute the **next target release version**.

### Format

- **Development Mode** (commits ahead of base tag):
  ```
  <target_major>.<target_minor>.<target_patch>-dev.<commit_count>+<date>.<commit_hash>
  ```
  *Example:* `0.3.0-dev.6+20260914.790f5b4`

- **Tagged Release** (exact checkout on a clean tag):
  ```
  <major>.<minor>.<patch>
  ```
  *Example:* `0.3.0`

### Next Version Bump Rules

| Conventional Commit Trigger | Inferred Bump | Target Version Calculation |
| :--- | :--- | :--- |
| `BREAKING CHANGE:` in body/footer, or `!` before `:` (e.g. `feat!:`, `refactor(auth)!:`) | **MAJOR** | `(major + 1).0.0` |
| `feat:` or `feat(<scope>):` | **MINOR** | `major.(minor + 1).0` |
| `fix:`, `perf:`, `refactor:`, `chore:`, etc. | **PATCH** | `major.minor.(patch + 1)` |
| Clean tag checkout (0 commits ahead) | **NONE** | Exact tag version |

The highest priority bump detected in the commit range determines the predicted version.

## Usage

### Commands

```bash
./cccp.sh [command] [options]
```

Available commands:
- `commit <message>`    - Create a commit after validating message formatting
- `install`             - Install Git hooks for commit message validation and automated versioning
- `version`             - Generate and write predictive version information to `VERSION`
- `tag [version]`       - Create release tag, update `VERSION` and regenerate `CHANGELOG.md`
- `changelog`           - Generate or update `CHANGELOG.md`
- `update`              - Self-update the script to the latest upstream release
- `help [command]`      - Display general help or deep-dive help on a specific command

### Release Tagging (`tag`)

The `tag` command enforces a clean working tree, updates `VERSION` and `CHANGELOG.md`, creates a release commit (`chore(release): <tag>`), and tags that commit with an annotated Git tag:

```bash
./cccp.sh tag                     # Auto-tags with the predicted SemVer version (e.g. v1.2.0)
./cccp.sh tag 2                   # Normalized to v2.0.0
./cccp.sh tag 2.1                 # Normalized to v2.1.0
./cccp.sh tag 2.1.0               # Normalized to v2.1.0
./cccp.sh tag 2.1.0 --no-v        # Tagged as 2.1.0 (without 'v' prefix)
./cccp.sh tag 1.0.0 -m "Release"  # Custom tag annotation message
```

### Git Hooks

- `commit-msg`: Validates commit messages for Conventional Commits compliance before saving the commit.
- `post-commit`: Automatically updates `CHANGELOG.md` and generates the predicted `VERSION` file.

### Commit Message Format

```text
<type>(<scope>): <subject>
<type>(<scope>)!: <subject>
<type>!: <subject>
```

#### Supported Types
- `feat`     - New feature (triggers Minor bump)
- `fix`      - Bug fix (triggers Patch bump)
- `perf`     - Performance improvement (triggers Patch bump)
- `refactor` - Code refactoring (triggers Patch bump)
- `revert`   - Revert changes
- `chore`    - Maintenance tasks
- `build`    - Build system changes
- `ci`       - CI configuration changes
- `docs`     - Documentation changes
- `ops`      - Operational changes
- `style`    - Code style changes
- `test`     - Test related changes
- `merge`    - Merge commits

#### Breaking Changes
Breaking changes can be signaled in two standard ways:
1. Appending an exclamation mark `!` right before the colon:
   `feat(api)!: drop deprecated endpoints` or `refactor!: restructure configuration format`
2. Adding a `BREAKING CHANGE:` or `BREAKING-CHANGE:` block in the commit body/footer.

#### Scopes and Subscopes
- **Scopes**: Categorize the module or subsystem (e.g., `ui`, `api`, `auth`, `updater`, `db`).
- **Multiple Scopes**: Comma-separated scopes are supported (e.g., `feat(api, ui): update endpoint`).
- **Subscopes**: Monorepo or hierarchical paths using `/` (e.g., `fix(api/auth): token expiration`).

### Configuration & Environment Variables

- `ALLOW_ANY_SCOPE=1` (Default: `1`): Allows any non-empty scope. Set to `0` to enforce strict matching against `COMMIT_SCOPES`.
- `ALLOW_ANY_SUBSCOPE=1` (Default: `1`): Allows any non-empty subscope. Set to `0` to enforce strict matching against `COMMIT_SUBSCOPES`.
- `DISABLE_SUBSCOPES=1`: Prohibits slash-delimited subscopes.
- `DISABLE_MULTIPLE_SCOPES=1`: Prohibits comma-separated multiple scopes.
- `DEFAULT_BASE_VERSION="0.2.0"`: Fallback base version used when no SemVer tags exist in the Git history.

### Examples

```bash
# Feature commit
./cccp.sh commit 'feat(updater): support rollback command'

# Breaking change commit
./cccp.sh commit 'refactor(api)!: migrate to typed response objects'

# Subscope commit
./cccp.sh commit 'fix(auth/token): validate PKCE verifier length'

# Multiple scopes
./cccp.sh commit 'docs(api, cli): update usage instructions'

# Generate predicted version
./cccp.sh version
```

## Benefits

- Consistent, audit-ready commit history
- Anticipatory, automated SemVer 2.0 version calculation
- Instant local Git hook validation with clear diagnostics
- Zero external runtime dependencies in production

## Support

If you encounter any issues or have questions, please open an issue in the project repository.

