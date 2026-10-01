//! Command Line Interface definition for CCCP using Clap derive macros.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// ★ CCCP - Conventional Commits Compliance Program ★
/// Native, fast, standalone binary for Git governance, SemVer, and Soviet aesthetics.
#[derive(Debug, Parser)]
#[command(
    name = "cccp",
    author = "Lumen Pink <hi@lumen.pink>",
    version,
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
    /// Interactive Conventional Commits wizard (Commitizen style)
    #[command(alias = "commit")]
    Cz(CzArgs),

    /// Validate commit messages against Conventional Commits specification
    Lint(LintArgs),

    /// Calculate predictive Semantic Versioning (SemVer 2.0.0) metadata
    #[command(alias = "version")]
    Semver(SemverArgs),

    /// Install, update, or remove CCCP Git hooks in repository
    Install(InstallArgs),

    /// Audit repository Conventional Commits compliance, hooks, and integrity
    Status(StatusArgs),

    /// Internal Git hook dispatcher (commit-msg, pre-push, post-commit)
    Hook(HookArgs),

    /// Synchronize forge issues and pull requests into local markdown
    #[command(alias = "sync-tickets")]
    Tickets(TicketsArgs),

    /// Interactive issue triage bot via AI (Gemini or local Ollama)
    #[command(alias = "ticket", alias = "issue", alias = "divan")]
    Diva(DivaArgs),

    /// Query and interact with remote Git forge (Codeberg, GitHub, GitLab, Gitea)
    #[command(alias = "forge", alias = "codeberg")]
    Forges(ForgesArgs),

    /// Gosplan cultural registry, Order 227 ("Not a step back!"), and Soviet easter eggs
    #[command(alias = "gosplan", alias = "sputnik", alias = "anthem")]
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
    #[command(alias = "check")]
    Gost(GostArgs),

    /// Display project contributors and comrades
    #[command(alias = "comrade", alias = "camaradas")]
    Comrades(ComradesArgs),

    /// Generate shell completion scripts (bash, zsh, fish, powershell)
    Completion(CompletionArgs),

    /// Manage repository catalog and workspace registry
    #[command(alias = "sync-catalog")]
    Catalog(CatalogArgs),

    /// Local CI runner and pre-push verification gate via act
    #[command(alias = "ci")]
    Act(ActArgs),

    /// Dependency catalog, constraint verification, and ecosystem tools
    #[command(alias = "dependencies", alias = "dep")]
    Deps(DepsArgs),

    /// Generate or update AGENTS.md directives with local variable interpolation
    #[command(alias = "agents")]
    Agent(AgentArgs),

    /// Shortcut to configure an agent variable in local .cccprc
    #[command(name = "set-av", alias = "set_av")]
    SetAv(SetAvArgs),
}

// =============================================================================
// Subcommand Arguments
// =============================================================================

#[derive(Debug, Args)]
pub struct CzArgs {
    /// Commit type (e.g. feat, fix, docs, style, refactor, perf, test, chore)
    #[arg(short = 't', long = "type")]
    pub commit_type: Option<String>,

    /// Commit scope (e.g. cli, parser, auth)
    #[arg(short = 's', long = "scope")]
    pub scope: Option<String>,

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

#[derive(Debug, Args)]
pub struct SemverArgs {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum VersionFormat {
    /// Clean SemVer (e.g. 1.2.0)
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

#[derive(Debug, Args)]
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

#[derive(Debug, Args)]
pub struct HookArgs {
    /// Hook trigger name or subcommand
    #[command(subcommand)]
    pub hook: HookSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum HookSubcommands {
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
}

#[derive(Debug, Args)]
pub struct TicketsArgs {
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
}

#[derive(Debug, Args)]
pub struct ForgesArgs {
    #[command(subcommand)]
    pub action: Option<ForgeActions>,
}

#[derive(Debug, Subcommand)]
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

#[derive(Debug, Args)]
pub struct PushArgs {
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

#[derive(Debug, Args)]
pub struct GostArgs {
    /// Fail immediately on any GOST compliance warning
    #[arg(long = "strict")]
    pub strict: bool,
}

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

    /// Filter repositories with minimum number of open tickets
    #[arg(short = 't', long = "tickets", alias = "min-tickets")]
    pub min_tickets: Option<usize>,

    /// Sort repositories by field ('name', 'tickets', 'status', 'forge')
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
            min_tickets: sub.min_tickets.or(self.min_tickets),
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
}

#[derive(Debug, Args, Clone)]
pub struct DivaArgs {
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
}

impl DivaArgs {
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
pub struct ActArgs {
    #[command(subcommand)]
    pub action: Option<ActSubcommands>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum ActSubcommands {
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli_structure() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parse_cz_args() {
        let args = Cli::parse_from([
            "cccp",
            "cz",
            "-t",
            "feat",
            "-s",
            "core",
            "-m",
            "hello world",
            "--breaking",
        ]);
        match args.command {
            Some(Commands::Cz(cz)) => {
                assert_eq!(cz.commit_type.as_deref(), Some("feat"));
                assert_eq!(cz.scope.as_deref(), Some("core"));
                assert_eq!(cz.message.as_deref(), Some("hello world"));
                assert!(cz.breaking);
            }
            _ => panic!("Expected Commands::Cz"),
        }
    }

    #[test]
    fn parse_semver_args() {
        let args = Cli::parse_from(["cccp", "semver", "-f", "dev", "-w"]);
        match args.command {
            Some(Commands::Semver(semver)) => {
                assert_eq!(semver.format, VersionFormat::Dev);
                assert!(semver.write);
            }
            _ => panic!("Expected Commands::Semver"),
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
    fn parse_diva_args() {
        let args = Cli::parse_from([
            "cccp",
            "diva",
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
            Some(Commands::Diva(diva)) => {
                assert_eq!(diva.provider.as_deref(), Some("ollama"));
                assert_eq!(diva.model.as_deref(), Some("llama3.2"));
                assert_eq!(diva.host.as_deref(), Some("http://127.0.0.1:11434"));
                assert!(diva.dry_run);
                let (repo, prompt) = diva.resolve_repo_and_prompt();
                assert_eq!(repo, None);
                assert_eq!(prompt.as_deref(), Some("fix something"));
            }
            _ => panic!("Expected Commands::Diva"),
        }
    }

    #[test]
    fn parse_diva_args_positional_repo_only() {
        let args = Cli::parse_from(["cccp", "diva", "joaju"]);
        match args.command {
            Some(Commands::Diva(diva)) => {
                let (repo, prompt) = diva.resolve_repo_and_prompt();
                assert_eq!(repo.as_deref(), Some("joaju"));
                assert_eq!(prompt, None);
            }
            _ => panic!("Expected Commands::Diva"),
        }
    }

    #[test]
    fn parse_diva_args_positional_repo_and_prompt() {
        let args = Cli::parse_from(["cccp", "diva", "joaju", "fix css styling"]);
        match args.command {
            Some(Commands::Diva(diva)) => {
                let (repo, prompt) = diva.resolve_repo_and_prompt();
                assert_eq!(repo.as_deref(), Some("joaju"));
                assert_eq!(prompt.as_deref(), Some("fix css styling"));
            }
            _ => panic!("Expected Commands::Diva"),
        }
    }

    #[test]
    fn parse_diva_args_flag_repo_and_positional_prompt() {
        let args = Cli::parse_from(["cccp", "diva", "-r", "joaju", "fix css styling"]);
        match args.command {
            Some(Commands::Diva(diva)) => {
                let (repo, prompt) = diva.resolve_repo_and_prompt();
                assert_eq!(repo.as_deref(), Some("joaju"));
                assert_eq!(prompt.as_deref(), Some("fix css styling"));
            }
            _ => panic!("Expected Commands::Diva"),
        }
    }

    #[test]
    fn parse_act_args() {
        let args = Cli::parse_from(["cccp", "act", "run", "-j", "lint"]);
        match args.command {
            Some(Commands::Act(act)) => match act.action {
                Some(ActSubcommands::Run { job }) => {
                    assert_eq!(job.as_deref(), Some("lint"));
                }
                _ => panic!("Expected ActSubcommands::Run"),
            },
            _ => panic!("Expected Commands::Act"),
        }

        let args_log = Cli::parse_from(["cccp", "act", "log", "-f"]);
        match args_log.command {
            Some(Commands::Act(act)) => match act.action {
                Some(ActSubcommands::Log { follow }) => {
                    assert!(follow);
                }
                _ => panic!("Expected ActSubcommands::Log"),
            },
            _ => panic!("Expected Commands::Act"),
        }
    }

    #[test]
    fn parse_deps_args() {
        let args = Cli::parse_from(["cccp", "deps", "list"]);
        match args.command {
            Some(Commands::Deps(deps)) => {
                assert!(matches!(deps.action, Some(DepsSubcommands::List)));
            }
            _ => panic!("Expected Commands::Deps"),
        }

        let args_check = Cli::parse_from(["cccp", "deps", "check", "git", "docker"]);
        match args_check.command {
            Some(Commands::Deps(deps)) => match deps.action {
                Some(DepsSubcommands::Check { deps: target_deps }) => {
                    assert_eq!(target_deps, vec!["git", "docker"]);
                }
                _ => panic!("Expected DepsSubcommands::Check"),
            },
            _ => panic!("Expected Commands::Deps"),
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

        let args_set_av = Cli::parse_from(["cccp", "set-av", "stack", "rust"]);
        match args_set_av.command {
            Some(Commands::SetAv(set_av)) => {
                assert_eq!(set_av.key, "stack");
                assert_eq!(set_av.value, "rust");
            }
            _ => panic!("Expected Commands::SetAv"),
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
}
