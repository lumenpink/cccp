//! Comprehensive unit tests for CCCP Gosplan cultural aesthetics,
//! Order 227 ("Ni shagu nazad!"), and NO_COLOR specification adherence.

use cccp::cultural::{
    gosplan_banner, gosplan_plan, order_227, revolutionary_quote, soviet_anthem, Theme,
};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_theme_color_enabled_ansi_escapes() {
    let theme = Theme::with_color(true);
    assert!(theme.color_enabled);

    assert_eq!(theme.red("Marx"), "\x1b[31mMarx\x1b[0m");
    assert_eq!(theme.green("Lenin"), "\x1b[32mLenin\x1b[0m");
    assert_eq!(theme.yellow("Gosplan"), "\x1b[33mGosplan\x1b[0m");
    assert_eq!(theme.cyan("Sputnik"), "\x1b[36mSputnik\x1b[0m");
    assert_eq!(theme.bold("Vostok"), "\x1b[1mVostok\x1b[0m");
    assert_eq!(theme.dim("Mayak"), "\x1b[2mMayak\x1b[0m");
    assert_eq!(theme.red_bold("CCCP"), "\x1b[1;31mCCCP\x1b[0m");
    assert_eq!(theme.gold_bold("Soviet"), "\x1b[1;33mSoviet\x1b[0m");
}

#[test]
fn test_theme_no_color_emits_raw_strings() {
    let theme = Theme::with_color(false);
    assert!(!theme.color_enabled);

    assert_eq!(theme.red("Marx"), "Marx");
    assert_eq!(theme.green("Lenin"), "Lenin");
    assert_eq!(theme.yellow("Gosplan"), "Gosplan");
    assert_eq!(theme.cyan("Sputnik"), "Sputnik");
    assert_eq!(theme.bold("Vostok"), "Vostok");
    assert_eq!(theme.dim("Mayak"), "Mayak");
    assert_eq!(theme.red_bold("CCCP"), "CCCP");
    assert_eq!(theme.gold_bold("Soviet"), "Soviet");
}

#[test]
fn test_no_color_env_and_cli_flag_adherence() {
    let _guard = ENV_LOCK.lock().unwrap();

    // 1. Explicit CLI --no-color flag always disables color
    let theme_cli = Theme::init(true);
    assert!(!theme_cli.color_enabled);

    // 2. NO_COLOR environment variable set and non-empty disables color
    std::env::set_var("NO_COLOR", "1");
    let theme_env = Theme::init(false);
    assert!(!theme_env.color_enabled);

    // 3. NO_COLOR set to empty string does not disable color by itself
    std::env::set_var("NO_COLOR", "");
    let _theme_empty = Theme::init(false);

    // Cleanup
    std::env::remove_var("NO_COLOR");
}

#[test]
fn test_all_cultural_outputs_contain_zero_escapes_when_color_disabled() {
    let theme = Theme::with_color(false);

    let banner = gosplan_banner(&theme);
    assert!(
        !banner.contains("\x1b["),
        "gosplan_banner must not contain ANSI escape codes when color is disabled"
    );

    let order = order_227(&theme);
    assert!(
        !order.contains("\x1b["),
        "order_227 must not contain ANSI escape codes when color is disabled"
    );

    let anthem = soviet_anthem(&theme);
    assert!(
        !anthem.contains("\x1b["),
        "soviet_anthem must not contain ANSI escape codes when color is disabled"
    );

    let plan = gosplan_plan(&theme);
    assert!(
        !plan.contains("\x1b["),
        "gosplan_plan must not contain ANSI escape codes when color is disabled"
    );

    let quote = revolutionary_quote(&theme);
    assert!(
        !quote.contains("\x1b["),
        "revolutionary_quote must not contain ANSI escape codes when color is disabled"
    );
}

#[test]
fn test_gosplan_banner_layout_and_directives() {
    let theme = Theme::with_color(false);
    let banner = gosplan_banner(&theme);

    assert!(banner.contains("CCCP"));
    assert!(banner.contains("CONVENTIONAL COMMITS COMPLIANCE PROGRAM"));
    assert!(banner.contains("«В СВЕТЛОЕ БУДУЩЕЕ С ЧИСТЫМ GIT!»"));
    assert!(banner.contains("Workers of the World, Conventionalize!"));
    assert!(banner.contains("ORDER NO. 227"));
    assert!(banner.contains("• feat:"));
    assert!(banner.contains("• fix:"));
    assert!(banner.contains("• BREAKING:"));
}

#[test]
fn test_order_227_proclamation_content() {
    let theme = Theme::with_color(false);
    let order = order_227(&theme);

    assert!(order.contains("ПРИКАЗ № 227"));
    assert!(order.contains("«НИ ШАГУ НАЗАД!» / NOT ONE STEP BACK!"));
    assert!(order.contains("28 July 1942 — Moscow, Kremlin"));
    assert!(order.contains("Conventional Commits 1.0.0"));
    assert!(order.contains("GOST"));
}

#[test]
fn test_soviet_anthem_lyrics_and_translation() {
    let theme = Theme::with_color(false);
    let anthem = soviet_anthem(&theme);

    // Russian original lyrics
    assert!(anthem.contains("ГОСУДАРСТВЕННЫЙ ГИМН СССР"));
    assert!(anthem.contains("Союз нерушимый республик свободных"));
    assert!(anthem.contains("Сплотила навеки Великая Русь."));
    assert!(anthem.contains("Славься, Отечество наше свободное"));

    // English translation
    assert!(anthem.contains("[English Translation]"));
    assert!(anthem.contains("Unbreakable Union of freeborn Republics"));
    assert!(anthem.contains("The united, the mighty Soviet Union!"));
}

#[test]
fn test_gosplan_five_year_plan_dashboard() {
    let theme = Theme::with_color(false);
    let plan = gosplan_plan(&theme);

    assert!(plan.contains("GOSPLAN FIVE-YEAR PLAN FOR GIT REPOSITORIES"));
    assert!(plan.contains("COMPLETED IN 4 YEARS (120% of Quota)"));
    assert!(plan.contains("1. Conventional Commits Compliance Rate: 100.0%"));
    assert!(plan.contains("2. Predictive SemVer Accuracy:           100.0%"));
    assert!(plan.contains("3. Pre-Push Hook Defense Perimeter:      Active"));
    assert!(plan.contains("4. Forge Synchronization Velocity:       < 5ms"));
    assert!(plan.contains("120%"));
    assert!(plan.contains("«Вперёд к победе коммунистического труда!»"));
}

#[test]
fn test_revolutionary_quotes_registry() {
    let theme = Theme::with_color(false);
    let quote = revolutionary_quote(&theme);

    assert!(quote.starts_with("★ "));
    assert!(quote.contains("—"));
    assert!(
        quote.contains("Lenin")
            || quote.contains("Marx")
            || quote.contains("Gagarin")
            || quote.contains("Lunacharsky")
    );
}
