use cccp::catalog::{CatalogRepoSummary, CiPipelineStatus};
use cccp::cultural::Theme;
use cccp::repo_top::{
    pad_visual, render_top_view, truncate_utf8_ellipsis, RepoTopState, FIXED_NON_DATA_ROWS,
};

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn sample_repo(name: &str, status: &str) -> CatalogRepoSummary {
    CatalogRepoSummary {
        name: name.to_string(),
        forge: "forgejo".to_string(),
        slug: format!("lumen/{}", name),
        aliases: vec![],
        ticket_count: 3,
        is_clean: Some(true),
        unpushed: 0,
        status_label: "clean [↑1]".to_string(),
        local_path: None,
        gost_version: Some("2.0.0".to_string()),
        ci_status: if status.is_empty() {
            vec![]
        } else {
            vec![CiPipelineStatus {
                name: "ci".to_string(),
                status: status.to_string(),
                description: None,
                target_url: None,
                updated_at: None,
            }]
        },
        is_mirror: false,
    }
}

#[test]
fn test_render_top_view_line_count_strictly_within_terminal_height() {
    let theme = Theme::init(true);
    let state = RepoTopState::new(5);
    let repos: Vec<CatalogRepoSummary> = (0..50)
        .map(|i| sample_repo(&format!("repo-{}", i), "success"))
        .collect();

    for height in [10, 15, 20, 24, 30, 50] {
        let rendered = render_top_view(&repos, &state, &theme, 80, height);
        let line_count = rendered.lines().count();
        assert!(
            line_count <= height as usize,
            "render_top_view produziu {} linhas, excedendo a altura do terminal de {} linhas",
            line_count,
            height
        );
    }
}

#[test]
fn test_render_top_view_fits_80_columns() {
    let theme = Theme::init(true);
    let state = RepoTopState::new(5);
    let repos = vec![
        sample_repo("autenticação-médica-longa-extensa", "failure"),
        sample_repo("curto", "success"),
        sample_repo("médio-projeto", "running"),
        sample_repo("sem-ci", ""),
    ];

    let rendered = render_top_view(&repos, &state, &theme, 80, 24);

    for (line_no, line) in rendered.lines().enumerate() {
        let clean = strip_ansi(line);
        let char_count = clean.chars().count();
        assert!(
            char_count <= 80,
            "Linha {} excedeu 80 colunas (teve {} caracteres): {:?}",
            line_no + 1,
            char_count,
            clean
        );
    }
}

#[test]
fn test_truncate_utf8_ellipsis() {
    assert_eq!(
        truncate_utf8_ellipsis("autenticação-de-usuário-longo", 15),
        "autenticação-d…"
    );
    assert_eq!(truncate_utf8_ellipsis("simples", 10), "simples");
    assert_eq!(truncate_utf8_ellipsis("exato", 5), "exato");
    assert_eq!(truncate_utf8_ellipsis("curto", 4), "cur…");
    assert_eq!(truncate_utf8_ellipsis("ab", 1), "…");
    assert_eq!(truncate_utf8_ellipsis("a", 1), "a");
    assert_eq!(truncate_utf8_ellipsis("", 5), "");
}

#[test]
fn test_pad_visual_ansi_alignment() {
    let theme = Theme::init(true);
    let red_text = theme.red("✖ 1 failing");
    let green_text = theme.green("✔ 10 passing");
    let none_text = theme.dim("— sem fila");

    let padded_red = pad_visual(&red_text, "✖ 1 failing".chars().count(), 18);
    let padded_green = pad_visual(&green_text, "✔ 10 passing".chars().count(), 18);
    let padded_none = pad_visual(&none_text, "— sem fila".chars().count(), 18);

    assert_eq!(strip_ansi(&padded_red).chars().count(), 18);
    assert_eq!(strip_ansi(&padded_green).chars().count(), 18);
    assert_eq!(strip_ansi(&padded_none).chars().count(), 18);
}

#[test]
fn test_mutation_layout_various_widths() {
    let theme = Theme::init(true);
    let state = RepoTopState::new(5);
    let repos = vec![
        sample_repo("curto", "failure"),
        sample_repo(
            "um-repositorio-com-nome-bem-longo-para-testar-o-comportamento",
            "success",
        ),
    ];

    for width in [40, 60, 80, 100, 120, 160] {
        let rendered = render_top_view(&repos, &state, &theme, width, 24);
        assert!(!rendered.is_empty());
        if width >= 80 {
            for (line_no, line) in rendered.lines().enumerate() {
                let clean = strip_ansi(line);
                let char_count = clean.chars().count();
                assert!(
                    char_count <= width as usize,
                    "Width {}: linha {} excedeu largura ({} > {}): {:?}",
                    width,
                    line_no + 1,
                    char_count,
                    width,
                    clean
                );
            }
        }
    }
}

#[test]
fn test_mutation_truncate_and_padding_limits() {
    assert_eq!(truncate_utf8_ellipsis("teste", 0), "");
    assert_eq!(truncate_utf8_ellipsis("teste", 2), "t…");
    assert_eq!(truncate_utf8_ellipsis("🚀✨🎉", 2), "🚀…");
    assert_eq!(truncate_utf8_ellipsis("🚀✨🎉", 3), "🚀✨🎉");

    // Padding when string is already >= target_width
    let uncolored = "exatamente-18-char";
    let padded = pad_visual(uncolored, uncolored.chars().count(), 18);
    assert_eq!(padded, uncolored);

    let longer = "string-muito-longa-que-passa-de-18";
    let padded_longer = pad_visual(longer, longer.chars().count(), 18);
    assert_eq!(padded_longer, longer);
}

#[test]
fn test_render_top_view_uses_crlf_raw_mode_newlines() {
    let theme = Theme::init(true);
    let state = RepoTopState::new(5);
    let repos = vec![
        sample_repo("hendu", "failure"),
        sample_repo("guara", "success"),
    ];

    let rendered = render_top_view(&repos, &state, &theme, 80, 24);

    // Must contain CRLF
    assert!(
        rendered.contains("\r\n"),
        "render_top_view output deve conter \\r\\n para modo raw"
    );

    // Every \n must be preceded by \r (no standalone \n)
    let bytes = rendered.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            assert!(
                i > 0 && bytes[i - 1] == b'\r',
                "Encontrado LF (\\n) isolado no índice {} sem CR (\\r) anterior no output",
                i
            );
        }
    }
}

#[test]
fn test_mutation_crlf_and_el_in_all_states() {
    let theme = Theme::init(true);
    let mut help_state = RepoTopState::new(5);
    help_state.show_help = true;
    let repos = vec![sample_repo("test-repo", "running")];

    // Help modal
    let help_rendered = render_top_view(&repos, &help_state, &theme, 80, 24);
    assert!(help_rendered.contains("\x1b[K\r\n"));
    for (i, &b) in help_rendered.as_bytes().iter().enumerate() {
        if b == b'\n' {
            assert!(i > 0 && help_rendered.as_bytes()[i - 1] == b'\r');
        }
    }

    // Empty repos
    let state = RepoTopState::new(5);
    let empty_rendered = render_top_view(&[], &state, &theme, 80, 24);
    assert!(empty_rendered.contains("\x1b[K\r\n"));
    for (i, &b) in empty_rendered.as_bytes().iter().enumerate() {
        if b == b'\n' {
            assert!(i > 0 && empty_rendered.as_bytes()[i - 1] == b'\r');
        }
    }
}

#[test]
fn test_mutation_fixed_rows_and_height_boundaries() {
    assert_eq!(FIXED_NON_DATA_ROWS, 5);

    let theme = Theme::init(true);
    let mut state = RepoTopState::new(5);
    state.has_converged = true;

    // 1. First line always contains header title and convergence badge
    let repos_single = vec![sample_repo("primeiro", "success")];
    let rendered_header = render_top_view(&repos_single, &state, &theme, 80, 24);
    let first_line = rendered_header.lines().next().unwrap_or("");
    let clean_first_line = strip_ansi(first_line);
    assert!(
        clean_first_line.contains("CCCP REPO TOP") && clean_first_line.contains("convergente"),
        "Primeira linha deve conter título e status de convergência: {:?}",
        clean_first_line
    );

    // 2. Heights from 5 up to 100 with various repo counts
    for repo_count in [0, 1, 3, 5, 20, 50, 100] {
        let repos: Vec<CatalogRepoSummary> = (0..repo_count)
            .map(|i| sample_repo(&format!("repo-{}", i), "success"))
            .collect();

        for height in [5, 6, 8, 10, 12, 15, 20, 24, 30, 50, 100] {
            let rendered = render_top_view(&repos, &state, &theme, 80, height);
            let line_count = rendered.lines().count();
            assert!(
                line_count <= height as usize,
                "repos: {}, height: {} -> gerou {} linhas",
                repo_count,
                height,
                line_count
            );
        }
    }
}
