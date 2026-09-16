//! Line endings and file encodings (S22, S24bis): the pure conversions the
//! content stage runs on one blob, the path filter the rules share and the
//! small `.gitattributes` reader of "follow the attributes".
//!
//! Nothing here reads a repository: the engine decides which blobs are text
//! (a NUL in the first 8 KiB means binary, except for UTF-16) and counts the
//! files these functions leave alone.

use serde::{Deserialize, Serialize};

use super::paths::GlobSet;

/// The line ending a text file gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineEnding {
    /// `\n` (Linux, macOS).
    Lf,
    /// `\r\n` (Windows).
    Crlf,
    /// `\r` (classic Mac OS).
    Cr,
}

impl LineEnding {
    pub const ALL: [LineEnding; 3] = [LineEnding::Lf, LineEnding::Crlf, LineEnding::Cr];

    /// The stable name used by the CLI (`--eol`) and in messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Lf => "lf",
            Self::Crlf => "crlf",
            Self::Cr => "cr",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim().to_ascii_lowercase();
        Self::ALL.into_iter().find(|e| e.name() == name)
    }

    fn bytes(self) -> &'static [u8] {
        match self {
            Self::Lf => b"\n",
            Self::Crlf => b"\r\n",
            Self::Cr => b"\r",
        }
    }
}

/// `bytes` with every line ending (`\r\n`, a lone `\r`, a lone `\n`) replaced
/// by `target`: a file with mixed endings comes out uniform. `None` when the
/// file already is.
pub fn convert_eol(bytes: &[u8], target: LineEnding) -> Option<Vec<u8>> {
    let end = target.bytes();
    let mut out = Vec::with_capacity(bytes.len() + bytes.len() / 16);
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' if bytes.get(i + 1) == Some(&b'\n') => {
                out.extend_from_slice(end);
                i += 2;
            }
            b'\r' | b'\n' => {
                out.extend_from_slice(end);
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    (out != bytes).then_some(out)
}

/// A text encoding a rule reads or writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextEncoding {
    /// UTF-8 without a byte order mark (a read tolerates one).
    Utf8,
    /// UTF-8 with the byte order mark `EF BB BF`.
    Utf8Bom,
    /// UTF-16 little endian (written with its byte order mark).
    Utf16Le,
    /// UTF-16 big endian (written with its byte order mark).
    Utf16Be,
    Windows1252,
    /// ISO-8859-1: byte `n` is U+00`n`, unlike Windows-1252 in `80`..`9F`.
    Latin1,
}

impl TextEncoding {
    pub const ALL: [TextEncoding; 6] = [
        TextEncoding::Utf8,
        TextEncoding::Utf8Bom,
        TextEncoding::Utf16Le,
        TextEncoding::Utf16Be,
        TextEncoding::Windows1252,
        TextEncoding::Latin1,
    ];

    /// The stable name used by the CLI and in messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Utf8 => "utf-8",
            Self::Utf8Bom => "utf-8-bom",
            Self::Utf16Le => "utf-16le",
            Self::Utf16Be => "utf-16be",
            Self::Windows1252 => "windows-1252",
            Self::Latin1 => "iso-8859-1",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.trim().to_ascii_lowercase().replace('_', "-");
        let alias = match name.as_str() {
            "utf8" => "utf-8",
            "utf8-bom" | "utf-8-with-bom" => "utf-8-bom",
            "utf16le" | "utf-16" => "utf-16le",
            "utf16be" => "utf-16be",
            "cp1252" | "windows1252" => "windows-1252",
            "latin1" | "latin-1" | "iso8859-1" => "iso-8859-1",
            other => other,
        };
        Self::ALL.into_iter().find(|e| e.name() == alias)
    }

    fn utf16(self) -> bool {
        matches!(self, Self::Utf16Le | Self::Utf16Be)
    }
}

/// What [`reencode`] did to one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reencoded {
    /// Already in the target encoding (or empty).
    Unchanged,
    New(Vec<u8>),
    /// Not valid in the source encoding, or the target cannot hold a
    /// character: the file stays as it is.
    Undecodable,
}

const BOM_UTF8: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// The encoding a byte order mark announces.
fn bom_of(bytes: &[u8]) -> Option<TextEncoding> {
    if bytes.starts_with(&BOM_UTF8) {
        Some(TextEncoding::Utf8Bom)
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        Some(TextEncoding::Utf16Le)
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        Some(TextEncoding::Utf16Be)
    } else {
        None
    }
}

/// Whether the file is UTF-16 text, so its NUL bytes do not make it binary:
/// `from` says so, or (without `from`) a UTF-16 byte order mark does.
pub fn is_utf16_text(bytes: &[u8], from: Option<TextEncoding>) -> bool {
    match from {
        Some(e) => e.utf16(),
        None => bom_of(bytes).is_some_and(TextEncoding::utf16),
    }
}

/// The source encoding of `bytes` when the rule names none: a byte order
/// mark, else valid UTF-8, else `fallback`.
pub fn detect(bytes: &[u8], fallback: TextEncoding) -> TextEncoding {
    bom_of(bytes).unwrap_or(if std::str::from_utf8(bytes).is_ok() {
        TextEncoding::Utf8
    } else {
        fallback
    })
}

fn decode(bytes: &[u8], from: TextEncoding) -> Option<String> {
    match from {
        TextEncoding::Utf8 | TextEncoding::Utf8Bom => {
            let body = bytes.strip_prefix(&BOM_UTF8).unwrap_or(bytes);
            std::str::from_utf8(body).ok().map(str::to_string)
        }
        TextEncoding::Utf16Le | TextEncoding::Utf16Be => {
            let le = from == TextEncoding::Utf16Le;
            let bom: [u8; 2] = if le { [0xFF, 0xFE] } else { [0xFE, 0xFF] };
            let body = bytes.strip_prefix(&bom).unwrap_or(bytes);
            if body.len() % 2 != 0 {
                return None;
            }
            let units = body.chunks_exact(2).map(|c| {
                if le {
                    u16::from_le_bytes([c[0], c[1]])
                } else {
                    u16::from_be_bytes([c[0], c[1]])
                }
            });
            char::decode_utf16(units)
                .collect::<Result<String, _>>()
                .ok()
        }
        TextEncoding::Windows1252 => encoding_rs::WINDOWS_1252
            .decode_without_bom_handling_and_without_replacement(bytes)
            .map(|c| c.into_owned()),
        TextEncoding::Latin1 => Some(bytes.iter().map(|&b| b as char).collect()),
    }
}

fn encode(text: &str, to: TextEncoding) -> Option<Vec<u8>> {
    match to {
        TextEncoding::Utf8 => Some(text.as_bytes().to_vec()),
        TextEncoding::Utf8Bom => {
            let mut out = BOM_UTF8.to_vec();
            out.extend_from_slice(text.as_bytes());
            Some(out)
        }
        TextEncoding::Utf16Le | TextEncoding::Utf16Be => {
            let le = to == TextEncoding::Utf16Le;
            let mut out: Vec<u8> = if le {
                vec![0xFF, 0xFE]
            } else {
                vec![0xFE, 0xFF]
            };
            for u in text.encode_utf16() {
                out.extend_from_slice(&if le { u.to_le_bytes() } else { u.to_be_bytes() });
            }
            Some(out)
        }
        TextEncoding::Windows1252 => {
            let (bytes, _, unmappable) = encoding_rs::WINDOWS_1252.encode(text);
            (!unmappable).then(|| bytes.into_owned())
        }
        TextEncoding::Latin1 => text
            .chars()
            .map(|c| u8::try_from(u32::from(c)).ok())
            .collect(),
    }
}

/// `bytes` from `from` (when `None`: detected, see [`detect`], with
/// `fallback` for what is neither marked nor UTF-8) to `to`.
pub fn reencode(
    bytes: &[u8],
    to: TextEncoding,
    from: Option<TextEncoding>,
    fallback: TextEncoding,
) -> Reencoded {
    if bytes.is_empty() {
        return Reencoded::Unchanged;
    }
    let source = from.unwrap_or_else(|| detect(bytes, fallback));
    let converted = decode(bytes, source).and_then(|text| encode(&text, to));
    match converted {
        None => Reencoded::Undecodable,
        Some(new) if new == bytes => Reencoded::Unchanged,
        Some(new) => Reencoded::New(new),
    }
}

/// Include globs (empty: every file) minus exclude globs.
#[derive(Debug, Clone)]
pub struct PathFilter {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl PathFilter {
    pub fn new(paths: &[String], exclude: &[String]) -> Result<Self, String> {
        let set = |globs: &[String]| -> Result<Option<GlobSet>, String> {
            if globs.is_empty() {
                Ok(None)
            } else {
                GlobSet::new(globs).map(Some)
            }
        };
        Ok(Self {
            include: set(paths)?,
            exclude: set(exclude)?,
        })
    }

    pub fn matches(&self, path: &str) -> bool {
        self.include.as_ref().map_or(true, |g| g.is_match(path))
            && !self.exclude.as_ref().is_some_and(|g| g.is_match(path))
    }

    /// The filter names no include glob: it reaches every file.
    pub fn everywhere(&self) -> bool {
        self.include.is_none()
    }
}

/// What the root `.gitattributes` says about a file's line endings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EolAttr {
    /// Nothing: the rule's own target applies.
    Unset,
    /// `-text` or `binary`: the file is left alone.
    Skip,
    /// `eol=lf` / `eol=crlf`: this file gets that ending instead.
    Force(LineEnding),
}

/// The `text` / `eol` lines of a `.gitattributes` (the last matching line
/// wins, as in git). Only the repository's root file is read.
#[derive(Debug, Clone, Default)]
pub struct Attributes {
    lines: Vec<(GlobSet, Option<bool>, Option<LineEnding>)>,
}

impl Attributes {
    pub fn parse(text: &[u8]) -> Self {
        let text = String::from_utf8_lossy(text);
        let mut lines = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split_whitespace();
            let Some(pattern) = parts.next() else {
                continue;
            };
            let Ok(set) = GlobSet::new(&[pattern]) else {
                continue;
            };
            // `Some(true)`: the file is text, `Some(false)`: it is not.
            let (mut is_text, mut eol) = (None, None);
            for attr in parts {
                match attr {
                    "-text" | "binary" => is_text = Some(false),
                    "text" | "text=auto" | "!text" => is_text = Some(true),
                    other => {
                        if let Some(v) = other.strip_prefix("eol=") {
                            eol = match v {
                                "lf" => Some(LineEnding::Lf),
                                "crlf" => Some(LineEnding::Crlf),
                                _ => eol,
                            };
                        }
                    }
                }
            }
            if is_text.is_some() || eol.is_some() {
                lines.push((set, is_text, eol));
            }
        }
        Self { lines }
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn eol_for(&self, path: &str) -> EolAttr {
        let (mut is_text, mut eol) = (true, None);
        for (set, t, e) in &self.lines {
            if set.is_match(path) {
                is_text = t.unwrap_or(is_text);
                eol = e.or(eol);
            }
        }
        match (is_text, eol) {
            (false, _) => EolAttr::Skip,
            (true, Some(e)) => EolAttr::Force(e),
            (true, None) => EolAttr::Unset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CP: TextEncoding = TextEncoding::Windows1252;

    #[test]
    fn every_ending_becomes_the_target() {
        let mixed = b"a\r\nb\nc\rd";
        assert_eq!(convert_eol(mixed, LineEnding::Lf).unwrap(), b"a\nb\nc\nd");
        assert_eq!(
            convert_eol(mixed, LineEnding::Crlf).unwrap(),
            b"a\r\nb\r\nc\r\nd"
        );
        assert_eq!(convert_eol(mixed, LineEnding::Cr).unwrap(), b"a\rb\rc\rd");
        assert_eq!(convert_eol(b"a\nb\n", LineEnding::Lf), None);
        assert_eq!(convert_eol(b"no ending", LineEnding::Crlf), None);
    }

    #[test]
    fn windows_1252_has_the_euro_and_latin1_does_not() {
        let euro = "5 \u{20AC}".as_bytes();
        assert_eq!(
            reencode(&[b'5', b' ', 0x80], TextEncoding::Utf8, Some(CP), CP),
            Reencoded::New(euro.to_vec())
        );
        assert_eq!(
            reencode(euro, TextEncoding::Latin1, None, CP),
            Reencoded::Undecodable
        );
        assert_eq!(
            reencode(&[0xE9], TextEncoding::Utf8, None, CP),
            Reencoded::New("\u{e9}".as_bytes().to_vec())
        );
        assert_eq!(
            reencode("\u{e9}".as_bytes(), TextEncoding::Latin1, None, CP),
            Reencoded::New(vec![0xE9])
        );
    }

    #[test]
    fn utf16_with_a_mark_is_detected() {
        let mut utf16 = vec![0xFF, 0xFE];
        for u in "h\u{e9}".encode_utf16() {
            utf16.extend_from_slice(&u.to_le_bytes());
        }
        assert!(is_utf16_text(&utf16, None));
        assert_eq!(
            reencode(&utf16, TextEncoding::Utf8, None, CP),
            Reencoded::New("h\u{e9}".as_bytes().to_vec())
        );
        // Odd length: not UTF-16.
        assert_eq!(
            reencode(&[0xFF, 0xFE, 0x41], TextEncoding::Utf8, None, CP),
            Reencoded::Undecodable
        );
    }

    #[test]
    fn the_utf8_mark_comes_and_goes() {
        assert_eq!(
            reencode(b"abc", TextEncoding::Utf8Bom, None, CP),
            Reencoded::New(b"\xEF\xBB\xBFabc".to_vec())
        );
        assert_eq!(
            reencode(b"\xEF\xBB\xBFabc", TextEncoding::Utf8, None, CP),
            Reencoded::New(b"abc".to_vec())
        );
        assert_eq!(
            reencode(b"abc", TextEncoding::Utf8, None, CP),
            Reencoded::Unchanged
        );
    }

    #[test]
    fn a_file_the_source_cannot_read_is_undecodable() {
        assert_eq!(
            reencode(&[0xE9], TextEncoding::Utf16Le, Some(TextEncoding::Utf8), CP),
            Reencoded::Undecodable
        );
    }

    #[test]
    fn a_filter_includes_then_excludes() {
        let f = PathFilter::new(&["*.txt".into()], &["vendor/".into()]).unwrap();
        assert!(f.matches("a.txt"));
        assert!(!f.matches("vendor/a.txt"));
        assert!(!f.matches("a.rs"));
        assert!(PathFilter::new(&[], &[]).unwrap().matches("anything"));
    }

    #[test]
    fn attributes_pick_skip_or_force() {
        let a = Attributes::parse(b"*.bat eol=crlf\n*.png binary\n# c\n*.txt text\n");
        assert_eq!(a.eol_for("x/run.bat"), EolAttr::Force(LineEnding::Crlf));
        assert_eq!(a.eol_for("a.png"), EolAttr::Skip);
        assert_eq!(a.eol_for("a.txt"), EolAttr::Unset);
        assert_eq!(a.eol_for("a.rs"), EolAttr::Unset);
    }

    #[test]
    fn names_round_trip() {
        for e in TextEncoding::ALL {
            assert_eq!(TextEncoding::from_name(e.name()), Some(e));
        }
        assert_eq!(
            TextEncoding::from_name("Latin1"),
            Some(TextEncoding::Latin1)
        );
        assert_eq!(LineEnding::from_name("CRLF"), Some(LineEnding::Crlf));
    }
}
