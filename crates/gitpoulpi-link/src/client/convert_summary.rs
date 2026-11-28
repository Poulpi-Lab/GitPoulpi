//! The status line after a bulk conversion: "converted 12 files, skipped 3
//! (2 binary, 1 cannot be encoded ...)". Pure, so the wording is tested
//! without a daemon.

use gitpoulpi_core::textconv::{ConvertReport, ConvertSkip};
use gitpoulpi_i18n::msg::link as l;

pub(super) fn text(report: &ConvertReport) -> String {
    let converted = report.converted.len() as u64;
    let skipped = report.skipped.len() as u64;
    if skipped == 0 {
        return if converted == 0 {
            l::CONVERT_NOTHING_TO_DO.get().to_string()
        } else {
            l::converted(converted)
        };
    }
    let why = why(report);
    if converted == 0 {
        l::convert_nothing_skipped(skipped, why)
    } else {
        l::converted_skipped(converted, skipped, why)
    }
}

/// "2 binary, 1 cannot be encoded: a.txt line 3, 'x'", reasons in a fixed
/// order.
fn why(report: &ConvertReport) -> String {
    let count =
        |f: fn(&ConvertSkip) -> bool| report.skipped.iter().filter(|(_, s)| f(s)).count() as u64;
    let mut parts = Vec::new();
    let binary = count(|s| matches!(s, ConvertSkip::Binary));
    if binary > 0 {
        parts.push(l::skip_binary(binary));
    }
    let unencodable = report.skipped.iter().find_map(|(path, s)| match s {
        ConvertSkip::Unencodable { line, ch, .. } => Some((path, *line, *ch)),
        _ => None,
    });
    if let Some((path, line, ch)) = unencodable {
        let n = count(|s| matches!(s, ConvertSkip::Unencodable { .. }));
        parts.push(l::skip_unencodable(n, path, line, ch));
    }
    let undecodable = count(|s| matches!(s, ConvertSkip::Undecodable));
    if undecodable > 0 {
        parts.push(l::skip_undecodable(undecodable));
    }
    let large = count(|s| matches!(s, ConvertSkip::TooLarge));
    if large > 0 {
        parts.push(l::skip_too_large(large));
    }
    let missing = count(|s| matches!(s, ConvertSkip::Missing));
    if missing > 0 {
        parts.push(l::skip_missing(missing));
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(converted: usize, skipped: Vec<(&str, ConvertSkip)>) -> ConvertReport {
        ConvertReport {
            converted: (0..converted).map(|i| format!("f{i}")).collect(),
            skipped: skipped
                .into_iter()
                .map(|(p, s)| (p.to_string(), s))
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn the_summary_counts_converted_and_skipped_by_reason() {
        let r = report(
            12,
            vec![
                ("a.bin", ConvertSkip::Binary),
                ("b.bin", ConvertSkip::Binary),
                (
                    "c.txt",
                    ConvertSkip::Unencodable {
                        count: 2,
                        line: 3,
                        ch: '\u{4e2d}',
                    },
                ),
            ],
        );
        assert_eq!(
            text(&r),
            "converted 12 files, skipped 3 (2 binary, 1 cannot be encoded: c.txt line 3, '\u{4e2d}')"
        );
    }

    #[test]
    fn nothing_converted_says_so() {
        assert_eq!(
            text(&report(0, vec![])),
            "nothing to convert: every file is already in that form"
        );
        assert_eq!(text(&report(1, vec![])), "converted 1 file");
        assert_eq!(
            text(&report(0, vec![("a", ConvertSkip::Missing)])),
            "nothing converted, skipped 1 (1 not in the working tree)"
        );
    }
}
