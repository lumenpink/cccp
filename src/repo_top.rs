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
use std::path::Path;
use std::str::FromStr;
use std::time::{Duration, Instant};

/// Filter options for CI status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CiTopFilter {
    #[default]
    All,
    Ok,
    Failure,
    Queued,
    NoQueue,
}

impl CiTopFilter {
    pub fn next(self) -> Self {
        match self {
            CiTopFilter::All => CiTopFilter::Ok,
            CiTopFilter::Ok => CiTopFilter::Failure,
            CiTopFilter::Failure => CiTopFilter::Queued,
            CiTopFilter::Queued => CiTopFilter::NoQueue,
            CiTopFilter::NoQueue => CiTopFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CiTopFilter::All => "all",
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
    #[default]
    ProcessOrder,
    CiStatus,
    RepoName,
}

impl CiTopSort {
    pub fn next(self) -> Self {
        match self {
            CiTopSort::ProcessOrder => CiTopSort::CiStatus,
            CiTopSort::CiStatus => CiTopSort::RepoName,
            CiTopSort::RepoName => CiTopSort::ProcessOrder,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CiTopSort::ProcessOrder => "process",
            CiTopSort::CiStatus => "status",
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

/// State of the interactive CI top monitor
#[derive(Debug, Clone)]
pub struct RepoTopState {
    pub filter: CiTopFilter,
    pub sort: CiTopSort,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub show_help: bool,
    pub interval_secs: u64,
}

impl RepoTopState {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            filter: CiTopFilter::All,
            sort: CiTopSort::ProcessOrder,
            selected_index: 0,
            scroll_offset: 0,
            show_help: false,
            interval_secs: interval_secs.max(1),
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
    let has_fail = ci.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "failure" || st == "fail" || st == "error"
    });
    if has_fail {
        return "fail";
    }
    let has_queued = ci.iter().any(|p| {
        let st = p.status.to_ascii_lowercase();
        st == "pending" || st == "running" || st == "waiting" || st == "progress"
    });
    if has_queued {
        return "queued";
    }
    "ok"
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
            filtered.sort_by_key(|r| match repo_ci_category(&r.ci_status) {
                "fail" => 0,
                "queued" => 1,
                "ok" => 2,
                _ => 3,
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
    let rows = height.max(10) as usize;

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

    let header_line = format!(
        "{} {} {} {}",
        theme.gold_bold(&header_title),
        theme.cyan(&filter_badge),
        theme.dim(&sort_badge),
        theme.bold(&total_badge)
    );
    out.push_str(&header_line);
    out.push('\n');

    // Table Column Header (line 2)
    let col_header = format!(
        "  {:<repo_width$} {:<9} {:<18} {:<8} {:<16}",
        "REPOSITÓRIO", "FORJA", "ESTADO CI", "TICKETS", "LOCAL"
    );
    out.push_str(&theme.dim(&col_header));
    out.push('\n');

    // Separator line (line 3)
    let max_table_width = repo_width + 57;
    let sep_len = cols.min(max_table_width);
    let sep = "─".repeat(sep_len);
    out.push_str(&theme.dim(&sep));
    out.push('\n');

    // Help modal overlay if active
    if state.show_help {
        out.push_str(&format!(
            "{}\n",
            theme.gold_bold("┌─ [ Ajuda / Teclas de Atalho ] ────────────────────────────────┐")
        ));
        out.push_str(&format!(
            "│  {:<12} : Alternar esta janela de ajuda                    │\n",
            "Ctrl+H / F1"
        ));
        out.push_str(&format!(
            "│  {:<12} : Alternar ordenação (process -> status -> name)   │\n",
            "Ctrl+S / ^O"
        ));
        out.push_str(&format!(
            "│  {:<12} : Alternar filtro de CI (all, ok, fail, queue...)  │\n",
            "Ctrl+F / ^T"
        ));
        out.push_str(&format!(
            "│  {:<12} : Forçar atualização imediata do catálogo/CI      │\n",
            "Ctrl+R"
        ));
        out.push_str(&format!(
            "│  {:<12} : Linha anterior / Linha seguinte                  │\n",
            "↑ / ↓ / j / k"
        ));
        out.push_str(&format!(
            "│  {:<12} : Página anterior / Página seguinte                │\n",
            "PgUp / PgDn"
        ));
        out.push_str(&format!(
            "│  {:<12} : Sair do monitor top                              │\n",
            "Ctrl+Q / q"
        ));
        out.push_str(&format!(
            "{}\n",
            theme.gold_bold("└───────────────────────────────────────────────────────────────┘")
        ));
    } else {
        // Table body: available visible lines = rows - 4 (header, col_header, sep, bottom_bar)
        let visible_lines = rows.saturating_sub(4).max(1);
        let start = state.scroll_offset;

        if repos.is_empty() {
            out.push_str(&theme.dim("  Nenhum repositório corresponde ao filtro ativo.\n"));
        } else {
            for (idx, r) in repos.iter().enumerate().skip(start).take(visible_lines) {
                let (raw_ci, color) = format_ci_summary(&r.ci_status);
                let ci_text = truncate_utf8_ellipsis(&raw_ci, 18);
                let colored_ci = match color {
                    "red" => theme.red(&ci_text),
                    "yellow" => theme.yellow(&ci_text),
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
                out.push('\n');
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
    out.push('\n');
    out.push_str(&theme.cyan(&bottom_display));

    out
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

    // Helper closure to reload catalog and merge consolidated runner jobs
    let refresh_data = |summaries: &mut Vec<CatalogRepoSummary>| {
        let cat = crate::catalog::load_catalog(None, catalog_path);
        *summaries = crate::catalog::summarize_catalog(&cat);
        let hosts = crate::forges::discover_forgejo_hosts(&cat);
        let client = crate::forges::ForgeClient::new();
        for host in hosts {
            if let Ok(jobs) = client.fetch_user_runner_jobs(&host, None) {
                crate::forges::merge_live_runner_jobs(summaries, &jobs);
            }
        }
    };

    // Load initial catalog with live CI runner jobs
    let mut raw_summaries = Vec::new();
    refresh_data(&mut raw_summaries);

    // Enter Alternate Screen and enable raw mode
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout_handle = stdout();
    let _ = execute!(stdout_handle, EnterAlternateScreen, crossterm::cursor::Hide);

    let mut last_refresh = Instant::now();
    let interval = Duration::from_secs(state.interval_secs);

    loop {
        // Compute filtered list
        let filtered = apply_top_filters_and_sorting(&raw_summaries, state.filter, state.sort);

        // Keep selected index within bounds
        if !filtered.is_empty() && state.selected_index >= filtered.len() {
            state.selected_index = filtered.len() - 1;
        }

        // Adjust scroll offset
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        let visible_lines = (rows as usize).saturating_sub(4).max(1);
        if state.selected_index >= state.scroll_offset + visible_lines {
            state.scroll_offset = state.selected_index + 1 - visible_lines;
        } else if state.selected_index < state.scroll_offset {
            state.scroll_offset = state.selected_index;
        }

        // Render frame
        let frame = render_top_view(&filtered, &state, theme, cols, rows);
        let _ = execute!(
            stdout_handle,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        );
        let _ = write!(stdout_handle, "{}", frame);
        let _ = stdout_handle.flush();

        // Calculate remaining timeout
        let elapsed = last_refresh.elapsed();
        let timeout = if elapsed >= interval {
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
                        refresh_data(&mut raw_summaries);
                        last_refresh = Instant::now();
                    } else {
                        state.handle_action(act, filtered.len());
                    }
                }
            }
        }

        // Auto refresh tick
        if last_refresh.elapsed() >= interval {
            refresh_data(&mut raw_summaries);
            last_refresh = Instant::now();
        }
    }

    // Teardown terminal
    let _ = execute!(stdout_handle, crossterm::cursor::Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();

    Ok(())
}
