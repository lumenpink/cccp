//! Comprehensive integration tests for CCCP Internationalization (i18n) module.

use cccp::config::Config;
use cccp::i18n::{get_locale, interpolate, lookup_for_locale, resolve_locale, set_locale, Locale};
use cccp::t;

#[test]
fn test_all_locales_display_and_tag() {
    assert_eq!(Locale::EnUs.as_str(), "en-US");
    assert_eq!(Locale::PtBr.as_str(), "pt-BR");
    assert_eq!(Locale::RuRu.as_str(), "ru-RU");
    assert_eq!(Locale::EsEs.as_str(), "es-ES");

    assert_eq!(format!("{}", Locale::EnUs), "en-US");
    assert_eq!(format!("{}", Locale::PtBr), "pt-BR");
    assert_eq!(format!("{}", Locale::RuRu), "ru-RU");
    assert_eq!(format!("{}", Locale::EsEs), "es-ES");
}

#[test]
fn test_loose_string_parsing_variants() {
    // English
    assert_eq!(Locale::from_str_loose("en"), Some(Locale::EnUs));
    assert_eq!(Locale::from_str_loose("EN-US"), Some(Locale::EnUs));
    assert_eq!(Locale::from_str_loose("en_US.UTF-8"), Some(Locale::EnUs));
    assert_eq!(Locale::from_str_loose("en_GB.iso88591"), Some(Locale::EnUs));
    assert_eq!(Locale::from_str_loose("english"), Some(Locale::EnUs));

    // Portuguese
    assert_eq!(Locale::from_str_loose("pt"), Some(Locale::PtBr));
    assert_eq!(Locale::from_str_loose("pt-br"), Some(Locale::PtBr));
    assert_eq!(Locale::from_str_loose("PT_BR.UTF-8"), Some(Locale::PtBr));
    assert_eq!(
        Locale::from_str_loose("pt_PT.UTF-8@euro"),
        Some(Locale::PtBr)
    );
    assert_eq!(Locale::from_str_loose("portugues"), Some(Locale::PtBr));

    // Russian
    assert_eq!(Locale::from_str_loose("ru"), Some(Locale::RuRu));
    assert_eq!(Locale::from_str_loose("RU-RU"), Some(Locale::RuRu));
    assert_eq!(Locale::from_str_loose("ru_RU.CP1251"), Some(Locale::RuRu));
    assert_eq!(Locale::from_str_loose("russian"), Some(Locale::RuRu));

    // Spanish
    assert_eq!(Locale::from_str_loose("es"), Some(Locale::EsEs));
    assert_eq!(Locale::from_str_loose("es-es"), Some(Locale::EsEs));
    assert_eq!(Locale::from_str_loose("es_AR.UTF-8"), Some(Locale::EsEs));
    assert_eq!(Locale::from_str_loose("espanol"), Some(Locale::EsEs));

    // Unrecognized
    assert_eq!(Locale::from_str_loose("de"), None);
    assert_eq!(Locale::from_str_loose("zh-CN"), None);
    assert_eq!(Locale::from_str_loose(""), None);
}

#[test]
fn test_precedence_cascade_all_layers() {
    // Layer 1: CLI flag wins
    assert_eq!(
        resolve_locale(Some("es-ES"), Some("pt-BR"), Some("ru-RU"), Some("en-US")),
        Locale::EsEs
    );

    // Layer 2: CCCP_LANG wins over config and system
    assert_eq!(
        resolve_locale(None, Some("pt-BR"), Some("ru-RU"), Some("en-US")),
        Locale::PtBr
    );

    // Layer 3: Tool config wins over generic OS LANG
    assert_eq!(
        resolve_locale(None, None, Some("ru-RU"), Some("en_US.UTF-8")),
        Locale::RuRu
    );

    // Layer 4: Generic OS LANG wins over default fallback
    assert_eq!(
        resolve_locale(None, None, None, Some("es_ES.UTF-8")),
        Locale::EsEs
    );

    // Layer 5: Fallback to en-US
    assert_eq!(resolve_locale(None, None, None, None), Locale::EnUs);
    assert_eq!(
        resolve_locale(Some("unknown-lang"), None, None, None),
        Locale::EnUs
    );
}

#[test]
fn test_dictionary_parity_across_all_locales() {
    let en_raw = include_str!("../locales/en-US.toml");
    let pt_raw = include_str!("../locales/pt-BR.toml");
    let ru_raw = include_str!("../locales/ru-RU.toml");
    let es_raw = include_str!("../locales/es-ES.toml");

    let en_table: toml::Table = en_raw.parse().expect("en-US.toml parse failed");
    let pt_table: toml::Table = pt_raw.parse().expect("pt-BR.toml parse failed");
    let ru_table: toml::Table = ru_raw.parse().expect("ru-RU.toml parse failed");
    let es_table: toml::Table = es_raw.parse().expect("es-ES.toml parse failed");

    for (section, sec_val) in &en_table {
        if let toml::Value::Table(sub_table) = sec_val {
            for (key, _) in sub_table {
                let full_key = format!("{}.{}", section, key);

                // Verify pt-BR has key
                let pt_has = pt_table
                    .get(section)
                    .and_then(|v| v.as_table())
                    .and_then(|t| t.get(key));
                assert!(pt_has.is_some(), "Missing key in pt-BR.toml: {}", full_key);

                // Verify ru-RU has key
                let ru_has = ru_table
                    .get(section)
                    .and_then(|v| v.as_table())
                    .and_then(|t| t.get(key));
                assert!(ru_has.is_some(), "Missing key in ru-RU.toml: {}", full_key);

                // Verify es-ES has key
                let es_has = es_table
                    .get(section)
                    .and_then(|v| v.as_table())
                    .and_then(|t| t.get(key));
                assert!(es_has.is_some(), "Missing key in es-ES.toml: {}", full_key);
            }
        }
    }
}

#[test]
fn test_translation_lookups_and_fallbacks() {
    // English
    assert_eq!(
        lookup_for_locale("commit.summary_prompt", Locale::EnUs, &[]),
        "Short, imperative summary of the change (max 72 chars):"
    );

    // Portuguese
    assert_eq!(
        lookup_for_locale("commit.summary_prompt", Locale::PtBr, &[]),
        "Resumo curto e imperativo da alteração (máx 72 caracteres):"
    );

    // Russian
    assert_eq!(
        lookup_for_locale("commit.summary_prompt", Locale::RuRu, &[]),
        "Краткое описание изменений (макс. 72 символа):"
    );

    // Spanish
    assert_eq!(
        lookup_for_locale("commit.summary_prompt", Locale::EsEs, &[]),
        "Resumen breve e imperativo del cambio (máx 72 caracteres):"
    );

    // Non-existent key falls back to key itself
    assert_eq!(
        lookup_for_locale("nonexistent.section.key", Locale::PtBr, &[]),
        "nonexistent.section.key"
    );
}

#[test]
fn test_interpolation_and_macros() {
    let templ = "Found {count} errors in {file}.";
    let res = interpolate(templ, &[("count", "3"), ("file", "src/main.rs")]);
    assert_eq!(res, "Found 3 errors in src/main.rs.");

    // Test macro with active locale
    set_locale(Locale::PtBr);
    assert_eq!(get_locale(), Locale::PtBr);
    let pt_msg = t!("commit.wizard_title");
    assert_eq!(pt_msg, "Assistente de Conventional Commits do CCCP");

    set_locale(Locale::RuRu);
    assert_eq!(get_locale(), Locale::RuRu);
    let ru_msg = t!("commit.wizard_title");
    assert_eq!(ru_msg, "Мастер Conventional Commits CCCP");

    // Reset back to EnUs
    set_locale(Locale::EnUs);
    assert_eq!(get_locale(), Locale::EnUs);
    let en_msg = t!("commit.wizard_title");
    assert_eq!(en_msg, "CCCP Conventional Commits Wizard");

    // Interpolation via t! macro
    let msg = t!("update.up_to_date", version = "3.1.0");
    assert_eq!(msg, "CCCP is up to date (3.1.0).");
}

#[test]
fn test_init_from_config() {
    let mut config = Config::default();
    config
        .extra
        .insert("language".to_string(), "ru".to_string());

    cccp::i18n::init(None, &config);
    assert_eq!(get_locale(), Locale::RuRu);

    // CLI override wins over config
    cccp::i18n::init(Some("pt-BR"), &config);
    assert_eq!(get_locale(), Locale::PtBr);

    // Clean up
    set_locale(Locale::EnUs);
}
