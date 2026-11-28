//! The diff pane's rows: the shared `gitpoulpi_app::diff` model (which hunks survive the whitespace
//! toggle, line numbers, Split pairing, changed words, the notices and the LFS card) turned into the
//! list the pane draws. Nothing here knows about the window; [`Look`] says how a span reads and
//! [`Look::style`] resolves it against the theme.

use gitpoulpi_app::diff::{
    effective_hunks, intraline, line_numbers, split_change, split_pairs, styled, DiffRole, DiffRow,
    RowChange, StyledLine, StyledSpan,
};
use gitpoulpi_core::domain::{lfs_change, FileDiff, LineOrigin};
use gitpoulpi_i18n::msg::diff_view as msg;

use crate::theme::DiffColors;

/// What the user chose for the pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opts {
    /// Side by side (`true`) or one column (`false`, "Inline").
    pub split: bool,
    /// The whitespace toggle: `false` folds whitespace-only changes into context.
    pub show_whitespace: bool,
    pub line_numbers: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            split: true,
            show_whitespace: true,
            line_numbers: true,
        }
    }
}

/// One logical row (no wrapping: that is layout).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// A header, notice, card line or an Inline line: the whole width.
    Full(StyledLine),
    /// A Side-by-side row: old half, new half (`None` is a blank half).
    Split(Option<StyledLine>, Option<StyledLine>),
}

/// The rows of one file.
#[derive(Clone, Debug, Default)]
pub struct Rows {
    pub lines: Vec<Line>,
    /// What each row stands for, for hunk lookups (same index as `lines`).
    pub meta: Vec<DiffRow>,
    /// Row index of every hunk header.
    pub hunk_starts: Vec<usize>,
    /// The longest text, in chars, of one half (Split) or one row (Inline): sizes the horizontal scroll.
    pub max_chars: usize,
}

/// Tabs to spaces (4 columns), so emphasis ranges and the painted text agree.
pub fn expand_tabs(text: &str) -> String {
    if !text.contains('\t') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() + 8);
    let mut col = 0usize;
    for c in text.chars() {
        if c == '\t' {
            let n = 4 - col % 4;
            out.extend(std::iter::repeat_n(' ', n));
            col += n;
        } else {
            out.push(c);
            col += 1;
        }
    }
    out
}

fn single(text: String, role: DiffRole) -> Rows {
    Rows {
        max_chars: text.chars().count(),
        lines: vec![Line::Full(StyledLine::single(text, role))],
        meta: vec![DiffRow::default()],
        hunk_starts: Vec::new(),
    }
}

/// Builds the rows of `file`: the LFS card, a notice when there is nothing to show, else every hunk
/// that survives the whitespace toggle under its `@@` header.
pub fn build(file: &FileDiff, opts: Opts) -> Rows {
    if let Some(change) = lfs_change(file) {
        let lines = styled::lfs_card(&change, None);
        let max = lines.iter().map(|l| l.plain().chars().count()).max();
        return Rows {
            meta: vec![DiffRow::default(); lines.len()],
            lines: lines.into_iter().map(Line::Full).collect(),
            hunk_starts: Vec::new(),
            max_chars: max.unwrap_or(0),
        };
    }
    if file.hunks.is_empty() {
        return single(msg::NOTICE_NO_CHANGES.get().to_string(), DiffRole::Notice);
    }
    let hunks = effective_hunks(file, opts.show_whitespace);
    if hunks.is_empty() {
        return single(
            msg::NOTICE_WHITESPACE_ONLY.get().to_string(),
            DiffRole::Notice,
        );
    }

    let mut rows = Rows::default();
    for eh in &hunks {
        let hunk = &eh.hunk;
        rows.hunk_starts.push(rows.lines.len());
        rows.lines.push(Line::Full(styled::hunk_header(hunk)));
        rows.meta.push(DiffRow::header(hunk.id));
        let texts: Vec<String> = hunk
            .lines
            .iter()
            .map(|l| expand_tabs(l.content.trim_end_matches(['\n', '\r'])))
            .collect();
        for t in &texts {
            rows.max_chars = rows.max_chars.max(t.chars().count());
        }
        // The emphasis is computed on the text as painted (tabs expanded).
        let emph = intraline::hunk_emphasis(&hunk.lines, |s| {
            expand_tabs(s.trim_end_matches(['\n', '\r']))
        });
        if opts.split {
            for (left, right) in split_pairs(hunk) {
                let half = |side: Option<(usize, u32)>| {
                    side.map(|(i, n)| {
                        styled::split_side_line(
                            &hunk.lines[i].origin,
                            &texts[i],
                            n,
                            &emph[i],
                            opts.line_numbers,
                        )
                    })
                };
                rows.lines.push(Line::Split(half(left), half(right)));
                let pick = |s: Option<(usize, u32)>| s.map(|(i, _)| eh.origin[i]);
                let mut row = DiffRow::body(hunk.id, [pick(left), pick(right)])
                    .with_change(split_change(hunk, left, right));
                row.numbers = [left.map(|(_, n)| n), right.map(|(_, n)| n)];
                rows.meta.push(row);
            }
        } else {
            let numbers = line_numbers(hunk);
            for (i, line) in hunk.lines.iter().enumerate() {
                rows.lines.push(Line::Full(styled::unified_line(
                    &line.origin,
                    &texts[i],
                    numbers[i],
                    &emph[i],
                    opts.line_numbers,
                )));
                let changed = line.origin != LineOrigin::Context;
                let mut row = DiffRow::body(hunk.id, [changed.then_some(eh.origin[i]), None])
                    .with_change(RowChange::of(&line.origin));
                row.numbers = [numbers[i].0, numbers[i].1];
                rows.meta.push(row);
            }
        }
    }
    rows
}

/// How a span reads in the desktop UI: the role of the shared model resolved to what the pane has a
/// colour for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    Add,
    AddEmph,
    Del,
    DelEmph,
    Context,
    Gutter,
    Hunk,
    Notice,
    Title,
    Muted,
}

impl Look {
    pub fn of(span: &StyledSpan) -> Look {
        match (span.role, span.emph) {
            (DiffRole::Add, true) => Look::AddEmph,
            (DiffRole::Add, false) => Look::Add,
            (DiffRole::Delete, true) => Look::DelEmph,
            (DiffRole::Delete, false) => Look::Del,
            (DiffRole::Context | DiffRole::Text, _) => Look::Context,
            (DiffRole::Gutter, _) => Look::Gutter,
            (DiffRole::HunkHeader, _) => Look::Hunk,
            (DiffRole::Notice, _) => Look::Notice,
            (DiffRole::Title, _) => Look::Title,
            (DiffRole::Muted, _) => Look::Muted,
        }
    }

    /// Text colour, background (a changed word's stronger one included) and weight / slant.
    pub fn style(self, c: &DiffColors) -> LookStyle {
        let plain = |fg| LookStyle {
            fg,
            bg: None,
            bold: false,
            italic: false,
        };
        match self {
            Look::Add => LookStyle {
                bg: Some(c.add_bg),
                ..plain(c.add_fg)
            },
            Look::AddEmph => LookStyle {
                bg: Some(c.add_emph_bg),
                ..plain(c.add_fg)
            },
            Look::Del => LookStyle {
                bg: Some(c.del_bg),
                ..plain(c.del_fg)
            },
            Look::DelEmph => LookStyle {
                bg: Some(c.del_emph_bg),
                ..plain(c.del_fg)
            },
            Look::Context => plain(c.text),
            Look::Gutter => plain(c.gutter_fg),
            Look::Hunk => LookStyle {
                bold: true,
                ..plain(c.hunk_fg)
            },
            Look::Notice => LookStyle {
                italic: true,
                ..plain(c.muted)
            },
            Look::Title => LookStyle {
                bold: true,
                ..plain(c.text)
            },
            Look::Muted => plain(c.muted),
        }
    }

    /// The background a whole row of this look carries (an added or removed line is tinted across the
    /// pane, not only under its text).
    pub fn row_bg(self, c: &DiffColors) -> Option<gpui_kit::Hsla> {
        match self {
            Look::Add | Look::AddEmph => Some(c.add_bg),
            Look::Del | Look::DelEmph => Some(c.del_bg),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LookStyle {
    pub fg: gpui_kit::Hsla,
    pub bg: Option<gpui_kit::Hsla>,
    pub bold: bool,
    pub italic: bool,
}

/// The look of a line's body (past the gutter): what tints the row.
pub fn body_look(line: &StyledLine) -> Option<Look> {
    line.spans
        .iter()
        .find(|s| s.role != DiffRole::Gutter)
        .map(Look::of)
}

/// A line as text with the looks of its runs, for the golden tests: `[gutter]    1 [add]+[add]foo`
/// with an emphasised run written `[add+]`.
#[cfg(test)]
pub fn describe(line: &StyledLine) -> String {
    let mut out = String::new();
    for s in &line.spans {
        let look = format!("{:?}", Look::of(s)).to_lowercase();
        let look = look.replace("emph", "+");
        out.push_str(&format!("[{look}]{}", s.text));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{DiffLine, FileStatus, Hunk, HunkId};
    use std::path::PathBuf;

    fn line(origin: LineOrigin, content: &str) -> DiffLine {
        DiffLine {
            origin,
            content: content.into(),
            staged: false,
        }
    }

    /// A hunk with a rewritten line (changed words), a removal, an addition and a whitespace-only edit.
    fn fixture() -> FileDiff {
        use LineOrigin::*;
        FileDiff {
            path: PathBuf::from("src/lib.rs"),
            status: FileStatus::Modified,
            renamed_from: None,
            similarity: None,
            hunks: vec![
                Hunk {
                    id: HunkId(0),
                    old_start: 10,
                    old_lines: 5,
                    new_start: 10,
                    new_lines: 5,
                    lines: vec![
                        line(Context, "fn main() {\n"),
                        line(Delete, "    let total = count + 1;\n"),
                        line(Add, "    let total = count + 2;\n"),
                        line(Delete, "    old();\n"),
                        line(Context, "}\n"),
                    ],
                },
                Hunk {
                    id: HunkId(1),
                    old_start: 40,
                    old_lines: 2,
                    new_start: 40,
                    new_lines: 2,
                    lines: vec![
                        line(Delete, "let x = 1;  \n"),
                        line(Add, "let x = 1;\n"),
                        line(Context, "end\n"),
                    ],
                },
            ],
        }
    }

    fn text(rows: &Rows) -> Vec<String> {
        rows.lines
            .iter()
            .map(|l| match l {
                Line::Full(l) => describe(l),
                Line::Split(a, b) => format!(
                    "{} || {}",
                    a.as_ref().map(describe).unwrap_or_else(|| "-".into()),
                    b.as_ref().map(describe).unwrap_or_else(|| "-".into())
                ),
            })
            .collect()
    }

    #[test]
    fn inline_rows_carry_line_numbers_markers_and_changed_words() {
        let rows = build(
            &fixture(),
            Opts {
                split: false,
                ..Opts::default()
            },
        );
        assert_eq!(
            text(&rows),
            vec![
                "[hunk]@@ -10,5 +10,5 @@",
                "[gutter]   10    10 [context] [context]fn main() {",
                "[gutter]   11       [del]-[del]    let total = count + [del+]1[del];",
                "[gutter]         11 [add]+[add]    let total = count + [add+]2[add];",
                "[gutter]   12       [del]-[del]    old();",
                "[gutter]   13    12 [context] [context]}",
                "[hunk]@@ -40,2 +40,2 @@",
                "[gutter]   40       [del]-[del]let x = 1;[del+]  ",
                "[gutter]         40 [add]+[add]let x = 1;",
                "[gutter]   41    41 [context] [context]end",
            ]
        );
        assert_eq!(rows.hunk_starts, vec![0, 6]);
    }

    #[test]
    fn split_rows_pair_deletions_with_additions() {
        let rows = build(&fixture(), Opts::default());
        assert_eq!(
            text(&rows),
            vec![
                "[hunk]@@ -10,5 +10,5 @@",
                "[gutter]   10 [context] [context]fn main() { || [gutter]   10 [context] [context]fn main() {",
                "[gutter]   11 [del]-[del]    let total = count + [del+]1[del]; || [gutter]   11 [add]+[add]    let total = count + [add+]2[add];",
                "[gutter]   12 [del]-[del]    old(); || -",
                "[gutter]   13 [context] [context]} || [gutter]   12 [context] [context]}",
                "[hunk]@@ -40,2 +40,2 @@",
                "[gutter]   40 [del]-[del]let x = 1;[del+]   || [gutter]   40 [add]+[add]let x = 1;",
                "[gutter]   41 [context] [context]end || [gutter]   41 [context] [context]end",
            ]
        );
    }

    #[test]
    fn turning_whitespace_off_folds_a_whitespace_only_change_into_context() {
        let rows = build(
            &fixture(),
            Opts {
                split: false,
                show_whitespace: false,
                ..Opts::default()
            },
        );
        // The second hunk has nothing left to show.
        assert_eq!(rows.hunk_starts, vec![0]);
        assert_eq!(rows.lines.len(), 6);
        let only_ws = FileDiff {
            hunks: vec![fixture().hunks[1].clone()],
            ..fixture()
        };
        let rows = build(
            &only_ws,
            Opts {
                show_whitespace: false,
                ..Opts::default()
            },
        );
        assert_eq!(
            text(&rows),
            vec![format!("[notice]{}", msg::NOTICE_WHITESPACE_ONLY.get())]
        );
    }

    #[test]
    fn a_file_without_hunks_says_so() {
        let file = FileDiff {
            hunks: vec![],
            ..fixture()
        };
        let rows = build(&file, Opts::default());
        assert_eq!(
            text(&rows),
            vec![format!("[notice]{}", msg::NOTICE_NO_CHANGES.get())]
        );
        assert!(rows.hunk_starts.is_empty());
    }

    #[test]
    fn line_numbers_can_be_turned_off_without_moving_the_text() {
        let on = build(
            &fixture(),
            Opts {
                split: false,
                ..Opts::default()
            },
        );
        let off = build(
            &fixture(),
            Opts {
                split: false,
                line_numbers: false,
                ..Opts::default()
            },
        );
        let (Line::Full(a), Line::Full(b)) = (&on.lines[1], &off.lines[1]) else {
            panic!("inline rows are whole lines");
        };
        assert_eq!(a.spans[0].text.len(), b.spans[0].text.len());
        assert!(b.spans[0].text.trim().is_empty());
    }

    #[test]
    fn tabs_expand_before_the_emphasis_is_computed() {
        assert_eq!(expand_tabs("a\tb"), "a   b");
        assert_eq!(expand_tabs("\t\tx"), "        x");
        let file = FileDiff {
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 1,
                old_lines: 1,
                new_start: 1,
                new_lines: 1,
                lines: vec![
                    line(LineOrigin::Delete, "\tfoo(1)\n"),
                    line(LineOrigin::Add, "\tfoo(2)\n"),
                ],
            }],
            ..fixture()
        };
        let rows = build(
            &file,
            Opts {
                split: false,
                ..Opts::default()
            },
        );
        let Line::Full(add) = &rows.lines[2] else {
            panic!()
        };
        let body: String = add.spans.iter().skip(2).map(|s| s.text.as_str()).collect();
        assert_eq!(body, "    foo(2)", "the tab is four columns");
        let emphasised: String = add
            .spans
            .iter()
            .filter(|s| s.emph)
            .map(|s| s.text.as_str())
            .collect();
        assert_eq!(emphasised, "2", "the range points at the expanded text");
    }

    #[test]
    fn the_row_map_points_back_at_the_stageable_lines() {
        let rows = build(
            &fixture(),
            Opts {
                split: false,
                ..Opts::default()
            },
        );
        // Row 2 is the first deletion: index 1 of hunk 0.
        assert_eq!(rows.meta[2].lines, [Some(1), None]);
        assert_eq!(gitpoulpi_app::diff::hunk_at(&rows.meta, 3), Some(HunkId(0)));
        assert_eq!(gitpoulpi_app::diff::hunk_at(&rows.meta, 8), Some(HunkId(1)));
        let (hunk, lines, spilled) = gitpoulpi_app::diff::selected_lines(&rows.meta, 2, 4).unwrap();
        assert_eq!((hunk, lines, spilled), (HunkId(0), vec![1, 2, 3], false));
        let (_, _, spilled) = gitpoulpi_app::diff::selected_lines(&rows.meta, 4, 8).unwrap();
        assert!(spilled, "a range over two hunks says so");
    }

    #[test]
    fn every_role_has_a_look_and_a_colour() {
        use gpui_kit::hsla;
        let c = DiffColors {
            text: hsla(0.0, 0.0, 0.9, 1.0),
            muted: hsla(0.0, 0.0, 0.5, 1.0),
            add_fg: hsla(0.33, 0.5, 0.6, 1.0),
            add_bg: hsla(0.33, 0.5, 0.2, 1.0),
            add_emph_bg: hsla(0.33, 0.5, 0.3, 1.0),
            del_fg: hsla(0.0, 0.5, 0.6, 1.0),
            del_bg: hsla(0.0, 0.5, 0.2, 1.0),
            del_emph_bg: hsla(0.0, 0.5, 0.3, 1.0),
            gutter_fg: hsla(0.0, 0.0, 0.4, 1.0),
            hunk_fg: hsla(0.6, 0.5, 0.6, 1.0),
        };
        let span = |role, emph| StyledSpan {
            emph,
            ..StyledSpan::new("x", role)
        };
        let add = Look::of(&span(DiffRole::Add, false)).style(&c);
        assert_eq!((add.fg, add.bg), (c.add_fg, Some(c.add_bg)));
        let word = Look::of(&span(DiffRole::Delete, true)).style(&c);
        assert_eq!((word.fg, word.bg), (c.del_fg, Some(c.del_emph_bg)));
        assert_eq!(Look::of(&span(DiffRole::Text, false)), Look::Context);
        assert!(Look::of(&span(DiffRole::HunkHeader, false)).style(&c).bold);
        assert!(Look::of(&span(DiffRole::Notice, false)).style(&c).italic);
        assert!(Look::of(&span(DiffRole::Title, false)).style(&c).bold);
        assert_eq!(
            Look::of(&span(DiffRole::Muted, false)).style(&c).fg,
            c.muted
        );
        assert_eq!(
            Look::of(&span(DiffRole::Gutter, false)).style(&c).fg,
            c.gutter_fg
        );
        assert_eq!(Look::Add.row_bg(&c), Some(c.add_bg));
        assert_eq!(Look::Context.row_bg(&c), None);
    }
}
