//! The right pane for a selected commit or stash: details above the changed files.
//!
//! The details: the id, the author with the date, every parent (a click jumps to it in the Log), the
//! refs on the commit, the change counts and the whole message as selectable text. Copy chips put the id,
//! the author's email and the message on the clipboard. The changed files list (Path or Tree, filter,
//! sort) opens a file's diff on a click or Enter; Up / Down walk it.

use gitpoulpi_core::domain::{CommitNode, FileStatus, Oid, RefKind};
use gitpoulpi_i18n::msg;
use gpui_kit::component::text::TextView;
use gpui_kit::component::{h_flex, input::Input, v_flex, ActiveTheme as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    div, px, ClipboardItem, Context, FontWeight, InteractiveElement as _, KeyDownEvent,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _,
};

use super::widgets::{boxed, ListKind};
use super::{Mode, RightEvent, RightPanel};
use crate::daemon::commit_ops::short;
use crate::theme::Skin;

/// Files added, modified and deleted among a commit's diffs.
fn change_counts(files: &[gitpoulpi_core::domain::FileDiff]) -> (u64, u64, u64) {
    let (mut added, mut modified, mut deleted) = (0u64, 0u64, 0u64);
    for f in files {
        match f.status {
            FileStatus::Added | FileStatus::Untracked => added += 1,
            FileStatus::Deleted => deleted += 1,
            _ => modified += 1,
        }
    }
    (added, modified, deleted)
}

/// The text as a fenced block longer than any run of backticks inside it, so it shows verbatim and the
/// text view can select it.
pub(super) fn fenced(text: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in text.chars() {
        if c == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    let fence = "`".repeat((longest + 1).max(3));
    format!("{fence}\n{text}\n{fence}")
}

/// How a ref on the commit is named: a branch or tag by its name, a remote branch with its remote.
fn ref_label(kind: &RefKind) -> String {
    match kind {
        RefKind::LocalBranch { name } | RefKind::Tag { name } => name.clone(),
        RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}"),
    }
}

impl RightPanel {
    /// Puts `text` on the clipboard and says what was copied.
    pub(super) fn copy_text(&mut self, what: &str, text: String, cx: &mut Context<Self>) {
        let echo: String = text.lines().next().unwrap_or("").chars().take(40).collect();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.say(msg::input::copied(what, echo), cx);
    }

    /// A small "Copy" control next to a detail.
    fn copy_chip(
        &self,
        id: &'static str,
        what: &'static str,
        text: String,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        div()
            .id(id)
            .flex_none()
            .px(px(5.0))
            .text_xs()
            .rounded(px(2.0))
            .text_color(skin.accent)
            .cursor_pointer()
            .hover(|d| d.bg(skin.hover))
            .child(msg::menu_ref::COPY.get())
            .on_click(cx.listener(move |this, _, _, cx| this.copy_text(what, text.clone(), cx)))
    }

    /// Up / Down walk the changed files, Enter opens the one under the cursor.
    pub(super) fn commit_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let m = &ev.keystroke.modifiers;
        if m.control || m.alt || m.platform {
            return;
        }
        let handled = match ev.keystroke.key.as_str() {
            "up" => self.step_file(-1, cx),
            "down" => self.step_file(1, cx),
            "enter" => self.open_selected_commit_file(cx),
            _ => false,
        };
        if handled {
            cx.stop_propagation();
        }
    }

    /// Moves the cursor to the next or previous file row (folders are skipped).
    fn step_file(&mut self, delta: isize, cx: &mut Context<Self>) -> bool {
        let files: Vec<usize> = self
            .rows
            .iter()
            .filter(|r| !r.folder)
            .map(|r| r.ix)
            .collect();
        if files.is_empty() {
            return false;
        }
        let at = self
            .selected
            .and_then(|s| files.iter().position(|f| *f == s))
            .map_or(if delta > 0 { 0 } else { files.len() - 1 }, |p| {
                (p as isize + delta).clamp(0, files.len() as isize - 1) as usize
            });
        self.selected = Some(files[at]);
        cx.notify();
        true
    }

    /// Enter: the cursor file's diff in the centre.
    pub fn open_selected_commit_file(&mut self, cx: &mut Context<Self>) -> bool {
        match self.selected {
            Some(ix) => {
                self.open_commit_file(ix, cx);
                true
            }
            None => false,
        }
    }

    /// The files of the commit or comparison on show, as the list's rows in order.
    pub fn commit_file_paths(&self) -> Vec<String> {
        self.rows
            .iter()
            .filter(|r| !r.folder)
            .map(|r| r.path.clone())
            .collect()
    }

    /// Whether the pane shows a commit (not the working changes or a comparison).
    pub fn shows_commit(&self) -> bool {
        matches!(self.mode, Mode::Commit(..))
    }

    /// A click on parent `i` of the shown commit: select it in the Log.
    pub fn jump_to_parent(&mut self, i: usize, cx: &mut Context<Self>) {
        if let Some(p) = self.shown_parents().get(i).copied() {
            cx.emit(RightEvent::SelectParent(p));
        }
    }

    /// The names of the refs the details list on the commit.
    pub fn shown_ref_names(&self) -> Vec<String> {
        match &self.mode {
            Mode::Commit(node, _) => node.refs.iter().map(ref_label).collect(),
            _ => Vec::new(),
        }
    }

    /// The parents the details list, for tests.
    pub fn shown_parents(&self) -> Vec<Oid> {
        match &self.mode {
            Mode::Commit(node, _) => node.parents.clone(),
            _ => Vec::new(),
        }
    }

    pub(super) fn commit_view(
        &self,
        node: &CommitNode,
        stash: &Option<String>,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let (added, modified, deleted) = change_counts(&self.files);
        let date = crate::prefs::PrefsStore::get(cx)
            .dates()
            .format_verbose(node.timestamp);
        let mono = cx.theme().mono_font_family.clone();
        let head = stash
            .clone()
            .unwrap_or_else(|| node.id.to_hex()[..7].to_string());
        let parents = node.parents.iter().enumerate().map(|(i, p)| {
            let p = *p;
            div()
                .id(("parent", i))
                .px(px(3.0))
                .rounded(px(2.0))
                .font_family(mono.clone())
                .text_color(skin.accent)
                .cursor_pointer()
                .hover(|d| d.bg(skin.hover))
                .child(short(p))
                .on_click(cx.listener(move |this, _, _, cx| this.jump_to_parent(i, cx)))
        });
        let parents: Vec<_> = parents.collect();
        let refs: Vec<String> = node.refs.iter().map(ref_label).collect();
        let details =
            boxed(skin, msg::commit_detail::TITLE.get().into(), None).child(
                v_flex()
                    .px(px(8.0))
                    .pb(px(8.0))
                    .gap(px(3.0))
                    .child(
                        h_flex()
                            .gap(px(6.0))
                            .items_center()
                            .child(
                                div()
                                    .text_color(skin.muted)
                                    .child(msg::commit_detail::header("").trim_end().to_string()),
                            )
                            .child(
                                div()
                                    .cursor_text()
                                    .text_color(skin.accent)
                                    .font_family(mono.clone())
                                    .child(head),
                            )
                            .child(div().flex_1())
                            .child(self.copy_chip(
                                "copy-sha",
                                crate::daemon::commit_ops::field_label(
                                    gitpoulpi_app::palette::vocab::log_menu::CommitField::LongSha,
                                ),
                                node.id.to_hex(),
                                skin,
                                cx,
                            )),
                    )
                    .child(
                        h_flex()
                            .gap(px(6.0))
                            .items_center()
                            .child(
                                div()
                                    .cursor_text()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(node.author_name.clone()),
                            )
                            .when(!node.author_email.is_empty(), |d| {
                                d.child(
                                    div()
                                        .cursor_text()
                                        .text_color(skin.muted)
                                        .child(format!("<{}>", node.author_email)),
                                )
                                .child(self.copy_chip(
                                    "copy-email",
                                    msg::menu_log::FIELD_AUTHOR_EMAIL.get(),
                                    node.author_email.clone(),
                                    skin,
                                    cx,
                                ))
                            }),
                    )
                    .child(
                        h_flex()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_color(skin.muted)
                                    .child(msg::commit_detail::AUTHORED.get()),
                            )
                            .child(
                                div()
                                    .cursor_text()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(date),
                            ),
                    )
                    .when(stash.is_none(), |d| {
                        d.child(
                            h_flex()
                                .gap(px(4.0))
                                .flex_wrap()
                                .items_center()
                                .child(div().text_color(skin.muted).child(
                                    if node.parents.is_empty() {
                                        msg::commit_detail::ROOT.get().to_string()
                                    } else {
                                        msg::commit_detail::parent("").trim_end().to_string()
                                    },
                                ))
                                .children(parents),
                        )
                    })
                    .when(!refs.is_empty(), |d| {
                        d.child(
                            h_flex()
                                .gap(px(4.0))
                                .flex_wrap()
                                .children(refs.into_iter().map(|r| {
                                    div()
                                        .px(px(5.0))
                                        .text_xs()
                                        .rounded(px(2.0))
                                        .border_1()
                                        .border_color(skin.border)
                                        .cursor_text()
                                        .child(r)
                                })),
                        )
                    })
                    .child(
                        h_flex()
                            .gap(px(12.0))
                            .text_xs()
                            .when(added > 0, |d| {
                                d.child(
                                    div()
                                        .text_color(skin.success)
                                        .child(format!("+ {}", msg::commit_detail::added(added))),
                                )
                            })
                            .when(modified > 0, |d| {
                                d.child(div().text_color(skin.warning).child(format!(
                                    "\u{270E} {}",
                                    msg::commit_detail::modified(modified)
                                )))
                            })
                            .when(deleted > 0, |d| {
                                d.child(div().text_color(skin.danger).child(format!(
                                    "\u{2212} {}",
                                    msg::commit_detail::deleted(deleted)
                                )))
                            }),
                    )
                    .child(
                        v_flex()
                            .mt(px(6.0))
                            .p(px(8.0))
                            .gap(px(4.0))
                            .border_1()
                            .border_color(skin.border)
                            .rounded(px(3.0))
                            .child(h_flex().items_center().child(div().flex_1()).child(
                                self.copy_chip(
                                    "copy-message",
                                    msg::menu_log::FIELD_MESSAGE.get(),
                                    node.message.clone(),
                                    skin,
                                    cx,
                                ),
                            ))
                            .child(
                                // The whole message, selectable with the mouse (the text view shows the I-beam).
                                div()
                                    .id("commit-message")
                                    .max_h(px(220.0))
                                    .overflow_y_scroll()
                                    .child(
                                        TextView::markdown(
                                            "commit-message-text",
                                            fenced(&node.message),
                                        )
                                        .selectable(true),
                                    ),
                            ),
                    ),
            );
        self.changed_files(details, skin, cx)
    }

    /// The details box above the changed-files box (shared with the comparison).
    pub(super) fn changed_files(
        &self,
        details: gpui_kit::Div,
        skin: &Skin,
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        let list = self.file_list(
            self.rows.clone(),
            &self.scroll,
            "cfiles",
            ListKind::Commit,
            cx,
            skin,
        );
        let files_box = boxed(
            skin,
            msg::commit_detail::files_title(
                msg::commit_detail::CHANGED_FILES.get(),
                self.rows.iter().filter(|r| !r.folder).count() as u64,
            )
            .into(),
            None,
        )
        .flex_1()
        .min_h_0()
        .child(self.controls(skin, cx))
        .child(
            h_flex()
                .flex_none()
                .px(px(8.0))
                .pb(px(4.0))
                .gap(px(6.0))
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(skin.muted)
                        .child(msg::commit_detail::FILTER_FILES.get()),
                )
                .child(
                    div()
                        .flex_1()
                        .child(Input::new(&self.filter).small().cleanable(true)),
                ),
        )
        .child(list);
        v_flex()
            .size_full()
            .gap(px(8.0))
            .child(details.flex_none())
            .child(files_box)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fence_is_longer_than_any_backtick_run_in_the_text() {
        assert_eq!(fenced("one"), "```\none\n```");
        let f = fenced("a ```` b");
        assert!(f.starts_with("`````\n") && f.ends_with("\n`````"), "{f}");
    }

    #[test]
    fn refs_read_like_the_refs_pane() {
        assert_eq!(
            ref_label(&RefKind::LocalBranch {
                name: "main".into()
            }),
            "main"
        );
        assert_eq!(
            ref_label(&RefKind::RemoteBranch {
                remote: "origin".into(),
                name: "main".into()
            }),
            "origin/main"
        );
    }
}
