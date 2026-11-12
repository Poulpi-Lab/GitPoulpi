//! Answers the questions a pty layer asks its terminal (cursor position,
//! device attributes, ...), so a child that waits for a reply starts.

/// A terminal is expected to answer questions, and a ConPTY asks two of
/// them the moment a session opens:
///
/// ```text
/// ESC[1t  ESC[6n  ESC[c  ESC[?1004h  ESC[?9001h
///         ^^^^^^  ^^^^^
///         DSR     Device Attributes
/// ```
///
/// `ESC[6n` is a cursor-position report request and `ESC[c` asks what
/// kind of terminal this is. **The child does not start until they are
/// answered** on the console hosts that enforce it (`microsoft/terminal#1810`,
/// which `portable-pty` does not handle). This is machine-dependent: a
/// Windows 11 desktop runs the child regardless, while on `windows-latest`
/// every child writes nothing at all, forever.
///
/// Answering is also simply *correct*, which is what makes it cfg-free:
/// this module emulates a terminal, and terminals reply to DSR and DA. A
/// child that never asks pays nothing. A canned reply injected after spawn
/// would put bytes no real terminal sends unprompted into the child's stdin.
///
/// Returns the bytes to send back, if any.
pub fn answer_queries(
    haystack: &[u8],
    from: usize,
    cursor: (u16, u16),
    kitty: bool,
    background: Option<&str>,
) -> Vec<u8> {
    answer_queries_with(
        haystack,
        from,
        cursor,
        &Replies {
            kitty,
            background,
            ..Replies::default()
        },
    )
}

/// What the emulated terminal answers beyond the always-answered cursor
/// position and device attributes.
#[derive(Debug, Clone, Copy, Default)]
pub struct Replies<'a> {
    /// Answer the kitty keyboard protocol's flags query.
    pub kitty: bool,
    /// The `rgb:rrrr/gggg/bbbb` body to answer an OSC 11 query with.
    pub background: Option<&'a str>,
    /// The parameters of the primary device attributes reply, after the
    /// `?` (`"62;4;22"` is a VT220 with sixel); `None` is `"1;2"`.
    pub attributes: Option<&'a str>,
    /// The text of the XTVERSION reply (`"XTerm(388)"`); `None` stays silent,
    /// as a terminal that does not know the query does.
    pub version: Option<&'a str>,
}

/// [`answer_queries`] with every optional answer in one place.
pub fn answer_queries_with(
    haystack: &[u8],
    from: usize,
    cursor: (u16, u16),
    replies: &Replies<'_>,
) -> Vec<u8> {
    /// The OSC 11 background-color query (ST-terminated, as the child sends it).
    const OSC11: &[u8] = b"\x1b]11;?";
    /// The kitty keyboard protocol's flags query; the answer is the
    /// current flags (none pushed yet), sent *before* the device
    /// attributes reply that follows the query as its sentinel.
    const KITTY: &[u8] = b"\x1b[?u";
    /// XTVERSION: the terminal names itself in a DCS string.
    const XTVERSION: &[u8] = b"\x1b[>0q";
    /// Cursor position report. The wire protocol is one-based.
    const DSR: &[u8] = b"[6n";
    /// Device attributes. `ESC[?1;2c` is "VT100 with advanced video
    /// option" - the reply a great many emulators give, and enough for
    /// anything that only wants to know someone is home.
    const DA: &[u8] = b"[c";

    let mut reply = Vec::new();
    let mut queries = vec![(DSR, format!("\x1b[{};{}R", cursor.0 + 1, cursor.1 + 1))];
    if replies.kitty {
        queries.push((KITTY, "\x1b[?0u".to_string()));
    }
    if let Some(rgb) = replies.background {
        queries.push((OSC11, format!("\x1b]11;{rgb}\x1b\\")));
    }
    if let Some(name) = replies.version {
        queries.push((XTVERSION, format!("\x1bP>|{name}\x1b\\")));
    }
    queries.push((
        DA,
        format!("\x1b[?{}c", replies.attributes.unwrap_or("1;2")),
    ));
    for (needle, answer) in queries {
        let mut at = 0;
        while let Some(found) = find(&haystack[at..], needle) {
            let end = at + found + needle.len();
            // Only matches that *end* in the new bytes: anything ending
            // earlier lay wholly inside the carried tail and was
            // answered on the previous chunk.
            if end > from {
                reply.extend_from_slice(answer.as_bytes());
            }
            at += found + 1;
        }
    }
    reply
}

/// First index of `needle` in `haystack`. `slice::windows` rather than a
/// dependency: the needles here are four bytes long.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// How much of the previous chunk to carry forward when scanning for
/// queries, so one split across a read boundary is still seen. The
/// longest needle is four bytes.
pub const QUERY_CARRY: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;

    /// Exactly what a ConPTY writes when a session opens, captured from
    /// a Windows 11 host. Two questions in it, and a child that CI's
    /// `windows-latest` runner will not start until both are answered.
    const CONPTY_PREAMBLE: &[u8] = b"\x1b[1t\x1b[6n\x1b[c\x1b[?1004h\x1b[?9001h";

    #[test]
    fn the_conpty_preamble_is_answered() {
        let reply = answer_queries(CONPTY_PREAMBLE, 0, (0, 0), false, None);
        let reply = String::from_utf8(reply).unwrap();
        assert!(reply.contains("\x1b[1;1R"), "cursor position: {reply:?}");
        assert!(reply.contains("\x1b[?1;2c"), "device attributes: {reply:?}");
    }

    /// The cursor report has to carry the *actual* cursor, one-based --
    /// a program that positions itself from the reply would land in the
    /// wrong cell otherwise.
    #[test]
    fn the_cursor_report_is_one_based_and_current() {
        let reply = answer_queries(b"\x1b[6n", 0, (4, 9), false, None);
        assert_eq!(String::from_utf8(reply).unwrap(), "\x1b[5;10R");
    }

    /// Nothing asked, nothing said. A reply nobody requested is input
    /// the child never expected.
    #[test]
    fn ordinary_output_is_not_answered() {
        assert!(answer_queries(b"just some text\r\n", 0, (0, 0), false, None).is_empty());
    }

    /// A query split across two reads must still be answered - the
    /// reader's chunk boundaries are the kernel's business, not the
    /// child's.
    #[test]
    fn a_query_split_across_reads_is_still_answered() {
        // First read ends mid-sequence, so it carries no complete query.
        assert!(answer_queries(b"\x1b[", 0, (0, 0), false, None).is_empty());
        // Second read: the carried tail plus the rest. `from` is the
        // length of the tail, so the match ends in the new bytes.
        let reply = answer_queries(b"\x1b[6n", 2, (0, 0), false, None);
        assert_eq!(String::from_utf8(reply).unwrap(), "\x1b[1;1R");
    }

    /// And it must not be answered *twice*: the tail is rescanned on
    /// every chunk, so a match lying wholly inside it is old news.
    /// Doubling the reply would put bytes the child never asked for
    /// into its input.
    #[test]
    fn a_query_already_answered_is_not_answered_again() {
        let carried = b"\x1b[6nabc";
        assert!(
            answer_queries(carried, carried.len(), (0, 0), false, None).is_empty(),
            "a query ending inside the carried tail was already answered"
        );
    }

    /// A terminal that was told what its background is answers the OSC 11
    /// query, ahead of the device-attributes reply the child uses as its
    /// sentinel; one that was not stays silent.
    #[test]
    fn the_background_query_is_answered_before_the_sentinel() {
        let query = b"\x1b]11;?\x1b\\\x1b[c";
        let reply = answer_queries(query, 0, (0, 0), false, Some("rgb:ffff/ffff/ffff"));
        assert_eq!(
            String::from_utf8(reply).unwrap(),
            "\x1b]11;rgb:ffff/ffff/ffff\x1b\\\x1b[?1;2c"
        );
        let silent = answer_queries(query, 0, (0, 0), false, None);
        assert_eq!(String::from_utf8(silent).unwrap(), "\x1b[?1;2c");
    }

    #[test]
    fn the_attributes_and_the_version_are_answered_when_configured() {
        let query = b"\x1b[?u\x1b]11;?\x1b\\\x1b[>0q\x1b[c";
        let replies = Replies {
            attributes: Some("62;4;22"),
            version: Some("XTerm(388)"),
            ..Replies::default()
        };
        let reply = answer_queries_with(query, 0, (0, 0), &replies);
        assert_eq!(
            String::from_utf8(reply).unwrap(),
            "\x1bP>|XTerm(388)\x1b\\\x1b[?62;4;22c"
        );
    }

    #[test]
    fn without_configuration_the_version_stays_silent_and_the_attributes_are_plain() {
        let reply = answer_queries_with(b"\x1b[>0q\x1b[c", 0, (0, 0), &Replies::default());
        assert_eq!(String::from_utf8(reply).unwrap(), "\x1b[?1;2c");
    }
}
