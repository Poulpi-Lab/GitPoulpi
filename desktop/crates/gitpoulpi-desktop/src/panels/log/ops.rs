//! The Log's commit operations on the selected row(s): cherry-pick, revert, reset, detached checkout,
//! copying a field, and opening the commit menu. Each is one daemon action (one undo entry); revert and
//! `reset --hard` ask first, the way the terminal UI's `log_action_confirmation` does.

use gitpoulpi_app::palette::vocab::log_menu::CommitField;
use gitpoulpi_app::palette::PaletteContext;
use gitpoulpi_core::domain::{CommitNode, Oid, ResetMode};
use gitpoulpi_i18n::msg;
use gpui_kit::{point, px, AppContext as _, ClipboardItem, Context, Pixels, Point, Window};

use super::menu::CommitMenu;
use super::menu_items::{build, MenuCtx};
use super::{LogEvent, LogPanel};
use crate::daemon::commit_ops::{self as ops, short};
use crate::dialogs::{self, Choice, Spec};
use gitpoulpi_app::log_multiselect::RowKey as MarkKey;

/// A one-line echo of what was copied, so the confirmation is verifiable.
fn preview(value: &str) -> String {
    let line = value.lines().next().unwrap_or("");
    let mut out: String = line.chars().take(40).collect();
    if out.chars().count() < line.chars().count() || value.contains('\n') {
        out.push('\u{2026}');
    }
    out
}

impl LogPanel {
    pub(super) fn say(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        cx.emit(LogEvent::Say(text.into()));
    }

    fn node(&self, oid: Oid) -> Option<&CommitNode> {
        self.data.nodes.iter().find(|n| n.id == oid)
    }

    /// The commits an operation acts on, oldest first: the marked commits, else the selected one.
    /// Stashes and the WIP row are not commits to apply.
    pub fn target_commits(&self) -> Vec<Oid> {
        if self.marks.is_empty() {
            return self.selected_commit().into_iter().collect();
        }
        let display = self.data.display_marks();
        // Stashes and the WIP row are not commits to apply: only a commit of the graph counts.
        let mut commits: Vec<Oid> = self
            .marks
            .in_display_order(&display)
            .into_iter()
            .filter_map(|k| match k {
                MarkKey::Id(oid) if self.node(oid).is_some() => Some(oid),
                _ => None,
            })
            .collect();
        commits.reverse();
        commits
    }

    /// The commit single-commit operations act on, or says why there is none.
    fn single(&mut self, cx: &mut Context<Self>) -> Option<Oid> {
        let commit = self.selected_commit();
        if commit.is_none() {
            self.say(msg::input::SELECT_COMMIT_NOT_WIP.get(), cx);
        }
        commit
    }

    // ---- cherry-pick ---------------------------------------------------------------------------------

    /// Applies the selected commit, or every marked commit oldest first, on top of HEAD.
    pub fn cherry_pick(&mut self, cx: &mut Context<Self>) {
        let commits = self.target_commits();
        match commits.as_slice() {
            [] => self.say(msg::input::SELECT_COMMIT_NOT_WIP.get(), cx),
            [one] => {
                self.backend.submit(ops::cherry_pick_action(*one));
                self.say(msg::menu_log::cherry_picking(short(*one)), cx);
            }
            many => {
                use gitpoulpi_core::domain::CherryPickOutcome as Outcome;
                use msg::link as l;
                let fut = self.backend.cherry_pick_many(many);
                self.say(
                    msg::menu_log::cherry_picking(
                        many.iter()
                            .map(|c| short(*c))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    cx,
                );
                cx.spawn(async move |this, cx| {
                    let text = match fut.await {
                        Ok(Outcome::Completed(done)) => l::cherry_picked(
                            done.iter()
                                .map(|c| short(*c))
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                        Ok(Outcome::Conflicted { paused_at, .. }) => {
                            l::cherry_pick_paused(short(paused_at))
                        }
                        Err(why) => why,
                    };
                    let _ = this.update(cx, |this, cx| this.say(text, cx));
                })
                .detach();
            }
        }
    }

    // ---- revert --------------------------------------------------------------------------------------

    /// Reverts the selected commit after asking; a merge commit asks which parent stays first.
    pub fn revert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(commit) = self.single(cx) else {
            return;
        };
        let parents = self
            .node(commit)
            .map(|n| n.parents.clone())
            .unwrap_or_default();
        let me = cx.entity().downgrade();
        if parents.len() < 2 {
            return self.ask_revert(commit, None, window, cx);
        }
        let choices = parents
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let summary = self.node(*p).map_or("", |n| n.summary.as_str());
                Choice::new(msg::revert_mainline::parent(i + 1, short(*p), summary))
            })
            .collect();
        dialogs::open(
            window,
            cx,
            Spec {
                title: msg::revert_mainline::TITLE.get().to_string(),
                lines: vec![msg::revert_mainline::QUESTION.get().to_string()],
                choices,
                hint: String::new(),
                highlight_first: true,
                yes_no_keys: false,
            },
            move |ix, cx| {
                let Some(window) = cx.active_window() else {
                    return;
                };
                let _ = window.update(cx, |_, window, cx| {
                    let _ = me.update(cx, |this, cx| {
                        this.ask_revert(commit, Some(ix as u32 + 1), window, cx)
                    });
                });
            },
            || {},
        );
    }

    fn ask_revert(
        &mut self,
        commit: Oid,
        mainline: Option<u32>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let backend = self.backend.clone();
        let started = msg::link::ACT_REVERTING.get().to_string();
        let me = cx.entity().downgrade();
        dialogs::open(
            window,
            cx,
            confirm_spec(
                ops::revert_prompt(commit, mainline),
                msg::menu_log::REVERT.get().trim_end_matches("..."),
            ),
            move |ix, cx| {
                if ix == 0 {
                    backend.submit(ops::revert_action(commit, mainline));
                    let _ = me.update(cx, |this, cx| this.say(started, cx));
                }
            },
            || {},
        );
    }

    // ---- reset and checkout --------------------------------------------------------------------------

    /// Moves the current branch to the selected commit; a hard reset asks first.
    pub fn reset(&mut self, mode: ResetMode, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.single(cx) else {
            return;
        };
        let started = msg::menu_log::resetting(ops::mode_word(mode), short(target));
        match ops::reset_prompt(target, mode) {
            None => {
                self.backend.submit(ops::reset_action(target, mode));
                self.say(started, cx);
            }
            Some(prompt) => {
                let backend = self.backend.clone();
                let me = cx.entity().downgrade();
                dialogs::open(
                    window,
                    cx,
                    confirm_spec(
                        prompt,
                        msg::menu_log::RESET_HARD.get().trim_end_matches("..."),
                    ),
                    move |ix, cx| {
                        if ix == 0 {
                            backend.submit(ops::reset_action(target, mode));
                            let _ = me.update(cx, |this, cx| this.say(started, cx));
                        }
                    },
                    || {},
                );
            }
        }
    }

    /// Detaches HEAD onto the selected commit (nothing is lost, so it never asks).
    pub fn checkout_commit(&mut self, cx: &mut Context<Self>) {
        let Some(commit) = self.single(cx) else {
            return;
        };
        self.backend.submit(ops::checkout_action(commit));
        self.say(msg::menu_log::checking_out(short(commit)), cx);
    }

    // ---- copy ----------------------------------------------------------------------------------------

    /// Puts one field of the selected commit on the clipboard.
    pub fn copy_field(&mut self, field: CommitField, cx: &mut Context<Self>) {
        let Some(commit) = self.single(cx) else {
            return;
        };
        let label = ops::field_label(field);
        let text = match self.node(commit) {
            Some(node) => ops::field_value(field, node, crate::prefs::PrefsStore::get(cx).dates()),
            // The ids are still copyable once the commit has left the loaded page.
            None => match field {
                CommitField::ShortSha => short(commit),
                CommitField::LongSha => commit.to_hex(),
                _ => return self.say(msg::menu_log::not_loaded(short(commit)), cx),
            },
        };
        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
        self.say(msg::input::copied(label, preview(&text)), cx);
    }

    // ---- the menu ------------------------------------------------------------------------------------

    /// Right-click on row `ix`: a click outside the selection selects that row first.
    pub fn open_menu_on(
        &mut self,
        ix: usize,
        at: Point<Pixels>,
        palette: PaletteContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = self.data.mark_at(ix);
        if !key.as_ref().is_some_and(|k| self.marks.contains(k)) {
            self.select(ix, cx);
            self.marks.reset_to(key);
        }
        self.focus.focus(window, cx);
        self.open_menu_at(at, palette, window, cx);
    }

    /// The menu key / Shift+F10 / `.`: the menu of the selected row, beside it.
    pub fn open_menu(
        &mut self,
        palette: PaletteContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_commit().is_none() && self.marks.is_empty() {
            return self.say(msg::input::SELECT_COMMIT_NOT_WIP.get(), cx);
        }
        let at = self.row_anchor();
        self.open_menu_at(at, palette, window, cx);
    }

    /// Where a keyboard-opened menu appears: under the selected row when the list knows where it is,
    /// else near the top of the Log.
    fn row_anchor(&self) -> Point<Pixels> {
        let bounds = self.scroll.0.borrow().base_handle.bounds();
        let scrolled = -self.scroll.0.borrow().base_handle.offset().y.as_f32();
        let first = (scrolled / crate::graph::CH).max(0.0) as usize;
        let below = self.selected.saturating_sub(first) as f32 + 1.0;
        let y = (bounds.origin.y + px(crate::graph::CH * below))
            .min(bounds.origin.y + bounds.size.height);
        point(
            bounds.origin.x + px(80.0),
            y.max(bounds.origin.y + px(20.0)),
        )
    }

    fn open_menu_at(
        &mut self,
        at: Point<Pixels>,
        palette: PaletteContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let marked = self.target_commits();
        let commit = self.selected_commit().or_else(|| marked.first().copied());
        let Some(commit) = commit else {
            return self.say(msg::input::SELECT_COMMIT_NOT_WIP.get(), cx);
        };
        let ctx = MenuCtx {
            head: self
                .data
                .head_branch
                .clone()
                .unwrap_or_else(|| "HEAD".to_string()),
            detached: self.data.head_branch.is_none(),
            selected: marked.len().max(1),
            merge_commit: self.node(commit).is_some_and(|n| n.parents.len() > 1),
            palette: PaletteContext {
                selected_commit: true,
                ..palette
            },
        };
        let title = match self.node(commit) {
            Some(n) => format!("{} {}", short(commit), n.summary),
            None => short(commit),
        };
        let items = build(&ctx);
        let back_to = self.focus.clone();
        let menu = cx.new(|cx| CommitMenu::new(title, items, at, back_to, window, cx));
        self._menu_sub = Some(cx.subscribe(&menu, |this, _, _: &super::menu::Closed, cx| {
            this.menu = None;
            this._menu_sub = None;
            cx.notify();
        }));
        self.menu = Some(menu);
        cx.notify();
    }
}

/// A plain confirmation: `confirm` (danger colour) or Cancel.
fn confirm_spec(title: String, confirm: &str) -> Spec {
    Spec {
        title,
        lines: Vec::new(),
        choices: vec![
            Choice::destructive(confirm.to_string()),
            Choice::new(msg::dialog::CANCEL.get()),
        ],
        hint: msg::footer::CONFIRM_HINT.get().trim().to_string(),
        highlight_first: true,
        yes_no_keys: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_echo_is_one_short_line() {
        assert_eq!(preview("short"), "short");
        assert!(preview("first\nsecond").ends_with('\u{2026}'));
        assert!(preview(&"x".repeat(100)).chars().count() <= 41);
    }
}
