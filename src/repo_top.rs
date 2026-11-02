//! Interactive top-style terminal monitoring for repository CI pipelines (GOST DD-TDD #81)

use crate::catalog::{format_ci_summary, CatalogRepoSummary, CiPipelineStatus};
use crate::cultural::Theme;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use std::fmt;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{Duration, Instant};

/// Filter options for CI status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CiTopFilter {
    #[default]
    All,
    FailOrPending,
    Ok,
    Failure,
    Queued,
    NoQueue,
}

impl CiTopFilter {
    pub fn next(self) -> Self {
        match self {
            CiTopFilter::All => CiTopFilter::FailOrPending,
            CiTopFilter::FailOrPending => CiTopFilter::Queued,
            CiTopFilter::Queued => CiTopFilter::Failure,
            CiTopFilter::Failure => CiTopFilter::Ok,
            CiTopFilter::Ok => CiTopFilter::NoQueue,
            CiTopFilter::NoQueue => CiTopFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CiTopFilter::All => "all",
            CiTopFilter::FailOrPending => "fail+pending",
            CiTopFilter::Ok => "ok",
            CiTopFilter::Failure => "fail",
            CiTopFilter::Queued => "queued",
            CiTopFilter::NoQueue => "no-queue",
        }
    }
}

impl fmt::Display for CiTopFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl FromStr for CiTopFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().trim() {
            "all" | "todos" => Ok(CiTopFilter::All),
            "fail+pending" | "pending+fail" | "fail-pending" | "issues" | "alertas"
            | "falha+pendente" | "pendente+falha" => Ok(CiTopFilter::FailOrPending),
            "ok" | "pass" | "success" => Ok(CiTopFilter::Ok),
            "fail" | "failure" | "falha" | "error" => Ok(CiTopFilter::Failure),
            "queued" | "queue" | "fila" | "running" | "pending" => Ok(CiTopFilter::Queued),
            "noqueue" | "no-queue" | "none" | "sem-fila" => Ok(CiTopFilter::NoQueue),
            other => Err(format!("Unknown filter: '{}'", other)),
        }
    }
}

/// Sort criteria for repository list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CiTopSort {
    ProcessOrder,
    #[default]
    CiStatus,
    RepoName,
}

impl CiTopSort {
    pub fn next(self) -> Self {
        match self {
            CiTopSort::CiStatus => CiTopSort::ProcessOrder,
            CiTopSort::ProcessOrder => CiTopSort::RepoName,
            CiTopSort::RepoName => CiTopSort::CiStatus,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CiTopSort::CiStatus => "status",
            CiTopSort::ProcessOrder => "process",
            CiTopSort::RepoName => "name",
        }
    }
}

impl fmt::Display for CiTopSort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl FromStr for CiTopSort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().trim() {
            "process" | "process-order" | "ordem" => Ok(CiTopSort::ProcessOrder),
            "status" | "ci" | "estado" => Ok(CiTopSort::CiStatus),
            "name" | "nome" | "repo" => Ok(CiTopSort::RepoName),
            other => Err(format!("Unknown sort: '{}'", other)),
        }
    }
}

/// Navigation and control actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoTopAction {
    CursorUp,
    CursorDown,
    PageUp,
    PageDown,
    CycleFilter,
    CycleSort,
    ToggleHelp,
    Refresh,
    Quit,
}

/// Frames for the discrete TUI background spinner
pub const TOP_SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Messages dispatched by the background worker thread in repo top
#[derive(Debug, Clone)]
pub enum TopBackgroundMsg {
    SyncStarted,
    RepoUpdate {
        slug: String,
        ci_status: Option<Vec<CiPipelineStatus>>,
        status_label: Option<String>,
        unpushed: Option<usize>,
        is_clean: Option<bool>,
    },
    RunnerJobsUpdated(Vec<crate::forges::ForgeActionRunnerJob>),
    SyncCompleted,
}

/// State of the interactive CI top monitor
#[derive(Debug, Clone)]
pub struct RepoTopState {
    pub filter: CiTopFilter,
    pub sort: CiTopSort,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub show_help: bool,
    pub interval_secs: u64,
    pub is_syncing: bool,
    pub has_converged: bool,
    pub spinner_frame: usize,
}

impl RepoTopState {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            filter: CiTopFilter::All,
            sort: CiTopSort::CiStatus,
            selected_index: 0,
            scroll_offset: 0,
            show_help: false,
            interval_secs: interval_secs.max(1),
            is_syncing: false,
            has_converged: false,
            spinner_frame: 0,
        }
    }

    pub fn handle_action(&mut self, action: RepoTopAction, total_items: usize) {
        match action {
            RepoTopAction::ToggleHelp => {
                self.show_help = !self.show_help;
            }
            RepoTopAction::CycleFilter => {
                self.filter = self.filter.next();
                self.selected_index = 0;
                self.scroll_offset = 0;
            }
            RepoTopAction::CycleSort => {
                self.sort = self.sort.next();
                self.selected_index = 0;
                self.scroll_offset = 0;
            }
            RepoTopAction::CursorUp => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
            }
            RepoTopAction::CursorDown => {
                if total_items > 0 && self.selected_index + 1 < total_items {
                    self.selected_index += 1;
                }
            }
            RepoTopAction::PageUp => {
                let page_size = 10;
                if self.selected_index >= page_size {
                    self.selected_index -= page_size;
                } else {
                    self.selected_index = 0;
                }
                if self.selected_index < self.scroll_offset {
                    self.scroll_offset = self.selected_index;
                }
            }
            RepoTopAction::PageDown => {
                let page_size = 10;
                if total_items > 0 {
                    self.selected_index = (self.selected_index + page_size).min(total_items - 1);
                }
            }
            RepoTopAction::Refresh | RepoTopAction::Quit => {}
        }
    }
}

/// Helper to classify CI pipeline status of a repository
fn repo_ci_category(ci: &[CiPipelineStatus]) -> &'static str {
    if ci.is_empty() {
        return "none";
    }
    let has_queued = ci.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "pending" || st == "running" || st == "waiting" || st == "progress" || st == "blocked"
    });
    if has_queued {
        return "queued";
    }
    let has_fail = ci.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "failure" || st == "fail" || st == "error"
    });
    if has_fail {
        return "fail";
    }
    "ok"
}

/// Helper to determine numeric CI priority for monitor sorting.
/// 0: Running (actively compiling on runner)
/// 1: Waiting (queued and waiting for runner)
/// 2: Blocked / Pending (waiting on dependencies or external conditions)
/// 3: Failure
/// 4: Pass / Ok
/// 5: None
pub fn repo_ci_priority(ci: &[CiPipelineStatus]) -> usize {
    if ci.is_empty() {
        return 5;
    }
    let has_running = ci.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "running" || st == "progress"
    });
    if has_running {
        return 0;
    }
    let has_waiting = ci.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "waiting" || st == "queued"
    });
    if has_waiting {
        return 1;
    }
    let has_blocked = ci.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "blocked" || st == "pending"
    });
    if has_blocked {
        return 2;
    }
    let has_fail = ci.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "failure" || st == "fail" || st == "error"
    });
    if has_fail {
        return 3;
    }
    4
}

/// Extract numeric runner job ID from pipeline descriptions (e.g. "Runner Job #1234 on run #56")
pub fn extract_runner_job_id(ci: &[CiPipelineStatus]) -> Option<u64> {
    for p in ci {
        let st = p.status.to_ascii_lowercase();
        if st == "pending" || st == "running" || st == "waiting" || st == "progress" {
            if let Some(ref desc) = p.description {
                if let Some(pos) = desc.find("Runner Job #") {
                    let rest = &desc[pos + 12..];
                    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(num) = digits.parse::<u64>() {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

/// Filter and sort catalog repository summaries for the top view
pub fn apply_top_filters_and_sorting(
    repos: &[CatalogRepoSummary],
    filter: CiTopFilter,
    sort: CiTopSort,
) -> Vec<CatalogRepoSummary> {
    let mut filtered: Vec<CatalogRepoSummary> = repos
        .iter()
        .filter(|r| match filter {
            CiTopFilter::All => true,
            CiTopFilter::FailOrPending => {
                let cat = repo_ci_category(&r.ci_status);
                cat == "fail" || cat == "queued"
            }
            CiTopFilter::Ok => repo_ci_category(&r.ci_status) == "ok",
            CiTopFilter::Failure => repo_ci_category(&r.ci_status) == "fail",
            CiTopFilter::Queued => repo_ci_category(&r.ci_status) == "queued",
            CiTopFilter::NoQueue => {
                let cat = repo_ci_category(&r.ci_status);
                cat == "none" || cat == "ok"
            }
        })
        .cloned()
        .collect();

    match sort {
        CiTopSort::ProcessOrder => {
            // Preserves original catalog ordering
        }
        CiTopSort::RepoName => {
            filtered.sort_by_key(|a| a.name.to_lowercase());
        }
        CiTopSort::CiStatus => {
            filtered.sort_by(|a, b| {
                let prio_a = repo_ci_priority(&a.ci_status);
                let prio_b = repo_ci_priority(&b.ci_status);
                if prio_a != prio_b {
                    return prio_a.cmp(&prio_b);
                }
                // Within active queue tiers (running / waiting / blocked), order by runner job id
                if prio_a <= 2 {
                    let job_a = extract_runner_job_id(&a.ci_status);
                    let job_b = extract_runner_job_id(&b.ci_status);
                    match (job_a, job_b) {
                        (Some(ja), Some(jb)) if ja != jb => return ja.cmp(&jb),
                        (Some(_), None) => return std::cmp::Ordering::Less,
                        (None, Some(_)) => return std::cmp::Ordering::Greater,
                        _ => {}
                    }
                }
                // Tie break by repository name
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            });
        }
    }

    filtered
}

/// Truncate UTF-8 string safely with ellipsis if exceeding max_chars
pub fn truncate_utf8_ellipsis(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = s.chars().count();
    if count <= max_chars {
        return s.to_string();
    }
    if max_chars == 1 {
        return "…".to_string();
    }
    let mut truncated: String = s.chars().take(max_chars - 1).collect();
    truncated.push('…');
    truncated
}

/// Pad visual string with spaces, taking into account visible character count
pub fn pad_visual(colored_str: &str, plain_text_len: usize, target_width: usize) -> String {
    let padding = target_width.saturating_sub(plain_text_len);
    format!("{}{}", colored_str, " ".repeat(padding))
}

/// Number of fixed non-data lines in the top view layout:
/// 1. Header (title, badges, convergence indicator)
/// 2. Column headers (REPOSITÓRIO, FORJA...)
/// 3. Top table separator line
/// 4. Bottom table separator line
/// 5. Bottom shortcuts / help line
pub const FIXED_NON_DATA_ROWS: usize = 5;

/// Pure string rendering function for testing and terminal painting
pub fn render_top_view(
    repos: &[CatalogRepoSummary],
    state: &RepoTopState,
    theme: &Theme,
    width: u16,
    height: u16,
) -> String {
    let mut out = String::new();
    let cols = width.max(40) as usize;
    let rows = (height as usize).max(FIXED_NON_DATA_ROWS);

    // Responsive column budget:
    // Base 80-col budget:
    // marker: 2 ("▶ " / "  ")
    // REPOSITÓRIO: repo_width (22 in 80 cols, expands dynamically with extra space)
    // FORJA: 9
    // ESTADO CI: 18
    // TICKETS: 8
    // LOCAL: 16
    // Inter-column spaces: 4
    // Total for 80 cols = 2 + 22 + 1 + 9 + 1 + 18 + 1 + 8 + 1 + 16 = 79 chars.
    let extra = cols.saturating_sub(80);
    let deficit = 80usize.saturating_sub(cols);
    let repo_width = if cols >= 80 {
        22 + extra.min(30)
    } else {
        22usize.saturating_sub(deficit).max(8)
    };

    // Header (line 1)
    let header_title = if cols >= 88 {
        format!(
            "★ CCCP REPO TOP — CI MONITOR ({}s refresh)",
            state.interval_secs
        )
    } else {
        format!("★ CCCP REPO TOP ({}s)", state.interval_secs)
    };
    let filter_badge = format!("[Filtro: {}]", state.filter);
    let sort_badge = format!("[Ordem: {}]", state.sort);
    let total_badge = format!("Total: {}", repos.len());

    let sync_badge = if state.is_syncing {
        let frame = TOP_SPINNER_FRAMES[state.spinner_frame % TOP_SPINNER_FRAMES.len()];
        theme.cyan(&format!("{} sincronizando", frame))
    } else if state.has_converged {
        theme.green("✓ convergente")
    } else {
        String::new()
    };

    let header_line = if sync_badge.is_empty() {
        format!(
            "{} {} {} {}",
            theme.gold_bold(&header_title),
            theme.cyan(&filter_badge),
            theme.dim(&sort_badge),
            theme.bold(&total_badge)
        )
    } else {
        format!(
            "{} {} {} {} {}",
            theme.gold_bold(&header_title),
            theme.cyan(&filter_badge),
            theme.dim(&sort_badge),
            theme.bold(&total_badge),
            sync_badge
        )
    };
    out.push_str(&header_line);
    out.push_str("\x1b[K\r\n");

    // Table Column Header (line 2)
    let col_header = format!(
        "  {:<repo_width$} {:<9} {:<18} {:<8} {:<16}",
        "REPOSITÓRIO", "FORJA", "ESTADO CI", "TICKETS", "LOCAL"
    );
    out.push_str(&theme.dim(&col_header));
    out.push_str("\x1b[K\r\n");

    // Separator line (line 3)
    let max_table_width = repo_width + 57;
    let sep_len = cols.min(max_table_width);
    let sep = "─".repeat(sep_len);
    out.push_str(&theme.dim(&sep));
    out.push_str("\x1b[K\r\n");

    // Help modal overlay if active
    if state.show_help {
        out.push_str(&format!(
            "{}\x1b[K\r\n",
            theme.gold_bold("┌─ [ Ajuda / Teclas de Atalho ] ────────────────────────────────┐")
        ));
        out.push_str(&format!(
            "│  {:<12} : Alternar esta janela de ajuda                    │\x1b[K\r\n",
            "Ctrl+H / F1"
        ));
        out.push_str(&format!(
            "│  {:<12} : Alternar ordenação (status -> process -> name)   │\x1b[K\r\n",
            "Ctrl+S / ^O"
        ));
        out.push_str(&format!(
            "│  {:<12} : Alternar filtro de CI (all, fail+pending, q...)  │\x1b[K\r\n",
            "Ctrl+F / ^T"
        ));
        out.push_str(&format!(
            "│  {:<12} : Forçar atualização imediata do catálogo/CI      │\x1b[K\r\n",
            "Ctrl+R"
        ));
        out.push_str(&format!(
            "│  {:<12} : Linha anterior / Linha seguinte                  │\x1b[K\r\n",
            "↑ / ↓ / j / k"
        ));
        out.push_str(&format!(
            "│  {:<12} : Página anterior / Página seguinte                │\x1b[K\r\n",
            "PgUp / PgDn"
        ));
        out.push_str(&format!(
            "│  {:<12} : Sair do monitor top                              │\x1b[K\r\n",
            "Ctrl+Q / q"
        ));
        out.push_str(&format!(
            "{}\x1b[K\r\n",
            theme.gold_bold("└───────────────────────────────────────────────────────────────┘")
        ));
    } else {
        // Table body: available visible lines = rows - FIXED_NON_DATA_ROWS
        let visible_lines = rows.saturating_sub(FIXED_NON_DATA_ROWS);
        let start = state.scroll_offset;

        if repos.is_empty() {
            if visible_lines > 0 {
                out.push_str(
                    &theme.dim("  Nenhum repositório corresponde ao filtro ativo.\x1b[K\r\n"),
                );
            }
        } else {
            for (idx, r) in repos.iter().enumerate().skip(start).take(visible_lines) {
                let (raw_ci, color) = format_ci_summary(&r.ci_status);
                let ci_text = truncate_utf8_ellipsis(&raw_ci, 18);
                let colored_ci = match color {
                    "red" => theme.red(&ci_text),
                    "yellow" => theme.yellow(&ci_text),
                    "cyan" => theme.cyan(&ci_text),
                    "green" => theme.green(&ci_text),
                    _ => theme.dim(&ci_text),
                };
                let ci_col = pad_visual(&colored_ci, ci_text.chars().count(), 18);

                let repo_name = if r.name.is_empty() { &r.slug } else { &r.name };
                let repo_display = truncate_utf8_ellipsis(repo_name, repo_width);
                let repo_padding = repo_width.saturating_sub(repo_display.chars().count());
                let repo_col = format!("{}{}", repo_display, " ".repeat(repo_padding));

                let forge_display = truncate_utf8_ellipsis(&r.forge, 9);
                let forge_padding = 9usize.saturating_sub(forge_display.chars().count());
                let forge_col = format!("{}{}", forge_display, " ".repeat(forge_padding));

                let ticket_str = r.ticket_count.to_string();
                let ticket_display = truncate_utf8_ellipsis(&ticket_str, 8);
                let ticket_padding = 8usize.saturating_sub(ticket_display.chars().count());
                let ticket_col = format!("{}{}", ticket_display, " ".repeat(ticket_padding));

                let status_display = truncate_utf8_ellipsis(&r.status_label, 16);
                let status_padding = 16usize.saturating_sub(status_display.chars().count());
                let status_col = format!("{}{}", status_display, " ".repeat(status_padding));

                let line = format!(
                    "{} {} {} {} {}",
                    repo_col, forge_col, ci_col, ticket_col, status_col
                );

                if idx == state.selected_index {
                    out.push_str(&theme.bold(&format!("▶ {}", line)));
                } else {
                    out.push_str(&format!("  {}", line));
                }
                out.push_str("\x1b[K\r\n");
            }
        }
    }

    // Bottom Bar
    let bottom_text = if cols < 78 {
        "Ctrl+H aj | Ctrl+S ord | Ctrl+F flt | Ctrl+R rfr | Ctrl+Q sai"
    } else {
        "★ Ctrl+H help | Ctrl+S sort | Ctrl+F filter | Ctrl+R refresh | Ctrl+Q quit"
    };
    let bottom_display = truncate_utf8_ellipsis(bottom_text, cols.min(max_table_width));
    out.push_str(&theme.dim(&sep));
    out.push_str("\x1b[K\r\n");
    out.push_str(&theme.cyan(&bottom_display));
    out.push_str("\x1b[K");

    out
}

/// Merge new CI statuses onto a repository summary while preserving active runner jobs
pub fn apply_repo_ci_update(
    repo: &mut CatalogRepoSummary,
    new_ci: Vec<crate::catalog::CiPipelineStatus>,
) {
    if new_ci.is_empty() {
        return;
    }
    let active_runners: Vec<_> = repo
        .ci_status
        .iter()
        .filter(|p| {
            let st = p.status.to_ascii_lowercase();
            st == "running" || st == "waiting" || st == "pending"
        })
        .cloned()
        .collect();

    repo.ci_status = new_ci;

    for ar in active_runners {
        if let Some(existing) = repo
            .ci_status
            .iter_mut()
            .find(|p| crate::forges::pipeline_matches_job_name(&p.name, &ar.name))
        {
            existing.status = ar.status;
            if ar.target_url.is_some() {
                existing.target_url = ar.target_url;
            }
            if ar.description.is_some() {
                existing.description = ar.description;
            }
        } else {
            repo.ci_status.insert(0, ar);
        }
    }
}

/// Interactive loop running the TUI monitor
pub fn run_repo_top(
    catalog_path: Option<&Path>,
    interval_secs: u64,
    initial_filter: Option<CiTopFilter>,
    initial_sort: Option<CiTopSort>,
    theme: &Theme,
) -> Result<(), String> {
    let mut state = RepoTopState::new(interval_secs);
    if let Some(f) = initial_filter {
        state.filter = f;
    }
    if let Some(s) = initial_sort {
        state.sort = s;
    }

    let (tx, rx) = std::sync::mpsc::channel::<TopBackgroundMsg>();

    let trigger_background_sync =
        |entries: &[crate::catalog::CatalogEntry],
         sender: std::sync::mpsc::Sender<TopBackgroundMsg>| {
            let hosts = crate::forges::discover_forgejo_hosts(entries);
            let local_targets: Vec<(String, Option<PathBuf>)> = entries
                .iter()
                .map(|e| (e.slug.clone(), crate::catalog::find_local_repo_path(e)))
                .collect();

            std::thread::spawn(move || {
                let _ = sender.send(TopBackgroundMsg::SyncStarted);
                let client = crate::forges::ForgeClient::new();

                // 1. Consolidated live runner jobs (1 HTTP call per forge host)
                let mut all_jobs = Vec::new();
                for host in &hosts {
                    if let Ok(jobs) = client.fetch_user_runner_jobs(host, None) {
                        all_jobs.extend(jobs);
                    }
                }
                let _ = sender.send(TopBackgroundMsg::RunnerJobsUpdated(all_jobs));

                // 2. Fast local git status inspection (pure local read, NO network git fetch)
                for (slug, local_path) in local_targets {
                    if let Some(ref path) = local_path {
                        if let Some(st) = crate::git::inspect_local_status(path) {
                            let label = match (st.is_clean, st.ahead, st.modified_files) {
                                (true, 0, _) => "clean".to_string(),
                                (true, n, _) => format!("clean [↑{}]", n),
                                (false, 0, m) => {
                                    if m > 0 {
                                        format!("dirty [{}*]", m)
                                    } else {
                                        "dirty".to_string()
                                    }
                                }
                                (false, n, m) => {
                                    if m > 0 {
                                        format!("dirty [{}* ↑{}]", m, n)
                                    } else {
                                        format!("dirty [↑{}]", n)
                                    }
                                }
                            };
                            let _ = sender.send(TopBackgroundMsg::RepoUpdate {
                                slug,
                                ci_status: None,
                                status_label: Some(label),
                                unpushed: Some(st.ahead),
                                is_clean: Some(st.is_clean),
                            });
                        }
                    }
                }

                let _ = sender.send(TopBackgroundMsg::SyncCompleted);
            });
        };

    // Helper closure to reload catalog, merge live runner jobs, and trigger background live forge CI sync
    let refresh_data = |summaries: &mut Vec<CatalogRepoSummary>,
                        sender: &std::sync::mpsc::Sender<TopBackgroundMsg>| {
        let cat = crate::catalog::load_catalog(None, catalog_path);
        *summaries = crate::catalog::summarize_catalog(&cat);
        let hosts = crate::forges::discover_forgejo_hosts(&cat);
        let client = crate::forges::ForgeClient::new();
        for host in hosts {
            if let Ok(jobs) = client.fetch_user_runner_jobs(&host, None) {
                crate::forges::merge_live_runner_jobs(summaries, &jobs);
            }
        }
        trigger_background_sync(&cat, sender.clone());
    };

    // Load initial catalog with live CI runner jobs
    let mut raw_summaries = Vec::new();
    state.is_syncing = true;
    refresh_data(&mut raw_summaries, &tx);

    // Enter Alternate Screen and enable raw mode
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout_handle = stdout();
    let _ = execute!(
        stdout_handle,
        EnterAlternateScreen,
        crossterm::cursor::Hide,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    );

    let mut last_refresh = Instant::now();
    let interval = Duration::from_secs(state.interval_secs);

    loop {
        // Drain any incoming background sync and live CI status updates
        while let Ok(msg) = rx.try_recv() {
            match msg {
                TopBackgroundMsg::SyncStarted => {
                    state.is_syncing = true;
                }
                TopBackgroundMsg::RepoUpdate {
                    slug,
                    ci_status,
                    status_label,
                    unpushed,
                    is_clean,
                } => {
                    for r in raw_summaries.iter_mut() {
                        if r.slug.eq_ignore_ascii_case(&slug) || r.name.eq_ignore_ascii_case(&slug)
                        {
                            if let Some(new_ci) = ci_status {
                                apply_repo_ci_update(r, new_ci);
                            }
                            if let Some(lbl) = status_label {
                                r.status_label = lbl;
                            }
                            if let Some(unp) = unpushed {
                                r.unpushed = unp;
                            }
                            if let Some(cln) = is_clean {
                                r.is_clean = Some(cln);
                            }
                            break;
                        }
                    }
                }
                TopBackgroundMsg::RunnerJobsUpdated(jobs) => {
                    let cat = crate::catalog::load_catalog(None, catalog_path);
                    let base_summaries = crate::catalog::summarize_catalog(&cat);
                    for r in raw_summaries.iter_mut() {
                        if let Some(base) = base_summaries
                            .iter()
                            .find(|b| b.slug.eq_ignore_ascii_case(&r.slug))
                        {
                            r.ci_status = base.ci_status.clone();
                        }
                    }
                    crate::forges::merge_live_runner_jobs(&mut raw_summaries, &jobs);
                }
                TopBackgroundMsg::SyncCompleted => {
                    state.is_syncing = false;
                    state.has_converged = true;
                    last_refresh = Instant::now();
                }
            }
        }

        // Compute filtered list
        let filtered = apply_top_filters_and_sorting(&raw_summaries, state.filter, state.sort);

        // Keep selected index within bounds
        if !filtered.is_empty() && state.selected_index >= filtered.len() {
            state.selected_index = filtered.len() - 1;
        }

        // Adjust scroll offset
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let visible_lines = (rows as usize).saturating_sub(FIXED_NON_DATA_ROWS).max(1);
        if state.selected_index >= state.scroll_offset + visible_lines {
            state.scroll_offset = state.selected_index + 1 - visible_lines;
        } else if state.selected_index < state.scroll_offset {
            state.scroll_offset = state.selected_index;
        }

        // Render frame
        let frame = render_top_view(&filtered, &state, theme, cols, rows);
        let _ = execute!(stdout_handle, crossterm::cursor::MoveTo(0, 0));
        let _ = write!(stdout_handle, "{}\x1b[J", frame);
        let _ = stdout_handle.flush();

        // Advance spinner animation while background sync is active
        if state.is_syncing {
            state.spinner_frame = state.spinner_frame.wrapping_add(1);
        }

        // Calculate remaining timeout: 100ms smooth animation while syncing, or interval-based
        let elapsed = last_refresh.elapsed();
        let timeout = if state.is_syncing {
            Duration::from_millis(100)
        } else if elapsed >= interval {
            Duration::from_millis(10)
        } else {
            interval - elapsed
        };

        if event::poll(timeout).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                let action = match (key.code, key.modifiers) {
                    (KeyCode::Char('c'), KeyModifiers::CONTROL)
                    | (KeyCode::Char('q'), KeyModifiers::CONTROL)
                    | (KeyCode::Char('q'), KeyModifiers::NONE)
                    | (KeyCode::Esc, _) => Some(RepoTopAction::Quit),

                    (KeyCode::Char('h'), KeyModifiers::CONTROL) | (KeyCode::F(1), _) => {
                        Some(RepoTopAction::ToggleHelp)
                    }

                    (KeyCode::Char('s'), KeyModifiers::CONTROL)
                    | (KeyCode::Char('o'), KeyModifiers::CONTROL) => Some(RepoTopAction::CycleSort),

                    (KeyCode::Char('f'), KeyModifiers::CONTROL)
                    | (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                        Some(RepoTopAction::CycleFilter)
                    }

                    (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(RepoTopAction::Refresh),

                    (KeyCode::Up, _)
                    | (KeyCode::Char('k'), KeyModifiers::NONE)
                    | (KeyCode::Char('p'), KeyModifiers::CONTROL) => Some(RepoTopAction::CursorUp),

                    (KeyCode::Down, _)
                    | (KeyCode::Char('j'), KeyModifiers::NONE)
                    | (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                        Some(RepoTopAction::CursorDown)
                    }

                    (KeyCode::PageUp, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                        Some(RepoTopAction::PageUp)
                    }

                    (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                        Some(RepoTopAction::PageDown)
                    }

                    _ => None,
                };

                if let Some(act) = action {
                    if act == RepoTopAction::Quit {
                        break;
                    }
                    if act == RepoTopAction::Refresh {
                        state.is_syncing = true;
                        state.has_converged = false;
                        refresh_data(&mut raw_summaries, &tx);
                        last_refresh = Instant::now();
                    } else {
                        state.handle_action(act, filtered.len());
                    }
                }
            }
        }

        // Auto refresh tick: only when not already syncing and interval has passed since convergence
        if !state.is_syncing && last_refresh.elapsed() >= interval {
            state.is_syncing = true;
            state.has_converged = false;
            let cat = crate::catalog::load_catalog(None, catalog_path);
            trigger_background_sync(&cat, tx.clone());
            last_refresh = Instant::now();
        }
    }

    // Teardown terminal
    let _ = execute!(stdout_handle, crossterm::cursor::Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();

    Ok(())
}
