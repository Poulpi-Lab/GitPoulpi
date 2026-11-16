//! From a code thread of the Overview to its code: the Files page opens the
//! file, the cursor lands on the commented line (on the side it was written
//! on) with the thread open under it, and Back (Esc) returns to the thread's
//! entry in the Overview.

use gitpoulpi_i18n::msg::review_view as msg;

use super::diffrows::RowKind;
use super::model::Focus;
use super::ReviewView;
use crate::item_cmd::CodeRef;
use crate::review_cmd::Tab;

/// A jump waiting for the file's rows (the diff is read on demand).
#[derive(Debug, Clone)]
pub(super) struct Goto {
    /// The file as the Files list names it.
    pub(super) path: String,
    pub(super) code: CodeRef,
}

impl ReviewView {
    /// Opens the Files page on the code `code` is about.
    pub(super) fn open_code(&mut self, code: CodeRef) {
        let Some(file) = self.host.as_ref().and_then(|h| {
            h.files.iter().position(|f| {
                f.path == code.path || f.previous_path.as_deref() == Some(code.path.as_str())
            })
        }) else {
            self.message = Some(msg::thread_not_in_diff(&code.path));
            return;
        };
        let path = self
            .host
            .as_ref()
            .and_then(|h| h.files.get(file))
            .map(|f| f.path.clone())
            .unwrap_or_default();
        self.set_tab(Tab::Files);
        self.ov_return = Some(code.thread_id.clone());
        self.select_file(file);
        self.focus = Focus::Diff;
        // The thread must be readable there: a folded one is opened.
        let id = self
            .thread_index(&code)
            .and_then(|i| self.host.as_ref().map(|h| h.threads[i].id.clone()));
        if let Some(id) = id {
            if self.collapsed.remove(&id) {
                self.revision += 1;
            }
        }
        self.goto = Some(Goto { path, code });
    }

    /// Which of the host's threads the conversation's thread is: by id, else
    /// by place.
    fn thread_index(&self, code: &CodeRef) -> Option<usize> {
        let threads = &self.host.as_ref()?.threads;
        threads
            .iter()
            .position(|t| t.id == code.thread_id)
            .or_else(|| {
                threads.iter().position(|t| {
                    t.path == code.path
                        && t.line == code.line
                        && t.original_line == code.original_line
                        && (t.side == gitpoulpi_hosting::DiffSide::Old) == code.old_side
                })
            })
    }

    /// Puts the cursor where a waiting jump wants it, once the rows of
    /// `path` are drawn.
    pub(super) fn apply_goto(&mut self, path: &str) {
        let Some(goto) = self.goto.as_ref() else {
            return;
        };
        if goto.path != path || self.files_pane.is_empty() {
            return;
        }
        let code = goto.code.clone();
        self.goto = None;
        let thread = self.thread_index(&code);
        let want = if code.outdated {
            code.original_line.or(code.line)
        } else {
            code.line.or(code.original_line)
        };
        let pane = &self.files_pane;
        let number = |kind: RowKind| match kind {
            RowKind::Line { old, new, .. } => {
                if code.old_side {
                    old
                } else {
                    new
                }
            }
            _ => None,
        };
        let head = thread.and_then(|t| {
            (0..pane.len()).find(|&i| pane.kind(i) == Some(RowKind::ThreadHead { thread: t }))
        });
        let mut exact: Option<usize> = None;
        let mut nearest: Option<(u32, usize)> = None;
        if let Some(want) = want {
            for i in 0..pane.len() {
                let Some(n) = pane.kind(i).and_then(number) else {
                    continue;
                };
                if n == want && exact.is_none() {
                    exact = Some(i);
                }
                let d = n.abs_diff(want);
                if nearest.is_none_or(|(best, _)| d < best) {
                    nearest = Some((d, i));
                }
            }
        }
        // A live thread sits right under its line; walk up from its head.
        let above = head.filter(|_| !code.outdated).and_then(|h| {
            (0..h)
                .rev()
                .find(|&i| matches!(pane.kind(i), Some(RowKind::Line { .. })))
        });
        let target = if code.outdated {
            nearest.map(|(_, i)| i).or(head)
        } else {
            above.or(exact).or(head).or(nearest.map(|(_, i)| i))
        };
        let Some(row) = target else {
            self.message = Some(msg::thread_not_in_diff(&code.path));
            return;
        };
        let pane = &mut self.files_pane;
        pane.side = usize::from(!code.old_side);
        pane.jump(row);
        // Room above the line, so the thread under it shows too.
        pane.offset = row
            .saturating_sub(3)
            .min(pane.len().saturating_sub(pane.visible()));
        pane.reveal();
        if code.outdated {
            self.message = Some(msg::thread_outdated_nearest(&code.path));
        }
    }
}
