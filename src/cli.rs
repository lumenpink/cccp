//! Command Line Interface definition for CCCP using Clap derive macros.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// ★ CCCP - Conventional Commits Compliance Program ★
/// Native, fast, standalone binary for Git governance, SemVer, and Soviet aesthetics.
#[derive(Debug, Parser)]
#[command(
    name = "cccp",
    author = "Lumen Pink <hi@lumen.pink>",
    version = env!("CCCP_CANONICAL_VERSION"),
    about = "★ CCCP - Conventional Commits Compliance Program ★",
    long_about = "Conventional Commits Compliance Program (CCCP)\n\
                  High-performance, standalone Git tool for Conventional Commits enforcement,\n\
                  predictive Semantic Versioning 2.0.0, automated changelogs, and Forge integration.",
    styles = styling()
)]
pub struct Cli {
    /// Change working directory before executing commands
    #[arg(short = 'C', long = "dir", global = true, value_name = "DIR")]
    pub dir: Option<PathBuf>,

    /// Increase logging verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', long = "verbose", global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress non-essential output
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,

    /// Disable ANSI color output (also respects NO_COLOR env var)
    #[arg(long = "no-color", global = true)]
    pub no_color: bool,

    /// Interface language/locale (en-US, pt-BR, ru-RU, es-ES)
    #[arg(long = "lang", global = true, value_name = "LOCALE")]
    pub lang: Option<String>,

    /// Primary subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

fn styling() -> clap::builder::styling::Styles {
    clap::builder::styling::Styles::styled()
        .header(
            clap::builder::styling::AnsiColor::Red.on_default()
                | clap::builder::styling::Effects::BOLD,
        )
        .usage(
            clap::builder::styling::AnsiColor::Yellow.on_default()
                | clap::builder::styling::Effects::BOLD,
        )
        .literal(
            clap::builder::styling::AnsiColor::Cyan.on_default()
                | clap::builder::styling::Effects::BOLD,
        )
        .placeholder(clap::builder::styling::AnsiColor::BrightBlack.on_default())
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create or switch to ticket branch (GOST Phase 2)
    Start(StartArgs),

    /// Interactive Conventional Commits wizard or automated DD-TDD phase commit
    Commit(CommitArgs),

    /// Close development cycle with resolution report (GOST Phase 9)
    Close(CloseArgs),

    /// Validate commit messages against Conventional Commits specification
    Lint(LintArgs),

    /// Display current installed CCCP, GOST, and repository versions
    Version(VersionArgs),

    /// Calculate predictive Semantic Versioning (SemVer 2.0.0) metadata
    #[command(name = "gen-version")]
    GenVersion(GenVersionArgs),

    /// Manage or dispatch Git hooks in repository (--install, --uninstall)
    Hook(HookArgs),

    /// Audit repository Conventional Commits compliance, hooks, and integrity
    Status(StatusArgs),

    /// Query or manage forge issues and pull requests
    #[command(alias = "tickets")]
    Ticket(TicketArgs),

    /// Interactive AI ticket triage and consultation bot
    #[command(alias = "diva")]
    Divan(DivanArgs),

    /// Synchronize forge tickets, AGENTS.md directives, and repository catalog
    Sync(SyncArgs),

    /// Query and interact with remote Git forge (Codeberg, GitHub, GitLab, Gitea)
    Forge(ForgeArgs),

    /// Gosplan cultural registry, Order 227 ("Not a step back!"), and Soviet comrades
    Soviet(SovietArgs),

    /// Create an annotated Git release tag with predicted or explicit SemVer
    Tag(TagArgs),

    /// Generate or update CHANGELOG.md based on conventional commit history
    Changelog(ChangelogArgs),

    /// Push commits and tags to remote with pre-push compliance checks
    Push(PushArgs),

    /// Inspect or manage hierarchical CCCP configuration
    Config(ConfigArgs),

    /// Comprehensive repository GOST quality and compliance check
    Check(CheckArgs),

    /// Generate shell completion scripts (bash, zsh, fish, powershell)
    Completion(CompletionArgs),

    /// Manage repositories, workspace catalog, and GOST scaffolding
    Repo(RepoArgs),

    /// Dependency catalog, constraint verification, and ecosystem tools
    Deps(DepsArgs),

    /// Generate, inspect, and manage AGENTS.md directives
    Agent(AgentArgs),

    /// Mobile & offline-tolerant queue and spool for Antigravity CLI (agy)
    Spool(SpoolArgs),

    /// Gosplan Interactive Web Cockpit & Mayak Command Bridge
    Cockpit(CockpitArgs),

    /// Check for updates and automatically upgrade CCCP
    Update(UpdateArgs),

    /// Merge a branch with strict GOST Phase 9 closes tag validation
    Merge(MergeArgs),

    /// Manage canonical GOST profiles (get, list, set)
    #[command(subcommand)]
    Profile(ProfileCommand),
}

#[derive(Debug, Subcommand, Clone)]
pub enum ProfileCommand {
    /// Get the active GOST profile URL and alias
    Get,

    /// List all known canonical GOST profiles
    List,

    /// Set the active GOST profile in GOSTVER (supports aliases like lumen, kollontai or custom URL)
    Set(ProfileSetArgs),

    /// Assemble full cascading GOST specification into .gost/GOST.md including CUSTOM.md overrides
    Assemble(ProfileAssembleArgs),
}

#[derive(Debug, Args, Clone, Default)]
pub struct ProfileAssembleArgs {
    /// Batch assemble GOST specifications for all catalog repositories
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Preview repositories without executing file generation
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,
}

#[derive(Debug, Args, Clone)]
pub struct ProfileSetArgs {
    /// Profile alias (e.g. lumen, kollontai) or raw profile URL
    #[arg(required = true)]
    pub profile: String,
}

#[derive(Debug, Args, Clone, Default)]
pub struct MergeArgs {
    /// Branch name to merge into current branch
    #[arg(required = true)]
    pub branch: String,

    /// Force merge even if canonical (closes #<id>) tag is missing
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Custom commit message for the merge commit
    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,
}

// =============================================================================
// Subcommand Arguments
// =============================================================================

#[derive(Debug, Args, Clone, Default)]
pub struct StartArgs {
    /// Ticket ID (e.g. 72)
    pub id: u64,

    /// Base branch to branch off from (defaults to main)
    #[arg(short = 'b', long = "base")]
    pub base: Option<String>,
}

#[derive(Debug, Args, Clone, Default)]
pub struct CloseArgs {
    /// Executive summary of the closing commit (<= 72 chars)
    pub summary: String,

    /// Resolution report body (must contain [Resumo]:, [Solução]:, [Validação]:)
    #[arg(short = 'b', long = "body")]
    pub body: Option<String>,

    /// Open editor to write the resolution report
    #[arg(long = "editor")]
    pub editor: bool,
}

#[derive(Debug, Args, Clone, Default)]
pub struct CommitArgs {
    /// Commit short subject / summary (enables automated DD-TDD phase commit)
    pub summary: Option<String>,

    /// Commit type (e.g. feat, fix, docs, style, refactor, perf, test, chore)
    #[arg(short = 't', long = "type")]
    pub commit_type: Option<String>,

    /// Commit scope (e.g. cli, parser, auth)
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

    /// Explicit DD-TDD phase override (e.g. ph3, ph4, ph5, ph6, ph7, ph8)
    #[arg(short = 'p', long = "phase")]
    pub phase: Option<String>,

    /// Commit short subject / description
    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,

    /// Extended commit body description
    #[arg(short = 'b', long = "body")]
    pub body: Option<String>,

    /// Mark commit as containing BREAKING CHANGE
    #[arg(long = "breaking")]
    pub breaking: bool,

    /// Stage all tracked modified files before committing (like git commit -a)
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Preview formatted commit message without creating commit
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// GPG or SSH sign the commit
    #[arg(short = 'S', long = "sign")]
    pub sign: bool,

    /// Do not sign the commit
    #[arg(long = "no-sign")]
    pub no_sign: bool,
}

pub type CzArgs = CommitArgs;

#[derive(Debug, Args)]
pub struct LintArgs {
    /// Commit message string to validate directly
    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,

    /// Path to file containing commit message (e.g. .git/COMMIT_EDITMSG)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,

    /// Git revision range to validate (e.g. origin/main..HEAD, HEAD~5..HEAD)
    #[arg(short = 'r', long = "range")]
    pub range: Option<String>,

    /// Enforce strict GOST / Conventional Commits rules (reject warnings as errors)
    #[arg(long = "strict")]
    pub strict: bool,
}

#[derive(Debug, Args, Clone, Default)]
pub struct GenVersionArgs {
    /// Output format for calculated version
    #[arg(short = 'f', long = "format", value_enum, default_value_t = VersionFormat::Clean)]
    pub format: VersionFormat,

    /// Write calculated version into VERSION file
    #[arg(short = 'w', long = "write")]
    pub write: bool,

    /// Manually override bump level (major, minor, patch)
    #[arg(short = 'b', long = "bump", value_enum)]
    pub bump: Option<BumpType>,
}

pub type SemverArgs = GenVersionArgs;

#[derive(Debug, Args, Clone, Default)]
pub struct VersionArgs {
    /// Output format (text, json)
    #[arg(short = 'f', long = "format", default_value = "text")]
    pub format: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum, Default)]
pub enum VersionFormat {
    /// Clean SemVer (e.g. 1.2.0)
    #[default]
    Clean,
    /// Development SemVer with build metadata (e.g. 1.2.0-dev.4+20260929.abc1234)
    Dev,
    /// Detailed JSON payload with bump analysis
    Json,
    /// Only the Next SemVer tag (e.g. v1.2.0)
    Tag,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Args, Clone, Default)]
pub struct InstallArgs {
    /// Overwrite existing git hooks without prompting
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Remove installed CCCP hooks from the repository
    #[arg(short = 'u', long = "uninstall")]
    pub uninstall: bool,

    /// Target hooks directory (defaults to .git/hooks)
    #[arg(long = "hook-dir")]
    pub hook_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Show detailed commit breakdown and hook integrity
    #[arg(short = 'd', long = "detailed")]
    pub detailed: bool,

    /// Output status as JSON
    #[arg(long = "json")]
    pub json: bool,
}

#[derive(Debug, Args, Clone, Default)]
pub struct HookArgs {
    /// Install CCCP Git hooks in repository
    #[arg(short = 'i', long = "install")]
    pub install: bool,

    /// Remove installed CCCP hooks from the repository
    #[arg(short = 'u', long = "uninstall")]
    pub uninstall: bool,

    /// Overwrite existing git hooks without prompting
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Target hooks directory (defaults to .git/hooks)
    #[arg(long = "hook-dir")]
    pub hook_dir: Option<PathBuf>,

    /// Hook trigger name or subcommand
    #[command(subcommand)]
    pub hook: Option<HookSubcommands>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum HookSubcommands {
    /// Run pre-commit hook actions (automatic VERSION sync into stage)
    PreCommit,
    /// Run commit-msg hook validation on commit message file
    CommitMsg {
        /// Path to commit message file (e.g. .git/COMMIT_EDITMSG)
        file: PathBuf,
        /// Commit source (message, template, merge, squash, or commit SHA)
        source: Option<String>,
        /// Commit SHA (if amend or rebase)
        sha: Option<String>,
    },
    /// Run pre-push hook validation
    PrePush {
        /// Remote destination name
        remote: Option<String>,
        /// Remote URL
        url: Option<String>,
    },
    /// Run post-commit hook actions (version sync, notices)
    PostCommit,
    /// Run reference-transaction hook actions
    ReferenceTransaction {
        /// Transaction state (prepared, committed, aborted)
        state: Option<String>,
    },
    /// Run post-merge hook actions (close tickets on merge)
    PostMerge {
        /// Flag whether merge was a squash merge (1 or 0)
        is_squash: Option<String>,
    },
}

#[derive(Debug, Args, Clone, Default)]
pub struct TicketArgs {
    /// Remote git forge name or URL
    #[arg(short = 'r', long = "remote")]
    pub remote: Option<String>,

    /// Destination file for markdown sync (default: .antigravity/tickets.md)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Maximum number of tickets to synchronize
    #[arg(short = 'l', long = "limit", default_value_t = 50)]
    pub limit: usize,

    /// Filter tickets by state (open, closed, all)
    #[arg(short = 's', long = "state", default_value = "open")]
    pub state: String,

    /// Ticket action or subcommand (e.g. close)
    #[command(subcommand)]
    pub action: Option<TicketAction>,
}

#[derive(Debug, Subcommand, Clone, PartialEq)]
pub enum TicketAction {
    /// Create a new ticket directly on the remote forge
    New {
        /// Ticket title (Conventional Commits style)
        #[arg(short = 't', long = "title")]
        title: Option<String>,

        /// Ticket description / body
        #[arg(short = 'b', long = "body")]
        body: Option<String>,

        /// Labels to associate with the ticket
        #[arg(short = 'l', long = "label", value_delimiter = ',')]
        labels: Vec<String>,

        /// Remote forge override
        #[arg(short = 'r', long = "remote")]
        remote: Option<String>,
    },

    /// Interactive AI ticket triage and consultation bot
    Divan(DivanArgs),

    /// Synchronize forge issues and pull requests into local .antigravity/tickets.md
    Sync {
        /// Remote git forge name or URL
        #[arg(short = 'r', long = "remote")]
        remote: Option<String>,

        /// Destination file for markdown sync (default: .antigravity/tickets.md)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Maximum number of tickets to synchronize
        #[arg(short = 'l', long = "limit", default_value_t = 50)]
        limit: usize,

        /// Filter tickets by state (open, closed, all)
        #[arg(short = 's', long = "state", default_value = "open")]
        state: String,
    },

    /// List locally synchronized tickets from .antigravity/tickets.md
    List,

    /// Close one or more tickets on the remote forge
    Close {
        /// One or more ticket IDs to close (e.g. 133 134)
        #[arg(required = true)]
        ids: Vec<u64>,

        /// Optional closing comment message
        #[arg(short = 'm', long = "comment")]
        comment: Option<String>,
    },

    /// Post a comment to an existing ticket on the remote forge
    Comment {
        /// Ticket ID to comment on (e.g. 148)
        #[arg(required = true)]
        id: u64,

        /// Comment message content (if not provided, opens in CCCP editor)
        #[arg(short = 'm', long = "message")]
        message: Option<String>,
    },
}

pub type TicketsArgs = TicketArgs;

#[derive(Debug, Args, Clone, Default)]
pub struct SyncArgs {
    /// Synchronize forge issues and pull requests into local tickets.md
    #[arg(short = 't', long = "tickets")]
    pub tickets: bool,

    /// Synchronize AGENTS.md template with local variable interpolation
    #[arg(short = 'a', long = "agents")]
    pub agents: bool,

    /// Synchronize workspace repository catalog (~/src)
    #[arg(short = 'c', long = "catalog")]
    pub catalog: bool,

    /// Synchronize ticket counts for all repositories in the catalog
    #[arg(long = "catalog-tickets")]
    pub catalog_tickets: bool,

    /// Force full synchronization / overwrite
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Target directory override
    #[arg(long = "dir")]
    pub dir: Option<PathBuf>,
}

#[derive(Debug, Args, Clone, Default)]
pub struct ForgeArgs {
    #[command(subcommand)]
    pub action: Option<ForgeActions>,
}

pub type ForgesArgs = ForgeArgs;

#[derive(Debug, Subcommand, Clone)]
pub enum ForgeActions {
    /// Detect remote forge and output connection info
    Detect,
    /// List remote issues/tickets
    Issues {
        #[arg(short = 's', long = "state", default_value = "open")]
        state: String,
    },
    /// List pull requests
    Prs {
        #[arg(short = 's', long = "state", default_value = "open")]
        state: String,
    },
}

#[derive(Debug, Args)]
pub struct SovietArgs {
    /// Display the State Anthem of the Soviet Union
    #[arg(long = "anthem")]
    pub anthem: bool,

    /// Display inspiring socialist / Gosplan quote
    #[arg(long = "quote")]
    pub quote: bool,

    /// Display the Five-Year Plan quota and compliance banner
    #[arg(long = "plan")]
    pub plan: bool,

    /// Display Order No. 227 ("Ni shagu nazad!" / "Not a step back!")
    #[arg(long = "order227")]
    pub order227: bool,

    /// Display project contributors and comrades
    #[arg(long = "comrades")]
    pub comrades: bool,
}

#[derive(Debug, Args)]
pub struct TagArgs {
    /// Target version to tag (e.g. 2.1.0, v2.1.0, 2). If omitted, uses predicted SemVer.
    pub version: Option<String>,

    /// Release title for tag annotation, commit, and CHANGELOG
    #[arg(short = 't', long = "title")]
    pub title: Option<String>,

    /// Custom tag annotation message
    #[arg(short = 'm', long = "message")]
    pub message: Option<String>,

    /// Do not prefix tag name with 'v'
    #[arg(long = "no-v")]
    pub no_v: bool,

    /// GPG or SSH sign the release tag
    #[arg(short = 's', long = "sign")]
    pub sign: bool,
}

#[derive(Debug, Args)]
pub struct ChangelogArgs {
    /// Output file for CHANGELOG (default: CHANGELOG.md)
    #[arg(short = 'o', long = "output", default_value = "CHANGELOG.md")]
    pub output: PathBuf,

    /// Preview changelog content without writing to disk
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Revision range for changelog (e.g. v1.0.0..HEAD)
    #[arg(short = 'r', long = "range")]
    pub range: Option<String>,
}

#[derive(Debug, Args, Clone, Default)]
pub struct PushArgs {
    /// Batch push all clean catalog repositories with unpushed commits
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Preview repositories without executing push
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// Arguments to forward directly to git push
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub git_args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    /// Configuration key to read or write
    pub key: Option<String>,

    /// Configuration value to set
    pub value: Option<String>,

    /// List all resolved configuration entries
    #[arg(short = 'l', long = "list")]
    pub list: bool,

    /// Set configuration globally (~/.cccprc or ~/.config/cccp/config)
    #[arg(short = 'g', long = "global")]
    pub is_global: bool,
}

#[derive(Debug, Args, Clone, Default)]
pub struct AuditArgs {
    /// Fail immediately on any GOST compliance warning
    #[arg(long = "strict")]
    pub strict: bool,
}

pub type GostArgs = AuditArgs;
pub type CheckArgs = AuditArgs;

#[derive(Debug, Args)]
pub struct ComradesArgs {
    /// Output format (table, json, plain)
    #[arg(short = 'f', long = "format", default_value = "table")]
    pub format: String,
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    /// Shell type for completion generation
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

#[derive(Debug, Args, Clone, Default)]
pub struct CatalogFilterArgs {
    /// Filter repositories by status ('clean', 'dirty', 'unpushed', 'remote')
    #[arg(short = 's', long = "status")]
    pub status: Option<String>,

    /// Filter exclusively dirty repositories (with unstaged or staged modifications)
    #[arg(long = "dirty", conflicts_with = "unpushed")]
    pub dirty: bool,

    /// Filter exclusively clean repositories with commits pending push
    #[arg(long = "unpushed", alias = "pending-push", conflicts_with = "dirty")]
    pub unpushed: bool,

    /// Filter repositories with minimum number of open tickets
    #[arg(short = 't', long = "tickets", alias = "min-tickets")]
    pub min_tickets: Option<usize>,

    /// Filter repositories by CI status ('pass'/'success', 'fail'/'failure', 'pending'/'running', 'none')
    #[arg(long = "ci")]
    pub ci: Option<String>,

    /// Display detailed CI pipeline execution status for each repository
    #[arg(long = "ci-details")]
    pub ci_details: bool,

    /// Sort repositories by field ('name', 'tickets', 'status', 'forge', 'gost', 'ci')
    #[arg(long = "sort", default_value = "name")]
    pub sort: String,

    /// Reverse sort order
    #[arg(short = 'r', long = "reverse")]
    pub reverse: bool,
}

impl CatalogFilterArgs {
    pub fn merge(&self, sub: &CatalogFilterArgs) -> Self {
        Self {
            status: sub.status.clone().or_else(|| self.status.clone()),
            dirty: sub.dirty || self.dirty,
            unpushed: sub.unpushed || self.unpushed,
            min_tickets: sub.min_tickets.or(self.min_tickets),
            ci: sub.ci.clone().or_else(|| self.ci.clone()),
            ci_details: sub.ci_details || self.ci_details,
            sort: if sub.sort != "name" {
                sub.sort.clone()
            } else {
                self.sort.clone()
            },
            reverse: sub.reverse || self.reverse,
        }
    }
}

#[derive(Debug, Args)]
pub struct CatalogArgs {
    #[command(subcommand)]
    pub action: Option<CatalogSubcommands>,

    #[command(flatten)]
    pub filter: CatalogFilterArgs,
}

#[derive(Debug, Subcommand)]
pub enum CatalogSubcommands {
    /// Add a repository to the catalog directly by URL or slug
    #[command(alias = "add_repo", alias = "add-repo")]
    Add {
        /// Remote repository URL or slug (e.g. 'codeberg.org/lumenpink/repo' or 'owner/repo')
        repo: String,

        /// Optional custom name for the repository
        #[arg(short = 'n', long = "name")]
        name: Option<String>,

        /// Optional description for the repository
        #[arg(short = 'd', long = "description")]
        description: Option<String>,

        /// Optional aliases for the repository (e.g. -a core -a cccp-core)
        #[arg(short = 'a', long = "alias", value_delimiter = ',')]
        aliases: Vec<String>,
    },
    /// Scan source directory and add all local git repositories to the catalog
    #[command(alias = "add_new")]
    AddNew {
        /// Source directory to scan for git repositories (defaults to ~/src if flag is given without argument)
        #[arg(long = "from-src", default_missing_value = "~/src", num_args = 0..=1)]
        from_src: Option<String>,
    },
    /// List all catalog repositories with optional filters and sorting
    List(CatalogFilterArgs),
    /// Manage repository aliases in catalog
    Alias(CatalogAliasArgs),
    /// Initialize and scaffold repository under strict GOST standard
    #[command(alias = "new")]
    Init(InitArgs),
    /// Push all clean repositories in catalog with pending commits
    Push {
        /// Preview repositories without executing push
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
    },
    /// Assemble cascading GOST specification (.gost/GOST.md) for catalog repositories
    #[command(alias = "gost")]
    Assemble {
        /// Preview repositories without executing assembly
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,
    },
    /// Remove a repository from the catalog by name, slug, or alias
    #[command(alias = "rm")]
    Remove {
        /// Repository name, slug, or alias to remove from catalog
        #[arg(required = true)]
        repo: String,

        /// Also delete the local repository directory if it is completely clean
        #[arg(long = "delete-dir", alias = "rm-dir")]
        delete_dir: bool,
    },
    /// Update open ticket count for repositories in the catalog
    #[command(alias = "sync-tickets", alias = "sync")]
    UpdateTickets {
        /// Repositories to update (slug, name, or alias)
        #[arg(num_args = 0..)]
        repos: Vec<String>,

        /// Update all repositories in the catalog
        #[arg(short = 'a', long = "all")]
        all: bool,
    },
}

pub type RepoSubcommands = CatalogSubcommands;
pub type RepoArgs = CatalogArgs;

#[derive(Debug, Args, Clone)]
pub struct CatalogAliasArgs {
    #[command(subcommand)]
    pub action: Option<CatalogAliasSubcommands>,

    /// Target repository name or slug
    pub repo: Option<String>,

    /// Aliases to add (when used without subcommand)
    #[arg(num_args = 0..)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum CatalogAliasSubcommands {
    /// Add one or more aliases to a repository
    #[command(alias = "set")]
    Add {
        /// Target repository name or slug
        repo: String,
        /// Aliases to associate
        #[arg(required = true, num_args = 1..)]
        aliases: Vec<String>,
    },
    /// Remove one or more aliases from a repository
    #[command(alias = "rm")]
    Remove {
        /// Target repository name or slug
        repo: String,
        /// Aliases to remove
        #[arg(required = true, num_args = 1..)]
        aliases: Vec<String>,
    },
    /// List configured aliases for a repository or entire catalog
    #[command(alias = "ls")]
    List {
        /// Optional repository name or slug
        repo: Option<String>,
    },
}

#[derive(Debug, Args, Clone, PartialEq, Default)]
pub struct DivanArgs {
    /// Target repository or problem description
    pub target: Option<String>,

    /// Problem description or user report to triage (when target repository is first argument)
    pub prompt: Option<String>,

    /// Target repository (e.g. 'cccp', 'owner/repo', or forge URL)
    #[arg(short = 'r', long = "repo")]
    pub repo: Option<String>,

    /// AI provider to use ('gemini' or 'ollama')
    #[arg(short = 'p', long = "provider")]
    pub provider: Option<String>,

    /// Quick shortcut to use local Ollama provider (-o, --ollama, --po)
    #[arg(short = 'o', long = "ollama", visible_alias = "po")]
    pub ollama: bool,

    /// Quick shortcut to use local OpenAI-compatible AI provider (-l, --local, --pl)
    #[arg(short = 'l', long = "local", visible_alias = "pl")]
    pub local: bool,

    /// Quick shortcut to use Google Gemini provider (-g, --gemini, --pg)
    #[arg(short = 'g', long = "gemini", visible_alias = "pg")]
    pub gemini: bool,

    /// Model name override (e.g. 'llama3.2', 'qwen2.5-coder', 'gemini-flash-latest')
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    /// Local Ollama host URL override (default: 'http://localhost:11434')
    #[arg(long = "host")]
    pub host: Option<String>,

    /// Generation temperature override (e.g. 0.2)
    #[arg(long = "temperature")]
    pub temperature: Option<f32>,

    /// Bypass AI decomposition and create single ticket directly
    #[arg(long = "direct", alias = "no-llm")]
    pub direct: bool,

    /// Dry run: display decomposed tickets without publishing to remote forge
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Target issue/ticket ID to comment on or interact with
    #[arg(short = 'c', long = "comment")]
    pub comment: Option<u64>,
}

pub type DivaArgs = DivanArgs;

impl DivanArgs {
    /// Resolves target repository and prompt description from arguments.
    ///
    /// Precedence & inference:
    /// - If `--repo` is specified, it is always the target repository. Any positional args become prompt.
    /// - If both `target` and `prompt` positional arguments are given, `target` is the repository and `prompt` is the report.
    /// - If only a single positional argument is given:
    ///   - If it contains whitespace, it is treated as a prompt description (for current repo or interactive selection).
    ///   - Otherwise, it is inferred as the target repository (closes #108).
    pub fn resolve_repo_and_prompt(&self) -> (Option<String>, Option<String>) {
        if let Some(ref r) = self.repo {
            let prompt = match (&self.target, &self.prompt) {
                (Some(t), Some(p)) => Some(format!("{} {}", t.trim(), p.trim())),
                (Some(t), None) => Some(t.trim().to_string()),
                (None, Some(p)) => Some(p.trim().to_string()),
                (None, None) => None,
            };
            (Some(r.trim().to_string()), prompt)
        } else {
            match (&self.target, &self.prompt) {
                (Some(t), Some(p)) => (Some(t.trim().to_string()), Some(p.trim().to_string())),
                (Some(t), None) => {
                    let trimmed = t.trim();
                    if trimmed.contains(' ') {
                        (None, Some(trimmed.to_string()))
                    } else {
                        (Some(trimmed.to_string()), None)
                    }
                }
                (None, Some(p)) => (None, Some(p.trim().to_string())),
                (None, None) => (None, None),
            }
        }
    }
}

#[derive(Debug, Args, Clone, Default)]
pub struct CiArgs {
    #[command(subcommand)]
    pub action: Option<CiSubcommands>,
}

pub type ActArgs = CiArgs;

#[derive(Debug, Subcommand, Clone)]
pub enum CiSubcommands {
    /// Inspect local runner status, Docker daemon, workflows, and logs
    Status,
    /// Run local CI workflow with act
    Run {
        /// Job name to execute (defaults to CCCP_ACT_JOB or 'test')
        #[arg(short = 'j', long = "job")]
        job: Option<String>,
    },
    /// Inspect or tail the most recent CI log in .git/act-logs/
    #[command(alias = "logs")]
    Log {
        /// Continuously stream and follow the log output
        #[arg(short = 'f', long = "follow")]
        follow: bool,
    },
}

pub type ActSubcommands = CiSubcommands;

#[derive(Debug, Args, Clone, Default)]
pub struct DepsArgs {
    #[command(subcommand)]
    pub action: Option<DepsSubcommands>,

    /// Specific dependencies to check when no subcommand is specified
    #[arg(trailing_var_arg = true)]
    pub deps: Vec<String>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum DepsSubcommands {
    /// List all catalog ecosystem dependencies
    #[command(alias = "ls")]
    List,
    /// Audit and verify dependencies against SemVer requirements
    #[command(alias = "status")]
    Check {
        /// Specific dependencies to check
        #[arg(trailing_var_arg = true)]
        deps: Vec<String>,
    },
    /// Guide or install ecosystem tools and dependencies
    #[command(alias = "get", alias = "add")]
    Install {
        /// Install all standalone ecosystem tools
        #[arg(long = "all")]
        all: bool,

        /// Specific dependencies to install
        #[arg(trailing_var_arg = true)]
        deps: Vec<String>,
    },
}

#[derive(Debug, Args, Clone, Default)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub action: Option<AgentSubcommands>,

    /// URL or local file path for the AGENTS template
    #[arg(short = 'u', long = "url", global = true)]
    pub url: Option<String>,

    /// Destination file path for generated AGENTS.md (default: .antigravity/AGENTS.md)
    #[arg(short = 'f', long = "file", global = true)]
    pub file: Option<String>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum AgentSubcommands {
    /// Generate or update AGENTS.md with variable interpolation (default)
    Update,
    /// Inspect template variables and verify resolution status
    #[command(alias = "vars", alias = "list-vars")]
    Check,
    /// Set an agent variable in local .cccprc
    #[command(alias = "set-var")]
    Set {
        /// Variable name (e.g. project_name, stack)
        key: String,
        /// Variable value
        value: String,
    },
    /// Get the resolved value of an agent variable
    Get {
        /// Variable name
        key: String,
    },
    /// Unset an agent variable from local .cccprc
    Unset {
        /// Variable name
        key: String,
    },
    /// Interactively prompt for missing variables, then regenerate AGENTS.md
    #[command(alias = "fill-vars")]
    Fill,
}

#[derive(Debug, Args, Clone)]
pub struct SetAvArgs {
    /// Variable name (e.g. project_name, stack)
    pub key: String,
    /// Variable value
    pub value: String,
}

#[derive(Debug, Args, Clone, Default)]
pub struct InitArgs {
    /// Repository name and directory to create
    pub name: Option<String>,

    /// Explicit repository name (alternative to positional argument)
    #[arg(long = "name")]
    pub explicit_name: Option<String>,

    /// Target root directory (default: ./<repository-name>)
    #[arg(long = "root")]
    pub root: Option<std::path::PathBuf>,

    /// Short project description for README and portal
    #[arg(long = "desc")]
    pub desc: Option<String>,

    /// Comma-separated active programming languages (default: shell)
    #[arg(long = "lang", default_value = "shell")]
    pub lang: String,

    /// SPDX license identifier (default: CC0-1.0; options: MIT, GPL-3.0, CC-BY-SA-4.0, Apache-2.0)
    #[arg(long = "license", default_value = "CC0-1.0")]
    pub license: String,

    /// Profile implementation URL (default: profiles/lumen.md)
    #[arg(long = "profile", default_value = "profiles/lumen.md")]
    pub profile: String,

    /// Do not initialize a local Git repository
    #[arg(long = "no-git", action = clap::ArgAction::SetTrue)]
    pub no_git: bool,

    /// Do not install CCCP Git hooks
    #[arg(long = "no-hooks", action = clap::ArgAction::SetTrue)]
    pub no_hooks: bool,

    /// Do not generate CI workflow (.forgejo/workflows/ci.yaml)
    #[arg(long = "no-ci", action = clap::ArgAction::SetTrue)]
    pub no_ci: bool,

    /// Do not synchronize GOST specifications (.gost/)
    #[arg(long = "no-sync", action = clap::ArgAction::SetTrue)]
    pub no_sync: bool,

    /// Quiet mode, suppress non-essential output
    #[arg(short = 'q', long = "quiet", action = clap::ArgAction::SetTrue)]
    pub quiet: bool,
}

pub type NewrepoArgs = InitArgs;

impl From<InitArgs> for crate::newrepo::NewrepoOptions {
    fn from(args: InitArgs) -> Self {
        let name = args.explicit_name.or(args.name);
        Self {
            name,
            root: args.root,
            desc: args.desc,
            lang: args.lang,
            license: args.license,
            profile: args.profile,
            git: !args.no_git,
            hooks: !args.no_hooks,
            ci: !args.no_ci,
            sync: !args.no_sync,
            quiet: args.quiet,
        }
    }
}

#[derive(Debug, Args, Clone, Default)]
pub struct ServeArgs {
    /// Directory to serve (defaults to ./site, ./dist, ./public, or ./)
    pub path: Option<String>,

    /// Development mode (builder watch, PHP or Node script execution)
    #[arg(long = "dev", action = clap::ArgAction::SetTrue)]
    pub dev: bool,

    /// VDNKh demo mode with fixtures and automatic browser launch
    #[arg(long = "demo", action = clap::ArgAction::SetTrue)]
    pub demo: bool,

    /// Bind only to loopback interface 127.0.0.1 (default is 0.0.0.0)
    #[arg(short = 'l', long = "localhost", action = clap::ArgAction::SetTrue)]
    pub localhost: bool,

    /// Custom IP address for binding
    #[arg(long = "host")]
    pub host: Option<String>,

    /// Starting port number (default: 8080)
    #[arg(short = 'p', long = "port")]
    pub port: Option<u16>,

    /// Open default browser automatically
    #[arg(short = 'o', long = "open", action = clap::ArgAction::SetTrue)]
    pub open: bool,

    /// Quiet mode (suppress per-request access logs)
    #[arg(short = 'q', long = "quiet", action = clap::ArgAction::SetTrue)]
    pub quiet: bool,
}

impl From<ServeArgs> for crate::serve::ServeOptions {
    fn from(args: ServeArgs) -> Self {
        let host = if args.localhost {
            "127.0.0.1".to_string()
        } else if let Some(h) = args.host {
            h
        } else {
            "0.0.0.0".to_string()
        };

        Self {
            target: args.path,
            dev: args.dev,
            demo: args.demo,
            host,
            port: args.port.unwrap_or(8080),
            open: args.open,
            quiet: args.quiet,
        }
    }
}

#[derive(Debug, Args, Clone, Default)]
pub struct SpoolArgs {
    #[command(subcommand)]
    pub action: Option<SpoolSubcommands>,

    /// Raw prompt to send if no subcommand was specified
    #[arg(trailing_var_arg = true)]
    pub prompt: Vec<String>,
}

pub type MatracaArgs = SpoolArgs;

#[derive(Debug, Subcommand, Clone)]
pub enum SpoolSubcommands {
    /// Enqueue prompt into FIFO queue (<10ms)
    Send(SpoolSendArgs),
    /// Query status and result of a job by ID or client ID
    Poll {
        /// Job ID or Client ID to query
        id: String,
    },
    /// Process queued jobs with agy
    Worker(SpoolWorkerArgs),
    /// List recently finished or active jobs
    History {
        /// Maximum number of messages to return
        #[arg(short = 'n', long = "limit", default_value = "50")]
        limit: usize,
    },
    /// Show queue health and statistics
    Status {
        /// Output status in structured JSON format
        #[arg(long = "json", action = clap::ArgAction::SetTrue)]
        json: bool,
    },
    /// Clean old completed and failed jobs
    Clean {
        /// Number of days to keep jobs (default: 7)
        #[arg(short = 'd', long = "days", default_value = "7")]
        days: u64,
    },
}

pub type MatracaSubcommands = SpoolSubcommands;

#[derive(Debug, Args, Clone, Default)]
pub struct SpoolSendArgs {
    /// Client ID for idempotency tracking
    #[arg(long = "client-id")]
    pub client_id: Option<String>,

    /// Target workspace directory (default: current directory)
    #[arg(long = "workspace")]
    pub workspace: Option<String>,

    /// Prompt or command for Antigravity
    #[arg(trailing_var_arg = true)]
    pub prompt: Vec<String>,
}

pub type MatracaSendArgs = SpoolSendArgs;

#[derive(Debug, Args, Clone, Default)]
pub struct SpoolWorkerArgs {
    /// Run worker continuously in daemon mode
    #[arg(long = "daemon", action = clap::ArgAction::SetTrue)]
    pub daemon: bool,

    /// Run single pass over pending queue (default)
    #[arg(long = "once", action = clap::ArgAction::SetTrue)]
    pub once: bool,

    /// Target specific job ID
    #[arg(long = "job-id")]
    pub job_id: Option<String>,
}

pub type MatracaWorkerArgs = SpoolWorkerArgs;

#[derive(Debug, Args, Clone, Default)]
pub struct CockpitArgs {
    /// Initial tab to display (matraca, diva, status, tickets)
    pub tab: Option<String>,

    /// Port to listen on (default: 8080)
    #[arg(short = 'p', long = "port")]
    pub port: Option<u16>,

    /// Host address to bind to (default: 127.0.0.1)
    #[arg(long = "host")]
    pub host: Option<String>,

    /// Open default web browser automatically
    #[arg(short = 'o', long = "open", action = clap::ArgAction::SetTrue)]
    pub open: bool,

    /// Do not open web browser
    #[arg(long = "no-open", action = clap::ArgAction::SetTrue)]
    pub no_open: bool,

    /// API token for authentication (or via CCCP_TOKEN/CCCP_API_KEY)
    #[arg(short = 't', long = "token", alias = "key", alias = "api-key")]
    pub token: Option<String>,

    /// Disable Matraca (Web REPL) tab
    #[arg(long = "no-matraca", action = clap::ArgAction::SetTrue)]
    pub no_matraca: bool,

    /// Disable O Divã (triage bot) tab
    #[arg(long = "no-diva", action = clap::ArgAction::SetTrue)]
    pub no_diva: bool,

    /// Disable Repository Status tab
    #[arg(long = "no-status", action = clap::ArgAction::SetTrue)]
    pub no_status: bool,

    /// Disable Active Tickets tab
    #[arg(long = "no-tickets", action = clap::ArgAction::SetTrue)]
    pub no_tickets: bool,

    /// Quiet mode (suppress verbose logs)
    #[arg(short = 'q', long = "quiet", action = clap::ArgAction::SetTrue)]
    pub quiet: bool,
}

impl From<CockpitArgs> for crate::web::CockpitOptions {
    fn from(args: CockpitArgs) -> Self {
        let initial_tab = args.tab.unwrap_or_else(|| "matraca".to_string());
        let open = if args.no_open { false } else { args.open };
        Self {
            port: args.port.unwrap_or(8080),
            host: args.host.unwrap_or_else(|| "127.0.0.1".to_string()),
            token: args.token,
            open,
            quiet: args.quiet,
            initial_tab,
            enable_matraca: !args.no_matraca,
            enable_diva: !args.no_diva,
            enable_status: !args.no_status,
            enable_tickets: !args.no_tickets,
        }
    }
}

#[derive(Debug, Args, Clone, Default)]
pub struct UpdateArgs {
    /// Release channel to track (stable or nightly)
    #[arg(short = 'c', long = "channel")]
    pub channel: Option<String>,

    /// Check for available updates without applying
    #[arg(long = "check")]
    pub check: bool,

    /// Pin version to prevent updates (or pin to specified version)
    #[arg(short = 'p', long = "pin", num_args = 0..=1, default_missing_value = "")]
    pub pin: Option<String>,

    /// Remove version pin and resume standard updates
    #[arg(short = 'u', long = "unpin")]
    pub unpin: bool,

    /// Explicit path to target binary to update
    #[arg(long = "target")]
    pub target: Option<PathBuf>,
}

#[derive(Debug, Args, Clone, Default)]
pub struct CheckUpdateArgs {
    /// Release channel to check (stable or nightly)
    #[arg(short = 'c', long = "channel")]
    pub channel: Option<String>,
}

#[derive(Debug, Args, Clone, Default)]
pub struct EditorArgs {
    /// Path to file to open and edit
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli_structure() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_commit_args() {
        let args = Cli::parse_from([
            "cccp",
            "commit",
            "-t",
            "feat",
            "-s",
            "core",
            "-m",
            "hello world",
            "--breaking",
        ]);
        match args.command {
            Some(Commands::Commit(c)) => {
                assert_eq!(c.commit_type.as_deref(), Some("feat"));
                assert_eq!(c.scope.as_deref(), Some("core"));
                assert_eq!(c.message.as_deref(), Some("hello world"));
                assert!(c.breaking);
            }
            _ => panic!("Expected Commands::Commit"),
        }
    }

    #[test]
    fn parse_version_args() {
        let args = Cli::parse_from(["cccp", "version"]);
        assert!(matches!(args.command, Some(Commands::Version(_))));

        let gen_args = Cli::parse_from(["cccp", "gen-version", "-f", "dev", "-w"]);
        match gen_args.command {
            Some(Commands::GenVersion(v)) => {
                assert_eq!(v.format, VersionFormat::Dev);
                assert!(v.write);
            }
            _ => panic!("Expected Commands::GenVersion"),
        }
    }

    #[test]
    fn parse_hook_args() {
        let args = Cli::parse_from(["cccp", "hook", "--install", "--force"]);
        match args.command {
            Some(Commands::Hook(h)) => {
                assert!(h.install);
                assert!(h.force);
                assert!(!h.uninstall);
            }
            _ => panic!("Expected Commands::Hook"),
        }

        let args_uninst = Cli::parse_from(["cccp", "hook", "--uninstall"]);
        match args_uninst.command {
            Some(Commands::Hook(h)) => {
                assert!(h.uninstall);
                assert!(!h.install);
            }
            _ => panic!("Expected Commands::Hook"),
        }
    }

    #[test]
    fn parse_sync_args() {
        let args = Cli::parse_from(["cccp", "sync", "--tickets", "--agents", "-f"]);
        match args.command {
            Some(Commands::Sync(s)) => {
                assert!(s.tickets);
                assert!(s.agents);
                assert!(!s.catalog);
                assert!(s.force);
            }
            _ => panic!("Expected Commands::Sync"),
        }
    }

    #[test]
    fn parse_ticket_args() {
        let args = Cli::parse_from(["cccp", "ticket", "--state", "closed", "--limit", "10"]);
        match args.command {
            Some(Commands::Ticket(t)) => {
                assert_eq!(t.state, "closed");
                assert_eq!(t.limit, 10);
            }
            _ => panic!("Expected Commands::Ticket"),
        }
    }

    #[test]
    fn parse_soviet_args() {
        let args = Cli::parse_from(["cccp", "soviet", "--anthem", "--plan"]);
        match args.command {
            Some(Commands::Soviet(soviet)) => {
                assert!(soviet.anthem);
                assert!(soviet.plan);
            }
            _ => panic!("Expected Commands::Soviet"),
        }
    }

    #[test]
    fn parse_divan_args() {
        let args = Cli::parse_from([
            "cccp",
            "ticket",
            "divan",
            "--provider",
            "ollama",
            "--model",
            "llama3.2",
            "--host",
            "http://127.0.0.1:11434",
            "--dry-run",
            "fix something",
        ]);
        match args.command {
            Some(Commands::Ticket(t)) => match t.action {
                Some(TicketAction::Divan(divan)) => {
                    assert_eq!(divan.provider.as_deref(), Some("ollama"));
                    assert_eq!(divan.model.as_deref(), Some("llama3.2"));
                    assert_eq!(divan.host.as_deref(), Some("http://127.0.0.1:11434"));
                    assert!(divan.dry_run);
                    let (repo, prompt) = divan.resolve_repo_and_prompt();
                    assert_eq!(repo, None);
                    assert_eq!(prompt.as_deref(), Some("fix something"));
                }
                _ => panic!("Expected TicketAction::Divan"),
            },
            _ => panic!("Expected Commands::Ticket"),
        }
    }

    #[test]
    fn parse_divan_args_positional_repo_only() {
        let args = Cli::parse_from(["cccp", "ticket", "divan", "joaju"]);
        match args.command {
            Some(Commands::Ticket(t)) => match t.action {
                Some(TicketAction::Divan(divan)) => {
                    let (repo, prompt) = divan.resolve_repo_and_prompt();
                    assert_eq!(repo.as_deref(), Some("joaju"));
                    assert_eq!(prompt, None);
                }
                _ => panic!("Expected TicketAction::Divan"),
            },
            _ => panic!("Expected Commands::Ticket"),
        }
    }

    #[test]
    fn parse_divan_args_positional_repo_and_prompt() {
        let args = Cli::parse_from(["cccp", "ticket", "divan", "joaju", "fix css styling"]);
        match args.command {
            Some(Commands::Ticket(t)) => match t.action {
                Some(TicketAction::Divan(divan)) => {
                    let (repo, prompt) = divan.resolve_repo_and_prompt();
                    assert_eq!(repo.as_deref(), Some("joaju"));
                    assert_eq!(prompt.as_deref(), Some("fix css styling"));
                }
                _ => panic!("Expected TicketAction::Divan"),
            },
            _ => panic!("Expected Commands::Ticket"),
        }
    }

    #[test]
    fn parse_divan_args_flag_repo_and_positional_prompt() {
        let args = Cli::parse_from(["cccp", "ticket", "divan", "-r", "joaju", "fix css styling"]);
        match args.command {
            Some(Commands::Ticket(t)) => match t.action {
                Some(TicketAction::Divan(divan)) => {
                    let (repo, prompt) = divan.resolve_repo_and_prompt();
                    assert_eq!(repo.as_deref(), Some("joaju"));
                    assert_eq!(prompt.as_deref(), Some("fix css styling"));
                }
                _ => panic!("Expected TicketAction::Divan"),
            },
            _ => panic!("Expected Commands::Ticket"),
        }
    }

    #[test]
    fn parse_agent_args() {
        let args = Cli::parse_from(["cccp", "agent", "set", "project_name", "my-app"]);
        match args.command {
            Some(Commands::Agent(agent)) => match agent.action {
                Some(AgentSubcommands::Set { key, value }) => {
                    assert_eq!(key, "project_name");
                    assert_eq!(value, "my-app");
                }
                _ => panic!("Expected AgentSubcommands::Set"),
            },
            _ => panic!("Expected Commands::Agent"),
        }

        assert!(Cli::try_parse_from(["cccp", "set-av", "stack", "rust"]).is_err());
    }

    #[test]
    fn parse_init_args() {
        let args = Cli::parse_from([
            "cccp",
            "repo",
            "init",
            "my-service",
            "--lang",
            "rust,shell",
            "--license",
            "MIT",
            "--desc",
            "My description",
            "--no-git",
            "--quiet",
        ]);
        match args.command {
            Some(Commands::Repo(cat)) => match cat.action {
                Some(RepoSubcommands::Init(init)) => {
                    assert_eq!(init.name.as_deref(), Some("my-service"));
                    assert_eq!(init.lang, "rust,shell");
                    assert_eq!(init.license, "MIT");
                    assert_eq!(init.desc.as_deref(), Some("My description"));
                    assert!(init.no_git);
                    assert!(init.quiet);
                    let opts: crate::newrepo::NewrepoOptions = init.into();
                    assert_eq!(opts.name.as_deref(), Some("my-service"));
                    assert!(!opts.git);
                    assert!(opts.quiet);
                }
                _ => panic!("Expected RepoSubcommands::Init"),
            },
            _ => panic!("Expected Commands::Repo"),
        }
    }

    #[test]
    fn parse_spool_args() {
        let args = Cli::parse_from([
            "cccp",
            "spool",
            "send",
            "--client-id",
            "cid-99",
            "--workspace",
            "/tmp/work",
            "run",
            "agent",
            "task",
        ]);
        match args.command {
            Some(Commands::Spool(spool)) => match spool.action {
                Some(SpoolSubcommands::Send(send_args)) => {
                    assert_eq!(send_args.client_id.as_deref(), Some("cid-99"));
                    assert_eq!(send_args.workspace.as_deref(), Some("/tmp/work"));
                    assert_eq!(send_args.prompt, vec!["run", "agent", "task"]);
                }
                _ => panic!("Expected SpoolSubcommands::Send"),
            },
            _ => panic!("Expected Commands::Spool"),
        }

        let args_poll = Cli::parse_from(["cccp", "spool", "poll", "job-12345"]);
        match args_poll.command {
            Some(Commands::Spool(spool)) => match spool.action {
                Some(SpoolSubcommands::Poll { id }) => {
                    assert_eq!(id, "job-12345");
                }
                _ => panic!("Expected SpoolSubcommands::Poll"),
            },
            _ => panic!("Expected Commands::Spool"),
        }
    }

    #[test]
    fn parse_cockpit_args() {
        let args = Cli::parse_from([
            "cccp",
            "cockpit",
            "divan",
            "--port",
            "8090",
            "--host",
            "0.0.0.0",
            "--open",
            "--token",
            "my-tok",
            "--no-matraca",
            "--quiet",
        ]);
        match args.command {
            Some(Commands::Cockpit(cockpit)) => {
                assert_eq!(cockpit.tab.as_deref(), Some("divan"));
                assert_eq!(cockpit.port, Some(8090));
                assert_eq!(cockpit.host.as_deref(), Some("0.0.0.0"));
                assert!(cockpit.open);
                assert_eq!(cockpit.token.as_deref(), Some("my-tok"));
                assert!(cockpit.no_matraca);
                assert!(cockpit.quiet);

                let opts: crate::web::CockpitOptions = cockpit.into();
                assert_eq!(opts.initial_tab, "divan");
                assert_eq!(opts.port, 8090);
                assert_eq!(opts.host, "0.0.0.0");
                assert!(opts.open);
                assert_eq!(opts.token.as_deref(), Some("my-tok"));
                assert!(!opts.enable_matraca);
                assert!(opts.enable_diva);
                assert!(opts.quiet);
            }
            _ => panic!("Expected Commands::Cockpit"),
        }
    }

    #[test]
    fn parse_global_flags() {
        let args = Cli::parse_from(["cccp", "-vv", "--no-color", "-C", "/tmp", "status"]);
        assert_eq!(args.verbose, 2);
        assert!(args.no_color);
        assert_eq!(args.dir, Some(PathBuf::from("/tmp")));
        assert!(matches!(args.command, Some(Commands::Status(_))));
    }

    #[test]
    fn parse_update_and_check_update() {
        let args = Cli::parse_from(["cccp", "update", "--channel", "nightly", "--pin", "3.0.0"]);
        match args.command {
            Some(Commands::Update(up)) => {
                assert_eq!(up.channel.as_deref(), Some("nightly"));
                assert_eq!(up.pin.as_deref(), Some("3.0.0"));
                assert!(!up.unpin);
            }
            _ => panic!("Expected Commands::Update"),
        }

        let args = Cli::parse_from(["cccp", "update", "--unpin"]);
        match args.command {
            Some(Commands::Update(up)) => {
                assert!(up.unpin);
            }
            _ => panic!("Expected Commands::Update"),
        }

        let args = Cli::parse_from(["cccp", "update", "--check", "--channel", "nightly"]);
        match args.command {
            Some(Commands::Update(up)) => {
                assert!(up.check);
                assert_eq!(up.channel.as_deref(), Some("nightly"));
            }
            _ => panic!("Expected Commands::Update"),
        }
        assert!(Cli::try_parse_from(["cccp", "check-update"]).is_err());
    }
}
