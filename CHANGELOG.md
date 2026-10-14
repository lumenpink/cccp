# Changelog

## [Unreleased]

### Multi-Repository Ticket Cache Isolation
- **Target Repository Ticket Cache Isolation**: Garante que tanto o `divan` quanto o comando `cccp ticket` sincronizem e atualizem `.antigravity/tickets.md` estritamente na raiz do repositório alvo (via catálogo ou caminho local), impedindo a poluição indevida do diretório corrente (`cwd`) ao operar sobre repositórios externos. (closes #140)

### Local CI Runner & Pre-Push Pause Optimization
- **Isolate Test Jobs in Local CI (`act`)**: Filtra e isola exclusivamente jobs de teste/verificação (`test`, `lint`, `check`), descartando jobs de deploy, release e publicação do workflow temporário passado ao `act`, limpando dependências `needs:` obsoletas e evitando falhas de validação de CD no pre-push hook. (closes #130)
- **Auto-pause forgejo-runner during local CI (`act`)**: Congela via `docker pause` o container daemon do `forgejo-runner` durante a execução do CI local (pre-push hook e `cccp act run`), descongelando-o deterministicamente ao término via guard RAII para evitar contenção de CPU e RAM no host. (closes #144)

### Automated Merge Ticket Closing & Merge Lock Gate
- **Merge Lock Gate (`cccp merge <branch>`)**: Bloqueia a integração de branches que não contêm commits com a tag canônica `(closes #<id>)`, garantindo conformidade irrestrita com a Fase 9 do GOST DD-TDD. (closes #133)
- **Post-Merge Git Hook (`post-merge`)**: Hook nativo de Git que inspeciona `ORIG_HEAD..HEAD`, identifica tags `(closes #<id>)` e encerra automaticamente as issues na API remota do forge. (closes #133)
- **Ticket Closure CLI (`cccp ticket close <ids...> [-m <comment>]`)**: Encerramento direto de tickets via API do forge (Codeberg/Forgejo/GitHub/GitLab) sem depender de ferramentas de terceiros. (closes #133)

## [v3.1.0] - 2026-10-03

### Multilingual Support, Divan Context Injection & Catalog Aliases

CCCP 3.1.0 expands the native standalone Rust binary with embedded internationalization, intelligent codebase context extraction for AI triage, enhanced terminal aesthetics, source installation tooling, and flexible repository catalog aliases.

#### Key Features & Improvements
- **Embedded Multilingual Internationalization (i18n)**: Zero-overhead embedded localization system supporting `en-US` (default fallback), `pt-BR`, `ru-RU`, and `es-ES` across the entire CLI with 4-tier precedence cascade (`CLI --lang < Config < ENV < Defaults`). (closes #110, #118)
- **Automatic Project Context & Symbols Injection**: Built-in manifest scanner (`Cargo.toml`, `package.json`, `pyproject.toml`, etc.), exported symbol extractor, and Gosplan token-budgeted prompt injection for Divan with `.git/cccp-context.cache` caching. (closes #116)
- **Constructivist Card Visual Formatting for Tickets**: High-aesthetic box-drawing cards with ANSI-aware padding and word-wrapping for ticket reviews in Divan. (closes #119)
- **CLI Commands Overhaul & Rationalization**: Unified commands under clear mnemonic English (`commit`, `version`, `lint`, `audit`, `sync`, `divan`), deprecating redundant aliases and unifying synchronization. (closes #115)
- **Soviet Constructivist Terminology Restored**: Reinstated ideological Party Line compliance, Order No. 227 directives, and full 9-point GOST compliance audit. (closes #120)
- **Compilation & Installation Infrastructure**: Root `install.sh` and POSIX `Makefile` with automatic toolchain verification, custom prefix/bindir, staging destdir, and shell PATH validation. (closes #121)
- **Repository Aliases in Catalog**: Support for short aliases (`alias` and `aliases`) in `catalog.json` with safety collision detection, enabling fast repo references in CCCP commands and Divan. (closes #123)
- **Comprehensive Test Suite & Quality Inspectorate**: 27 test targets covering linting, cultural themes, command parsing, E2E binaries, i18n dictionaries, and installer verification with 100% pass rate. (closes #111, #112, #113, #114)

## [v3.0.0] - 2026-09-30

### Complete Rust Migration & Multiplatform Release

CCCP 3.0.0 is a complete, ground-up rewrite in high-performance Rust, replacing the legacy POSIX shell implementation with a single, statically linked binary with zero external runtime dependencies (no libc, no curl, no OpenSSL).

#### Architecture & Multiplatform Support (32-bit & 64-bit)
- **First-Class 32-bit & 64-bit Binaries**: Official static builds for Linux and macOS via `cargo-zigbuild`:
  - `linux-386` (`i686-unknown-linux-musl`) — 32-bit x86 systems
  - `linux-armv7` (`armv7-unknown-linux-musleabihf`) — 32-bit ARM (Raspberry Pi, embedded devices)
  - `linux-amd64` (`x86_64-unknown-linux-musl`) — 64-bit x86_64 systems
  - `linux-arm64` (`aarch64-unknown-linux-musl`) — 64-bit ARM (AArch64)
  - `darwin-amd64` (`x86_64-apple-darwin`) — macOS Intel
  - `darwin-arm64` (`aarch64-apple-darwin`) — macOS Apple Silicon
- **Universal POSIX Installer**: Automated bootstrap script (`site/install.sh`) detecting all 6 targets with SHA-256 verification and zero external tools.

#### Local Offline AI & Forge Integrations
- **Local Ollama Integration**: Added offline AI-driven ticket triage via local Ollama daemon (`cccp diva --provider ollama --model <model>`) alongside Google Gemini API (`cccp diva --provider gemini`) and direct manual triage (`--direct`).
- **Multi-Forge Tickets Synchronizer**: Unified issue synchronization (`cccp tickets`) supporting Codeberg, Forgejo, GitHub, and GitLab with embedded TLS (`rustls`).

#### Developer Experience & Performance
- **Sub-3ms Git Hooks**: Lightning-fast git hook execution (`cccp hook commit-msg` and `cccp hook pre-push`) with automated hook installation (`cccp hook install`).
- **Interactive Commit Wizard**: Terminal commit wizard (`cccp cz`) built with `inquire`, offering intuitive type/scope selection, breaking change prompts, and GPG/SSH signing passthrough.
- **Strict Linting & SemVer Calculation**: Conventional Commits 1.0.0 validator (`cccp lint`) and predictive SemVer 2.0.0 release calculator (`cccp semver`) with tag inspection via `gix`.
- **Gosplan Cultural Aesthetics & Config**: 4-tier hierarchical configuration (`.cccprc`, XDG, ENV, CLI flags), authentic ASCII banners, and strict `NO_COLOR` standard compliance.

#### Legacy Features & Tooling

  - (triage) adicionar opcao para postagem direta sem intermedio do gemini (closes #55)
  - (cli) aceitar repositorio como parametro posicional unico no diva (closes #54)
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
  - (apps) corrigir identificacao de forge e metadados no app tickets e subapps (closes #48)
  - (diva) usar comrade kv (kvant) como editor de texto padrao (closes #50)
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

