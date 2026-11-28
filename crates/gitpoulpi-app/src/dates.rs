//! Dates, rendered the same way in every frontend: the `ui.date_*` patterns,
//! the tokenizer behind them and the Gregorian conversion.
//!
//! Token set: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D` `dddd` `ddd` `HH`
//! `mm` `ss`. Month and weekday names come from the UI language's catalog
//! (`gitpoulpi-i18n`), or stay English with `ui.date_locale = "english"`.
//!
//! | Key | Reads it |
//! |---|---|
//! | `ui.date_format` | A file's Blame/History date column ([`DateSettings::format_short`]) |
//! | `ui.date_time_format` | The Log's Date column ([`DateSettings::format_time`]) |
//! | `ui.date_verbose_format` | The commit-detail Date line, the Date column's tooltip, "Copy > Date" ([`DateSettings::format_verbose`]) |
//!
//! Each pattern is independent, so changing one never silently changes another
//! column. `ui.date_word_format` is a legacy key, parsed and read by nothing.
//! Times are UTC unix seconds; there is deliberately no datetime crate.

use gitpoulpi_config::UiConfig;

/// Days-since-epoch -> proleptic Gregorian (y, m, d), Howard Hinnant's
/// `civil_from_days` - avoids pulling in a datetime crate for one
/// display column.
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Renders `ts` (unix seconds, UTC) through a date pattern (`YYYY-MM-DD HH:mm`-style tokens).
///
/// Longest match first, which is the whole subtlety - scanning `MM`
/// before `MMMM` turns `MMMM` into two numeric months rather than a
/// month name. Anything else passes through verbatim, so `DD.MM.YYYY @
/// HH:mm` keeps its separators and its `@`. `english` is the
/// `ui.date_locale = "english"` switch: names stay English whatever the
/// interface language.
pub fn format_date(pattern: &str, ts: i64, english: bool) -> String {
    DatePattern::parse(pattern).render(ts, english)
}

/// The three `ui.date_*` patterns and the locale switch, tokenized once.
/// A frontend builds it from the config (again when Preferences changes one)
/// and renders through it, so both frontends print a date identically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateSettings {
    pub short: DatePattern,
    pub time: DatePattern,
    pub verbose: DatePattern,
    /// `ui.date_locale` is `english`: month and weekday names ignore the UI
    /// language.
    pub english: bool,
}

impl DateSettings {
    pub fn from_config(ui: &UiConfig) -> Self {
        Self {
            short: DatePattern::parse(&ui.date_format),
            time: DatePattern::parse(&ui.date_time_format),
            verbose: DatePattern::parse(&ui.date_verbose_format),
            english: ui.date_locale.trim().eq_ignore_ascii_case("english"),
        }
    }

    /// The Blame/History date (`ui.date_format`).
    pub fn format_short(&self, ts: i64) -> String {
        self.short.render(ts, self.english)
    }

    /// The Log's Date column (`ui.date_time_format`).
    pub fn format_time(&self, ts: i64) -> String {
        self.time.render(ts, self.english)
    }

    /// The commit-detail Date line, tooltip and "Copy > Date"
    /// (`ui.date_verbose_format`).
    pub fn format_verbose(&self, ts: i64) -> String {
        self.verbose.render(ts, self.english)
    }
}

impl Default for DateSettings {
    fn default() -> Self {
        Self::from_config(&UiConfig::default())
    }
}

/// One piece of a tokenized date pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
enum DatePiece {
    Year4,
    Year2,
    MonthName,
    MonthAbbr,
    Month2,
    Month,
    Day2,
    Day,
    WeekdayName,
    WeekdayAbbr,
    Hour,
    Minute,
    Second,
    /// The whole `D MMMM YYYY` run (`DD` when `pad`), written the way the
    /// language writes it (`msg::date::day_month_year`: `5 March 2024`,
    /// `2024年3月5日`, `2024년 3월 5일`).
    DayMonthYear {
        pad: bool,
    },
    Literal(String),
}

/// A date pattern tokenized once, so the Log's Date column (one call per
/// visible row, every frame) does no scanning and builds no throwaway
/// token strings. `format_date` is `parse` + `render`; a frontend parses its
/// configured patterns once ([`DateSettings`]).
///
/// Month and weekday names come straight from the catalog as `&'static str`
/// (no allocation, no lookup table to build per frame); a month written next
/// to a day number uses its genitive form (`5 мая`, `5 maja`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatePattern {
    pieces: Vec<DatePiece>,
    /// The pattern has a day token, so a spelled-out month is the form that
    /// follows a day number.
    with_day: bool,
}

fn date_text(english: bool, m: gitpoulpi_i18n::M) -> &'static str {
    if english {
        m.english()
    } else {
        m.get()
    }
}

impl DatePattern {
    /// Longest token first, which is the whole subtlety (see [`format_date`]).
    pub fn parse(pattern: &str) -> Self {
        const TOKENS: [(&str, DatePiece); 13] = [
            ("YYYY", DatePiece::Year4),
            ("YY", DatePiece::Year2),
            ("MMMM", DatePiece::MonthName),
            ("MMM", DatePiece::MonthAbbr),
            ("MM", DatePiece::Month2),
            ("M", DatePiece::Month),
            ("DD", DatePiece::Day2),
            ("D", DatePiece::Day),
            ("dddd", DatePiece::WeekdayName),
            ("ddd", DatePiece::WeekdayAbbr),
            ("HH", DatePiece::Hour),
            ("mm", DatePiece::Minute),
            ("ss", DatePiece::Second),
        ];
        let mut pieces: Vec<DatePiece> = Vec::new();
        let mut i = 0;
        'outer: while i < pattern.len() {
            for (token, piece) in &TOKENS {
                if pattern[i..].starts_with(token) {
                    pieces.push(piece.clone());
                    i += token.len();
                    continue 'outer;
                }
            }
            // Not a token: copy one whole char, not one byte.
            let ch = pattern[i..].chars().next().expect("i is a char boundary");
            match pieces.last_mut() {
                Some(DatePiece::Literal(text)) => text.push(ch),
                _ => pieces.push(DatePiece::Literal(ch.to_string())),
            }
            i += ch.len_utf8();
        }
        let with_day = pieces
            .iter()
            .any(|p| matches!(p, DatePiece::Day | DatePiece::Day2));
        // `D MMMM YYYY` as one piece, so the CJK languages can reorder it.
        let space = |p: &DatePiece| matches!(p, DatePiece::Literal(s) if s == " ");
        if let Some(at) = pieces.windows(5).position(|w| {
            matches!(w[0], DatePiece::Day | DatePiece::Day2)
                && space(&w[1])
                && w[2] == DatePiece::MonthName
                && space(&w[3])
                && w[4] == DatePiece::Year4
        }) {
            let pad = pieces[at] == DatePiece::Day2;
            pieces.splice(at..at + 5, [DatePiece::DayMonthYear { pad }]);
        }
        Self { pieces, with_day }
    }

    pub fn render(&self, ts: i64, english: bool) -> String {
        use gitpoulpi_i18n::{month_abbr, month_name, month_name_of, weekday_abbr, weekday_name};
        use std::fmt::Write as _;
        let days = ts.div_euclid(86_400);
        let secs = ts.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        let (hour, minute, second) = (secs / 3600, (secs % 3600) / 60, secs % 60);
        // 1970-01-01 was a Thursday; 0 = Monday.
        let weekday = (days + 3).rem_euclid(7) as u32;
        let mut out = String::with_capacity(24);
        for piece in &self.pieces {
            // Writing into a `String` cannot fail.
            let _ = match piece {
                DatePiece::Year4 => write!(out, "{year:04}"),
                DatePiece::Year2 => write!(out, "{:02}", year.rem_euclid(100)),
                DatePiece::MonthName => {
                    let m = if self.with_day {
                        month_name_of(month)
                    } else {
                        month_name(month)
                    };
                    write!(out, "{}", date_text(english, m))
                }
                DatePiece::MonthAbbr => write!(out, "{}", date_text(english, month_abbr(month))),
                DatePiece::Month2 => write!(out, "{month:02}"),
                DatePiece::Month => write!(out, "{month}"),
                DatePiece::Day2 => write!(out, "{day:02}"),
                DatePiece::Day => write!(out, "{day}"),
                DatePiece::WeekdayName => {
                    write!(out, "{}", date_text(english, weekday_name(weekday)))
                }
                DatePiece::WeekdayAbbr => {
                    write!(out, "{}", date_text(english, weekday_abbr(weekday)))
                }
                DatePiece::Hour => write!(out, "{hour:02}"),
                DatePiece::Minute => write!(out, "{minute:02}"),
                DatePiece::Second => write!(out, "{second:02}"),
                DatePiece::DayMonthYear { pad } => {
                    let day = if *pad {
                        format!("{day:02}")
                    } else {
                        day.to_string()
                    };
                    let month = date_text(english, month_name_of(month));
                    if english {
                        write!(out, "{day} {month} {year:04}")
                    } else {
                        // The language's own order and marks (`2024年3月5日`).
                        out.push_str(&gitpoulpi_i18n::msg::date::day_month_year(
                            day,
                            month,
                            format_args!("{year:04}"),
                        ));
                        Ok(())
                    }
                }
                DatePiece::Literal(text) => {
                    out.push_str(text);
                    Ok(())
                }
            };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_i18n::Lang;

    #[test]
    fn month_names_follow_the_language() {
        let pattern = DatePattern::parse("D MMMM YYYY MMM");
        // 2024-03-05 00:00 UTC.
        let ts = 1_709_596_800;
        assert_eq!(pattern.render(ts, false), "5 March 2024 Mar");
        let _g = gitpoulpi_i18n::override_lang(Lang::Fr);
        assert_eq!(pattern.render(ts, false), "5 mars 2024 mars");
    }

    /// 2024-03-05 12:34:56 UTC, a Tuesday.
    const TS: i64 = 1_709_642_096;

    fn render_in(lang: Lang, pattern: &str) -> String {
        let _g = gitpoulpi_i18n::override_lang(lang);
        DatePattern::parse(pattern).render(TS, false)
    }

    fn in_lang(lang: Lang, pattern: &str) -> String {
        render_in(lang, pattern)
    }

    #[test]
    fn a_month_after_a_day_number_is_genitive_where_the_language_has_one() {
        assert_eq!(in_lang(Lang::Ru, "D MMMM YYYY"), "5 марта 2024");
        assert_eq!(in_lang(Lang::Uk, "D MMMM YYYY"), "5 березня 2024");
        assert_eq!(in_lang(Lang::Pl, "D MMMM YYYY"), "5 marca 2024");
        assert_eq!(in_lang(Lang::Cs, "D MMMM YYYY"), "5. března 2024");
        // On its own it is the nominative.
        assert_eq!(in_lang(Lang::Ru, "MMMM YYYY"), "март 2024");
        assert_eq!(in_lang(Lang::Uk, "MMMM"), "березень");
        assert_eq!(in_lang(Lang::Pl, "MMMM"), "marzec");
        assert_eq!(in_lang(Lang::Cs, "MMMM"), "březen");
        // May is the case where the two forms differ most.
        let may = 1_715_000_000; // 2024-05-06
        let _g = gitpoulpi_i18n::override_lang(Lang::Ru);
        assert_eq!(DatePattern::parse("D MMMM").render(may, false), "6 мая");
        assert_eq!(DatePattern::parse("MMMM").render(may, false), "май");
    }

    #[test]
    fn weekday_names_follow_the_language() {
        for (lang, full, abbr) in [
            (Lang::En, "Tuesday", "Tue"),
            (Lang::Fr, "mardi", "mar."),
            (Lang::Es, "martes", "mar"),
            (Lang::PtBr, "terça-feira", "ter"),
            (Lang::PtPt, "terça-feira", "ter"),
            (Lang::It, "martedì", "mar"),
            (Lang::De, "Dienstag", "Di"),
            (Lang::ZhCn, "星期二", "周二"),
            (Lang::Ja, "火曜日", "火"),
            (Lang::Ko, "화요일", "화"),
            (Lang::Ru, "вторник", "вт"),
            (Lang::Uk, "вівторок", "вт"),
            (Lang::Pl, "wtorek", "wt"),
            (Lang::Cs, "úterý", "út"),
            (Lang::Nl, "dinsdag", "di"),
            (Lang::Sv, "tisdag", "tis"),
            (Lang::Tr, "Salı", "Sal"),
        ] {
            assert_eq!(in_lang(lang, "dddd"), full, "{lang:?}");
            assert_eq!(in_lang(lang, "ddd"), abbr, "{lang:?}");
        }
        // The week starts on a Monday and the epoch was a Thursday.
        let _g = gitpoulpi_i18n::override_lang(Lang::En);
        let p = DatePattern::parse("ddd");
        assert_eq!(p.render(0, false), "Thu");
        let names: Vec<String> = (0..7)
            .map(|d| p.render(4 * 86_400 + d * 86_400, false))
            .collect();
        assert_eq!(names, ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]);
    }

    #[test]
    fn every_language_spells_the_default_verbose_date() {
        for (lang, want) in [
            (Lang::En, "5 March 2024 12:34"),
            (Lang::Fr, "5 mars 2024 12:34"),
            (Lang::Es, "5 de marzo de 2024 12:34"),
            (Lang::PtBr, "5 de março de 2024 12:34"),
            (Lang::PtPt, "5 de março de 2024 12:34"),
            (Lang::It, "5 marzo 2024 12:34"),
            (Lang::De, "5. März 2024 12:34"),
            (Lang::ZhCn, "2024年3月5日 12:34"),
            (Lang::Ja, "2024年3月5日 12:34"),
            (Lang::Ko, "2024년 3월 5일 12:34"),
            (Lang::Ru, "5 марта 2024 12:34"),
            (Lang::Uk, "5 березня 2024 12:34"),
            (Lang::Pl, "5 marca 2024 12:34"),
            (Lang::Cs, "5. března 2024 12:34"),
            (Lang::Nl, "5 maart 2024 12:34"),
            (Lang::Sv, "5 mars 2024 12:34"),
            (Lang::Tr, "5 Mart 2024 12:34"),
        ] {
            assert_eq!(in_lang(lang, "D MMMM YYYY HH:mm"), want, "{lang:?}");
        }
        // Zero-padded day, CJK order too; other patterns keep their order.
        assert_eq!(in_lang(Lang::Ja, "DD MMMM YYYY"), "2024年3月05日");
        assert_eq!(in_lang(Lang::Ja, "YYYY-MM-DD"), "2024-03-05");
        assert_eq!(in_lang(Lang::Ja, "MMMM D"), "3月 5");
    }

    #[test]
    fn the_english_date_locale_ignores_the_ui_language() {
        let mut ui = UiConfig {
            date_locale: "english".into(),
            date_verbose_format: "dddd D MMMM YYYY".into(),
            ..UiConfig::default()
        };
        let _g = gitpoulpi_i18n::override_lang(Lang::Ru);
        assert_eq!(
            DateSettings::from_config(&ui).format_verbose(TS),
            "Tuesday 5 March 2024"
        );
        ui.date_locale = "system".into();
        assert_eq!(
            DateSettings::from_config(&ui).format_verbose(TS),
            "вторник 5 марта 2024"
        );
    }

    #[test]
    fn the_patterns_default_as_documented() {
        let d = DateSettings::default();
        let _g = gitpoulpi_i18n::override_lang(Lang::En);
        assert_eq!(d.format_short(TS), "2024-03-05");
        assert_eq!(d.format_verbose(TS), "5 March 2024 12:34");
        assert!(!d.format_time(TS).is_empty());
    }

    #[test]
    fn civil_from_days_knows_the_epoch_and_a_leap_day() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    }

    /// The Log renders a date per visible row per frame: a parsed pattern is
    /// pure catalog reads and one small `String` per call.
    #[test]
    fn rendering_dates_stays_cheap() {
        let pattern = DatePattern::parse("D MMMM YYYY HH:mm");
        let _g = gitpoulpi_i18n::override_lang(Lang::Ru);
        let started = std::time::Instant::now();
        let mut total = 0usize;
        for i in 0..50_000i64 {
            total += pattern.render(TS + i * 3600, false).len();
        }
        assert!(total > 0);
        // ~1 microsecond each in release; the budget is two orders looser
        // so a loaded debug build never flakes.
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "50k renders took {:?}",
            started.elapsed()
        );
    }
}
