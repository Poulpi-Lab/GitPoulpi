//! Syntax highlighting for the built-in editor, over
//! `syntect` and the Sublime grammars it ships compiled in.
//!
//! # Why syntect, and which build of it
//!
//! Nothing else in the tree highlights. `syntect` is the crate `bat`,
//! `delta` and most Rust TUIs use: grammars for every mainstream language in
//! one dependency, no grammar compilation step, and a line-at-a-time API
//! whose parse state can be cloned, which is what incremental highlighting
//! needs. `tree-sitter` would mean one C grammar crate per language. It is
//! built with `regex-fancy` (pure Rust) rather than the default Oniguruma
//! binding, so the Windows build needs no C toolchain for it (see the
//! workspace `Cargo.toml`).
//!
//! # Only what is on screen, and never twice
//!
//! Highlighting is stateful: how line 900 colors depends on every line
//! above it (an open block comment, a string). [`Highlight`] therefore
//! keeps, per document:
//!
//! - the spans of every line already highlighted, from line 0 down to
//!   the furthest line ever shown -- nothing past it is ever parsed;
//! - the parser state at the start of every [`STRIDE`]th line (a
//!   checkpoint), so an edit on line `L` throws away only what is at or
//!   below the checkpoint before `L` and re-parses from there -- at most
//!   `STRIDE` lines above the view plus the view itself, per keystroke.
//!
//! [`Highlight::advance`] parses under a time budget: jumping to the end
//! of a very long file fills the gap over a few frames (drawing those
//! lines plain meanwhile) instead of freezing input. The event loop
//! redraws on its own tick, so the rest arrives unasked.
//!
//! The `SyntaxSet` and the theme are loaded once per process
//! (`OnceLock`): loading the default set takes tens of milliseconds.

use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use ratatui::style::{Color, Modifier, Style};
use syntect::highlighting::{
    FontStyle, HighlightIterator, HighlightState, Highlighter, Theme, ThemeSet,
};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

use crate::theme::{th, theme_generation};

/// Lines between two parser checkpoints.
pub(crate) const STRIDE: usize = 16;

/// Lines longer than this are drawn plain: a regex grammar over a
/// minified one-megabyte line is where highlighters hang.
const MAX_LINE_BYTES: usize = 4096;

fn syntaxes() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// The syntect theme the fallback uses when a theme names an unknown one.
const FALLBACK_THEME: &str = "base16-ocean.dark";

fn syntax_themes() -> &'static ThemeSet {
    static SET: OnceLock<ThemeSet> = OnceLock::new();
    SET.get_or_init(|| {
        let mut set = ThemeSet::load_defaults();
        set.themes.insert(EMBER.to_string(), ember_theme());
        set
    })
}

/// The name presets give `syntax` for [`ember_theme`].
const EMBER: &str = "GitPoulpi Ember"; // i18n-english

/// "GitPoulpi Ember", an original syntax theme for presets no bundled one
/// can be read on: Fire Light's orange background. Every color is dark.
fn ember_theme() -> Theme {
    use std::str::FromStr;
    use syntect::highlighting::{Color, ScopeSelectors, StyleModifier, ThemeItem, ThemeSettings};
    let rgb = |v: u32| Color {
        r: (v >> 16) as u8,
        g: (v >> 8) as u8,
        b: v as u8,
        a: 0xff,
    };
    let rules: [(&str, u32, FontStyle); 13] = [
        ("comment", 0x5a2a00, FontStyle::ITALIC),
        ("string", 0x1f4000, FontStyle::empty()),
        ("constant.numeric", 0x4a0a6b, FontStyle::empty()),
        (
            "constant.language, constant.character, constant.other",
            0x5a0a3a,
            FontStyle::empty(),
        ),
        ("keyword, storage", 0x7a0a12, FontStyle::BOLD),
        (
            "entity.name.type, entity.name.class, support.type, support.class, storage.type",
            0x5a0040,
            FontStyle::empty(),
        ),
        (
            "entity.name.function, support.function, meta.function-call",
            0x002a6b,
            FontStyle::empty(),
        ),
        ("variable, variable.parameter", 0x2a0c00, FontStyle::empty()),
        (
            "entity.name.tag, markup.heading",
            0x7a0a12,
            FontStyle::empty(),
        ),
        ("entity.other.attribute-name", 0x3a2a00, FontStyle::empty()),
        (
            "support.macro, entity.name.function.macro",
            0x003a3c,
            FontStyle::empty(),
        ),
        ("invalid, markup.deleted", 0x9c0a0a, FontStyle::empty()),
        ("markup.inserted", 0x1f4000, FontStyle::empty()),
    ];
    Theme {
        name: Some(EMBER.to_string()),
        author: None,
        settings: ThemeSettings {
            foreground: Some(rgb(0x2a0c00)),
            background: Some(rgb(0xffd0a2)),
            caret: Some(rgb(0x2a0c00)),
            selection: Some(rgb(0xfff07a)),
            line_highlight: Some(rgb(0xffdcb8)),
            ..ThemeSettings::default()
        },
        scopes: rules
            .iter()
            .filter_map(|&(scope, fg, font_style)| {
                Some(ThemeItem {
                    scope: ScopeSelectors::from_str(scope).ok()?,
                    style: StyleModifier {
                        foreground: Some(rgb(fg)),
                        background: None,
                        font_style: Some(font_style),
                    },
                })
            })
            .collect(),
    }
}

/// The syntect theme called `name` (from the defaults compiled in), else
/// the fallback, else an empty one.
fn theme_named(name: &str) -> &'static Theme {
    static EMPTY: OnceLock<Theme> = OnceLock::new();
    let set = syntax_themes();
    set.themes
        .get(name)
        .or_else(|| set.themes.get(FALLBACK_THEME))
        .unwrap_or_else(|| EMPTY.get_or_init(Theme::default))
}

/// A run of bytes of one line, in one style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Run {
    pub(crate) len: usize,
    pub(crate) style: Style,
}

type State = (ParseState, HighlightState);

/// One document's highlighting cache. See the module doc.
pub(crate) struct Highlight {
    syntax: Option<&'static SyntaxReference>,
    highlighter: Highlighter<'static>,
    /// `theme_generation()` the cached runs were colored under.
    generation: u64,
    /// `checkpoints[k]` is the state at the start of line `k * STRIDE`.
    checkpoints: Vec<State>,
    /// The runs of lines `0..lines.len()`.
    lines: Vec<Vec<Run>>,
    /// The state at the start of line `lines.len()`.
    state: Option<State>,
    /// What an edit threw away, kept as the best guess for lines not
    /// re-parsed yet: the runs of lines `stale_from..`. Shown only below
    /// `lines`; the person typing sees colors stay put while the parse
    /// catches up.
    stale: Vec<Vec<Run>>,
    stale_from: usize,
    /// A window parsed cold, far below `lines` ([`Highlight::advance_window`]):
    /// the runs of lines `window_from..`, and the state after the last.
    window: Vec<Vec<Run>>,
    window_from: usize,
    window_state: Option<State>,
    /// `window_marks[k]` is the state at the start of window line
    /// `k * WINDOW_STRIDE`, so an edit inside the window re-parses a few
    /// lines, not the window.
    window_marks: Vec<State>,
}

/// Lines between two checkpoints inside the cold window.
const WINDOW_STRIDE: usize = 8;

/// A gap this long (or shorter) below the parsed lines is parsed exactly,
/// line by line; a longer one is crossed with a cold window.
const JUMP: usize = 400;
/// Lines parsed above the first wanted line of a cold window, so a string
/// or comment opened just above it is usually seen.
const CONTEXT: usize = 40;

impl Highlight {
    /// Picks the grammar from the file name (extension, then the whole
    /// name for `Makefile` and friends), then from the first line
    /// (`#!/bin/sh`). `None` of those: plain text, nothing parsed.
    pub(crate) fn for_file(path: &Path, first_line: &str) -> Highlight {
        let set = syntaxes();
        let by_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(|e| set.find_syntax_by_extension(e));
        let by_name = || {
            path.file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| set.find_syntax_by_extension(n))
        };
        let syntax = by_ext
            .or_else(by_name)
            .or_else(|| set.find_syntax_by_first_line(first_line))
            .filter(|s| s.name != "Plain Text");
        Highlight::with_syntax(syntax)
    }

    /// The grammar a Markdown fence names (`rust`, `py`, `sh`), plain text
    /// when it is unknown or empty.
    pub(crate) fn for_token(token: &str) -> Highlight {
        let token = token.trim();
        let syntax = (!token.is_empty())
            .then(|| syntaxes().find_syntax_by_token(token))
            .flatten()
            .filter(|s| s.name != "Plain Text");
        Highlight::with_syntax(syntax)
    }

    fn with_syntax(syntax: Option<&'static SyntaxReference>) -> Highlight {
        let highlighter = Highlighter::new(theme_named(th().syntax));
        let state = syntax.map(|s| {
            (
                ParseState::new(s),
                HighlightState::new(&highlighter, ScopeStack::new()),
            )
        });
        Highlight {
            syntax,
            highlighter,
            generation: theme_generation(),
            checkpoints: Vec::new(),
            lines: Vec::new(),
            state,
            stale: Vec::new(),
            stale_from: 0,
            window: Vec::new(),
            window_from: 0,
            window_state: None,
            window_marks: Vec::new(),
        }
    }

    /// When the theme changed since the runs were cached, drops every
    /// cached run and rebuilds the highlighter under the new syntect theme:
    /// the next `advance` re-highlights from the top.
    fn sync_theme(&mut self) {
        let generation = theme_generation();
        if self.generation != generation {
            *self = Highlight::with_syntax(self.syntax);
            self.generation = generation;
        }
    }

    /// Pays the grammar's lazy compile (tens of milliseconds, on the first
    /// parse of a language) now, on whatever thread calls this, so the first
    /// frame does not.
    pub(crate) fn warm(&self) {
        if let Some(syntax) = self.syntax {
            let mut parse = ParseState::new(syntax);
            let _ = parse.parse_line("fn main() { let a = \"b\"; }\n", syntaxes());
        }
    }

    /// The grammar's name, for the status bar.
    pub(crate) fn name(&self) -> &str {
        self.syntax
            .map_or(gitpoulpi_i18n::msg::editor::PLAIN_TEXT.get(), |s| {
                s.name.as_str()
            })
    }

    /// The runs of `line`, when it has been highlighted. `None` for a
    /// plain-text file and for a line not reached yet.
    pub(crate) fn line(&self, line: usize) -> Option<&[Run]> {
        if self.generation != theme_generation() {
            return None;
        }
        if let Some(runs) = self.lines.get(line) {
            return Some(runs);
        }
        fn at(start: usize, runs: &[Vec<Run>], line: usize) -> Option<&[Run]> {
            line.checked_sub(start)
                .and_then(|k| runs.get(k))
                .map(Vec::as_slice)
        }
        at(self.window_from, &self.window, line).or_else(|| at(self.stale_from, &self.stale, line))
    }

    /// `delta` lines were inserted (or, negative, removed) after line
    /// `line`: the stale guesses below it move with them.
    pub(crate) fn shift_stale(&mut self, line: usize, delta: isize) {
        if self.stale.is_empty() || delta == 0 {
            return;
        }
        let at = (line + 1).saturating_sub(self.stale_from);
        if at >= self.stale.len() {
            return;
        }
        if delta > 0 {
            let fill = vec![Vec::new(); delta as usize];
            self.stale.splice(at..at, fill);
        } else {
            let end = (at + delta.unsigned_abs()).min(self.stale.len());
            self.stale.drain(at..end);
        }
    }

    #[cfg(test)]
    pub(crate) fn window_span(&self) -> (usize, usize) {
        (self.window_from, self.window.len())
    }

    /// Lines highlighted so far.
    #[cfg(test)]
    pub(crate) fn highlighted(&self) -> usize {
        self.lines.len()
    }

    /// Line `line` changed (and everything after it may have moved):
    /// drops what was derived from it, back to the checkpoint before it.
    pub(crate) fn invalidate_from(&mut self, line: usize) {
        if self.syntax.is_none() {
            return;
        }
        if line >= self.window_from && line < self.window_from + self.window.len() {
            let k = (line - self.window_from) / WINDOW_STRIDE;
            let cut = k * WINDOW_STRIDE;
            let tail = self.window.split_off(cut);
            self.window_marks.truncate(k + 1);
            self.window_state = self.window_marks.get(k).cloned();
            self.keep_stale(self.window_from + cut, tail);
        } else if line < self.window_from {
            // The window sits below the edit: its colors are a guess now.
            let tail = std::mem::take(&mut self.window);
            self.keep_stale(self.window_from, tail);
            self.window_state = None;
            self.window_marks.clear();
        }
        if line >= self.lines.len() {
            return;
        }
        let k = line / STRIDE;
        self.checkpoints.truncate(k + 1);
        // What was parsed below the checkpoint becomes the stale guess.
        let boundary = k * STRIDE;
        let tail = self.lines.split_off(boundary);
        self.keep_stale(boundary, tail);
        self.state = self.checkpoints.get(k).cloned();
    }

    /// `tail` (the runs of lines `boundary..`) becomes the stale guess,
    /// followed by any older guess that reached further down.
    fn keep_stale(&mut self, boundary: usize, mut tail: Vec<Vec<Run>>) {
        let old_end = self.stale_from + self.stale.len();
        if old_end > boundary + tail.len() && self.stale_from <= boundary + tail.len() {
            let skip = boundary + tail.len() - self.stale_from;
            tail.extend(self.stale.drain(skip..));
        }
        self.stale = tail;
        self.stale_from = boundary;
    }

    /// Like [`Highlight::advance`] for lines `from..to` being on screen: a
    /// gap of more than [`JUMP`] lines between what is parsed and `from`
    /// is not parsed through (a few thousand lines is seconds of regex
    /// work). A window starting [`CONTEXT`] lines above `from` is parsed
    /// cold instead, and kept while the view moves down from it. The
    /// colors are right unless a string or comment opened above the
    /// window and runs into it. Returns whether `to` is covered.
    pub(crate) fn advance_window<'a>(
        &mut self,
        line_text: impl Fn(usize) -> &'a str,
        count: usize,
        from: usize,
        to: usize,
        budget: Duration,
    ) -> bool {
        self.sync_theme();
        let to = to.min(count);
        if self.syntax.is_none() || to <= self.lines.len() {
            return true;
        }
        let start = from.saturating_sub(CONTEXT);
        if start <= self.lines.len() + JUMP {
            return self.advance(line_text, count, to, budget);
        }
        let end = self.window_from + self.window.len();
        let resumable =
            self.window_state.is_some() && self.window_from <= from && from <= end + CONTEXT;
        if !resumable {
            self.window.clear();
            self.window_marks.clear();
            self.window_from = start;
            self.window_state = self.syntax.map(|s| {
                (
                    ParseState::new(s),
                    HighlightState::new(&self.highlighter, ScopeStack::new()),
                )
            });
        }
        let set = syntaxes();
        let started = Instant::now();
        let mut buf = String::new();
        while self.window_from + self.window.len() < to {
            let i = self.window_from + self.window.len();
            if i.is_multiple_of(8) && started.elapsed() >= budget {
                return false;
            }
            let Some((parse, hl)) = self.window_state.as_mut() else {
                return true;
            };
            let at = i - self.window_from;
            if at.is_multiple_of(WINDOW_STRIDE) && self.window_marks.len() == at / WINDOW_STRIDE {
                self.window_marks.push((parse.clone(), hl.clone()));
            }
            let runs = parse_line(parse, hl, &self.highlighter, set, line_text(i), &mut buf);
            self.window.push(runs);
        }
        true
    }

    /// Highlights forward until lines `0..upto` are done (`count` being
    /// how many lines the document has) or `budget` has run out.
    /// Returns whether `upto` was reached.
    pub(crate) fn advance<'a>(
        &mut self,
        line_text: impl Fn(usize) -> &'a str,
        count: usize,
        upto: usize,
        budget: Duration,
    ) -> bool {
        self.sync_theme();
        let upto = upto.min(count);
        if self.syntax.is_none() {
            return true;
        }
        let set = syntaxes();
        let started = Instant::now();
        let mut buf = String::new();
        while self.lines.len() < upto {
            let i = self.lines.len();
            // Checked every few lines: `Instant::now` is not free.
            if i.is_multiple_of(8) && started.elapsed() >= budget {
                return false;
            }
            let Some((parse, hl)) = self.state.as_mut() else {
                return true;
            };
            if i.is_multiple_of(STRIDE) && self.checkpoints.len() == i / STRIDE {
                self.checkpoints.push((parse.clone(), hl.clone()));
            }
            let runs = parse_line(parse, hl, &self.highlighter, set, line_text(i), &mut buf);
            self.lines.push(runs);
        }
        true
    }
}

/// Parses one line (without its break) and returns its runs.
fn parse_line(
    parse: &mut ParseState,
    hl: &mut HighlightState,
    highlighter: &Highlighter<'static>,
    set: &SyntaxSet,
    text: &str,
    buf: &mut String,
) -> Vec<Run> {
    if text.len() > MAX_LINE_BYTES {
        return vec![Run {
            len: text.len(),
            style: Style::default(),
        }];
    }
    // The `_newlines` grammars expect each line with its break.
    buf.clear();
    buf.push_str(text);
    buf.push('\n');
    match parse.parse_line(buf, set) {
        Ok(ops) => {
            let mut runs: Vec<Run> = Vec::new();
            let mut used = 0;
            for (style, piece) in HighlightIterator::new(hl, &ops, buf, highlighter) {
                let len = piece.len().min(text.len() - used);
                if len == 0 {
                    continue;
                }
                used += len;
                let style = to_style(style);
                match runs.last_mut() {
                    Some(last) if last.style == style => last.len += len,
                    _ => runs.push(Run { len, style }),
                }
            }
            runs
        }
        Err(_) => vec![Run {
            len: text.len(),
            style: Style::default(),
        }],
    }
}

/// A syntect style as a ratatui one: the foreground and the font style.
/// The theme's background is left out on purpose - the editor draws on
/// the app's own background, like every other pane.
fn to_style(s: syntect::highlighting::Style) -> Style {
    let fg = s.foreground;
    let mut style = Style::default().fg(Color::Rgb(fg.r, fg.g, fg.b));
    if s.font_style.contains(FontStyle::BOLD) {
        style = style.add_modifier(Modifier::BOLD);
    }
    if s.font_style.contains(FontStyle::ITALIC) {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if s.font_style.contains(FontStyle::UNDERLINE) {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

#[cfg(test)]
mod tests {
    use super::*;

    const FOREVER: Duration = Duration::from_secs(60);

    fn lines(text: &str) -> Vec<&str> {
        text.split('\n').collect()
    }

    #[test]
    fn the_grammar_comes_from_the_extension_the_name_or_the_first_line() {
        assert_eq!(
            Highlight::for_file(Path::new("src/a.rs"), "").name(),
            "Rust"
        );
        assert_eq!(
            Highlight::for_file(Path::new("Makefile"), "").name(),
            "Makefile"
        );
        assert_eq!(
            Highlight::for_file(Path::new("run"), "#!/bin/bash").name(),
            "Bourne Again Shell (bash)"
        );
        assert_eq!(
            Highlight::for_file(Path::new("notes.unknownext"), "hello").name(),
            "Plain Text"
        );
    }

    #[test]
    fn a_theme_change_re_highlights_the_cached_lines() {
        use crate::theme::{override_theme, Theme};
        let ls = lines("fn main() {}");
        let mut hl = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(hl.advance(|i| ls[i], ls.len(), ls.len(), FOREVER));
        let dark_runs = hl.line(0).unwrap().to_vec();
        // Same theme again: the cache is kept as is.
        assert_eq!(hl.highlighted(), 1);
        let mut light = Theme::dark();
        light.syntax = "InspiredGitHub";
        let _g = override_theme(&light);
        assert!(hl.line(0).is_none(), "stale colors are never served");
        assert!(hl.advance(|i| ls[i], ls.len(), ls.len(), FOREVER));
        let light_runs = hl.line(0).unwrap();
        assert_ne!(
            light_runs,
            dark_runs.as_slice(),
            "recolored by the new theme"
        );
    }

    /// The syntect theme a preset names has the preset's lightness: a dark
    /// syntect theme on a light background (or the reverse) gives text that
    /// cannot be read.
    #[test]
    fn every_preset_names_a_syntax_theme_of_its_own_lightness() {
        let mut bad = Vec::new();
        for spec in gitpoulpi_config::theme::presets() {
            let Some(theme) = syntax_themes().themes.get(spec.syntax.as_str()) else {
                bad.push(format!(
                    "{}: unknown syntect theme {:?}",
                    spec.slug, spec.syntax
                ));
                continue;
            };
            let fg = theme.settings.foreground.expect("a foreground");
            let lum = |c: syntect::highlighting::Color| {
                0.299 * f64::from(c.r) + 0.587 * f64::from(c.g) + 0.114 * f64::from(c.b)
            };
            // The text color is dark on a light theme, light on a dark one.
            if (lum(fg) < 128.0) != spec.light {
                bad.push(format!(
                    "{}: {:?} text color {fg:?}",
                    spec.slug, spec.syntax
                ));
            }
        }
        assert!(bad.is_empty(), "{bad:#?}");
    }

    #[test]
    fn a_keyword_and_a_name_get_different_colors() {
        let text = "fn main() {}";
        let ls = lines(text);
        let mut hl = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(hl.advance(|i| ls[i], ls.len(), ls.len(), FOREVER));
        let runs = hl.line(0).unwrap();
        assert_eq!(runs.iter().map(|r| r.len).sum::<usize>(), text.len());
        let style_at = |byte: usize| {
            let mut at = 0;
            runs.iter()
                .find(|r| {
                    at += r.len;
                    byte < at
                })
                .unwrap()
                .style
        };
        assert!(matches!(style_at(0).fg, Some(Color::Rgb(..))));
        assert_ne!(style_at(0).fg, style_at(3).fg, "`fn` vs `main`");
    }

    #[test]
    fn only_the_lines_asked_for_are_parsed_and_an_edit_rewinds_to_a_checkpoint() {
        let text: String = (0..200).map(|i| format!("let x{i} = {i};\n")).collect();
        let ls = lines(&text);
        let mut hl = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(hl.advance(|i| ls[i], ls.len(), 40, FOREVER));
        assert_eq!(hl.highlighted(), 40, "nothing past the view");
        assert!(hl.line(40).is_none());

        hl.invalidate_from(37);
        assert_eq!(hl.highlighted(), 32, "back to the checkpoint at 32");
        assert!(hl.advance(|i| ls[i], ls.len(), 40, FOREVER));
        assert_eq!(hl.highlighted(), 40);

        // An edit below what was highlighted costs nothing.
        hl.invalidate_from(150);
        assert_eq!(hl.highlighted(), 40);
    }

    #[test]
    fn a_block_comment_opened_above_colors_the_lines_below_after_an_edit() {
        let before = "let a = 1;\nlet b = 2;\n";
        let after = "/* let a = 1;\nlet b = 2;\n";
        let ls = lines(before);
        let mut hl = Highlight::for_file(Path::new("a.rs"), ls[0]);
        hl.advance(|i| ls[i], ls.len(), ls.len(), FOREVER);
        let plain_b = hl.line(1).unwrap().to_vec();
        let ls2 = lines(after);
        hl.invalidate_from(0);
        hl.advance(|i| ls2[i], ls2.len(), ls2.len(), FOREVER);
        let commented_b = hl.line(1).unwrap().to_vec();
        assert_ne!(plain_b, commented_b, "line 1 is inside the comment now");
        assert_eq!(commented_b.len(), 1, "one run: all comment");
    }

    #[test]
    fn a_zero_budget_stops_early_and_says_so() {
        let text: String = (0..100).map(|i| format!("x{i}\n")).collect();
        let ls = lines(&text);
        let mut hl = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(!hl.advance(|i| ls[i], ls.len(), 100, Duration::ZERO));
        assert!(hl.highlighted() < 100);
    }

    fn code(n: usize) -> String {
        (0..n).map(|i| format!("let x{i} = {i}; // c\n")).collect()
    }

    #[test]
    fn a_far_window_is_colored_without_parsing_everything_above_it() {
        let text = code(5000);
        let ls = lines(&text);
        let mut hl = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(hl.advance_window(|i| ls[i], ls.len(), 4000, 4030, FOREVER));
        assert_eq!(hl.highlighted(), 0, "nothing parsed from the top");
        assert!(hl.line(10).is_none());
        let far = hl.line(4010).expect("the window is colored").to_vec();
        // The same line, parsed the long way, gets the same colors.
        let mut exact = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(exact.advance(|i| ls[i], ls.len(), 4011, FOREVER));
        assert_eq!(far, exact.line(4010).unwrap());
        // A short gap is parsed exactly instead.
        let mut near = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(near.advance_window(|i| ls[i], ls.len(), 100, 130, FOREVER));
        assert_eq!(near.highlighted(), 130);
    }

    #[test]
    fn an_edit_keeps_the_old_colors_below_it_until_they_are_parsed_again() {
        let text = code(100);
        let ls = lines(&text);
        let mut hl = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(hl.advance(|i| ls[i], ls.len(), 60, FOREVER));
        let before = hl.line(50).unwrap().to_vec();
        hl.invalidate_from(20);
        assert_eq!(hl.highlighted(), 16);
        assert_eq!(hl.line(50), Some(before.as_slice()), "stale, not plain");
        // A line inserted at 20 moves the guesses below it down by one.
        hl.shift_stale(20, 1);
        assert_eq!(hl.line(51), Some(before.as_slice()));
        assert!(hl.advance(|i| ls[i], ls.len(), 60, FOREVER));
        assert_eq!(hl.line(50).unwrap(), hl.line(50).unwrap());
    }

    #[test]
    fn an_edit_inside_the_window_re_parses_from_its_checkpoint() {
        let text = code(5000);
        let ls = lines(&text);
        let mut hl = Highlight::for_file(Path::new("a.rs"), ls[0]);
        assert!(hl.advance_window(|i| ls[i], ls.len(), 4000, 4030, FOREVER));
        let (from, len) = hl.window_span();
        hl.invalidate_from(4020);
        let (from2, len2) = hl.window_span();
        assert_eq!(from2, from);
        assert!(len2 < len && len2 <= 4020 - from, "{len2} of {len}");
        assert!(hl.line(4025).is_some(), "the guess below stays");
        assert!(hl.advance_window(|i| ls[i], ls.len(), 4000, 4030, FOREVER));
        assert_eq!(hl.window_span().1, 4030 - from);
    }
}
