//! Pure text handling for the terminal mode: the sentinel instruction,
//! the answer between the markers, first-run dialog detection.
//!
//! The prompt asks the agent to wrap its answer in two marker lines. The
//! prompt itself is echoed on the screen too, so a marker only counts when
//! it stands **alone on its line** (after the agent's bullet or gutter is
//! stripped): the instruction mentions the markers inside a sentence, never
//! on a line of their own.

/// The opening and closing marker for `nonce`.
pub fn markers(nonce: &str) -> (String, String) {
    (format!("<<GP-{nonce}>>"), format!("<<GP-END-{nonce}>>"))
}

/// The sentence appended to the prompt. The markers sit mid-sentence on
/// purpose (see the module doc).
pub fn instruction(nonce: &str) -> String {
    let (open, close) = markers(nonce);
    format!(
        "Reply with your complete answer only, framed by two marker lines: first the line {open} \
         on its own, then the answer, then the line {close} on its own. Write nothing outside \
         the two markers, and do not use any tool."
    )
}

/// Characters an agent draws in front of its message or indents it with.
const GUTTER: &[char] = &[
    ' ', '\t', '\u{25cf}', '\u{2022}', '\u{2726}', '\u{25c6}', '\u{25cb}', '\u{2502}', '\u{2503}',
    '>', '\u{2514}', '\u{23bf}',
];

fn core(line: &str) -> &str {
    line.trim_start_matches(GUTTER).trim_end()
}

/// The text between the last standalone marker pair, with the agent's
/// indentation removed; `None` while the closing marker has not appeared.
pub fn answer(lines: &[String], nonce: &str) -> Option<String> {
    let (open, close) = markers(nonce);
    let end = lines.iter().rposition(|l| core(l) == close)?;
    let start = lines[..end].iter().rposition(|l| core(l) == open)?;
    let body: Vec<&str> = lines[start + 1..end].iter().map(String::as_str).collect();
    // Remove the gutter the agent indents every line with: the smallest
    // indentation of the non-blank lines, so relative indentation (code)
    // survives.
    let indent = body
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start_matches(' ').len())
        .min()
        .unwrap_or(0);
    let text = body
        .iter()
        .map(|l| l.get(indent.min(l.len())..).unwrap_or("").trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    let text = text.trim_matches('\n').to_string();
    Some(text)
}

/// Whether any of `markers` is on the screen.
pub fn any_marker(screen: &str, markers: &[&str]) -> bool {
    markers.iter().any(|m| screen.contains(m))
}

/// The first of `blockers` that is on the screen: a first-run dialog.
pub fn blocker<'a>(screen: &str, blockers: &[&'a str]) -> Option<&'a str> {
    blockers.iter().copied().find(|b| screen.contains(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_string).collect()
    }

    #[test]
    fn the_echoed_instruction_is_not_an_answer() {
        // Right after the paste: only the echo of the prompt is up.
        let screen = lines(&format!("> summarize this\n  {}\n", instruction("ab12")));
        assert_eq!(answer(&screen, "ab12"), None);
        // Wrapped at a marker, the marker still shares its line with words.
        let wrapped = instruction("ab12").replace(" on its own, then", "\n  on its own, then");
        assert_eq!(answer(&lines(&wrapped), "ab12"), None);
    }

    #[test]
    fn the_answer_is_the_text_between_the_last_marker_pair() {
        let screen = lines(
            "> prompt mentioning <<GP-n1>> and <<GP-END-n1>> inline\n\n\
             \u{25cf} <<GP-n1>>\n  feat: add thing\n\n  A body line.\n    indented code\n  <<GP-END-n1>>\n\n> ",
        );
        assert_eq!(
            answer(&screen, "n1").as_deref(),
            Some("feat: add thing\n\nA body line.\n  indented code")
        );
    }

    #[test]
    fn a_stale_pair_from_another_nonce_is_ignored() {
        let screen = lines("<<GP-old>>\nno\n<<GP-END-old>>\n");
        assert_eq!(answer(&screen, "new"), None);
    }

    #[test]
    fn an_open_marker_without_its_close_is_not_an_answer_yet() {
        let screen = lines("\u{2726} <<GP-n2>>\n  half an ans");
        assert_eq!(answer(&screen, "n2"), None);
    }

    #[test]
    fn the_last_pair_wins_when_the_agent_repeats_itself() {
        let screen =
            lines("<<GP-n3>>\nfirst\n<<GP-END-n3>>\nmore\n<<GP-n3>>\nsecond\n<<GP-END-n3>>");
        assert_eq!(answer(&screen, "n3").as_deref(), Some("second"));
    }

    #[test]
    fn first_run_dialogs_are_recognised() {
        let blockers = ["Do you trust the files in this folder?", "Select Theme"];
        assert_eq!(
            blocker("... Do you trust the files in this folder? ...", &blockers),
            Some("Do you trust the files in this folder?")
        );
        assert_eq!(blocker("Type your message", &blockers), None);
        assert!(any_marker("x esc to cancel y", &["esc to cancel"]));
    }
}
