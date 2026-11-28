//! Calendar helpers: UTC civil dates from Unix timestamps, the way the terminal UI formats them.

use gitpoulpi_i18n::msg;

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// (year, month, day, hour, minute), UTC.
pub fn civil(ts: i64) -> (i64, u32, u32, u32, u32) {
    let (y, m, d) = civil_from_days(ts.div_euclid(86_400));
    let secs = ts.rem_euclid(86_400);
    (y, m, d, (secs / 3600) as u32, ((secs % 3600) / 60) as u32)
}

/// `YYYY-MM-DD HH:MM`, UTC.
pub fn iso_date(ts: i64) -> String {
    let (y, m, d, hh, mm) = civil(ts);
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}")
}

/// "3 days ago" in the current UI language for recent commits, the ISO date after a year.
pub fn relative_date(ts: i64, now: i64) -> String {
    let d = (now - ts).max(0) as u64;
    match d {
        0..=59 => msg::reltime::JUST_NOW.get().to_string(),
        60..=3599 => msg::reltime::minutes_ago(d / 60),
        3600..=86_399 => msg::reltime::hours_ago(d / 3600),
        86_400..=604_799 => msg::reltime::days_ago(d / 86_400),
        604_800..=2_591_999 => msg::reltime::weeks_ago(d / 604_800),
        2_592_000..=31_535_999 => msg::reltime::months_ago(d / 2_592_000),
        _ => iso_date(ts)[..10].to_string(),
    }
}

pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970() {
        assert_eq!(iso_date(0), "1970-01-01 00:00");
    }

    #[test]
    fn leap_day_and_time_of_day() {
        // 2024-02-29 13:45:00 UTC
        assert_eq!(civil(1_709_214_300), (2024, 2, 29, 13, 45));
    }

    #[test]
    fn before_the_epoch() {
        assert_eq!(iso_date(-86_400), "1969-12-31 00:00");
    }
}
