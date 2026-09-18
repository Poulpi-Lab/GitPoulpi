use super::*;

/// Dates are what a reader scans a blame gutter by, so a wrong
/// conversion is worse than none. Pinned against known instants,
/// including a leap day and a pre-epoch commit.
#[test]
fn dates_convert_correctly_including_leap_days_and_pre_epoch() {
    // See `live_settings::DATE_PATTERNS_TEST_LOCK`'s doc comment -- this
    // test assumes the default `YYYY-MM-DD` pattern, and races with
    // `ymd_reads_the_configured_short_date_pattern` (below) and
    // `preferences`'s own date-pattern test without this guard.
    // Reproduced: flaky under ordinary parallel `cargo test -p
    // gitpoulpi-tui --lib` before the lock existed.
    let _guard = live_settings::DATE_PATTERNS_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    assert_eq!(ymd(0), "1970-01-01");
    assert_eq!(ymd(1_700_000_000), "2023-11-14");
    assert_eq!(ymd(951_782_400), "2000-02-29");
    assert_eq!(ymd(-86_400), "1969-12-31");
}

/// `ymd` reads `ui.date_format` (`live_settings::date_short_pattern`),
/// not a hard-coded `YYYY-MM-DD` - and a pattern shorter than the
/// gutter's fixed 10-character column is padded, never left ragged.
/// Falsified: called `live_settings::format_date` directly instead of
/// going through the published pattern, watched this fail with
/// `"1970-01-01"` instead of `"01/01/70"`, restored.
#[test]
fn ymd_reads_the_configured_short_date_pattern() {
    // See `live_settings::DATE_PATTERNS_TEST_LOCK`'s doc comment.
    let _guard = live_settings::DATE_PATTERNS_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let default_ui = gitpoulpi_config::UiConfig::default();
    let mut custom = default_ui.clone();
    custom.date_format = "DD/MM/YY".into();
    live_settings::install_date_patterns(&custom);

    assert_eq!(ymd(0), "01/01/70  "); // 8 chars, padded to DATE_WIDTH (10)

    live_settings::install_date_patterns(&default_ui);
    assert_eq!(ymd(0), "1970-01-01");
}
