# Changelog

## [Unreleased]

### Features
  - (deps) implementar gerenciador de dependencias com list, check e install (#61)
  - (cli) implementar comando newrepo para scaffolding de repositórios no modelo gost (closes #57)
  - (cockpit) unificar interface grafica no comando cockpit com selecao de tabs (closes #59)
  - (matraca) integrar matraca como web repl movil tolerante a falhas no cccp (closes #58)
  - (act,pre-push) implement pre-push hook with act: local CI gate, live spinner and structured logs (closes #37)
  - (repo) implementar visão geral e gestão unificada de múltiplos repositórios (closes #38)
  - (serve) implementar comando unificado cccp serve (estático, dev e demo) (closes #39)
  - (gost,check) suporte a conformidade GOST (leitura de GOSTVER, cccp gost sync, check e upgrade) (closes #47) (closes #42)
  - (cockpit) adicionar seletor de repositorio ativo e suporte global a -C (closes #43)
  - (repo,diva,catalog) unify forge identity by domain and software engine by forgeapp (closes #46) (closes #34)
  - (web) adotar opt-in --open, resolucao de chave via env/flag e autenticacao no cockpit (closes #44) (closes #45)
  - (web) merge web interface and mayak command bridge (closes #41)
  - (diva) add cccp ticket/issue command with branch suggestion (closes #36)
  - (comrades) support mayak (mk) static micro-server in comrades manager
  - (web) implement cccp web command and mayak command bridge (ref #41)
  - (comrades) support ARM32 architectures, 1960s-style raw bootstrap downloaders, and auto-recruitment
  - (deps) implement pure POSIX token reader for gh and issue url parser without jq (#33)
  - (diva) integrar editor kvant (kv) para edicao de tickets com navegacao por teclado
  - (llm) permitir rediscutir parametros e refinar triagem com a Gemini no diva
  - (ui) permitir edicao manual de titulo e corpo dos tickets no diva
  - (cli) suportar nome parcial de repositorio na opcao -r do diva

### Bug Fixes
  - (act) assegurar extensao yml em workflow temporario para o act (#61)
  - (act) corrigir flag pull e compatibilidade com workflows forgejo no runner (#61)
  - (diva) validar existência local do repositório informado via flag -r (closes #35)
  - (diva,tickets) suporte a forgejo, persistencia de drafts e correcao de hosts hardcoded

## Previous Releases

### [v2.3.0] - v2.3.0

### Features
  - (tag) recompile and stage cccp.sh in release commit
  - (catalog) suporte a diretório padrão via config global, env e flags
  - (diva) display target repository and forge in prompts and ticket preview
  - (diva) update default gemini model to gemini-3.6-flash
  - (diva) fallback to ~/.env for gemini api key resolution when unset in memory
  - (tooling) establish POSIX shell formatting standards and linting automation
  - (diva) integrate gemini-powered interactive ticket triage bot into posix shell
  - (catalog) port catalog metadata engine and forge synchronizer to posix shell
  - (web) implement hashbang polyglot installer and dedications section
  - (update) refactor cccp update and install routines for cccp.2lp.in channels (closes #17)
  - (installer) polyglot html/shell installer and loving tribute to Lisi (closes #15) (closes #16)
  - (dist) configure stable and nightly release publishing on cccp.2lp.in (closes #14)
  - (ci) migrate cloudflare deployment to workers static assets with custom domain
  - (site) update header to standalone svg badge, add author and unlicense links
  - (site) add programmer credit with trans flag to footer (closes #9)
  - (site) adicionar site estatico bilingue, design sovietico e servidor de preview (closes #7)
  - (tickets) implement generic forge detection and multi-driver ticket sync (#5)
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
  - (generator) prevent truncation of long/tall tickets in output
  - (generator) fix ASCII table right border and text wrapping in columns
  - (install) prevent same-file error and prioritize workspace script in install -g
  - (bin) include catalog and diva in unbundled development runner
  - (hooks) respect XDG_BIN_HOME and CCCP_INSTALL_DIR when running as root
  - (site) restore standard html5 document structure and use worker content negotiation
  - (hooks) keep only the latest hook backup on install (closes #6)
  - (tickets) handle empty issue lists gracefully without false error warning
  - (agent) persist --url to .cccprc automatically when passed via CLI
  - (agent) correct URL resolution precedence and add content validation
  - (agents) align confirmation output and JSON handling (#2)
  - (agents) support multi-line variables and exact awk substring replacement (#2)
  - (hooks) allow non-semver utility tags and bypass remote sync in reference-transaction

### [v2.2.0]

### Features
  - (hooks) implement pre-push and reference-transaction tag guards and CI check

### [v2.1.0] - v2.1.0 - Gosplan State Inspection & SemVer Protocol

### Features
  - (config) enforce global-only scope for tool update directives
  - (version) absorb breaking changes in X.0.0 pre-releases and promote on non-zero pre-releases
  - (hooks) provide detailed hooks audit, diffing, and intervention guide
  - (update) add check-update command and version pinning support
  - (help) display active version in all help headers

### Bug Fixes
  - (version) target base version when latest tag is a pre-release
  - (core) establish single-source version injection and eliminate recursive invocations

### [v2.0.0] - disable update check in spec_helper and cli_spec to prevent test warnings

### Features
  - (validation) enhance prerequisite checks without awk dependency
  - (core) introduce modular dispatcher and development runner

### Bug Fixes
  - (core) ensure safe sourcing and robust configuration hierarchy across modules
  - (changelog) omit empty section headings and add default release fallback

### [v1.7.0] - v1.7.0 - Teleprinter Telegraph (Shell Autocompletion)

### Features
  - (completion) add shell autocompletion for bash, zsh, and fish

### [v1.6.0] - v1.6.0 - Soviet Assembly (Interactive Commit Wizard)

### Features
  - (interactive) add pure shell terminal commit wizard

### [v1.5.0] - v1.5.0 - Party Line Compliance (Commit Linting)

### Features
  - (lint) add history and pull request commit linting

### [v1.4.0] - v1.4.0 - State Inspection (Repository Status)

### Features
  - (status) add repository status and health diagnostic command

### [v1.3.0] - v1.3.0 - Gosplan Directive (Custom Types and Scopes)

### Features
  - (config) support custom types, scopes, and descriptor lookup

### [v1.2.0] - v1.2.0 - Sputnik Protocol

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

### [v1.1.0] - v1.1.0

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

