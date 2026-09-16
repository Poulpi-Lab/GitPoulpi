//! The message-encoding fix (slice S24): re-encode what a commit declares
//! (`encoding ISO-8859-1`) or what was once mis-decoded (UTF-8 bytes read
//! as Latin-1 / Windows-1252, "RÃ©Ã©criture") as plain UTF-8. Pure
//! functions over bytes; the engine decides which commits and drops the
//! `encoding` header.
//!
//! Conservative by construction: a text is repaired only when re-encoding
//! the result the way it was mangled gives back the original exactly, and
//! anything undecidable is left alone and counted.

use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};

/// What to do with one text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fix {
    /// Nothing to repair (valid UTF-8, no declared encoding, no mojibake).
    Keep,
    /// The text as UTF-8; `changed` is `false` when only the `encoding`
    /// header goes (the bytes were already this UTF-8).
    Text { text: String, changed: bool },
    /// Cannot be decided: unknown or wrong encoding, or text that looks
    /// mangled but does not round-trip. Left as it is.
    Left,
}

#[derive(Clone, Copy)]
enum Declared {
    Latin1,
    Utf8,
    Other(&'static Encoding),
}

/// The encoding a header value names, as git understands it: strict
/// ISO-8859-1 is Latin-1 (not WHATWG's Windows-1252), the rest goes through
/// `encoding_rs`. `None` for an unknown label and for the encodings that
/// are not ASCII-compatible (a commit header cannot be in them).
fn resolve(label: &[u8]) -> Option<Declared> {
    let text = std::str::from_utf8(label).ok()?.trim().to_ascii_lowercase();
    match text.as_str() {
        "iso-8859-1" | "iso8859-1" | "iso_8859-1" | "latin1" | "latin-1" | "l1" | "iso88591" => {
            return Some(Declared::Latin1)
        }
        "utf-8" | "utf8" => return Some(Declared::Utf8),
        _ => {}
    }
    let enc = Encoding::for_label(text.as_bytes())?;
    if enc == UTF_8 {
        return Some(Declared::Utf8);
    }
    enc.is_ascii_compatible().then_some(Declared::Other(enc))
}

/// `bytes` read in the encoding `label` names, as UTF-8 text; `None` when
/// the label is unknown or the bytes are not valid in it.
pub fn decode_declared(label: &[u8], bytes: &[u8]) -> Option<String> {
    match resolve(label)? {
        Declared::Latin1 => Some(latin1_text(bytes)),
        Declared::Utf8 => std::str::from_utf8(bytes).ok().map(str::to_string),
        Declared::Other(enc) => enc
            .decode_without_bom_handling_and_without_replacement(bytes)
            .map(|c| c.into_owned()),
    }
}

/// `true` when the label names UTF-8 (the header is then redundant).
pub fn declares_utf8(label: &[u8]) -> bool {
    matches!(resolve(label), Some(Declared::Utf8))
}

/// Text that is UTF-8 bytes once read as Latin-1 or Windows-1252, repaired;
/// `None` when it is not (or not provably) that. The round trip must be
/// exact and the result must hold no control character or U+FFFD.
pub fn repair_mojibake(text: &str) -> Option<String> {
    if text.is_ascii() {
        return None;
    }
    let candidates = [latin1_bytes(text), windows_1252_bytes(text)];
    for bytes in candidates.into_iter().flatten() {
        let Ok(fixed) = String::from_utf8(bytes) else {
            continue;
        };
        if fixed == text
            || fixed
                .chars()
                .any(|c| c == '\u{FFFD}' || (c.is_control() && !matches!(c, '\n' | '\t' | '\r')))
        {
            continue;
        }
        // Mangling the result the same way must give the text back.
        let back = latin1_text(fixed.as_bytes()) == text
            || WINDOWS_1252.decode_without_bom_handling(fixed.as_bytes()).0 == text;
        if back {
            return Some(fixed);
        }
    }
    None
}

fn latin1_bytes(text: &str) -> Option<Vec<u8>> {
    text.chars()
        .map(|c| u8::try_from(u32::from(c)).ok())
        .collect()
}

fn latin1_text(bytes: &[u8]) -> String {
    bytes.iter().map(|b| char::from(*b)).collect()
}

fn windows_1252_bytes(text: &str) -> Option<Vec<u8>> {
    let (bytes, _, had_errors) = WINDOWS_1252.encode(text);
    (!had_errors).then(|| bytes.into_owned())
}

/// `true` for text that reads like mangled UTF-8 (a lead letter such as
/// "Ã" or "Â" in non-ASCII text): worth counting as "left" when it cannot
/// be repaired.
fn looks_mangled(text: &str) -> bool {
    text.contains(['\u{00C2}', '\u{00C3}', '\u{00E2}'])
}

/// One piece of text of a commit (message or a name) given its bytes and
/// the commit's `encoding` header, if any.
pub fn fix(bytes: &[u8], declared: Option<&[u8]>) -> Fix {
    let repaired = |text: &str| {
        repair_mojibake(text).map(|text| Fix::Text {
            text,
            changed: true,
        })
    };
    let Some(label) = declared else {
        return match std::str::from_utf8(bytes) {
            Ok(text) => repaired(text).unwrap_or(if looks_mangled(text) {
                Fix::Left
            } else {
                Fix::Keep
            }),
            Err(_) => Fix::Left,
        };
    };
    if declares_utf8(label) {
        let Ok(text) = std::str::from_utf8(bytes) else {
            return Fix::Left;
        };
        return repaired(text).unwrap_or(Fix::Text {
            text: text.to_string(),
            changed: false,
        });
    }
    match decode_declared(label, bytes) {
        Some(text) => Fix::Text {
            text,
            changed: true,
        },
        None => Fix::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(f: Fix) -> String {
        match f {
            Fix::Text { text, .. } => text,
            other => panic!("not a text: {other:?}"),
        }
    }

    #[test]
    fn a_declared_latin1_message_is_decoded() {
        assert_eq!(
            text(fix(b"caf\xe9 au lait\n", Some(b"ISO-8859-1"))),
            "caf\u{e9} au lait\n"
        );
        // Latin-1 keeps the C1 controls Windows-1252 would turn into letters.
        assert_eq!(text(fix(b"\x80", Some(b"latin1"))), "\u{80}");
        assert_eq!(text(fix(b"\x80", Some(b"windows-1252"))), "\u{20ac}");
    }

    #[test]
    fn other_declared_encodings_go_through_encoding_rs() {
        // "日本" in Shift_JIS.
        assert_eq!(
            text(fix(b"\x93\xfa\x96\x7b", Some(b"Shift_JIS"))),
            "\u{65e5}\u{672c}"
        );
        assert_eq!(fix(b"\x93", Some(b"Shift_JIS")), Fix::Left);
        assert_eq!(fix(b"x", Some(b"no-such-encoding")), Fix::Left);
    }

    #[test]
    fn mojibake_is_repaired_when_the_round_trip_is_exact() {
        assert_eq!(
            text(fix("R\u{c3}\u{a9}\u{c3}\u{a9}criture\n".as_bytes(), None)),
            "R\u{e9}\u{e9}criture\n"
        );
        // Windows-1252 flavour: a right single quote is E2 80 99.
        assert_eq!(
            text(fix("it\u{e2}\u{20ac}\u{2122}s".as_bytes(), None)),
            "it\u{2019}s"
        );
        // Latin-1 flavour of the same quote: the C1 controls stay.
        assert_eq!(
            text(fix("it\u{e2}\u{80}\u{99}s".as_bytes(), None)),
            "it\u{2019}s"
        );
    }

    #[test]
    fn a_text_that_is_not_provably_mojibake_is_left() {
        // A real e-acute next to the mangled one: no single mangling.
        assert_eq!(fix("\u{c3}\u{a9} et \u{e9}".as_bytes(), None), Fix::Left);
        // Letters outside Windows-1252 next to it.
        assert_eq!(
            fix("\u{c3}\u{a9} \u{65e5}\u{672c}".as_bytes(), None),
            Fix::Left
        );
        // A lone accented letter is just French.
        assert_eq!(fix("caf\u{e9}".as_bytes(), None), Fix::Keep);
        assert_eq!(fix(b"plain ascii", None), Fix::Keep);
        // Invalid UTF-8 and no header: undecidable.
        assert_eq!(fix(b"caf\xe9", None), Fix::Left);
    }

    #[test]
    fn a_declared_utf8_header_only_goes() {
        assert_eq!(
            fix("caf\u{e9}".as_bytes(), Some(b"UTF-8")),
            Fix::Text {
                text: "caf\u{e9}".into(),
                changed: false
            }
        );
    }
}
