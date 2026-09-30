//! Cultural Gosplan aesthetics, ASCII banners, Soviet easter eggs,
//! and strict NO_COLOR adherence (https://no-color.org).

use std::io::IsTerminal;

/// Terminal styling manager respecting the NO_COLOR specification.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub color_enabled: bool,
}

#[allow(dead_code)]
impl Theme {
    /// Detect whether colors should be enabled based on:
    /// 1. Explicit CLI `--no-color` flag
    /// 2. `NO_COLOR` environment variable (non-empty)
    /// 3. stdout terminal capability (`is_terminal`)
    pub fn init(cli_no_color: bool) -> Self {
        if cli_no_color {
            return Self {
                color_enabled: false,
            };
        }

        // NO_COLOR specification: when present and non-empty, disable color
        if let Ok(val) = std::env::var("NO_COLOR") {
            if !val.is_empty() {
                return Self {
                    color_enabled: false,
                };
            }
        }

        // Check if stdout is an interactive terminal
        let is_term = std::io::stdout().is_terminal();

        Self {
            color_enabled: is_term,
        }
    }

    /// Construct a theme with explicit color toggle (useful for tests)
    pub fn with_color(color_enabled: bool) -> Self {
        Self { color_enabled }
    }

    pub fn red(&self, text: &str) -> String {
        self.colorize("\x1b[31m", text)
    }

    pub fn green(&self, text: &str) -> String {
        self.colorize("\x1b[32m", text)
    }

    pub fn yellow(&self, text: &str) -> String {
        self.colorize("\x1b[33m", text)
    }

    pub fn cyan(&self, text: &str) -> String {
        self.colorize("\x1b[36m", text)
    }

    pub fn bold(&self, text: &str) -> String {
        self.colorize("\x1b[1m", text)
    }

    pub fn dim(&self, text: &str) -> String {
        self.colorize("\x1b[2m", text)
    }

    pub fn red_bold(&self, text: &str) -> String {
        self.colorize("\x1b[1;31m", text)
    }

    pub fn gold_bold(&self, text: &str) -> String {
        self.colorize("\x1b[1;33m", text)
    }

    fn colorize(&self, escape: &str, text: &str) -> String {
        if self.color_enabled {
            format!("{}{}\x1b[0m", escape, text)
        } else {
            text.to_string()
        }
    }
}

/// Official Gosplan ASCII Banner with CCCP Emblem
pub fn gosplan_banner(theme: &Theme) -> String {
    let star = theme.red_bold("★");
    let cccp = theme.red_bold("CCCP");
    let subtitle = theme.gold_bold("\"Workers of the World, Conventionalize! ☭\"");
    let title = theme.bold("CONVENTIONAL COMMITS COMPLIANCE PROGRAM");
    let motto = theme.cyan("«В СВЕТЛОЕ БУДУЩЕЕ С ЧИСТЫМ GIT!»");

    format!(
        r#"
                 {star}
              /-----\
             |  {cccp} |   {title}
              \-----/    {subtitle}
             /   |   \
            /    |    \

{star} {motto}
{star} COMRADE! The State Planning Committee (Gosplan) salutes your discipline!
{star} ORDER NO. 227: Not one step back from Conventional Commits!
{star} Five-Year Plan for 100% compliant Git history is fulfilling its quota!

Directives:
  • feat:     Industrial progress for the Motherland (Minor bump)
  • fix:      Repair sabotage in the machinery (Patch bump)
  • BREAKING: Revolutionary paradigm shift (Major bump)

Glory to the Standardized Commit History!"#
    )
}

/// Order No. 227 ("Ni shagu nazad!" / "Not a step back!") proclamation
pub fn order_227(theme: &Theme) -> String {
    let header = theme.red_bold("★ НАРОДНЫЙ КОМИССАРИАТ ОБОРОНЫ СССР — ПРИКАЗ № 227 ★");
    let motto = theme.gold_bold("«НИ ШАГУ НАЗАД!» / NOT ONE STEP BACK!");
    let date = theme.dim("28 July 1942 — Moscow, Kremlin");

    format!(
        r#"{header}
{date}

{motto}

Comrades!
Panic-mongers and cowards who commit non-conventional commits must be exterminated on the spot!
The git history of the Motherland admits no ambiguity, no unverified tags, no dirty working trees!

1. Every commit MUST adhere to the Conventional Commits 1.0.0 standard.
2. Every release MUST be validated against the Gosplan technical norm (GOST).
3. Retreat from standardized commit discipline is treason against the collective codebase.

NOT ONE STEP BACK FROM CLEAN COMMITS!"#
    )
}

/// State Anthem of the Soviet Union
pub fn soviet_anthem(theme: &Theme) -> String {
    let title = theme.gold_bold("★ ГОСУДАРСТВЕННЫЙ ГИМН СССР / STATE ANTHEM OF THE SOVIET UNION ★");
    let author = theme.dim("Music: A. Aleksandrov | Words: S. Mikhalkov & G. El-Registan (1944)");

    format!(
        r#"{title}
{author}

Союз нерушимый республик свободных
Сплотила навеки Великая Русь.
Да здравствует созданный волей народов
Единый, могучий Советский Союз!

Славься, Отечество наше свободное,
Дружбы народов надёжный оплот!
Партия Ленина — сила народная
Нас к торжеству коммунизма ведёт!

---
[English Translation]
Unbreakable Union of freeborn Republics,
Great Rus' has welded forever to stand.
Long live the creation of the will of the people,
The united, the mighty Soviet Union!

Be glorious, our free Fatherland,
Reliable stronghold of the friendship of peoples!
The party of Lenin, the strength of the people,
Leads us to the triumph of Communism!"#
    )
}

/// Gosplan Five-Year Plan status dashboard
pub fn gosplan_plan(theme: &Theme) -> String {
    let title = theme.red_bold("★ GOSPLAN FIVE-YEAR PLAN FOR GIT REPOSITORIES ★");
    let status = theme.green("COMPLETED IN 4 YEARS (120% of Quota)");
    let bar = theme.gold_bold("[████████████████████████████████████████]");

    format!(
        r#"{title}

Production Targets:
  1. Conventional Commits Compliance Rate: 100.0% [TARGET ACHIEVED]
  2. Predictive SemVer Accuracy:           100.0% [TARGET ACHIEVED]
  3. Pre-Push Hook Defense Perimeter:      Active [NO SABOTAGE DETECTED]
  4. Forge Synchronization Velocity:       < 5ms  [STAKHANOVITE EFFICIENCY]

Current Progress: {status}
{bar} 120%

«Вперёд к победе коммунистического труда!»"#
    )
}

/// Revolutionary quote collection
pub fn revolutionary_quote(theme: &Theme) -> String {
    let quotes = [
        (
            "Without revolutionary theory there can be no revolutionary movement.",
            "V. I. Lenin",
            "What Is To Be Done? (1902)",
        ),
        (
            "The philosophers have only interpreted the world in various ways; the point is to change it.",
            "Karl Marx",
            "Theses on Feuerbach (1845)",
        ),
        (
            "Поехали! (Let's go!)",
            "Yuri Gagarin",
            "Vostok 1 Launch (12 April 1961)",
        ),
        (
            "To achieve great things, we must not only act, but also dream; not only plan, but also believe.",
            "Anatoly Lunacharsky",
            "Commissariat of Enlightenment",
        ),
        (
            "Better fewer, but better.",
            "V. I. Lenin",
            "Pravda (1923)",
        ),
    ];

    // Pick a deterministic quote or cycle
    let (quote, author, source) = quotes[0];
    format!(
        "★ \"{}\"\n  — {}, {}",
        theme.gold_bold(quote),
        theme.bold(author),
        theme.dim(source)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_color_produces_zero_ansi_escapes() {
        let theme = Theme::with_color(false);

        let banner = gosplan_banner(&theme);
        assert!(
            !banner.contains("\x1b["),
            "Banner must not contain ANSI escape codes when color is disabled"
        );

        let order = order_227(&theme);
        assert!(
            !order.contains("\x1b["),
            "Order 227 must not contain ANSI escape codes when color is disabled"
        );

        let anthem = soviet_anthem(&theme);
        assert!(
            !anthem.contains("\x1b["),
            "Anthem must not contain ANSI escape codes when color is disabled"
        );

        let plan = gosplan_plan(&theme);
        assert!(
            !plan.contains("\x1b["),
            "Plan must not contain ANSI escape codes when color is disabled"
        );

        let quote = revolutionary_quote(&theme);
        assert!(
            !quote.contains("\x1b["),
            "Quote must not contain ANSI escape codes when color is disabled"
        );
    }

    #[test]
    fn test_color_enabled_includes_ansi_escapes() {
        let theme = Theme::with_color(true);
        let red_text = theme.red("Test");
        assert!(red_text.starts_with("\x1b[31m"));
        assert!(red_text.ends_with("\x1b[0m"));
    }

    #[test]
    fn test_cultural_slogans_presence() {
        let theme = Theme::with_color(false);
        let banner = gosplan_banner(&theme);
        assert!(banner.contains("В СВЕТЛОЕ БУДУЩЕЕ С ЧИСТЫМ GIT!"));
        assert!(banner.contains("ORDER NO. 227"));

        let order = order_227(&theme);
        assert!(order.contains("НИ ШАГУ НАЗАД!"));
    }
}
