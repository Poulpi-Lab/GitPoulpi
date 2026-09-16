//! What the editor knows about a file's bytes beyond "UTF-8 with `\n`":
//! its text encoding ([`Charset`]), its kind of line break ([`Eol`]) and
//! the indentation conversion. Pure functions and small types; the
//! document (`editor_buffer::Doc`) owns the state and the undo.
//!
//! # Encodings
//!
//! The document always holds Unicode text. A [`Charset`] says how that text
//! is turned into bytes on a save and how bytes become text on a load. The
//! list is the short one a developer meets (UTF-8 with or without BOM,
//! UTF-16 with BOM, the Western / Central / Cyrillic single-byte pages, and
//! the East Asian multi-byte ones). `encoding_rs` does the legacy pages;
//! UTF-16 is done by hand because `encoding_rs` only *decodes* it, and so
//! is ISO-8859-1, because the web's "iso-8859-1" label means Windows-1252
//! there (it fills 0x80-0x9F with letters, a real Latin-1 file has control
//! characters).
//!
//! Decoding never substitutes: a byte sequence the encoding cannot read is
//! an error, not U+FFFD, so a file is never silently damaged. Encoding is
//! the same: a character the page cannot hold is reported (count and first
//! line), never written as `?`.

use encoding_rs::Encoding;
use serde::{Deserialize, Serialize};

/// A text encoding the editor reads and writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Charset {
    Utf8,
    /// UTF-8 with a byte order mark.
    Utf8Bom,
    /// UTF-16 little endian, written with its BOM.
    Utf16Le,
    /// UTF-16 big endian, written with its BOM.
    Utf16Be,
    Windows1252,
    Iso8859_1,
    Iso8859_15,
    Windows1250,
    Windows1251,
    Koi8R,
    ShiftJis,
    EucJp,
    Gbk,
    Big5,
    EucKr,
}

/// Every encoding the menus offer, in the order they are listed.
pub const ALL: &[Charset] = &[
    Charset::Utf8,
    Charset::Utf8Bom,
    Charset::Utf16Le,
    Charset::Utf16Be,
    Charset::Windows1252,
    Charset::Iso8859_1,
    Charset::Iso8859_15,
    Charset::Windows1250,
    Charset::Windows1251,
    Charset::Koi8R,
    Charset::ShiftJis,
    Charset::EucJp,
    Charset::Gbk,
    Charset::Big5,
    Charset::EucKr,
];

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
const UTF16_LE_BOM: &[u8] = b"\xFF\xFE";
const UTF16_BE_BOM: &[u8] = b"\xFE\xFF";

/// Why bytes could not be read in an encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// Not valid UTF-8; the offset of the first bad byte.
    Utf8 { at: usize },
    /// Not valid in this (other) encoding.
    Invalid,
}

/// What an encoding cannot hold: how many characters, and the first one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unencodable {
    pub count: usize,
    /// 1-based line of the first one.
    pub line: usize,
    pub ch: char,
}

impl Charset {
    /// The name shown on the chip and in the menus (never translated).
    pub fn name(self) -> &'static str {
        match self {
            Charset::Utf8 => "UTF-8",
            Charset::Utf8Bom => "UTF-8 BOM",
            Charset::Utf16Le => "UTF-16 LE",
            Charset::Utf16Be => "UTF-16 BE",
            Charset::Windows1252 => "Windows-1252",
            Charset::Iso8859_1 => "ISO-8859-1",
            Charset::Iso8859_15 => "ISO-8859-15",
            Charset::Windows1250 => "Windows-1250",
            Charset::Windows1251 => "Windows-1251",
            Charset::Koi8R => "KOI8-R",
            Charset::ShiftJis => "Shift_JIS",
            Charset::EucJp => "EUC-JP",
            Charset::Gbk => "GBK",
            Charset::Big5 => "Big5",
            Charset::EucKr => "EUC-KR",
        }
    }

    /// The charset named `name` (as [`Charset::name`] spells it).
    pub fn from_name(name: &str) -> Option<Charset> {
        ALL.iter().copied().find(|c| c.name() == name)
    }

    fn legacy(self) -> Option<&'static Encoding> {
        Some(match self {
            Charset::Windows1252 => encoding_rs::WINDOWS_1252,
            Charset::Iso8859_15 => encoding_rs::ISO_8859_15,
            Charset::Windows1250 => encoding_rs::WINDOWS_1250,
            Charset::Windows1251 => encoding_rs::WINDOWS_1251,
            Charset::Koi8R => encoding_rs::KOI8_R,
            Charset::ShiftJis => encoding_rs::SHIFT_JIS,
            Charset::EucJp => encoding_rs::EUC_JP,
            Charset::Gbk => encoding_rs::GBK,
            Charset::Big5 => encoding_rs::BIG5,
            Charset::EucKr => encoding_rs::EUC_KR,
            _ => return None,
        })
    }

    pub fn is_utf16(self) -> bool {
        matches!(self, Charset::Utf16Le | Charset::Utf16Be)
    }

    /// The encoding a file's own first bytes announce, if any.
    pub fn from_bom(bytes: &[u8]) -> Option<Charset> {
        if bytes.starts_with(UTF8_BOM) {
            Some(Charset::Utf8Bom)
        } else if bytes.starts_with(UTF16_LE_BOM) {
            Some(Charset::Utf16Le)
        } else if bytes.starts_with(UTF16_BE_BOM) {
            Some(Charset::Utf16Be)
        } else {
            None
        }
    }

    /// Reads `bytes` as this encoding. A BOM of the matching family is
    /// consumed; the returned charset is `self`, except that the two UTF-8
    /// variants are told apart by whether the BOM was there.
    pub fn decode(self, bytes: &[u8]) -> Result<(String, Charset), DecodeError> {
        match self {
            Charset::Utf8 | Charset::Utf8Bom => {
                let (bom, body) = match bytes.strip_prefix(UTF8_BOM) {
                    Some(rest) => (true, rest),
                    None => (false, bytes),
                };
                let s = std::str::from_utf8(body).map_err(|e| DecodeError::Utf8 {
                    at: e.valid_up_to() + if bom { UTF8_BOM.len() } else { 0 },
                })?;
                let kind = if bom { Charset::Utf8Bom } else { Charset::Utf8 };
                Ok((s.to_string(), kind))
            }
            Charset::Utf16Le | Charset::Utf16Be => {
                let (bom, enc) = if self == Charset::Utf16Le {
                    (UTF16_LE_BOM, encoding_rs::UTF_16LE)
                } else {
                    (UTF16_BE_BOM, encoding_rs::UTF_16BE)
                };
                let body = bytes.strip_prefix(bom).unwrap_or(bytes);
                let s = enc
                    .decode_without_bom_handling_and_without_replacement(body)
                    .ok_or(DecodeError::Invalid)?;
                Ok((s.into_owned(), self))
            }
            Charset::Iso8859_1 => Ok((bytes.iter().map(|&b| b as char).collect(), self)),
            _ => {
                let enc = self.legacy().expect("a legacy charset");
                let s = enc
                    .decode_without_bom_handling_and_without_replacement(bytes)
                    .ok_or(DecodeError::Invalid)?;
                Ok((s.into_owned(), self))
            }
        }
    }

    /// The bytes of `text` in this encoding (with the BOM the variant
    /// carries), or `None` when some character has no code in it.
    pub fn encode(self, text: &str) -> Option<Vec<u8>> {
        match self {
            Charset::Utf8 => Some(text.as_bytes().to_vec()),
            Charset::Utf8Bom => {
                let mut out = Vec::with_capacity(text.len() + 3);
                out.extend_from_slice(UTF8_BOM);
                out.extend_from_slice(text.as_bytes());
                Some(out)
            }
            Charset::Utf16Le | Charset::Utf16Be => {
                let le = self == Charset::Utf16Le;
                let mut out = Vec::with_capacity(text.len() * 2 + 2);
                out.extend_from_slice(if le { UTF16_LE_BOM } else { UTF16_BE_BOM });
                for unit in text.encode_utf16() {
                    out.extend_from_slice(&if le {
                        unit.to_le_bytes()
                    } else {
                        unit.to_be_bytes()
                    });
                }
                Some(out)
            }
            Charset::Iso8859_1 => text
                .chars()
                .map(|c| u8::try_from(u32::from(c)).ok())
                .collect(),
            _ => {
                let enc = self.legacy().expect("a legacy charset");
                let (bytes, _, unmappable) = enc.encode(text);
                (!unmappable).then(|| bytes.into_owned())
            }
        }
    }

    /// Whether `c` has a code in this encoding.
    fn can_hold(self, c: char) -> bool {
        match self {
            Charset::Utf8 | Charset::Utf8Bom | Charset::Utf16Le | Charset::Utf16Be => true,
            Charset::Iso8859_1 => u32::from(c) <= 0xFF,
            _ => {
                let enc = self.legacy().expect("a legacy charset");
                !enc.encode(c.encode_utf8(&mut [0; 4])).2
            }
        }
    }

    /// The characters of `text` this encoding cannot hold: how many and
    /// where the first is; `None` when it holds them all.
    pub fn unencodable(self, text: &str) -> Option<Unencodable> {
        if self.encode(text).is_some() {
            return None;
        }
        let mut count = 0;
        let mut first = None;
        let mut line = 1;
        for c in text.chars() {
            if c == '\n' {
                line += 1;
            } else if !self.can_hold(c) {
                count += 1;
                first.get_or_insert((line, c));
            }
        }
        first.map(|(line, ch)| Unencodable { count, line, ch })
    }
}

/// The kind of line break a file uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Eol {
    Lf,
    Crlf,
    Cr,
    /// Several kinds in one file (or a stray `\r`): the document holds the
    /// text exactly as read until it is converted.
    Mixed,
}

impl Eol {
    /// `LF` / `CRLF` / `CR`; `None` for [`Eol::Mixed`] (a translated word).
    pub fn code(self) -> Option<&'static str> {
        match self {
            Eol::Lf => Some("LF"),
            Eol::Crlf => Some("CRLF"),
            Eol::Cr => Some("CR"),
            Eol::Mixed => None,
        }
    }
}

/// The line-break kind of `text` as read: the one kind it uses, `Lf` when
/// it has no line break, [`Eol::Mixed`] when it has several.
pub fn detect_eol(text: &str) -> Eol {
    let b = text.as_bytes();
    let (mut lf, mut crlf, mut cr) = (0usize, 0usize, 0usize);
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\r' if b.get(i + 1) == Some(&b'\n') => {
                crlf += 1;
                i += 1;
            }
            b'\r' => cr += 1,
            b'\n' => lf += 1,
            _ => {}
        }
        i += 1;
    }
    match (lf > 0, crlf > 0, cr > 0) {
        (_, false, false) => Eol::Lf,
        (false, true, false) => Eol::Crlf,
        (false, false, true) => Eol::Cr,
        _ => Eol::Mixed,
    }
}

/// `text` with every line break (CRLF, CR, LF) as `\n`.
pub fn normalize_eol(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// `text` (with `\n` breaks) with its leading whitespace rewritten as
/// spaces or as tabs (spaces for the remainder), measured with tab stops
/// every `tab` columns. Returns the new text and how many lines changed.
pub fn convert_indent(text: &str, to_tabs: bool, tab: usize) -> (String, usize) {
    let tab = tab.max(1);
    let mut out = String::with_capacity(text.len());
    let mut changed = 0;
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let lead = line
            .bytes()
            .take_while(|b| *b == b' ' || *b == b'\t')
            .count();
        let mut col = 0;
        for b in line[..lead].bytes() {
            col += if b == b'\t' { tab - col % tab } else { 1 };
        }
        let indent = if to_tabs {
            format!("{}{}", "\t".repeat(col / tab), " ".repeat(col % tab))
        } else {
            " ".repeat(col)
        };
        if indent != line[..lead] {
            changed += 1;
        }
        out.push_str(&indent);
        out.push_str(&line[lead..]);
    }
    (out, changed)
}

// ---------------------------------------------------------------- files

/// A conversion applied to the bytes of files on disk (the right pane's
/// "Convert" menu, `GitBackend::convert_files`). The editor's own
/// conversions are the same functions on the open document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvertOp {
    /// Write the text in this encoding.
    Encoding(Charset),
    /// Every line break becomes this one (`Lf`, `Crlf` or `Cr`).
    Eol(Eol),
    /// Leading whitespace as tabs or as spaces; `size` is the tab stop
    /// every existing tab is measured with (and the width of a new tab).
    Indent { tabs: bool, size: usize },
}

/// Why a file was left alone by a conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvertSkip {
    /// Not text (a NUL byte outside UTF-16).
    Binary,
    /// Not a file in the working tree (missing, or a folder).
    Missing,
    TooLarge,
    /// Its bytes are not valid in the encoding they announce.
    Undecodable,
    /// The target encoding cannot hold some of its characters.
    Unencodable {
        count: usize,
        line: usize,
        ch: char,
    },
}

/// What a conversion does to one file's bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertOutcome {
    /// Already in the requested form: nothing to write.
    Unchanged,
    Changed(Vec<u8>),
    Skipped(ConvertSkip),
}

/// The largest file a conversion reads (a text file, not a data blob).
pub const MAX_CONVERT_BYTES: usize = 64 * 1024 * 1024;

/// Applies `op` to a file's bytes. The source encoding is the one a BOM
/// announces, else UTF-8, else Windows-1252 (every byte of which means
/// something): the editor's own rule, so the two read a file alike.
/// Nothing is ever substituted: a character the target cannot hold skips
/// the file (reported), it is not written as `?`.
pub fn convert_bytes(bytes: &[u8], op: ConvertOp) -> ConvertOutcome {
    if bytes.len() > MAX_CONVERT_BYTES {
        return ConvertOutcome::Skipped(ConvertSkip::TooLarge);
    }
    let announced = Charset::from_bom(bytes);
    if !announced.is_some_and(Charset::is_utf16) && bytes.contains(&0) {
        return ConvertOutcome::Skipped(ConvertSkip::Binary);
    }
    let decoded = match announced {
        Some(cs) => cs.decode(bytes),
        None => Charset::Utf8
            .decode(bytes)
            .or_else(|_| Charset::Windows1252.decode(bytes)),
    };
    let Ok((text, source)) = decoded else {
        return ConvertOutcome::Skipped(ConvertSkip::Undecodable);
    };
    let eol = detect_eol(&text);
    let (target, out_text) = match op {
        ConvertOp::Encoding(to) => {
            if to == source {
                return ConvertOutcome::Unchanged;
            }
            (to, text)
        }
        ConvertOp::Eol(to) => {
            let plain = normalize_eol(&text);
            let out = match to {
                Eol::Crlf => plain.replace('\n', "\r\n"),
                Eol::Cr => plain.replace('\n', "\r"),
                Eol::Lf | Eol::Mixed => plain,
            };
            (source, out)
        }
        ConvertOp::Indent { tabs, size } => {
            let out = if eol == Eol::Cr {
                let (converted, _) = convert_indent(&normalize_eol(&text), tabs, size);
                converted.replace('\n', "\r")
            } else {
                convert_indent(&text, tabs, size).0
            };
            (source, out)
        }
    };
    if let Some(why) = target.unencodable(&out_text) {
        return ConvertOutcome::Skipped(ConvertSkip::Unencodable {
            count: why.count,
            line: why.line,
            ch: why.ch,
        });
    }
    match target.encode(&out_text) {
        Some(out) if out == bytes => ConvertOutcome::Unchanged,
        Some(out) => ConvertOutcome::Changed(out),
        None => ConvertOutcome::Skipped(ConvertSkip::Undecodable),
    }
}

/// What `GitBackend::convert_files` did, path by path (repo-relative, `/`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertReport {
    pub converted: Vec<String>,
    pub unchanged: Vec<String>,
    pub skipped: Vec<(String, ConvertSkip)>,
    /// Files that could not be read or written: path and the system's reason.
    pub failed: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eol_kinds_are_told_apart() {
        assert_eq!(detect_eol(""), Eol::Lf);
        assert_eq!(detect_eol("a"), Eol::Lf);
        assert_eq!(detect_eol("a\nb\n"), Eol::Lf);
        assert_eq!(detect_eol("a\r\nb\r\n"), Eol::Crlf);
        assert_eq!(detect_eol("a\rb\r"), Eol::Cr);
        assert_eq!(detect_eol("a\r\nb\n"), Eol::Mixed);
        assert_eq!(detect_eol("a\rb\n"), Eol::Mixed);
        assert_eq!(detect_eol("a\r\r\nb"), Eol::Mixed);
    }

    #[test]
    fn indentation_converts_by_columns() {
        let (t, n) = convert_indent("\tx\n  \ty\nz\n", false, 4);
        assert_eq!((t.as_str(), n), ("    x\n    y\nz\n", 2));
        let (t, n) = convert_indent("        x\n      y\n z\n", true, 4);
        assert_eq!((t.as_str(), n), ("\t\tx\n\t  y\n z\n", 2));
        let (t, n) = convert_indent("a\tb\n", false, 4);
        assert_eq!((t.as_str(), n), ("a\tb\n", 0), "only leading whitespace");
    }

    fn changed(bytes: &[u8], op: ConvertOp) -> Vec<u8> {
        match convert_bytes(bytes, op) {
            ConvertOutcome::Changed(b) => b,
            other => panic!("expected a change, got {other:?}"),
        }
    }

    #[test]
    fn line_breaks_convert_both_ways_and_from_mixed() {
        assert_eq!(changed(b"a\nb\n", ConvertOp::Eol(Eol::Crlf)), b"a\r\nb\r\n");
        assert_eq!(changed(b"a\r\nb\r\n", ConvertOp::Eol(Eol::Lf)), b"a\nb\n");
        assert_eq!(changed(b"a\r\nb\n", ConvertOp::Eol(Eol::Lf)), b"a\nb\n");
        assert_eq!(changed(b"a\nb\n", ConvertOp::Eol(Eol::Cr)), b"a\rb\r");
        assert_eq!(
            convert_bytes(b"a\nb\n", ConvertOp::Eol(Eol::Lf)),
            ConvertOutcome::Unchanged
        );
    }

    #[test]
    fn a_windows_1252_file_becomes_utf8_and_back() {
        let latin = b"caf\xe9\n";
        assert_eq!(
            changed(latin, ConvertOp::Encoding(Charset::Utf8)),
            "caf\u{e9}\n".as_bytes()
        );
        assert_eq!(
            changed(
                "caf\u{e9}\n".as_bytes(),
                ConvertOp::Encoding(Charset::Windows1252)
            ),
            latin
        );
        // Plain ASCII is already both: nothing to write.
        assert_eq!(
            convert_bytes(b"abc\n", ConvertOp::Encoding(Charset::Windows1252)),
            ConvertOutcome::Unchanged
        );
    }

    #[test]
    fn utf16_and_the_bom_variant_are_written_with_their_bom() {
        let out = changed(b"hi\n", ConvertOp::Encoding(Charset::Utf8Bom));
        assert_eq!(out, b"\xEF\xBB\xBFhi\n");
        let out = changed(b"hi", ConvertOp::Encoding(Charset::Utf16Le));
        assert_eq!(out, b"\xFF\xFEh\0i\0");
        // And the BOM is the source announcement on the way back.
        assert_eq!(changed(&out, ConvertOp::Encoding(Charset::Utf8)), b"hi");
    }

    #[test]
    fn a_character_the_target_cannot_hold_skips_the_file() {
        match convert_bytes(
            "a\n\u{4e2d}\n".as_bytes(),
            ConvertOp::Encoding(Charset::Windows1252),
        ) {
            ConvertOutcome::Skipped(ConvertSkip::Unencodable { count, line, ch }) => {
                assert_eq!((count, line, ch), (1, 2, '\u{4e2d}'));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn binary_files_are_skipped() {
        assert_eq!(
            convert_bytes(b"a\0b", ConvertOp::Eol(Eol::Crlf)),
            ConvertOutcome::Skipped(ConvertSkip::Binary)
        );
    }

    #[test]
    fn indentation_converts_on_files_and_keeps_crlf() {
        let op = ConvertOp::Indent {
            tabs: false,
            size: 4,
        };
        assert_eq!(changed(b"\tx\r\n\t\ty\r\n", op), b"    x\r\n        y\r\n");
        let op = ConvertOp::Indent {
            tabs: true,
            size: 4,
        };
        assert_eq!(changed(b"        x\n", op), b"\t\tx\n");
        assert_eq!(convert_bytes(b"x\n", op), ConvertOutcome::Unchanged);
    }
}
