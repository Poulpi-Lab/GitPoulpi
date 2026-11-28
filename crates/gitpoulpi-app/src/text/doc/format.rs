//! Changing a document's format from the editor's status bar: its
//! encoding, its line breaks and its indentation. Each is one undo step
//! (the format rides on the `Change`, see the parent module), and each
//! marks the document dirty by differing from what was last saved.

use gitpoulpi_core::textconv::{self as charset, Charset, Eol, Unencodable};

use super::{Doc, Format, Group, Indent};

impl Doc {
    /// Saves the document in `to` from now on, keeping the text. Refused,
    /// with what is in the way, when `to` cannot hold every character.
    /// `Ok(false)`: it already was that encoding.
    pub fn convert_charset(&mut self, to: Charset) -> Result<bool, Unencodable> {
        if to == self.fmt.charset {
            return Ok(false);
        }
        if let Some(why) = to.unencodable(&self.text) {
            return Err(why);
        }
        self.change_format(
            Format {
                charset: to,
                ..self.fmt
            },
            None,
        );
        Ok(true)
    }

    /// Every line break of the document becomes `to` (`Lf`, `Crlf` or
    /// `Cr`), as one undo step; the user's choice then wins over
    /// `editor.eol`. `false`: it already was that.
    pub fn convert_eol(&mut self, to: Eol) -> bool {
        debug_assert!(to != Eol::Mixed);
        if self.eol() == to {
            return false;
        }
        let normalized = (self.fmt.eol == Eol::Mixed).then(|| charset::normalize_eol(&self.text));
        self.change_format(
            Format {
                eol: to,
                eol_set: true,
                ..self.fmt
            },
            normalized,
        );
        true
    }

    /// Switches the format to `fmt`, optionally replacing the text, as one
    /// undo step. The caret keeps its place in the text.
    fn change_format(&mut self, fmt: Format, new_text: Option<String>) {
        let before = self.text.clone();
        let old = self.fmt;
        let cursor = self.caret.cursor(&self.text);
        let mut to = cursor;
        if let Some(text) = new_text {
            // Where the caret lands once the breaks are normalized.
            to = charset::normalize_eol(&before[..cursor]).len();
            self.text = text;
        }
        self.fmt = fmt;
        self.caret.set_cursor(&self.text, to, false);
        self.goal_col = None;
        self.record_inner(&before, Group::Alone, None, 0, Some((old, fmt)));
    }

    /// What `Tab` inserts from now on: tabs or spaces (as many as the
    /// file's tab size).
    pub fn set_indent_tabs(&mut self, tabs: bool) {
        self.indent = if tabs {
            Indent::Tabs
        } else {
            Indent::Spaces(self.tab_cols())
        };
    }

    /// The size of an indent step and the width of a tab in this file.
    pub fn set_tab_size(&mut self, size: usize) {
        let size = size.clamp(1, 8);
        self.tab = Some(size);
        if let Indent::Spaces(_) = self.indent {
            self.indent = Indent::Spaces(size);
        }
        self.goal_col = None;
        // Widths changed, nothing else: what is cached per version redraws.
        self.version += 1;
    }

    /// Rewrites the leading whitespace of every line as tabs or as spaces
    /// (with the file's tab size) and makes that what `Tab` inserts. One
    /// undo step; the number of lines changed.
    pub fn convert_indent(&mut self, to_tabs: bool) -> usize {
        self.set_indent_tabs(to_tabs);
        let (text, changed) = charset::convert_indent(&self.text, to_tabs, self.tab_cols());
        if changed == 0 {
            return 0;
        }
        // The caret keeps its place in its line, past the indentation.
        let at = self.cursor();
        let line = self.line_of(at);
        let start = self.line_start(line);
        let old_lead = lead_len(self.line(line));
        let col = at - start;
        let new_line = text.split('\n').nth(line).unwrap_or("");
        let new_lead = lead_len(new_line);
        let in_line = if col >= old_lead {
            col - old_lead + new_lead
        } else {
            col.min(new_lead)
        };
        let new_start: usize = text.split('\n').take(line).map(|l| l.len() + 1).sum();
        self.goal_col = None;
        self.edit_with(Group::Alone, |caret, t| {
            *t = text;
            caret.set_cursor(t, new_start + in_line, false);
        });
        changed
    }

    /// A document that replaces `old` (same file, read again): its version
    /// continues from `old`'s, so nothing cached per version goes stale.
    pub fn continue_from(&mut self, old: &Doc) {
        self.version = old.version + 1;
        self.changed_from = Some(0);
    }
}

fn lead_len(line: &str) -> usize {
    line.bytes()
        .take_while(|b| *b == b' ' || *b == b'\t')
        .count()
}
