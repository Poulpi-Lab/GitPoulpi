use super::*;

/// The month-name tokens are why the scan has to be longest-first:
/// matching `MM` before `MMMM` would turn `MMMM` into two numeric
/// months (`0303`) instead of `March`.
#[test]
fn a_pattern_renders_longest_tokens_first() {
    // 2026-03-22 14:05:07 UTC
    let ts = 1_774_188_307;
    assert_eq!(format_date("YYYY-MM-DD", ts), "2026-03-22");
    assert_eq!(format_date("DD.MM.YYYY", ts), "22.03.2026");
    assert_eq!(format_date("D MMMM YYYY", ts), "22 March 2026");
    assert_eq!(format_date("MMM D, YYYY", ts), "Mar 22, 2026");
    assert_eq!(format_date("DD.MM.YYYY @ HH:mm", ts), "22.03.2026 @ 14:05");
    assert_eq!(format_date("D MMMM YYYY HH:mm", ts), "22 March 2026 14:05");
}

#[test]
fn separators_and_non_tokens_pass_through_verbatim() {
    let ts = 0;
    assert_eq!(format_date("DD.MM.YYYY @ HH:mm", ts), "01.01.1970 @ 00:00");
    assert_eq!(format_date("at HH:mm:ss", ts), "at 00:00:00");
}

/// The pattern the Log's Date column renders with must be the one
/// the config holds, or the field in this screen is decoration.
#[test]
fn installing_a_pattern_changes_what_the_log_column_will_render() {
    // See `DATE_PATTERNS_TEST_LOCK`'s doc comment -- this test and
    // `file_view.rs`'s date tests share one process-wide static and
    // race without it.
    let _guard = DATE_PATTERNS_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    install_date_time_format("DD.MM.YYYY @ HH:mm");
    assert_eq!(date_time_pattern(), "DD.MM.YYYY @ HH:mm");
    assert_eq!(format_date(&date_time_pattern(), 0), "01.01.1970 @ 00:00");
    install_date_time_format(&gitpoulpi_config::UiConfig::default().date_time_format);
}
