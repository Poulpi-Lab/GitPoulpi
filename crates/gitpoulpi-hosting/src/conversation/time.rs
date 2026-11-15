//! Just enough ISO 8601 for "was this comment edited".

/// Seconds since the epoch of an ISO 8601 timestamp (`2024-05-01T10:00:00Z`,
/// with optional fraction and `+hh:mm` offset). `None` when it is not one.
pub(crate) fn iso_secs(text: &str) -> Option<i64> {
    let t = text.trim();
    let b = t.as_bytes();
    if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || !matches!(b[10], b'T' | b' ') {
        return None;
    }
    let num = |from: usize, to: usize| t.get(from..to)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, s) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    // Days from civil (Howard Hinnant).
    let y = if mo <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if mo > 2 { mo - 3 } else { mo + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let mut rest = &t[19..];
    if let Some(stripped) = rest.strip_prefix('.') {
        rest = stripped.trim_start_matches(|c: char| c.is_ascii_digit());
    }
    let offset = match rest.as_bytes().first() {
        Some(b'+' | b'-') => {
            let sign = if rest.starts_with('-') { -1 } else { 1 };
            let oh = rest.get(1..3)?.parse::<i64>().ok()?;
            let om = rest
                .get(4..6)
                .or_else(|| rest.get(3..5))
                .and_then(|m| m.parse::<i64>().ok())
                .unwrap_or(0);
            sign * (oh * 3600 + om * 60)
        }
        _ => 0,
    };
    Some(days * 86_400 + h * 3600 + mi * 60 + s - offset)
}

/// Hosts keep no "edited" flag on a plain comment: it counts as edited when
/// it was updated more than two seconds after it was created.
#[allow(dead_code)] // unused by a Gerrit-only build.
pub(crate) fn is_edited(created_at: &str, updated_at: Option<&str>) -> bool {
    match (iso_secs(created_at), updated_at.and_then(iso_secs)) {
        (Some(c), Some(u)) => u > c + 2,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_parse_with_fraction_and_offset() {
        assert_eq!(iso_secs("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(iso_secs("2024-05-01T10:00:00Z"), Some(1_714_557_600));
        assert_eq!(
            iso_secs("2024-05-01T12:00:00.250+02:00"),
            Some(1_714_557_600)
        );
        assert_eq!(iso_secs("nope"), None);
    }

    #[test]
    fn edited_needs_more_than_two_seconds() {
        let c = "2024-05-01T10:00:00Z";
        assert!(!is_edited(c, Some("2024-05-01T10:00:02Z")));
        assert!(is_edited(c, Some("2024-05-01T10:00:03Z")));
        assert!(!is_edited(c, None));
    }
}
