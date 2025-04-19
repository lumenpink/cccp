# cccp.sh - Conventional Commits Compliance Program

A comprehensive tool for enforcing and managing conventional commits in your Git workflow. This program helps maintain consistent commit messages and automate version management through semantic versioning.

## What is it?

cccp.sh is a Git hook-based solution that:
- Enforces conventional commit message formatting
- Validates commit messages locally before they're pushed
- Automates semantic version generation based on commit history
- Helps maintain clean and meaningful commit history
- Simplifies version management in your projects

## Prerequisites

Your computer must have the following technology(s) installed:
- git (https://git-scm.com/)

> [!IMPORTANT]
> If you are using Windows as your operating system, you must run all commands through the "git bash" application.

## Installation

To install the hook on your machine, run the following script inside this project folder:
```bash
chmod +x cccp.sh
./cccp.sh install
```

## Version Numbering

The version number follows the format: `<major>.<minor>.<patch>+<commit_count>.<date>.<commit_hash>`

For example: `1.0.0+31.20250419.88338c1`

Where:
- `1.0.0` is the semantic version tag
- `+31` is the number of commits since the last tag
- `20250419` is the current date in YYYYMMDD format
- `88338c1` is the short hash of the second-to-last commit

## Usage

### Commands

```bash
cccp.sh [command] [options]
```

Available commands:
- `commit <message>`    - Create a commit with a conventional commit message
- `install`            - Install git hooks for commit message validation
- `version`            - Generate version information file
- `changelog`          - Generate or update CHANGELOG.md
- `help`               - Show help message

### Git Hooks
- `commit-msg`         - Validates commit messages for conventional commit format
- `post-commit`        - Automatically updates changelog and version after commit

### Commit Message Format
```
<type>(<scope>): <subject>
```

#### Types
- `feat`     - New feature
- `fix`      - Bug fix
- `perf`     - Performance improvement
- `refactor` - Code refactoring
- `revert`   - Revert changes
- `chore`    - Maintenance tasks
- `build`    - Build system changes
- `ci`       - CI configuration changes
- `docs`     - Documentation changes
- `ops`      - Operational changes
- `style`    - Code style changes
- `test`     - Test related changes
- `merge`    - Merge commits

#### Scopes
- `ui`       - User interface changes
- `docs`     - Documentation changes
- `api`      - API changes
- `docker`   - Docker related changes
- `db`       - Database changes

#### Subscopes
- `components` - UI components
- `pages`      - Page components
- `services`   - Service layer
- `utils`      - Utility functions
- `auth`       - Authentication related

### Environment Variables
- `DISABLE_SUBSCOPES`         - Set to 1 to disable subscopes
- `DISABLE_MULTIPLE_SCOPES`   - Set to 1 to disable multiple scopes
- `ALLOW_ANY_SUBSCOPE`        - Set to 1 to allow any subscope
- `ALLOW_ANY_SCOPE`           - Set to 1 to allow any scope

### Examples
```bash
cccp.sh commit 'feat(ui): add new button'
cccp.sh commit 'fix(api/auth): resolve login issue'
cccp.sh install
cccp.sh version
cccp.sh changelog
```

> Note: After installation, git hooks will automatically validate commit messages and update the changelog and version information after each commit.

## Benefits

- Consistent commit history
- Automated version management
- Better changelog generation
- Improved collaboration
- Clear project documentation
- Simplified release process

## Support

If you encounter any issues or have questions, please open an issue in the project repository.
