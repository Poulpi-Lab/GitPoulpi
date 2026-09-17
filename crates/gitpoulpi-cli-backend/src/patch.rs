//! The partial patches behind hunk- and line-level staging, unstaging and
//! discarding: parse one file's `git diff` into hunks, cut a hunk down to
//! a selection of its changed lines, and render it back as a patch for
//! `git apply`.
//!
//! Byte-faithful on purpose: the parsed `FileDiff` loses the `\r` of a CRLF
//! line and the "\ No newline at end of file" marker, and a patch rebuilt
//! without them does not apply. So each line's text is kept exactly as git
//! printed it, with the marker carried on the line it follows.
//!
//! Pure string work, no I/O; `staging.rs` does the running of `git`.

use gitpoulpi_core::domain::LineOrigin;

/// One line of a hunk body, exactly as `git diff` printed it minus the
/// one-character origin prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchLine {
    pub origin: LineOrigin,
    /// Without the `\n`, but with a `\r` if the file has one.
    pub text: String,
    /// Followed by "\ No newline at end of file": this is the last line
    /// of its side of the diff and has no line terminator.
    pub no_eol: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<PatchLine>,
}

impl PatchHunk {
    /// Whether applying this hunk would change anything at all.
    pub fn has_changes(&self) -> bool {
        self.lines.iter().any(|l| l.origin != LineOrigin::Context)
    }
}

/// Which way the patch will be applied, which decides what an
/// *unselected* change turns into.
///
/// A patch is only ever applied onto the side that holds its preimage,
/// and an unselected change must leave that side exactly as it is:
///
/// - `Forward` (`git apply --cached`, staging from the working-tree
///   diff): the preimage is the diff's *old* side (the index). An
///   unselected addition is not in it, so it vanishes; an unselected
///   removal is in it and stays, so it becomes context.
/// - `Reverse` (`git apply --reverse`, unstaging from the staged diff or
///   discarding from the working-tree diff): the preimage is the diff's
///   *new* side (the index, resp. the working tree). There an unselected
///   addition is present, so it becomes context, and an unselected
///   removal is absent, so it vanishes.
///
/// Getting this backwards still produces a well-formed patch - it just
/// names context the file does not have, and `git apply` refuses it as
/// soon as the hunk holds more than one change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Reverse,
}

/// Every hunk in `raw`, the output of `git diff` for **one** file, in
/// order - the order `HunkId`s are numbered in.
pub fn parse_hunks(raw: &str) -> Vec<PatchHunk> {
    let mut hunks: Vec<PatchHunk> = Vec::new();
    let mut in_hunk = false;
    // `split('\n')`, not `lines()`: `lines()` would also eat the `\r` of
    // a CRLF file's lines, which is precisely what must survive.
    for line in raw.split('\n') {
        if line.starts_with("diff --git ") {
            in_hunk = false;
            continue;
        }
        if line.starts_with("@@ ") {
            match parse_header(line) {
                Some((old_start, old_lines, new_start, new_lines)) => {
                    hunks.push(PatchHunk {
                        old_start,
                        old_lines,
                        new_start,
                        new_lines,
                        lines: Vec::new(),
                    });
                    in_hunk = true;
                }
                None => in_hunk = false,
            }
            continue;
        }
        if !in_hunk {
            continue;
        }
        let Some(hunk) = hunks.last_mut() else {
            continue;
        };
        let (origin, text) = match line.as_bytes().first() {
            Some(b'+') => (LineOrigin::Add, &line[1..]),
            Some(b'-') => (LineOrigin::Delete, &line[1..]),
            Some(b' ') => (LineOrigin::Context, &line[1..]),
            Some(b'\\') => {
                if let Some(last) = hunk.lines.last_mut() {
                    last.no_eol = true;
                }
                continue;
            }
            // The empty string after the output's final `\n`.
            _ => continue,
        };
        hunk.lines.push(PatchLine {
            origin,
            text: text.to_string(),
            no_eol: false,
        });
    }
    hunks
}

/// `@@ -a[,b] +c[,d] @@ ...` -> `(a, b, c, d)`, a missing count being 1.
fn parse_header(line: &str) -> Option<(u32, u32, u32, u32)> {
    let mut parts = line.strip_prefix("@@ ")?.split(' ');
    let range = |s: &str| -> Option<(u32, u32)> {
        match s.split_once(',') {
            Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
            None => Some((s.parse().ok()?, 1)),
        }
    };
    let (old_start, old_lines) = range(parts.next()?.strip_prefix('-')?)?;
    let (new_start, new_lines) = range(parts.next()?.strip_prefix('+')?)?;
    Some((old_start, old_lines, new_start, new_lines))
}

/// `hunk` reduced to the changed lines at `indices` (indices into
/// `hunk.lines`, the numbering `LineSelector::line_indices` uses), every
/// other change neutralised for `direction` - see [`Direction`].
///
/// The counts in the header are recomputed from what is left. The start
/// lines are kept (git uses them only as a hint and searches for the
/// context), except where a side goes from empty to non-empty or back:
/// an empty range's start names the line *before* it (`-0,0` is "before
/// line 1"), so its start shifts by one when that changes - the case of
/// a selection at the very top of a file.
pub fn select_lines(hunk: &PatchHunk, indices: &[u32], direction: Direction) -> PatchHunk {
    let mut lines = Vec::with_capacity(hunk.lines.len());
    for (i, line) in hunk.lines.iter().enumerate() {
        let selected = indices.contains(&(i as u32));
        let keep_as = match (&line.origin, selected, direction) {
            (LineOrigin::Context, _, _) => Some(LineOrigin::Context),
            (origin, true, _) => Some(origin.clone()),
            (LineOrigin::Add, false, Direction::Forward) => None,
            (LineOrigin::Add, false, Direction::Reverse) => Some(LineOrigin::Context),
            (LineOrigin::Delete, false, Direction::Forward) => Some(LineOrigin::Context),
            (LineOrigin::Delete, false, Direction::Reverse) => None,
        };
        if let Some(origin) = keep_as {
            lines.push(PatchLine {
                origin,
                text: line.text.clone(),
                no_eol: line.no_eol,
            });
        }
    }
    let old_lines = lines.iter().filter(|l| l.origin != LineOrigin::Add).count() as u32;
    let new_lines = lines
        .iter()
        .filter(|l| l.origin != LineOrigin::Delete)
        .count() as u32;
    PatchHunk {
        old_start: shift_start(hunk.old_start, hunk.old_lines, old_lines),
        old_lines,
        new_start: shift_start(hunk.new_start, hunk.new_lines, new_lines),
        new_lines,
        lines,
    }
}

fn shift_start(start: u32, before: u32, after: u32) -> u32 {
    match (before, after) {
        (0, n) if n > 0 => start + 1,
        (n, 0) if n > 0 => start.saturating_sub(1),
        _ => start,
    }
}

/// The one all-added hunk of a file git has never seen: `content`'s lines,
/// exactly as `git diff --no-index` would print them. `None` for an empty
/// file (nothing to stage or discard line by line).
pub fn whole_file_hunk(content: &str) -> Option<PatchHunk> {
    if content.is_empty() {
        return None;
    }
    let ends_with_eol = content.ends_with('\n');
    let body = content.strip_suffix('\n').unwrap_or(content);
    let mut lines: Vec<PatchLine> = body
        .split('\n')
        .map(|text| PatchLine {
            origin: LineOrigin::Add,
            text: text.to_string(),
            no_eol: false,
        })
        .collect();
    if !ends_with_eol {
        if let Some(last) = lines.last_mut() {
            last.no_eol = true;
        }
    }
    Some(PatchHunk {
        old_start: 0,
        old_lines: 0,
        new_start: 1,
        new_lines: lines.len() as u32,
        lines,
    })
}

/// `hunk` (all additions) as a patch that creates `path`, mode `mode`
/// (`100644` / `100755`).
pub fn render_new_file(path: &str, mode: &str, hunk: &PatchHunk) -> String {
    let body = render(path, hunk);
    let (_, rest) = body.split_once("@@").unwrap_or(("", ""));
    format!(
        "diff --git a/{path} b/{path}\nnew file mode {mode}\n--- /dev/null\n+++ b/{path}\n@@{rest}"
    )
}

/// `hunk` as a complete one-file, one-hunk patch of `path`.
pub fn render(path: &str, hunk: &PatchHunk) -> String {
    render_hunks(path, std::slice::from_ref(hunk))
}

/// `hunks` (in file order, each with its own header) as one complete
/// one-file patch of `path`; several of these concatenated are one
/// multi-file patch for `git apply`.
pub fn render_hunks(path: &str, hunks: &[PatchHunk]) -> String {
    let mut out = format!("diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n");
    for hunk in hunks {
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        ));
        for line in &hunk.lines {
            out.push(match line.origin {
                LineOrigin::Add => '+',
                LineOrigin::Delete => '-',
                LineOrigin::Context => ' ',
            });
            out.push_str(&line.text);
            out.push('\n');
            if line.no_eol {
                out.push_str("\\ No newline at end of file\n");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whole_new_file_is_one_all_added_hunk_with_its_missing_final_newline() {
        let h = whole_file_hunk("a\r\nb").unwrap();
        assert_eq!((h.old_lines, h.new_start, h.new_lines), (0, 1, 2));
        assert_eq!(h.lines[0].text, "a\r");
        assert!(!h.lines[0].no_eol && h.lines[1].no_eol);
        assert!(whole_file_hunk("").is_none());
        let patch = render_new_file("d/f.txt", "100644", &h);
        assert!(patch.contains("new file mode 100644\n--- /dev/null\n+++ b/d/f.txt\n@@ -0,0 +1,2 @@\n+a\r\n+b\n\\ No newline"));
    }

    const RAW: &str = "diff --git a/f.txt b/f.txt\n\
index 1111111..2222222 100644\n\
--- a/f.txt\n\
+++ b/f.txt\n\
@@ -1,5 +1,6 @@\n \
a\n\
-b\n\
-c\n\
+B\n\
+C\n \
d\n\
+new\n \
e\n";

    fn origins(h: &PatchHunk) -> String {
        h.lines
            .iter()
            .map(|l| match l.origin {
                LineOrigin::Add => '+',
                LineOrigin::Delete => '-',
                LineOrigin::Context => ' ',
            })
            .collect()
    }

    fn texts(h: &PatchHunk) -> Vec<&str> {
        h.lines.iter().map(|l| l.text.as_str()).collect()
    }

    #[test]
    fn parses_the_hunk_and_numbers_its_lines_like_the_file_diff() {
        let hunks = parse_hunks(RAW);
        assert_eq!(hunks.len(), 1);
        let h = &hunks[0];
        assert_eq!(
            (h.old_start, h.old_lines, h.new_start, h.new_lines),
            (1, 5, 1, 6)
        );
        assert_eq!(origins(h), " --++ + ");
        assert_eq!(texts(h), ["a", "b", "c", "B", "C", "d", "new", "e"]);
    }

    /// A selection spanning a removal and an addition, reversed (discard /
    /// unstage): the selected pair stays a change, the other removal
    /// vanishes, the other additions become context - the working tree's
    /// (the preimage's) own lines, in order.
    #[test]
    fn a_reverse_selection_spanning_adds_and_deletes() {
        let h = &parse_hunks(RAW)[0];
        // `-b` (1) and `+B` (3).
        let s = select_lines(h, &[1, 3], Direction::Reverse);
        assert_eq!(origins(&s), " -+    ");
        assert_eq!(texts(&s), ["a", "b", "B", "C", "d", "new", "e"]);
        // new side unchanged: a B C d new e = 6; old: a b C d new e = 6.
        assert_eq!(
            (s.old_start, s.old_lines, s.new_start, s.new_lines),
            (1, 6, 1, 6)
        );
        assert_eq!(
            render("f.txt", &s),
            "diff --git a/f.txt b/f.txt\n--- a/f.txt\n+++ b/f.txt\n@@ -1,6 +1,6 @@\n \
             a\n-b\n+B\n C\n d\n new\n e\n"
        );
    }

    /// The same selection forward (stage): the other removal becomes
    /// context, the other additions vanish - the index's own lines.
    #[test]
    fn a_forward_selection_spanning_adds_and_deletes() {
        let h = &parse_hunks(RAW)[0];
        let s = select_lines(h, &[1, 3], Direction::Forward);
        assert_eq!(origins(&s), " - +  ");
        assert_eq!(texts(&s), ["a", "b", "c", "B", "d", "e"]);
        assert_eq!((s.old_lines, s.new_lines), (5, 5));
    }

    /// Nothing selected: no change left, which the caller refuses rather
    /// than handing `git apply` a patch that does nothing.
    #[test]
    fn an_empty_selection_has_no_changes() {
        let h = &parse_hunks(RAW)[0];
        for d in [Direction::Forward, Direction::Reverse] {
            assert!(!select_lines(h, &[], d).has_changes());
            assert!(!select_lines(h, &[0, 5], d).has_changes(), "context only");
        }
    }

    /// Top of a file: a new first line in a file that was empty
    /// (`-0,0`). Discarding one of two added lines leaves the other as
    /// context, so the old side stops being empty and its start moves
    /// from "before line 1" to line 1.
    #[test]
    fn a_selection_at_the_top_of_a_file_shifts_an_empty_ranges_start() {
        let raw = "diff --git a/n b/n\n--- a/n\n+++ b/n\n@@ -0,0 +1,2 @@\n+one\n+two\n";
        let h = &parse_hunks(raw)[0];
        let s = select_lines(h, &[0], Direction::Reverse);
        assert_eq!(origins(&s), "+ ");
        assert_eq!(
            (s.old_start, s.old_lines, s.new_start, s.new_lines),
            (1, 1, 1, 2)
        );

        // Line 1 replaced, only the addition staged: the removal stays as
        // context, so neither side is empty and neither start moves.
        let raw = "diff --git a/n b/n\n--- a/n\n+++ b/n\n@@ -1 +1 @@\n-old\n+new\n";
        let h = &parse_hunks(raw)[0];
        assert_eq!((h.old_start, h.old_lines), (1, 1), "a missing count is 1");
        let s = select_lines(h, &[1], Direction::Forward);
        assert_eq!(origins(&s), " +");
        assert_eq!(
            (s.old_start, s.old_lines, s.new_start, s.new_lines),
            (1, 1, 1, 2)
        );
    }

    /// End of a file: the marker follows its line through a selection and
    /// back out into the patch, and a CRLF line keeps its `\r`.
    #[test]
    fn the_end_of_a_file_keeps_its_missing_newline_and_its_crlf() {
        let raw = "diff --git a/e b/e\n--- a/e\n+++ b/e\n@@ -2,2 +2,2 @@\n two\r\n\
                   -three\n\\ No newline at end of file\n+THREE\n\\ No newline at end of file\n";
        let h = &parse_hunks(raw)[0];
        assert_eq!(texts(h), ["two\r", "three", "THREE"]);
        assert_eq!(
            h.lines.iter().map(|l| l.no_eol).collect::<Vec<_>>(),
            [false, true, true]
        );
        let s = select_lines(h, &[2], Direction::Reverse);
        assert_eq!(
            render("e", &s),
            "diff --git a/e b/e\n--- a/e\n+++ b/e\n@@ -2,1 +2,2 @@\n two\r\n\
             +THREE\n\\ No newline at end of file\n"
        );
    }

    #[test]
    fn several_hunks_render_as_one_file_patch() {
        let raw = "diff --git a/m b/m\n--- a/m\n+++ b/m\n@@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n\
                   @@ -17,3 +17,3 @@\n x\n-y\n+Y\n z\n";
        assert_eq!(render_hunks("m", &parse_hunks(raw)), raw);
    }

    #[test]
    fn several_hunks_are_numbered_in_order() {
        let raw = "diff --git a/m b/m\n--- a/m\n+++ b/m\n@@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n\
                   @@ -17,3 +17,3 @@\n x\n-y\n+Y\n z\n";
        let hunks = parse_hunks(raw);
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[1].old_start, 17);
        assert_eq!(texts(&hunks[1]), ["x", "y", "Y", "z"]);
    }
}
