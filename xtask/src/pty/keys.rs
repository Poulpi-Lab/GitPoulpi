//! Turning a human-readable key spec into the bytes a terminal would
//! actually put on the wire.
//!
//! The half of the driver with no pty in it, so the encoding (where every
//! "the shortcut did nothing" bug lives) is pinned by plain unit tests that
//! run identically on Windows and Linux.

/// Encodes a whitespace-separated key spec into the exact bytes a real
/// terminal emits for those keys.
///
/// A token is a named key (`Enter`, `Down`, `F5`), a single character
/// (`q`), or either of those with a modifier prefix: `C-` control, `M-`
/// alt/meta, `S-` shift. Tokens concatenate in order, so
/// `"C-g c Enter"` is the whole `Ctrl+G c Enter` chord sequence.
///
/// Unknown names are an error rather than a literal fallback: a typo'd
/// `PgDown` sent as six characters would look like an unwired shortcut.
pub fn encode_keys(spec: &str) -> Result<String, String> {
    let mut out = String::new();
    for token in spec.split_whitespace() {
        out.push_str(&encode_token(token)?);
    }
    Ok(out)
}

/// [`encode_keys`] for a ConPTY. The console host turns a VT byte stream
/// into key events itself, and four keys do not survive that: `\n`
/// (Ctrl+J) becomes Ctrl+Enter, `\0` (Ctrl+Space) is dropped, and the kitty
/// `CSI u` of Ctrl+, is not understood. A bare `ESC` is the fourth: sent alone
/// it often never reached the TUI (a confirm dialog never saw it). A real
/// Windows terminal sends all of those
/// as key events, which the host accepts as *win32-input-mode* sequences
/// (`CSI Vk ; Sc ; Uc ; Kd ; Cs ; Rc _`, announced by the host with
/// `CSI ? 9001 h`); this encodes exactly those chords that way and leaves
/// every other key as [`encode_keys`] does.
pub fn encode_keys_win32(spec: &str) -> Result<String, String> {
    const CTRL: u32 = 0x08; // LEFT_CTRL_PRESSED
    let mut out = String::new();
    for token in spec.split_whitespace() {
        match token {
            "C-j" | "C-J" => out.push_str(&win32_key(0x4A, 0x24, 10, CTRL)),
            "C-Space" => out.push_str(&win32_key(0x20, 0x39, 0, CTRL)),
            "C-," => out.push_str(&win32_key(0xBC, 0x33, 44, CTRL)),
            "Escape" | "Esc" => out.push_str(&win32_key(0x1B, 0x01, 27, 0)),
            other => out.push_str(&encode_token(other)?),
        }
    }
    Ok(out)
}

/// One key press then its release, as win32-input-mode sequences.
fn win32_key(vk: u32, scan: u32, ch: u32, state: u32) -> String {
    format!("\x1b[{vk};{scan};{ch};1;{state};1_\x1b[{vk};{scan};{ch};0;{state};1_")
}

fn encode_token(token: &str) -> Result<String, String> {
    if let Some(rest) = token.strip_prefix("C-") {
        return control(rest);
    }
    if let Some(rest) = token.strip_prefix("M-") {
        // An arrow, Home or End carries Alt in xterm's modifier parameter
        // (`CSI 1 ; 3 <final>`), as kitty, foot and Windows Terminal send it.
        if let Some(f) = csi_final(rest) {
            return Ok(format!("\x1b[1;3{f}"));
        }
        // Alt is ESC-prefixing, which is why alt chords and a real
        // Escape keypress are ambiguous on the wire -- crossterm
        // disambiguates on timing, and since we write the whole token in
        // one call the two bytes always arrive together.
        return Ok(format!("\x1b{}", encode_token(rest)?));
    }
    if let Some(rest) = token.strip_prefix("S-") {
        return shift(rest);
    }
    plain(token)
}

fn plain(name: &str) -> Result<String, String> {
    let sequence = match name {
        "Enter" | "Return" => "\r",
        "Tab" => "\t",
        "BTab" => "\x1b[Z",
        "Escape" | "Esc" => "\x1b",
        "Space" => " ",
        "Backspace" | "BSpace" => "\x7f",
        "Delete" | "DC" => "\x1b[3~",
        "Insert" | "IC" => "\x1b[2~",
        "Up" => "\x1b[A",
        "Down" => "\x1b[B",
        "Right" => "\x1b[C",
        "Left" => "\x1b[D",
        "Home" => "\x1b[H",
        "End" => "\x1b[F",
        "PageUp" | "PgUp" => "\x1b[5~",
        "PageDown" | "PgDn" => "\x1b[6~",
        "F1" => "\x1bOP",
        "F2" => "\x1bOQ",
        "F3" => "\x1bOR",
        "F4" => "\x1bOS",
        "F5" => "\x1b[15~",
        "F6" => "\x1b[17~",
        "F7" => "\x1b[18~",
        "F8" => "\x1b[19~",
        "F9" => "\x1b[20~",
        "F10" => "\x1b[21~",
        "F11" => "\x1b[23~",
        "F12" => "\x1b[24~",
        other => {
            let mut chars = other.chars();
            return match (chars.next(), chars.next()) {
                (Some(c), None) => Ok(c.to_string()),
                _ => Err(format!(
                    "unknown key name {other:?} - use send_text for literal strings"
                )),
            };
        }
    };
    Ok(sequence.to_string())
}

fn control(name: &str) -> Result<String, String> {
    if name == "Space" {
        return Ok("\0".to_string());
    }
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        // C0 control codes are the letter's position in the alphabet:
        // Ctrl+A is 0x01. Anything outside A-Z (or the four symbols
        // below) has no single-byte control form.
        (Some(c), None) if c.is_ascii_alphabetic() => {
            let byte = c.to_ascii_uppercase() as u8 - b'@';
            Ok((byte as char).to_string())
        }
        // Ctrl+, has no C0 byte: the kitty keyboard protocol's `CSI u`
        // (ConPTY: see `encode_keys_win32`).
        (Some(','), None) => Ok("\x1b[44;5u".to_string()),
        (Some('['), None) => Ok("\x1b".to_string()),
        (Some('\\'), None) => Ok("\x1c".to_string()),
        (Some(']'), None) => Ok("\x1d".to_string()),
        (Some('_'), None) => Ok("\x1f".to_string()),
        _ => Err(format!("{name:?} has no control-key encoding")),
    }
}

/// The final byte of a key that xterm encodes as `CSI 1 ; <mods> <final>`.
fn csi_final(name: &str) -> Option<char> {
    Some(match name {
        "Up" => 'A',
        "Down" => 'B',
        "Right" => 'C',
        "Left" => 'D',
        "Home" => 'H',
        "End" => 'F',
        _ => return None,
    })
}

fn shift(name: &str) -> Result<String, String> {
    // xterm's modifier encoding: `CSI 1 ; <1+mask> <final>`, mask 1 for
    // shift. Tab is the exception -- shift-tab has its own sequence.
    let final_byte = match name {
        "Tab" => return Ok("\x1b[Z".to_string()),
        // The menu key's chord: `CSI 21 ; 2 ~`.
        "F10" => return Ok("\x1b[21;2~".to_string()),
        "Up" => 'A',
        "Down" => 'B',
        "Right" => 'C',
        "Left" => 'D',
        "Home" => 'H',
        "End" => 'F',
        other => {
            let mut chars = other.chars();
            return match (chars.next(), chars.next()) {
                (Some(c), None) => Ok(c.to_uppercase().to_string()),
                _ => Err(format!("{other:?} has no shifted encoding")),
            };
        }
    };
    Ok(format!("\x1b[1;2{final_byte}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_keys_encode_to_their_xterm_sequences() {
        assert_eq!(encode_keys("Enter").unwrap(), "\r");
        assert_eq!(encode_keys("Tab").unwrap(), "\t");
        assert_eq!(encode_keys("Escape").unwrap(), "\x1b");
        assert_eq!(encode_keys("Down").unwrap(), "\x1b[B");
        assert_eq!(encode_keys("PageDown").unwrap(), "\x1b[6~");
        assert_eq!(encode_keys("F5").unwrap(), "\x1b[15~");
    }

    #[test]
    fn a_control_chord_is_one_c0_byte() {
        assert_eq!(encode_keys("C-g").unwrap(), "\x07");
        assert_eq!(encode_keys("C-c").unwrap(), "\x03");
        assert_eq!(encode_keys("C-Space").unwrap(), "\0");
    }

    #[test]
    fn the_chords_a_conpty_mangles_go_out_as_win32_input_mode() {
        assert_eq!(encode_keys("C-,").unwrap(), "\x1b[44;5u");
        assert_eq!(
            encode_keys_win32("C-j").unwrap(),
            "\x1b[74;36;10;1;8;1_\x1b[74;36;10;0;8;1_"
        );
        assert!(encode_keys_win32("C-Space")
            .unwrap()
            .starts_with("\x1b[32;57;0;1;8;1_"));
        assert!(encode_keys_win32("C-,")
            .unwrap()
            .starts_with("\x1b[188;51;44;1;8;1_"));
        // Everything else is the plain encoding.
        assert_eq!(encode_keys_win32("C-g Enter").unwrap(), "\x07\r");
    }

    #[test]
    fn alt_prefixes_escape() {
        assert_eq!(encode_keys("M-x").unwrap(), "\x1bx");
        assert_eq!(encode_keys("M-Down").unwrap(), "\x1b[1;3B");
    }

    #[test]
    fn shift_on_an_arrow_uses_the_modifier_parameter() {
        assert_eq!(encode_keys("S-Down").unwrap(), "\x1b[1;2B");
        assert_eq!(encode_keys("S-Tab").unwrap(), "\x1b[Z");
        assert_eq!(encode_keys("S-F10").unwrap(), "\x1b[21;2~");
    }

    #[test]
    fn several_keys_concatenate_in_order() {
        assert_eq!(encode_keys("Down Down Enter").unwrap(), "\x1b[B\x1b[B\r");
    }

    #[test]
    fn a_bare_character_is_itself() {
        assert_eq!(encode_keys("q").unwrap(), "q");
        assert_eq!(encode_keys("a b").unwrap(), "ab");
    }

    #[test]
    fn an_unknown_key_name_is_an_error_not_a_silent_literal() {
        // A typo'd key name that silently went out as six literal
        // characters would look exactly like "the shortcut did nothing".
        let err = encode_keys("PgDown").unwrap_err();
        assert!(err.contains("PgDown"), "{err}");
    }
}
