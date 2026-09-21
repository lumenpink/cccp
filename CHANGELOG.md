# Changelog

## [Unreleased]

### Features
  - (tickets) add GitHub issues sync and Gosplan rate limiter
  - (agent) add fill subcommand and generate template despite missing variables
  - (codeberg) add cccp codeberg setup command (#4)
  - (agent) standardise command to singular with plural fallback
  - (hooks) add sync-tickets and sync-agents to post-push hook (#3)
  - (cli) wire agents command and autocompletion into cccp (#2)
  - (agents) implement dynamic AGENTS.md generator and variable manager (#2)
  - (cli) add dynamic codeberg tickets synchronization command
  - (hooks) implement optional post-push hook automation and cccp push command

### Bug Fixes
  - (agent) persist --url to .cccprc automatically when passed via CLI
  - (agent) correct URL resolution precedence and add content validation
  - (agents) align confirmation output and JSON handling (#2)
  - (agents) support multi-line variables and exact awk substring replacement (#2)
  - (hooks) allow non-semver utility tags and bypass remote sync in reference-transaction

## Previous Releases

### [v2.2.0]

### Features
  - (hooks) implement pre-push and reference-transaction tag guards and CI check

### [v2.1.0] - Gosplan State Inspection & SemVer Protocol

### Features
  - (config) enforce global-only scope for tool update directives
  - (version) absorb breaking changes in X.0.0 pre-releases and promote on non-zero pre-releases
  - (hooks) provide detailed hooks audit, diffing, and intervention guide
  - (update) add check-update command and version pinning support
  - (help) display active version in all help headers

### Bug Fixes
  - (version) target base version when latest tag is a pre-release
  - (core) establish single-source version injection and eliminate recursive invocations

### [v2.0.0] - Pure POSIX Architecture & Gosplan Dual Matrix

### Features
  - (validation) enhance prerequisite checks without awk dependency
  - (core) introduce modular dispatcher and development runner

### Bug Fixes
  - (core) ensure safe sourcing and robust configuration hierarchy across modules
  - (changelog) omit empty section headings and add default release fallback

### [v1.7.0] - Teleprinter Telegraph (Shell Autocompletion)

### Features
  - (completion) add shell autocompletion for bash, zsh, and fish

### [v1.6.0] - Soviet Assembly (Interactive Commit Wizard)

### Features
  - (interactive) add pure shell terminal commit wizard

### [v1.5.0] - Party Line Compliance (Commit Linting)

### Features
  - (lint) add history and pull request commit linting

### [v1.4.0] - State Inspection (Repository Status)

### Features
  - (status) add repository status and health diagnostic command

### [v1.3.0] - Gosplan Directive (Custom Types and Scopes)

### Features
  - (config) support custom types, scopes, and descriptor lookup

### [v1.2.0] - Sputnik Protocol

### Features
  - (tag) support release titles with -t and --title flags
  - (update) add update checking, release channels and hook version synchronization
  - (install) support global CLI installation to PATH with shell profile integration
  - (config) add hierarchical configuration system and cccp config CLI command
  - (hooks) use standalone portable wrapper scripts with version tracking
  - (config) standardize default flags to 0 and set base version to 0.0.1

### Bug Fixes
  - (tag) exclude VERSION and CHANGELOG.md from pre-tag dirty check
  - (ci) configure git default branch to silence hints during spec tests
  - (ci) install shellspec to user space and export PATH
  - (ci) correct YAML escaping for multiline release notes and quote workflow name

### [v1.1.0]

### Features
  - (tag) automate release commit creation and working tree validation
  - (cli) add command-specific help and release tagging command
  - (version) implement predictive semantic versioning, prerequisite checks and scope validation fixes

### [v1.0.2] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v1.0.1] - test

### Features
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - add updatescript function

### [v1.0.0] - first version

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.9] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.8] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.7] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.6] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.5] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.4] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.3] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.2] - test

- Routine maintenance, documentation updates, and operational improvements.

### [v0.0.1] - test

### Features
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe
  - testttqwe

### [nightly] - v2.2.0 - Gosplan Tag Defense & Gatekeeper Bureau

### Features
  - (hooks) implement pre-push and reference-transaction tag guards and CI check
  - (config) enforce global-only scope for tool update directives
  - (version) absorb breaking changes in X.0.0 pre-releases and promote on non-zero pre-releases
  - (hooks) provide detailed hooks audit, diffing, and intervention guide
  - (update) add check-update command and version pinning support
  - (help) display active version in all help headers
  - (validation) enhance prerequisite checks without awk dependency
  - (core) introduce modular dispatcher and development runner
  - (completion) add shell autocompletion for bash, zsh, and fish
  - (interactive) add pure shell terminal commit wizard
  - (lint) add history and pull request commit linting
  - (status) add repository status and health diagnostic command
  - (config) support custom types, scopes, and descriptor lookup
  - (tag) support release titles with -t and --title flags
  - (update) add update checking, release channels and hook version synchronization
  - (install) support global CLI installation to PATH with shell profile integration
  - (config) add hierarchical configuration system and cccp config CLI command
  - (hooks) use standalone portable wrapper scripts with version tracking
  - (config) standardize default flags to 0 and set base version to 0.0.1
  - (tag) automate release commit creation and working tree validation
  - (cli) add command-specific help and release tagging command
  - (version) implement predictive semantic versioning, prerequisite checks and scope validation fixes
  - add updatescript function
  - first version

### Bug Fixes
  - (version) target base version when latest tag is a pre-release
  - (core) establish single-source version injection and eliminate recursive invocations
  - (core) ensure safe sourcing and robust configuration hierarchy across modules
  - (changelog) omit empty section headings and add default release fallback
  - (tag) exclude VERSION and CHANGELOG.md from pre-tag dirty check
  - (ci) configure git default branch to silence hints during spec tests
  - (ci) install shellspec to user space and export PATH
  - (ci) correct YAML escaping for multiline release notes and quote workflow name

