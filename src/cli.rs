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
    fn parse_global_flags() {
        let args = Cli::parse_from(["cccp", "-vv", "--no-color", "-C", "/tmp", "status"]);
        assert_eq!(args.verbose, 2);
        assert!(args.no_color);
        assert_eq!(args.dir, Some(PathBuf::from("/tmp")));
        assert!(matches!(args.command, Some(Commands::Status(_))));
    }
}
