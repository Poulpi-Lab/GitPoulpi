//! Which language to show: `GITPOULPI_LANG`, then the setting, then the OS.

use std::sync::OnceLock;

use crate::Lang;

/// The environment override; it beats `ui.language`. Accepts everything
/// [`Lang::parse`] does, plus `qps` (the pseudo-locale).
pub const ENV_VAR: &str = "GITPOULPI_LANG";

/// Test hook: overrides the OS locale `auto` resolves to (see [`system_lang`]).
pub const SYSTEM_LOCALE_ENV_VAR: &str = "GITPOULPI_SYSTEM_LOCALE";

/// The OS locale as a language, computed once per process (it reads the
/// environment or calls a platform API). `C`, `POSIX`, an unsupported
/// language or no answer at all give English.
pub fn system_lang() -> Lang {
    static SYSTEM: OnceLock<Lang> = OnceLock::new();
    *SYSTEM.get_or_init(|| {
        // `GITPOULPI_SYSTEM_LOCALE` stands in for the OS locale so tests can pin `auto`:
        // Windows ignores `LANG`/`LC_ALL` (the API answers with the user's locale).
        let pinned = std::env::var(SYSTEM_LOCALE_ENV_VAR)
            .ok()
            .filter(|v| !v.is_empty());
        from_locale(pinned.or_else(sys_locale::get_locale).as_deref())
    })
}

/// [`system_lang`]'s mapping, on an explicit locale string (`fr-CA`,
/// `de_CH.UTF-8`, `pt-BR`, ...). Never `qps`: the pseudo-locale is opt-in.
pub fn from_locale(locale: Option<&str>) -> Lang {
    match locale.and_then(Lang::parse) {
        Some(Lang::Qps) | None => Lang::En,
        Some(lang) => lang,
    }
}

/// The effective language. `env` is the raw `GITPOULPI_LANG` value, `setting`
/// the configured language (`None`: `auto`). A value of `env` that names no
/// language is ignored. A language that is not built into this binary
/// (`lang-*` features) falls back to English.
pub fn resolve(env: Option<&str>, setting: Option<Lang>) -> Lang {
    let lang = env
        .and_then(Lang::parse)
        .or(setting)
        .unwrap_or_else(system_lang);
    if lang.is_built() {
        lang
    } else {
        Lang::En
    }
}

/// [`resolve`] reading `GITPOULPI_LANG` from the process environment.
pub fn resolve_from_env(setting: Option<Lang>) -> Lang {
    resolve(std::env::var(ENV_VAR).ok().as_deref(), setting)
}

/// Whether `GITPOULPI_LANG` is set to a language, i.e. the setting is
/// currently overridden (the Settings row says so).
pub fn env_override() -> Option<Lang> {
    std::env::var(ENV_VAR).ok().as_deref().and_then(Lang::parse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locales_map_to_languages() {
        assert_eq!(from_locale(Some("fr-CA")), Lang::Fr);
        assert_eq!(from_locale(Some("de_CH.UTF-8")), Lang::De);
        assert_eq!(from_locale(Some("pt-BR")), Lang::PtBr);
        assert_eq!(from_locale(Some("pt-PT")), Lang::PtPt);
        for (locale, want) in [
            ("zh_CN.UTF-8", Lang::ZhCn),
            ("zh-CN", Lang::ZhCn),
            ("zh-Hans", Lang::ZhCn),
            ("zh-Hans-CN", Lang::ZhCn),
            ("zh_SG", Lang::ZhCn),
            ("ja_JP.UTF-8", Lang::Ja),
            ("ja", Lang::Ja),
            ("ko_KR.UTF-8", Lang::Ko),
            ("ru_RU.UTF-8", Lang::Ru),
            ("uk_UA", Lang::Uk),
            ("pl_PL.UTF-8", Lang::Pl),
            ("cs_CZ", Lang::Cs),
            ("sv_SE.UTF-8", Lang::Sv),
            ("sv-FI", Lang::Sv),
            ("tr_TR.UTF-8", Lang::Tr),
            ("nl_NL", Lang::Nl),
            ("nl_BE.UTF-8", Lang::Nl),
        ] {
            assert_eq!(from_locale(Some(locale)), want, "{locale}");
        }
        // Traditional Chinese is not supported: English, not simplified.
        for locale in ["zh_TW.UTF-8", "zh-Hant", "zh_HK", "zh-Hant-TW", "zh_MO"] {
            assert_eq!(from_locale(Some(locale)), Lang::En, "{locale}");
        }
        assert_eq!(from_locale(Some("C")), Lang::En);
        assert_eq!(from_locale(Some("POSIX")), Lang::En);
        assert_eq!(from_locale(Some("ar_EG.UTF-8")), Lang::En);
        assert_eq!(from_locale(Some("qps")), Lang::En);
        assert_eq!(from_locale(None), Lang::En);
    }

    #[cfg(feature = "all-langs")]
    #[test]
    fn env_beats_setting_beats_system() {
        assert_eq!(resolve(Some("de"), Some(Lang::Fr)), Lang::De);
        assert_eq!(resolve(Some("qps"), None), Lang::Qps);
        assert_eq!(resolve(Some("klingon"), Some(Lang::Fr)), Lang::Fr);
        assert_eq!(resolve(None, Some(Lang::Es)), Lang::Es);
    }

    #[test]
    fn an_unbuilt_language_resolves_to_english() {
        for l in Lang::ALL.into_iter().filter(|l| !l.is_built()) {
            assert_eq!(resolve(None, Some(l)), Lang::En, "{l:?}");
            assert_eq!(resolve(Some(l.code()), None), Lang::En, "{l:?}");
        }
        for l in Lang::ALL.into_iter().filter(|l| l.is_built()) {
            assert_eq!(resolve(None, Some(l)), l);
        }
    }

    #[test]
    fn rest_of_env_priority() {
        assert_eq!(resolve(None, None), system_lang());
    }
}
