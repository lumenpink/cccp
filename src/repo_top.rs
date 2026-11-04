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
    RecentUpdate,
    ProcessOrder,
    #[default]
    CiStatus,
    RepoName,
}

impl CiTopSort {
    pub fn next(self) -> Self {
        match self {
            CiTopSort::CiStatus => CiTopSort::RecentUpdate,
            CiTopSort::RecentUpdate => CiTopSort::ProcessOrder,
            CiTopSort::ProcessOrder => CiTopSort::RepoName,
            CiTopSort::RepoName => CiTopSort::CiStatus,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CiTopSort::RecentUpdate => "recent",
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
            "recent" | "recentupdate" | "updated" | "recente" => Ok(CiTopSort::RecentUpdate),
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

/// Sanitize legacy phantom pending statuses where description indicates completion
pub fn sanitize_pipeline_statuses(repos: &mut [CatalogRepoSummary]) {
    for repo in repos.iter_mut() {
        for pipe in repo.ci_status.iter_mut() {
            let st = pipe.status.to_ascii_lowercase();
            if st == "pending"
                || st == "running"
                || st == "waiting"
                || st == "blocked"
                || st == "queued"
            {
                if let Some(ref desc) = pipe.description {
                    if crate::catalog::is_skipped_or_cancelled_description(desc) {
                        pipe.status = "skipped".to_string();
                    } else {
                        let d = desc.to_ascii_lowercase();
                        if d.contains("success") || d.contains("successful") {
                            pipe.status = "success".to_string();
                        } else if d.contains("fail")
                            || d.contains("failure")
                            || d.contains("failing")
                            || d.contains("error")
                        {
                            pipe.status = "failure".to_string();
                        }
                    }
                }
            }
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
        CiTopSort::RecentUpdate => {
            filtered.sort_by(|a, b| {
                let time_a = a
                    .ci_status
                    .iter()
                    .filter_map(|p| p.updated_at.as_deref())
                    .max()
                    .unwrap_or("");
                let time_b = b
                    .ci_status
                    .iter()
                    .filter_map(|p| p.updated_at.as_deref())
                    .max()
                    .unwrap_or("");
                if time_a != time_b {
                    return time_b.cmp(time_a);
                }
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            });
        }
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

/// Parse an ISO-8601 timestamp string (e.g. "2026-11-02T05:00:00Z" or "2026-11-02T02:00:00-03:00")
/// into Unix epoch seconds in UTC without external datetime dependencies.
pub fn parse_iso8601_to_epoch(s: &str) -> Option<u64> {
    let clean = s.trim();
    if clean.len() < 19 {
        return None;
    }
    let year: i64 = clean.get(0..4)?.parse().ok()?;
    let month: u64 = clean.get(5..7)?.parse().ok()?;
    let day: u64 = clean.get(8..10)?.parse().ok()?;
    let hour: u64 = clean.get(11..13)?.parse().ok()?;
    let min: u64 = clean.get(14..16)?.parse().ok()?;
    let sec: u64 = clean.get(17..19)?.parse().ok()?;

    if month == 0 || month > 12 || day == 0 || day > 31 || hour > 23 || min > 59 || sec > 60 {
        return None;
    }

    // Days from civil formula (Howard Hinnant algorithm)
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let m_adj = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * m_adj + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe as i64 - 719468;

    let mut epoch = days * 86400 + (hour * 3600 + min * 60 + sec) as i64;

    // Timezone parsing
    let rest = &clean[19..];
    if !rest.is_empty() && rest != "Z" {
        let tz_part = if let Some(dot_idx) = rest.find('.') {
            let after_dot = &rest[dot_idx + 1..];
            if let Some(sign_idx) = after_dot.find(['+', '-', 'Z']) {
                &after_dot[sign_idx..]
            } else {
                ""
            }
        } else {
            rest
        };

        if tz_part.starts_with('+') || tz_part.starts_with('-') {
            let sign = if tz_part.starts_with('-') {
                -1i64
            } else {
                1i64
            };
            let tz_nums = &tz_part[1..];
            let (th, tm) = if let Some(colon) = tz_nums.find(':') {
                let h: i64 = tz_nums[..colon].parse().ok()?;
                let m: i64 = tz_nums[colon + 1..]
                    .chars()
                    .take(2)
                    .collect::<String>()
                    .parse()
                    .ok()?;
                (h, m)
            } else if tz_nums.len() >= 2 {
                let h: i64 = tz_nums[..2].parse().ok()?;
                (h, 0)
            } else {
                (0, 0)
            };
            let offset_sec = sign * (th * 3600 + tm * 60);
            epoch -= offset_sec;
        }
    }

    if epoch < 0 {
        None
    } else {
        Some(epoch as u64)
    }
}

/// Format duration in seconds to a concise string fitting a 7-column cell (e.g. "45s", "14m", "1h15m", "2d4h")
pub fn format_duration_concise(elapsed: u64) -> String {
    if elapsed < 60 {
        format!("{}s", elapsed)
    } else if elapsed < 3600 {
        format!("{}m", elapsed / 60)
    } else if elapsed < 86400 {
        format!("{}h{:02}m", elapsed / 3600, (elapsed % 3600) / 60)
    } else {
        format!("{}d{}h", elapsed / 86400, (elapsed % 86400) / 3600)
    }
}

/// Format elapsed time or duration for CI pipelines fitting a 7-character column.
pub fn format_ci_elapsed_time(ci: &[CiPipelineStatus], now_epoch: u64) -> String {
    if ci.is_empty() {
        return "-".to_string();
    }

    // 1. Check all active queued/running pipelines for any valid updated_at timestamp
    for p in ci {
        let st = p.status.to_ascii_lowercase();
        if st == "running"
            || st == "progress"
            || st == "waiting"
            || st == "queued"
            || st == "blocked"
            || st == "pending"
        {
            if let Some(ref updated) = p.updated_at {
                if let Some(epoch) = parse_iso8601_to_epoch(updated) {
                    let elapsed = now_epoch.saturating_sub(epoch);
                    return format_duration_concise(elapsed);
                }
            }
        }
    }

    // 2. Check for completed pipelines with duration in description (e.g. "Successful in 14m21s")
    for p in ci {
        if let Some(ref desc) = p.description {
            if let Some(pos) = desc.find(" in ") {
                let candidate = desc[pos + 4..].trim();
                let dur: String = candidate
                    .chars()
                    .take_while(|c| c.is_alphanumeric())
                    .collect();
                if !dur.is_empty() {
                    return dur;
                }
            } else if let Some(pos) = desc.find(" after ") {
                let candidate = desc[pos + 7..].trim();
                let dur: String = candidate
                    .chars()
                    .take_while(|c| c.is_alphanumeric())
                    .collect();
                if !dur.is_empty() {
                    return dur;
                }
            }
        }
    }

    // 3. Fallback: single completed job with updated_at
    if let Some(p) = ci.first() {
        if let Some(ref updated) = p.updated_at {
            if let Some(epoch) = parse_iso8601_to_epoch(updated) {
                let elapsed = now_epoch.saturating_sub(epoch);
                return format_duration_concise(elapsed);
            }
        }
    }

    "-".to_string()
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
    let now_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Responsive column budget:
    // Base 80-col budget:
    // marker: 2 ("▶ " / "  ")
    // REPOSITÓRIO: repo_width (20 in 80 cols, expands dynamically with extra space)
    // FORJA: 8
    // ESTADO CI: 16
    // TEMPO: 7
    // TICKETS: 7
    // LOCAL: 15
    // Inter-column spaces: 5
    // Total for 80 cols = 2 + 20 + 1 + 8 + 1 + 16 + 1 + 7 + 1 + 7 + 1 + 15 = 80 chars.
    let extra = cols.saturating_sub(80);
    let deficit = 80usize.saturating_sub(cols);
    let repo_width = if cols >= 80 {
        20 + extra.min(30)
    } else {
        20usize.saturating_sub(deficit).max(8)
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
        "  {:<repo_width$} {:<8} {:<16} {:<7} {:<7} {:<15}",
        "REPOSITÓRIO", "FORJA", "ESTADO CI", "TEMPO", "TICKETS", "LOCAL"
    );
    out.push_str(&theme.dim(&col_header));
    out.push_str("\x1b[K\r\n");

    // Separator line (line 3)
    let max_table_width = repo_width + 58;
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
                let ci_text = truncate_utf8_ellipsis(&raw_ci, 16);
                let colored_ci = match color {
                    "red" => theme.red(&ci_text),
                    "yellow" => theme.yellow(&ci_text),
                    "cyan" => theme.cyan(&ci_text),
                    "green" => theme.green(&ci_text),
                    _ => theme.dim(&ci_text),
                };
                let ci_col = pad_visual(&colored_ci, ci_text.chars().count(), 16);

                let raw_time = format_ci_elapsed_time(&r.ci_status, now_epoch);
                let time_text = truncate_utf8_ellipsis(&raw_time, 7);
                let colored_time = match color {
                    "cyan" => theme.cyan(&time_text),
                    "yellow" => theme.yellow(&time_text),
                    _ => theme.dim(&time_text),
                };
                let time_col = pad_visual(&colored_time, time_text.chars().count(), 7);

                let repo_name = if r.name.is_empty() { &r.slug } else { &r.name };
                let repo_display = truncate_utf8_ellipsis(repo_name, repo_width);
                let repo_padding = repo_width.saturating_sub(repo_display.chars().count());
                let repo_col = format!("{}{}", repo_display, " ".repeat(repo_padding));

                let forge_display = truncate_utf8_ellipsis(&r.forge, 8);
                let forge_padding = 8usize.saturating_sub(forge_display.chars().count());
                let forge_col = format!("{}{}", forge_display, " ".repeat(forge_padding));

                let ticket_str = r.ticket_count.to_string();
                let ticket_display = truncate_utf8_ellipsis(&ticket_str, 7);
                let ticket_padding = 7usize.saturating_sub(ticket_display.chars().count());
                let ticket_col = format!("{}{}", ticket_display, " ".repeat(ticket_padding));

                let status_display = truncate_utf8_ellipsis(&r.status_label, 15);
                let status_padding = 15usize.saturating_sub(status_display.chars().count());
                let status_col = format!("{}{}", status_display, " ".repeat(status_padding));

                let line = format!(
                    "{} {} {} {} {} {}",
                    repo_col, forge_col, ci_col, time_col, ticket_col, status_col
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
            let ex_st = existing.status.to_ascii_lowercase();
            let ar_st = ar.status.to_ascii_lowercase();
            if ar_st == "running"
                || (ex_st != "success" && ex_st != "failure" && ex_st != "skipped")
            {
                existing.status = ar.status;
                if ar.target_url.is_some() {
                    existing.target_url = ar.target_url;
                }
                if ar.description.is_some() {
                    existing.description = ar.description;
                }
            }
        } else {
            repo.ci_status.insert(0, ar);
        }
    }
}

/// Automatically synchronize reconciled CI statuses back to catalog.json on disk if modified.
/// If `target_path` is None, only the in-memory `cat` slice is updated and no disk writes occur.
pub fn auto_sync_catalog_ci_status(
    cat: &mut [crate::catalog::CatalogEntry],
    summaries: &[CatalogRepoSummary],
    target_path: Option<&Path>,
) -> bool {
    if cat.is_empty() || summaries.is_empty() {
        return false;
    }
    let mut modified = false;
    for r in summaries {
        if let Some(entry) = cat
            .iter_mut()
            .find(|e| e.slug.eq_ignore_ascii_case(&r.slug))
        {
            if entry.ci_status != r.ci_status {
                entry.ci_status = r.ci_status.clone();
                modified = true;
            }
        }
    }
    if modified {
        if let Some(path) = target_path {
            let _ = crate::catalog::save_catalog_to_path(cat, path);
        }
    }
    modified
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
    state.sort = initial_sort.unwrap_or(CiTopSort::RecentUpdate);

    let (tx, rx) = std::sync::mpsc::channel::<TopBackgroundMsg>();
    let effective_catalog_path: Option<PathBuf> = catalog_path
        .map(PathBuf::from)
        .or_else(|| crate::catalog::find_catalog_path(None, None));

    let trigger_background_sync =
        |entries: &[crate::catalog::CatalogEntry],
         sender: std::sync::mpsc::Sender<TopBackgroundMsg>| {
            let hosts = crate::forges::discover_forgejo_hosts(entries);
            let local_targets: Vec<(String, Option<PathBuf>)> = entries
                .iter()
                .map(|e| (e.slug.clone(), crate::catalog::find_local_repo_path(e)))
                .collect();
            let forge_targets: Vec<(String, String, String, String)> = entries
                .iter()
                .map(|e| {
                    let info = e.to_forge_info();
                    (e.slug.clone(), info.host, info.owner, info.repo)
                })
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

                // 3. Live commit status inspection from forges for catalog repositories
                for (slug, host, owner, repo) in forge_targets {
                    if let Ok(st) =
                        client.fetch_repo_commit_status(&host, &owner, &repo, "main", None)
                    {
                        if !st.is_empty() {
                            let _ = sender.send(TopBackgroundMsg::RepoUpdate {
                                slug,
                                ci_status: Some(st),
                                status_label: None,
                                unpushed: None,
                                is_clean: None,
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
        let mut cat = crate::catalog::load_catalog(None, catalog_path);
        *summaries = crate::catalog::summarize_catalog(&cat);
        sanitize_pipeline_statuses(summaries);
        auto_sync_catalog_ci_status(&mut cat, summaries, effective_catalog_path.as_deref());
        let hosts = crate::forges::discover_forgejo_hosts(&cat);
        let client = crate::forges::ForgeClient::new();
        for host in hosts {
            if let Ok(jobs) = client.fetch_user_runner_jobs(&host, None) {
                crate::forges::merge_live_runner_jobs(summaries, &jobs);
                auto_sync_catalog_ci_status(&mut cat, summaries, effective_catalog_path.as_deref());
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
                    let mut ci_updated = false;
                    for r in raw_summaries.iter_mut() {
                        if r.slug.eq_ignore_ascii_case(&slug) || r.name.eq_ignore_ascii_case(&slug)
                        {
                            if let Some(new_ci) = ci_status {
                                apply_repo_ci_update(r, new_ci);
                                ci_updated = true;
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
                    if ci_updated {
                        let mut cat = crate::catalog::load_catalog(None, catalog_path);
                        auto_sync_catalog_ci_status(
                            &mut cat,
                            &raw_summaries,
                            effective_catalog_path.as_deref(),
                        );
                    }
                }
                TopBackgroundMsg::RunnerJobsUpdated(jobs) => {
                    let mut cat = crate::catalog::load_catalog(None, catalog_path);
                    let mut base_summaries = crate::catalog::summarize_catalog(&cat);
                    sanitize_pipeline_statuses(&mut base_summaries);
                    for r in raw_summaries.iter_mut() {
                        if let Some(base) = base_summaries
                            .iter()
                            .find(|b| b.slug.eq_ignore_ascii_case(&r.slug))
                        {
                            if !base.ci_status.is_empty() {
                                r.ci_status = base.ci_status.clone();
                            }
                        }
                    }
                    crate::forges::merge_live_runner_jobs(&mut raw_summaries, &jobs);
                    auto_sync_catalog_ci_status(
                        &mut cat,
                        &raw_summaries,
                        effective_catalog_path.as_deref(),
                    );
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
