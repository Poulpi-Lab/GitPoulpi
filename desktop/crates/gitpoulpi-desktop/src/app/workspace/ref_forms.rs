//! The small forms of the ref commands: New branch, New tag, Annotate, Stash with a message,
//! Rename and Set upstream. Each opens a `form_dialog` whose validation is the rule of
//! `daemon::refs_ops`; the submit goes through the window like every other ref command.

use gitpoulpi_core::domain::Oid;
use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;
use gpui_kit::{Context, Window};

use super::Workspace;
use crate::app::actions::with_workspace;
use crate::daemon::refs_ops::{self as rules, NewRef, Snapshot};
use crate::form_dialog::{self, Field, FormSpec};
use crate::ref_target::RefTarget;
use crate::strings;

impl Workspace {
    /// HEAD's commit: the checked-out branch's, else asked from the daemon (a detached HEAD).
    pub(super) fn with_head(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        then: impl FnOnce(&mut Self, Oid, &mut Window, &mut Context<Self>) + 'static,
    ) {
        if let Some((_, at)) = self.snapshot(cx).head() {
            return then(self, at, window, cx);
        }
        let head = self.backend.head();
        cx.spawn_in(window, async move |this, cx| match head.await {
            Ok(at) => {
                let _ = this.update_in(cx, |this, window, cx| then(this, at, window, cx));
            }
            Err(_) => {
                let _ = this.update(cx, |this, cx| {
                    this.say(msg::input::NO_COMMIT_FOR_BRANCH.get(), cx)
                });
            }
        })
        .detach();
    }

    pub(super) fn start_new_branch(
        &mut self,
        at: Option<Oid>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match at {
            Some(at) => self.new_branch_form(at, true, window, cx),
            None => self.with_head(window, cx, |this, at, window, cx| {
                this.new_branch_form(at, false, window, cx)
            }),
        }
    }

    pub(super) fn new_branch_form(
        &mut self,
        at: Oid,
        at_commit: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use msg::menu_inline as m;
        let refs = self.snapshot(cx).refs;
        let sha = gitpoulpi_core::domain::short(at);
        let title = if at_commit {
            m::box_new_branch_at(&sha)
        } else {
            m::BOX_NEW_BRANCH.get().to_string()
        };
        let spec = FormSpec {
            title,
            intro: Vec::new(),
            fields: vec![
                Field::text(strings::NAME_LABEL, m::PLACEHOLDER_BRANCH.get(), ""),
                Field::check(strings::CHECKOUT_AFTER_CREATE, true),
            ],
            submit: m::CREATE_BUTTON.get().to_string(),
            cancel: msg::dialog::CANCEL.get().to_string(),
        };
        form_dialog::open(
            window,
            cx,
            spec,
            move |v| rules::validate_new_ref(NewRef::Branch, v.text(0), &refs).map(|_| ()),
            move |v, _, cx| {
                let (name, checkout) = (v.text(0).to_string(), v.check(1));
                with_workspace(cx, move |ws, _, cx| {
                    ws.create_branch(name, at, at_commit, checkout, cx)
                });
            },
        );
    }

    /// Creates the branch. Checked out as well, it is the terminal UI's action (create, then check
    /// out); otherwise HEAD stays where it is.
    pub(super) fn create_branch(
        &mut self,
        name: String,
        at: Oid,
        at_commit: bool,
        checkout: bool,
        cx: &mut Context<Self>,
    ) {
        let said = if at_commit {
            msg::input::creating_branch_at(&name, gitpoulpi_core::domain::short(at))
        } else {
            msg::input::creating_branch(&name)
        };
        if checkout {
            self.backend.create_branch_and_checkout(name, at);
        } else {
            let created = self.backend.create_branch_only(&name, at);
            cx.spawn(async move |this, cx| {
                if let Err(why) = created.await {
                    let _ = this.update(cx, |this, cx| {
                        this.notice = Some(crate::app::Notice::error(why));
                        cx.notify();
                    });
                }
            })
            .detach();
        }
        self.say(said, cx);
    }

    pub(super) fn start_new_tag(
        &mut self,
        at: Option<Oid>,
        annotated: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match at {
            Some(at) => self.new_tag_form(at, true, annotated, window, cx),
            None => self.with_head(window, cx, |this, at, window, cx| {
                this.new_tag_form(at, false, false, window, cx)
            }),
        }
    }

    pub(super) fn new_tag_form(
        &mut self,
        at: Oid,
        at_commit: bool,
        annotated: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use msg::menu_inline as m;
        let refs = self.snapshot(cx).refs;
        let sha = gitpoulpi_core::domain::short(at);
        let title = match (at_commit, annotated) {
            (false, _) => m::BOX_NEW_TAG.get().to_string(),
            (true, false) => m::box_new_tag_at(&sha),
            (true, true) => m::box_new_annotated_tag_at(&sha),
        };
        let spec = FormSpec {
            title,
            intro: Vec::new(),
            fields: vec![
                Field::text(strings::NAME_LABEL, m::PLACEHOLDER_TAG.get(), ""),
                Field::text(
                    if annotated {
                        strings::TAG_MESSAGE
                    } else {
                        strings::TAG_MESSAGE_OPTIONAL
                    },
                    msg::prompt_dialog::TAG_PLACEHOLDER.get(),
                    "",
                ),
            ],
            submit: m::CREATE_BUTTON.get().to_string(),
            cancel: msg::dialog::CANCEL.get().to_string(),
        };
        form_dialog::open(
            window,
            cx,
            spec,
            move |v| {
                rules::validate_new_ref(NewRef::Tag, v.text(0), &refs)?;
                if annotated && v.text(1).is_empty() {
                    return Err(msg::input::ANNOTATED_TAG_NEEDS_MESSAGE.get().to_string());
                }
                Ok(())
            },
            move |v, _, cx| {
                let name = v.text(0).to_string();
                let message = Some(v.text(1).to_string()).filter(|m| !m.is_empty());
                with_workspace(cx, move |ws, _, cx| {
                    ws.backend.create_tag(name.clone(), at, message.clone());
                    let said = match (at_commit, &message) {
                        (false, _) => msg::input::creating_tag(&name),
                        (true, None) => msg::input::creating_tag_at(&name, &sha),
                        (true, Some(_)) => msg::input::creating_annotated_tag_at(&name, &sha),
                    };
                    ws.say(said, cx);
                });
            },
        );
    }

    pub(super) fn start_annotate(
        &mut self,
        tag: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let spec = FormSpec {
            title: msg::prompt_dialog::tag_annotate_title(&tag),
            intro: Vec::new(),
            fields: vec![Field::text(
                strings::TAG_MESSAGE,
                msg::prompt_dialog::TAG_PLACEHOLDER.get(),
                "",
            )],
            submit: msg::prompt_dialog::TAG_ANNOTATE_BUTTON.get().to_string(),
            cancel: msg::dialog::CANCEL.get().to_string(),
        };
        form_dialog::open(
            window,
            cx,
            spec,
            |v| {
                if v.text(0).is_empty() {
                    Err(msg::input::ANNOTATED_TAG_NEEDS_MESSAGE.get().to_string())
                } else {
                    Ok(())
                }
            },
            move |v, _, cx| {
                let message = v.text(0).to_string();
                with_workspace(cx, move |ws, _, cx| {
                    ws.backend.submit(Action::TagAnnotate {
                        name: tag.clone(),
                        message,
                    });
                    ws.say(msg::input::annotating_tag(&tag), cx);
                });
            },
        );
    }

    pub(super) fn start_stash(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let spec = FormSpec {
            title: msg::prompt_dialog::STASH_TITLE.get().to_string(),
            intro: Vec::new(),
            fields: vec![Field::text(
                strings::STASH_MESSAGE_LABEL,
                msg::prompt_dialog::STASH_PLACEHOLDER.get(),
                "",
            )],
            submit: msg::prompt_dialog::STASH_BUTTON.get().to_string(),
            cancel: msg::dialog::CANCEL.get().to_string(),
        };
        form_dialog::open(
            window,
            cx,
            spec,
            |_| Ok(()),
            move |v, _, cx| {
                let message = Some(v.text(0).to_string()).filter(|m| !m.is_empty());
                with_workspace(cx, move |ws, _, cx| {
                    ws.backend.stash_push(message);
                    ws.say(msg::input::STASHING.get(), cx);
                });
            },
        );
    }

    pub(super) fn start_rename(
        &mut self,
        old: String,
        snap: Snapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use msg::menu_inline as m;
        let refs = snap.refs;
        let spec = FormSpec {
            title: m::box_rename(&old),
            intro: Vec::new(),
            fields: vec![Field::text(
                strings::NAME_LABEL,
                m::PLACEHOLDER_BRANCH.get(),
                &old,
            )],
            submit: m::RENAME_BUTTON.get().to_string(),
            cancel: msg::dialog::CANCEL.get().to_string(),
        };
        let old_check = old.clone();
        form_dialog::open(
            window,
            cx,
            spec,
            move |v| {
                // Keeping the name is not an error: it closes and changes nothing.
                if v.text(0) == old_check {
                    return Ok(());
                }
                rules::validate_rename(&old_check, v.text(0), &refs).map(|_| ())
            },
            move |v, _, cx| {
                let new_name = v.text(0).to_string();
                if new_name == old {
                    return;
                }
                with_workspace(cx, move |ws, _, cx| {
                    ws.backend.submit(Action::BranchRename {
                        old_name: old.clone(),
                        new_name: new_name.clone(),
                    });
                    ws.say(msg::input::renaming(&old, &new_name), cx);
                });
            },
        );
    }

    pub(super) fn start_upstream(
        &mut self,
        branch: String,
        snap: Snapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let current = snap
            .find(&RefTarget::Local(branch.clone()))
            .and_then(|r| r.upstream.clone())
            .unwrap_or_default();
        let refs = snap.refs;
        let spec = FormSpec {
            title: msg::prompt_dialog::upstream_title(&branch),
            intro: vec![strings::UPSTREAM_EMPTY_UNSETS.to_string()],
            fields: vec![Field::text(
                strings::UPSTREAM_LABEL,
                msg::prompt_dialog::UPSTREAM_PLACEHOLDER.get(),
                &current,
            )],
            submit: msg::prompt_dialog::SET_BUTTON.get().to_string(),
            cancel: msg::dialog::CANCEL.get().to_string(),
        };
        let refs_check = refs.clone();
        form_dialog::open(
            window,
            cx,
            spec,
            move |v| rules::validate_upstream(v.text(0), &refs_check).map(|_| ()),
            move |v, _, cx| {
                let upstream = rules::validate_upstream(v.text(0), &refs).ok().flatten();
                with_workspace(cx, move |ws, _, cx| {
                    let said = msg::prompt_dialog::upstream_title(&branch);
                    ws.backend.set_upstream(branch, upstream);
                    ws.say(said, cx);
                });
            },
        );
    }
}
