//! The one OSC 52 encoder: the escape sequence that asks a terminal to put
//! text on its (the user's) clipboard. Shared by the TUI's copy actions and
//! the CLI's `--copy`, so the two can never encode differently.

use base64::Engine as _;

/// The OSC 52 sequence that asks a terminal to set its clipboard to `text`
/// (selection `c`, terminated by BEL, which every terminal that implements
/// OSC 52 accepts).
pub fn osc52(text: &str) -> String {
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    format!("\x1b]52;c;{encoded}\x07")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_payload_is_base64_in_a_clipboard_sequence() {
        assert_eq!(osc52("abc"), "\x1b]52;c;YWJj\x07");
        assert_eq!(osc52("foo"), "\x1b]52;c;Zm9v\x07");
        assert_eq!(osc52(""), "\x1b]52;c;\x07");
    }

    #[test]
    fn both_padding_lengths_and_non_ascii_encode() {
        assert_eq!(osc52("f"), "\x1b]52;c;Zg==\x07");
        assert_eq!(osc52("fo"), "\x1b]52;c;Zm8=\x07");
        assert_eq!(osc52("\u{e9}"), "\x1b]52;c;w6k=\x07");
    }
}
