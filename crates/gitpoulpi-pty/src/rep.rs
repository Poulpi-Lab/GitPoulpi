//! Expands `CSI Ps b` (REP, "repeat the preceding graphic character") before
//! the bytes reach the cell grid.
//!
//! The Windows console host compresses a run of one character (a `─` rule, a
//! row of blanks) into `char CSI n b` when it forwards a frame, and `vt100`
//! 0.15 ignores REP: every run it compressed vanished from the grid, leaving
//! whatever an earlier frame had there (stray letters of a closed Settings
//! page across the Log, `Unstaged Files (10` for `(0)`). A unix pty never
//! sends it. The expansion is a plain byte filter in front of the parser;
//! the raw stream the tests inspect is kept as it arrived.

/// The most characters one REP may add: a bound, not a feature (a screen is
/// a few hundred columns wide).
const MAX_REPEAT: usize = 4096;

#[derive(Default)]
pub struct RepExpander {
    state: State,
    /// The last graphic character written, as UTF-8.
    last: Vec<u8>,
    /// Bytes of a character whose end has not arrived yet.
    pending: Vec<u8>,
    /// The parameter bytes of the CSI being read.
    csi: Vec<u8>,
    /// An `ESC` (and what followed) not yet forwarded: a CSI is held back
    /// until its final byte says whether it is a REP.
    held: Vec<u8>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Ground,
    Esc,
    Csi,
    /// OSC, DCS, APC, PM, SOS: a string up to BEL or `ESC \`.
    Str,
    StrEsc,
}

impl RepExpander {
    /// `chunk` with every REP replaced by the repeats it asks for.
    pub fn feed(&mut self, chunk: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(chunk.len());
        for &b in chunk {
            match self.state {
                State::Ground => match b {
                    0x1b => {
                        self.flush_pending(&mut out);
                        self.state = State::Esc;
                        self.held.push(b);
                    }
                    0x00..=0x1f | 0x7f => {
                        self.flush_pending(&mut out);
                        out.push(b);
                    }
                    _ => {
                        out.push(b);
                        self.pending.push(b);
                        if utf8_complete(&self.pending) {
                            self.last = std::mem::take(&mut self.pending);
                        }
                    }
                },
                State::Esc => {
                    self.state = match b {
                        b'[' => {
                            self.held.push(b);
                            self.csi.clear();
                            State::Csi
                        }
                        other => {
                            out.append(&mut self.held);
                            out.push(other);
                            match other {
                                b']' | b'P' | b'_' | b'^' | b'X' => State::Str,
                                // ESC ( B and friends carry one more byte;
                                // close enough: no REP follows them.
                                _ => State::Ground,
                            }
                        }
                    };
                }
                State::Csi => match b {
                    0x20..=0x3f => {
                        self.held.push(b);
                        self.csi.push(b);
                    }
                    b'b' if self.csi.iter().all(u8::is_ascii_digit) => {
                        let n = if self.csi.is_empty() {
                            1
                        } else {
                            std::str::from_utf8(&self.csi)
                                .ok()
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(1)
                        }
                        .clamp(1, MAX_REPEAT);
                        self.held.clear();
                        for _ in 0..n {
                            out.extend_from_slice(&self.last);
                        }
                        self.state = State::Ground;
                    }
                    _ => {
                        out.append(&mut self.held);
                        out.push(b);
                        self.state = State::Ground;
                    }
                },
                State::Str => {
                    out.push(b);
                    match b {
                        0x07 => self.state = State::Ground,
                        0x1b => self.state = State::StrEsc,
                        _ => {}
                    }
                }
                State::StrEsc => {
                    out.push(b);
                    self.state = if b == b'\\' {
                        State::Ground
                    } else {
                        State::Str
                    };
                }
            }
        }
        out
    }

    fn flush_pending(&mut self, _out: &mut [u8]) {
        // A character cut short by a control byte was malformed; forget it.
        self.pending.clear();
    }
}

/// Whether `bytes` is one whole UTF-8 character.
fn utf8_complete(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(parts: &[&[u8]]) -> String {
        let mut e = RepExpander::default();
        let mut out = Vec::new();
        for p in parts {
            out.extend(e.feed(p));
        }
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn a_repeat_becomes_the_character_n_times() {
        assert_eq!(run(&[b"a\x1b[3b"]), "aaaa");
        assert_eq!(
            run(&["\u{2500}\x1b[2b|".as_bytes()]),
            "\u{2500}\u{2500}\u{2500}|"
        );
    }

    #[test]
    fn a_repeat_split_across_reads_still_expands() {
        assert_eq!(run(&[b"x\x1b[", b"2", b"b"]), "xxx");
        assert_eq!(
            run(&["\u{2500}".as_bytes(), b"\x1b[1b"]),
            "\u{2500}\u{2500}"
        );
    }

    #[test]
    fn other_sequences_and_strings_pass_through_untouched() {
        let raw: &[u8] = b"\x1b[1;5Hz\x1b]0;title\x07\x1b[38;2;1;2;3mq";
        assert_eq!(run(&[raw]).as_bytes(), raw);
        // The payload of a string is not text to repeat.
        assert_eq!(run(&[b"a\x1b]0;zzz\x07\x1b[2b"]), "a\x1b]0;zzz\x07aa");
    }
}
