//! Pure-Rust internationalization (i18n) engine for CCCP.
//!
//! Provides compile-time embedded translations with zero dynamic library dependencies.
//! Falls back gracefully to `en-US` when keys or locales are missing.

use std::collections::HashMap;
use std::fmt;
use std::sync::{OnceLock, RwLock};

/// Supported locales in CCCP.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Locale {
    #[default]
    EnUs,
    PtBr,
    RuRu,
    EsEs,
}

impl Locale {
    /// Canonical IETF BCP 47 language tag.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EnUs => "en-US",
            Self::PtBr => "pt-BR",
            Self::RuRu => "ru-RU",
            Self::EsEs => "es-ES",
        }
    }

    /// Loose parser from arbitrary strings (e.g. "pt_BR.UTF-8", "ru", "es-ES").
    pub fn from_str_loose(s: &str) -> Option<Self> {
        let clean = s
            .trim()
            .split('.')
            .next()?
            .split('@')
            .next()?
            .to_ascii_lowercase()
            .replace('_', "-");

        if clean == "en" || clean.starts_with("en-") || clean == "english" {
            Some(Self::EnUs)
        } else if clean == "pt"
            || clean.starts_with("pt-")
            || clean == "portuguese"
            || clean == "portugues"
        {
            Some(Self::PtBr)
        } else if clean == "ru"
            || clean.starts_with("ru-")
            || clean == "russian"
            || clean == "russkiy"
        {
            Some(Self::RuRu)
        } else if clean == "es"
            || clean.starts_with("es-")
            || clean == "spanish"
            || clean == "espanol"
        {
            Some(Self::EsEs)
        } else {
            None
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Global active locale
static CURRENT_LOCALE: RwLock<Locale> = RwLock::new(Locale::EnUs);

/// Set the current active locale.
pub fn set_locale(locale: Locale) {
    if let Ok(mut lock) = CURRENT_LOCALE.write() {
        *lock = locale;
    }
}

/// Retrieve the current active locale.
pub fn get_locale() -> Locale {
    CURRENT_LOCALE.read().map(|l| *l).unwrap_or(Locale::EnUs)
}

/// Resolve locale according to the five-layer precedence cascade:
/// 1. CLI explicit flag (`--lang`)
/// 2. `CCCP_LANG` environment variable
/// 3. Tool configuration setting (`language` or `lang` in .cccprc)
/// 4. Generic OS environment (`LC_ALL` / `LC_MESSAGES` / `LANG`)
/// 5. Fallback: `en-US`
pub fn resolve_locale(
    cli_lang: Option<&str>,
    env_cccp: Option<&str>,
    config_lang: Option<&str>,
    env_sys: Option<&str>,
) -> Locale {
    if let Some(loc) = cli_lang.and_then(Locale::from_str_loose) {
        return loc;
    }
    if let Some(loc) = env_cccp.and_then(Locale::from_str_loose) {
        return loc;
    }
    if let Some(loc) = config_lang.and_then(Locale::from_str_loose) {
        return loc;
    }
    if let Some(loc) = env_sys.and_then(Locale::from_str_loose) {
        return loc;
    }
    Locale::EnUs
}

/// Initialize the global i18n system with configuration and environment precedence.
pub fn init(cli_lang: Option<&str>, config: &crate::config::Config) {
    let env_cccp = std::env::var("CCCP_LANG").ok();
    let env_sys = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .ok();
    let config_lang = config
        .extra
        .get("language")
        .or_else(|| config.extra.get("lang"))
        .map(|s| s.as_str());

    let resolved = resolve_locale(
        cli_lang,
        env_cccp.as_deref(),
        config_lang,
        env_sys.as_deref(),
    );
    set_locale(resolved);
}

// Embedded translation resources
static EN_US_RAW: &str = include_str!("../locales/en-US.toml");
static PT_BR_RAW: &str = include_str!("../locales/pt-BR.toml");
static RU_RU_RAW: &str = include_str!("../locales/ru-RU.toml");
static ES_ES_RAW: &str = include_str!("../locales/es-ES.toml");

type StringMap = HashMap<String, String>;

static EN_US_MAP: OnceLock<StringMap> = OnceLock::new();
static PT_BR_MAP: OnceLock<StringMap> = OnceLock::new();
static RU_RU_MAP: OnceLock<StringMap> = OnceLock::new();
static ES_ES_MAP: OnceLock<StringMap> = OnceLock::new();

fn parse_toml_map(raw: &str) -> StringMap {
    let mut map = HashMap::new();
    if let Ok(toml::Value::Table(table)) = raw.parse::<toml::Value>() {
        for (section, sec_val) in table {
            if let toml::Value::Table(sub_table) = sec_val {
                for (k, v) in sub_table {
                    if let toml::Value::String(s) = v {
                        map.insert(format!("{}.{}", section, k), s);
                    }
                }
            } else if let toml::Value::String(s) = sec_val {
                map.insert(section, s);
            }
        }
    }
    map
}

fn get_map_for_locale(locale: Locale) -> &'static StringMap {
    match locale {
        Locale::EnUs => EN_US_MAP.get_or_init(|| parse_toml_map(EN_US_RAW)),
        Locale::PtBr => PT_BR_MAP.get_or_init(|| parse_toml_map(PT_BR_RAW)),
        Locale::RuRu => RU_RU_MAP.get_or_init(|| parse_toml_map(RU_RU_RAW)),
        Locale::EsEs => ES_ES_MAP.get_or_init(|| parse_toml_map(ES_ES_RAW)),
    }
}

/// Look up a translation key for the currently active locale, falling back to en-US.
pub fn lookup(key: &str, vars: &[(&str, &str)]) -> String {
    lookup_for_locale(key, get_locale(), vars)
}

/// Look up a translation key for an explicit locale, falling back to en-US.
pub fn lookup_for_locale(key: &str, locale: Locale, vars: &[(&str, &str)]) -> String {
    let text = if locale != Locale::EnUs {
        let map = get_map_for_locale(locale);
        map.get(key).cloned().or_else(|| {
            let fallback_map = get_map_for_locale(Locale::EnUs);
            fallback_map.get(key).cloned()
        })
    } else {
        let map = get_map_for_locale(Locale::EnUs);
        map.get(key).cloned()
    };

    let template = text.unwrap_or_else(|| key.to_string());
    if vars.is_empty() {
        template
    } else {
        interpolate(&template, vars)
    }
}

/// Interpolate `{name}` variables in template.
pub fn interpolate(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (k, v) in vars {
        let pattern = format!("{{{}}}", k);
        result = result.replace(&pattern, v);
    }
    result
}

/// Macro for ergonomic translation retrieval with optional variable interpolation.
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::lookup($key, &[])
    };
    ($key:expr, $($k:ident = $v:expr),* $(,)?) => {
        $crate::i18n::lookup($key, &[
            $((stringify!($k), &$v.to_string())),*
        ])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_loose_parsing() {
        assert_eq!(Locale::from_str_loose("en"), Some(Locale::EnUs));
        assert_eq!(Locale::from_str_loose("en-US"), Some(Locale::EnUs));
        assert_eq!(Locale::from_str_loose("en_GB.UTF-8"), Some(Locale::EnUs));

        assert_eq!(Locale::from_str_loose("pt"), Some(Locale::PtBr));
        assert_eq!(Locale::from_str_loose("pt-BR"), Some(Locale::PtBr));
        assert_eq!(Locale::from_str_loose("pt_BR.utf8"), Some(Locale::PtBr));

        assert_eq!(Locale::from_str_loose("ru"), Some(Locale::RuRu));
        assert_eq!(Locale::from_str_loose("ru-RU"), Some(Locale::RuRu));
        assert_eq!(Locale::from_str_loose("ru_RU.UTF-8"), Some(Locale::RuRu));

        assert_eq!(Locale::from_str_loose("es"), Some(Locale::EsEs));
        assert_eq!(Locale::from_str_loose("es-ES"), Some(Locale::EsEs));
        assert_eq!(Locale::from_str_loose("es_MX.UTF-8"), Some(Locale::EsEs));

        assert_eq!(Locale::from_str_loose("de_DE.UTF-8"), None);
    }

    #[test]
    fn test_precedence_cascade() {
        assert_eq!(
            resolve_locale(Some("pt-BR"), Some("ru"), Some("es"), Some("en")),
            Locale::PtBr
        );
        assert_eq!(
            resolve_locale(None, Some("ru"), Some("es"), Some("en")),
            Locale::RuRu
        );
        assert_eq!(
            resolve_locale(None, None, Some("es"), Some("en")),
            Locale::EsEs
        );
        assert_eq!(resolve_locale(None, None, None, Some("pt")), Locale::PtBr);
        assert_eq!(resolve_locale(None, None, None, None), Locale::EnUs);
    }

    #[test]
    fn test_translations_and_fallback() {
        let en = lookup_for_locale("app.tagline", Locale::EnUs, &[]);
        assert_eq!(en, "Conventional Commits Compliance Program");

        let pt = lookup_for_locale("app.tagline", Locale::PtBr, &[]);
        assert_eq!(pt, "Programa de Conformidade com Conventional Commits");

        let ru = lookup_for_locale("app.tagline", Locale::RuRu, &[]);
        assert_eq!(ru, "Программа Соответствия Стандарту Conventional Commits");

        let es = lookup_for_locale("app.tagline", Locale::EsEs, &[]);
        assert_eq!(es, "Programa de Cumplimiento de Conventional Commits");

        // Missing key fallback to en-US / key string
        let missing = lookup_for_locale("nonexistent.key", Locale::PtBr, &[]);
        assert_eq!(missing, "nonexistent.key");
    }

    #[test]
    fn test_interpolation() {
        let templ = "Hello {user}, version {ver}!";
        let res = interpolate(templ, &[("user", "Comrade"), ("ver", "3.0.0")]);
        assert_eq!(res, "Hello Comrade, version 3.0.0!");

        let formatted =
            lookup_for_locale("update.up_to_date", Locale::EnUs, &[("version", "3.0.0")]);
        assert_eq!(formatted, "CCCP is up to date (3.0.0).");
    }
}
