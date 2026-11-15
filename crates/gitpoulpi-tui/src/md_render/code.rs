//! Highlighted code blocks, over the editor's syntect grammars.

use std::time::Duration;

use ratatui::style::Style;

use crate::editor_highlight::Highlight;

/// Longest time spent colouring one block; the rest is drawn plain.
const BUDGET: Duration = Duration::from_millis(40);

/// A line of a block as `(text, style)` runs; `None` is the body style.
pub(super) type Runs = Vec<(String, Option<Style>)>;

/// The language word of a fence info string (`rust`, `rust,no_run`,
/// `{.py}` -> `rust`, `rust`, `py`).
pub(super) fn language(info: &str) -> &str {
    let word = info
        .split(|c: char| c.is_whitespace() || c == ',')
        .next()
        .unwrap_or("");
    word.trim_matches(|c| c == '{' || c == '}' || c == '.')
}

/// Each of `lines` (tabs already expanded) as coloured runs.
pub(super) fn highlight(lang: &str, lines: &[String]) -> Vec<Runs> {
    let plain = |l: &String| vec![(l.clone(), None)];
    let mut hl = Highlight::for_token(lang);
    let done = hl.advance(|i| lines[i].as_str(), lines.len(), lines.len(), BUDGET);
    let _ = done;
    lines
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let Some(runs) = hl.line(i) else {
                return plain(text);
            };
            let mut out: Runs = Vec::new();
            let mut at = 0;
            for run in runs {
                let end = (at + run.len).min(text.len());
                let Some(piece) = text.get(at..end) else {
                    return plain(text);
                };
                if !piece.is_empty() {
                    out.push((piece.to_string(), Some(run.style)));
                }
                at = end;
            }
            if at < text.len() {
                out.push((text[at..].to_string(), None));
            }
            if out.is_empty() {
                out.push((String::new(), None));
            }
            out
        })
        .collect()
}
