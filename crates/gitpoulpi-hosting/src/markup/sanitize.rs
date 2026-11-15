//! Untrusted text hygiene: what a terminal must never be handed.
//!
//! Comment bodies come from other people. A C0/C1 control character in one
//! (ESC, BEL, CSI, OSC introducers) would be interpreted by the terminal, so
//! they are removed here, before any parsing. Emoji presentation selectors,
//! zero-width joiners, skin-tone modifiers and combining marks with no base
//! are removed too: terminals disagree on their cell width, which corrupts
//! the layout (see "emoji presentation" in `docs/developer/where-we-are.md`).

/// Characters kept as they are although they are controls.
fn keep_control(c: char) -> bool {
    c == '\n' || c == '\t'
}

fn is_combining_mark(c: char) -> bool {
    matches!(
        c as u32,
        0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F
    )
}

/// Format characters that change width or direction in a terminal.
fn is_unwanted_format(c: char) -> bool {
    matches!(
        c as u32,
        // variation selectors 15 and 16, zero-width joiner
        0xFE0E | 0xFE0F | 0x200D
        // zero-width space, non-joiner, LRM, RLM
        | 0x200B | 0x200C | 0x200E | 0x200F
        // bidi embedding and isolate controls (spoofing)
        | 0x202A..=0x202E | 0x2066..=0x2069
        // BOM / word joiner
        | 0xFEFF | 0x2060
        // emoji skin-tone modifiers
        | 0x1F3FB..=0x1F3FF
    )
}

/// `text` with every character a terminal could act on removed. `\r\n`
/// becomes `\n`; a lone `\r` is dropped.
pub fn sanitize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_control() {
            if keep_control(c) {
                out.push(c);
            }
            continue;
        }
        if is_unwanted_format(c) {
            continue;
        }
        if is_combining_mark(c) {
            // Lone: nothing to attach to at the start or after a blank.
            let lone = out.chars().next_back().is_none_or(|p| p.is_whitespace());
            if lone {
                continue;
            }
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn table() {
        let cases: &[(&str, &str)] = &[
            ("plain text", "plain text"),
            ("a\tb\nc", "a\tb\nc"),
            ("red \u{1b}[31mtext\u{1b}[0m", "red [31mtext[0m"),
            ("a\u{1b}]0;title\u{7}b", "a]0;titleb"),
            ("c1\u{9b}31m\u{85}x", "c131mx"),
            ("nul\u{0}del\u{7f}x", "nuldelx"),
            ("crlf\r\nx\ry", "crlf\nxy"),
            ("\u{2764}\u{fe0f} heart", "\u{2764} heart"),
            ("text\u{fe0e}", "text"),
            ("\u{1f468}\u{200d}\u{1f469}", "\u{1f468}\u{1f469}"),
            ("\u{1f44d}\u{1f3fd}", "\u{1f44d}"),
            ("e\u{301} stays", "e\u{301} stays"),
            ("\u{301}lone", "lone"),
            ("a \u{301}b", "a b"),
            ("\u{202e}evil", "evil"),
            ("caf\u{e9} \u{4e2d}\u{6587}", "caf\u{e9} \u{4e2d}\u{6587}"),
        ];
        for (input, want) in cases {
            assert_eq!(sanitize(input), *want, "{input:?}");
        }
    }

    #[test]
    fn no_escape_survives() {
        let nasty = "\u{1b}[2J\u{1b}]52;c;AAAA\u{7}\u{9d}x\u{90}y";
        let out = sanitize(nasty);
        assert!(!out.chars().any(|c| c.is_control()), "{out:?}");
    }
}
