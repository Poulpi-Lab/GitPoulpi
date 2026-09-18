//! The status bar's formats: encodings, line breaks, indentation.

use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_str(doc: &mut Doc, s: &str) {
    for c in s.chars() {
        let code = if c == '\n' {
            KeyCode::Enter
        } else {
            KeyCode::Char(c)
        };
        doc.on_key(&key(code), 10);
    }
}

#[test]
fn every_encoding_reads_and_writes_its_own_bytes() {
    // (encoding, text, the bytes as another tool writes them)
    let cases: &[(Charset, &str, &[u8])] = &[
        (
            Charset::Utf8,
            "h\u{e9}llo \u{20ac}\n",
            "h\u{e9}llo \u{20ac}\n".as_bytes(),
        ),
        (Charset::Utf8Bom, "h\u{e9}\n", b"\xEF\xBB\xBFh\xC3\xA9\n"),
        (
            Charset::Utf16Le,
            "h\u{e9}\u{1f600}\n",
            b"\xFF\xFEh\x00\xE9\x00\x3D\xD8\x00\xDE\n\x00",
        ),
        (
            Charset::Utf16Be,
            "h\u{e9}\n",
            b"\xFE\xFF\x00h\x00\xE9\x00\n",
        ),
        (
            Charset::Windows1252,
            "caf\u{e9} \u{20ac}\n",
            b"caf\xE9 \x80\n",
        ),
        (Charset::Iso8859_1, "caf\u{e9} \u{80}\n", b"caf\xE9 \x80\n"),
        (Charset::Iso8859_15, "\u{20ac}\n", b"\xA4\n"),
        (Charset::Windows1250, "\u{10d}\u{e1}\n", b"\xE8\xE1\n"),
        (
            Charset::Windows1251,
            "\u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}\n",
            b"\xCF\xF0\xE8\xE2\xE5\xF2\n",
        ),
        (
            Charset::Koi8R,
            "\u{43f}\u{440}\u{438}\u{432}\u{435}\u{442}\n",
            b"\xD0\xD2\xC9\xD7\xC5\xD4\n",
        ),
        (
            Charset::ShiftJis,
            "\u{65e5}\u{672c}\n",
            b"\x93\xFA\x96\x7B\n",
        ),
        (Charset::EucJp, "\u{65e5}\u{672c}\n", b"\xC6\xFC\xCB\xDC\n"),
        (Charset::Gbk, "\u{4e2d}\u{6587}\n", b"\xD6\xD0\xCE\xC4\n"),
        (Charset::Big5, "\u{4e2d}\u{6587}\n", b"\xA4\xA4\xA4\xE5\n"),
        (Charset::EucKr, "\u{d55c}\u{ae00}\n", b"\xC7\xD1\xB1\xDB\n"),
    ];
    assert_eq!(
        cases.len(),
        charset::ALL.len(),
        "every offered encoding is covered"
    );
    for (cs, text, bytes) in cases {
        let doc = Doc::decode_as(bytes, Some(*cs)).unwrap();
        assert_eq!(doc.charset(), *cs, "{}", cs.name());
        assert_eq!(doc.text(), *text, "{}", cs.name());
        assert_eq!(doc.encode(), *bytes, "{} round trip", cs.name());
        assert!(!doc.is_dirty());
        assert!(doc.pristine.is_none(), "{} needs no kept bytes", cs.name());
    }
}

#[test]
fn the_encoding_is_detected_on_open() {
    let utf16 = Doc::decode(b"\xFF\xFEa\x00").unwrap();
    assert_eq!((utf16.charset(), utf16.text()), (Charset::Utf16Le, "a"));
    let utf16 = Doc::decode(b"\xFE\xFF\x00a").unwrap();
    assert_eq!((utf16.charset(), utf16.text()), (Charset::Utf16Be, "a"));
    let bom = Doc::decode(b"\xEF\xBB\xBFa").unwrap();
    assert_eq!((bom.charset(), bom.text()), (Charset::Utf8Bom, "a"));
    let plain = Doc::decode("h\u{e9}".as_bytes()).unwrap();
    assert_eq!(plain.charset(), Charset::Utf8);
    let legacy = Doc::decode(b"h\xE9").unwrap();
    assert_eq!(
        (legacy.charset(), legacy.text()),
        (Charset::Windows1252, "h\u{e9}")
    );
    // A NUL outside UTF-16 is binary, even when forced to a legacy page.
    assert_eq!(
        Doc::decode_as(b"a\0b", Some(Charset::Windows1252)).err(),
        Some(LoadRefusal::Binary)
    );
    // Bytes the page cannot read are refused, not turned into U+FFFD.
    assert_eq!(
        Doc::decode_as(b"\xFF\xFEa", Some(Charset::Utf16Le)).err(),
        Some(LoadRefusal::Undecodable(Charset::Utf16Le))
    );
}

#[test]
fn a_file_the_encoding_cannot_reproduce_keeps_its_bytes_until_edited() {
    // Shift_JIS has two codes for U+2252; re-encoding picks the other.
    let original = b"\x87\x90\n";
    let mut doc = Doc::decode_as(original, Some(Charset::ShiftJis)).unwrap();
    assert_eq!(doc.text(), "\u{2252}\n");
    assert_eq!(doc.encode(), original);
    type_str(&mut doc, "x");
    assert_ne!(doc.encode(), original);
    doc.undo();
    assert!(!doc.is_dirty());
    assert_eq!(doc.encode(), original, "clean again, the bytes come back");
}

#[test]
fn converting_the_encoding_changes_what_is_written_and_undoes() {
    let original = b"caf\xE9 \x80\n";
    let mut doc = Doc::decode(original).unwrap();
    assert_eq!(doc.convert_charset(Charset::Windows1252), Ok(false));
    assert!(!doc.is_dirty());
    assert_eq!(doc.convert_charset(Charset::Utf8), Ok(true));
    assert!(doc.is_dirty());
    assert_eq!(doc.encode(), "caf\u{e9} \u{20ac}\n".as_bytes());
    assert_eq!(doc.text(), "caf\u{e9} \u{20ac}\n", "the text is untouched");
    assert!(doc.undo());
    assert_eq!(doc.charset(), Charset::Windows1252);
    assert!(!doc.is_dirty());
    assert_eq!(doc.encode(), original);
    assert!(doc.redo());
    assert_eq!(doc.charset(), Charset::Utf8);
    assert!(doc.is_dirty());
    // Saved: that is the new baseline.
    doc.mark_saved();
    assert!(!doc.is_dirty());
    assert!(doc.undo());
    assert!(doc.is_dirty(), "undoing past the save differs from disk");
}

#[test]
fn converting_to_an_encoding_that_cannot_hold_the_text_is_refused() {
    let mut doc = Doc::decode("ok\ncaf\u{e9} \u{20ac} \u{20ac}\n".as_bytes()).unwrap();
    let why = doc.convert_charset(Charset::Iso8859_1).unwrap_err();
    assert_eq!((why.count, why.line, why.ch), (2, 2, '\u{20ac}'));
    let why = doc.convert_charset(Charset::ShiftJis).unwrap_err();
    assert_eq!((why.count, why.line, why.ch), (3, 2, '\u{e9}'));
    assert_eq!(doc.charset(), Charset::Utf8);
    assert!(!doc.is_dirty() && !doc.can_undo());
    // Typing what the page cannot hold makes the save refuse, too.
    let mut doc = Doc::decode(b"abc\n").unwrap();
    doc.convert_charset(Charset::Windows1252).unwrap();
    type_str(&mut doc, "\u{4e2d}");
    let why = doc.try_encode().unwrap_err();
    assert_eq!((why.count, why.line, why.ch), (1, 1, '\u{4e2d}'));
}

#[test]
fn the_kind_of_line_break_is_detected() {
    let eol = |bytes: &[u8]| Doc::decode(bytes).unwrap().eol();
    assert_eq!(eol(b"a\nb\n"), Eol::Lf);
    assert_eq!(eol(b"no break"), Eol::Lf);
    assert_eq!(eol(b"a\r\nb\r\n"), Eol::Crlf);
    assert_eq!(eol(b"a\rb\r"), Eol::Cr);
    assert_eq!(eol(b"a\r\nb\n"), Eol::Mixed);
    assert_eq!(eol(b"a\rb\nc"), Eol::Mixed);
    // An all-CR file is lines, like a CRLF one.
    let doc = Doc::decode(b"a\rb\r").unwrap();
    assert_eq!(doc.line_count(), 3);
    assert_eq!(doc.line(1), "b");
}

#[test]
fn line_breaks_convert_between_all_three_as_one_undo_step() {
    let cases: [(&[u8], Eol, &[&str]); 4] = [
        (b"a\nb\nc", Eol::Lf, &["a", "b", "c"]),
        (b"a\r\nb\r\nc", Eol::Crlf, &["a", "b", "c"]),
        (b"a\rb\rc", Eol::Cr, &["a", "b", "c"]),
        (b"a\r\nb\nc\rd", Eol::Mixed, &["a", "b", "c", "d"]),
    ];
    for (original, from, lines) in cases {
        for to in [Eol::Lf, Eol::Crlf, Eol::Cr] {
            let mut doc = Doc::decode(original).unwrap();
            assert_eq!(doc.eol(), from);
            let sep = match to {
                Eol::Lf => "\n",
                Eol::Crlf => "\r\n",
                _ => "\r",
            };
            let want = lines.join(sep).into_bytes();
            let changed = doc.convert_eol(to);
            assert_eq!(changed, from != to, "{from:?} -> {to:?}");
            assert_eq!(doc.encode(), want, "{from:?} -> {to:?}");
            assert_eq!(doc.is_dirty(), changed);
            assert_eq!(doc.undo_len(), usize::from(changed), "one undo step");
            if changed {
                doc.undo();
                assert_eq!(doc.encode(), original, "{from:?} -> {to:?} undone");
                assert!(!doc.is_dirty());
                doc.redo();
                assert_eq!(doc.encode(), want);
            }
        }
    }
}

#[test]
fn a_mixed_file_round_trips_and_new_lines_use_the_files_break() {
    let original = b"a\r\nb\nc\rd\n";
    let doc = Doc::decode(original).unwrap();
    assert_eq!(doc.eol(), Eol::Mixed);
    assert_eq!(doc.encode(), original);
    let mut crlf = Doc::decode(b"x\r\ny\r\n").unwrap();
    crlf.on_key(&key(KeyCode::End), 10);
    type_str(&mut crlf, "\nz");
    assert_eq!(crlf.encode(), b"x\r\nz\r\ny\r\n");
    let mut cr = Doc::decode(b"x\ry\r").unwrap();
    cr.on_key(&key(KeyCode::End), 10);
    type_str(&mut cr, "\nz");
    assert_eq!(cr.encode(), b"x\rz\ry\r");
}

#[test]
fn the_explicit_choice_wins_over_the_setting() {
    use gitpoulpi_config::EditorEol;
    crate::editor_prefs::with_eol(EditorEol::Crlf, || {
        let mut doc = Doc::decode(b"a\nb\n").unwrap();
        // The setting forces CRLF on save until the user chooses.
        assert_eq!(doc.eol(), Eol::Crlf);
        type_str(&mut doc, "x");
        assert_eq!(doc.encode(), b"xa\r\nb\r\n");
        assert!(doc.convert_eol(Eol::Lf));
        assert_eq!(doc.encode(), b"xa\nb\n");
        // A mix is never rewritten by the setting.
        let mixed = Doc::decode(b"a\r\nb\n").unwrap();
        assert_eq!(mixed.encode(), b"a\r\nb\n");
    });
}

#[test]
fn indentation_converts_both_ways_as_one_undo_step() {
    let original = "fn a() {\n\tif x {\n\t\ty();\n\t}\n}\n";
    let mut doc = Doc::decode(original.as_bytes()).unwrap();
    assert_eq!(doc.indent(), Indent::Tabs);
    doc.set_tab_size(4);
    assert_eq!(doc.convert_indent(false), 3);
    assert_eq!(doc.text(), "fn a() {\n    if x {\n        y();\n    }\n}\n");
    assert_eq!(doc.indent(), Indent::Spaces(4));
    assert_eq!(doc.undo_len(), 1);
    assert!(doc.undo());
    assert_eq!(doc.text(), original);
    assert!(!doc.is_dirty());
    // And back: spaces to tabs, by the tab size.
    let mut doc = Doc::decode(b"a\n    b\n      c\n").unwrap();
    doc.set_tab_size(4);
    assert_eq!(doc.convert_indent(true), 2);
    assert_eq!(doc.text(), "a\n\tb\n\t  c\n");
    assert_eq!(doc.indent(), Indent::Tabs);
    doc.undo();
    assert_eq!(doc.text(), "a\n    b\n      c\n");
    // Nothing to do: no step.
    assert_eq!(doc.convert_indent(false), 0);
    assert_eq!(doc.undo_len(), 0);
}

#[test]
fn the_indent_size_and_style_steer_tab_and_the_display() {
    let mut doc = Doc::decode(b"a\n").unwrap();
    doc.set_indent_tabs(false);
    doc.set_tab_size(2);
    assert_eq!(doc.indent_label(), "Spaces: 2");
    doc.on_key(&key(KeyCode::Tab), 10);
    assert_eq!(doc.text(), "  a\n");
    doc.set_indent_tabs(true);
    doc.set_tab_size(8);
    assert_eq!(doc.indent_label(), "Tabs: 8");
    doc.on_key(&key(KeyCode::Tab), 10);
    assert_eq!(doc.text(), "  \ta\n");
    // The caret column counts the tab at this file's width.
    assert_eq!(doc.cursor_line_col(), (0, 8));
    doc.set_tab_size(4);
    assert_eq!(doc.cursor_line_col(), (0, 4));
}
