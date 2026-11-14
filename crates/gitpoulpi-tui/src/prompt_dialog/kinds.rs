//! The questions the dialog asks: one constructor each.

use super::*;
use crate::combo::ComboOption;
use gitpoulpi_core::domain::{Ref, RefKind, Remote};
use gitpoulpi_i18n::msg::prompt_dialog as msg;
use gitpoulpi_i18n::msg::branch_remotes as msg_br;

impl PromptDialog {
    /// `Ctrl+G m` and the palette's `Stash With Message...`: an optional
    /// message (empty is fine), with the ✦ twin when AI is on.
    pub(crate) fn stash(ai_enabled: bool) -> Self {
        let mut dialog = Self::base(
            PromptKind::Stash,
            msg::STASH_TITLE.get().to_string(),
            msg::STASH_PLACEHOLDER.get(),
            msg::STASH_BUTTON.get().to_string(),
            false,
            "",
        );
        dialog.sync_ai(ai_enabled, false);
        dialog
    }

    /// `Create annotated tag here...`, step two: the message of the tag
    /// `name` at `at`, prefilled with an agent's proposed one (empty
    /// otherwise).
    pub(crate) fn tag_message(at: Oid, name: &str, prefill: &str) -> Self {
        let mut dialog = Self::base(
            PromptKind::TagMessage {
                at,
                name: name.to_string(),
            },
            msg::tag_new_title(name, Self::short(at)),
            msg::TAG_PLACEHOLDER.get(),
            msg::TAG_CREATE_BUTTON.get().to_string(),
            true,
            prefill,
        );
        dialog.proposed_by = crate::ui_intent::proposal::active()
            .filter(|a| a.dialog == crate::ui_intent::proposal::Dialog::Tag(at))
            .map(|a| a.client);
        dialog
    }

    /// `Annotate <tag>...` on an existing tag: its (multi-line) message.
    pub(crate) fn annotate_tag(tag: &str) -> Self {
        Self::base(
            PromptKind::AnnotateTag {
                tag: tag.to_string(),
            },
            msg::tag_annotate_title(tag),
            msg::TAG_PLACEHOLDER.get(),
            msg::TAG_ANNOTATE_BUTTON.get().to_string(),
            true,
            "",
        )
    }

    /// `Remotes of <branch>`: the upstream in a searchable list that also
    /// takes a typed `remote/branch` (every `remote/branch` but `HEAD`, the
    /// same-named branches first, the current upstream preselected and
    /// marked), over the table of the repository's remotes.
    ///
    /// The list opens with the dialog when there is no upstream to keep (the
    /// pick is the point); with one, the keyboard starts in the table.
    /// `status` is the daemon's answer, read while the dialog is up.
    pub(crate) fn branch_remotes(
        branch: &str,
        current: Option<&str>,
        refs: &[Ref],
        remotes: &[Remote],
        status: crate::daemon_link::Reply<gitpoulpi_core::domain::RemoteGroupStatus>,
    ) -> Self {
        let mut names: Vec<(String, String)> = refs
            .iter()
            .filter_map(|r| match &r.kind {
                RefKind::RemoteBranch { remote, name } if name != "HEAD" => {
                    Some((name.clone(), format!("{remote}/{name}")))
                }
                _ => None,
            })
            .collect();
        names.sort_by(|a, b| ((a.0 != branch), &a.1).cmp(&((b.0 != branch), &b.1)));
        names.dedup();
        let mut options: Vec<ComboOption> = names
            .into_iter()
            .map(|(_, full)| ComboOption::new(full.clone(), full))
            .collect();
        // An upstream that is no listed ref (never fetched) still shows.
        if let Some(cur) = current.filter(|c| !options.iter().any(|o| o.value == *c)) {
            options.insert(0, ComboOption::new(cur, cur));
        }
        let mut combo = Combo::new(msg::UPSTREAM_PLACEHOLDER.get())
            .with_search()
            .with_custom();
        combo.set_options(options);
        if let Some(cur) = current {
            combo.select(Some(cur.to_string()));
            combo.set_marks(vec![cur.to_string()]);
        } else {
            combo.open();
        }
        let names: Vec<String> = remotes.iter().map(|r| r.name.clone()).collect();
        let mut dialog = Self::base(
            PromptKind::BranchRemotes {
                branch: branch.to_string(),
                remotes: names,
            },
            msg_br::title(branch),
            "",
            msg_br::APPLY_BUTTON.get().to_string(),
            false,
            "",
        );
        dialog.combo = Some(RefCell::new(combo));
        dialog.remotes = Some(Box::new(crate::branch_remotes_dialog::RemotesPane::new(
            branch, remotes, current, status,
        )));
        dialog.set_focus(if current.is_some() {
            PromptPart::Rows
        } else {
            PromptPart::Field
        });
        dialog
    }

    /// `LFS: Track Pattern...`: space-separated patterns, as `git lfs track` takes them.
    pub(crate) fn lfs_track() -> Self {
        Self::base(
            PromptKind::LfsTrack,
            msg::LFS_TRACK_TITLE.get().to_string(),
            msg::LFS_TRACK_PLACEHOLDER.get(),
            msg::TRACK_BUTTON.get().to_string(),
            false,
            "",
        )
    }

    /// `LFS: Untrack Pattern...`: the tracked patterns in a list that also
    /// takes a typed one; the list opens with the dialog.
    pub(crate) fn lfs_untrack(tracked: &[String]) -> Self {
        let mut combo = Combo::new(msg::LFS_UNTRACK_PLACEHOLDER.get())
            .with_search()
            .with_custom();
        combo.set_options(
            tracked
                .iter()
                .map(|p| ComboOption::new(p.clone(), p.clone()))
                .collect(),
        );
        combo.open();
        let mut dialog = Self::base(
            PromptKind::LfsUntrack,
            msg::LFS_UNTRACK_TITLE.get().to_string(),
            "",
            msg::UNTRACK_BUTTON.get().to_string(),
            false,
            "",
        );
        dialog.combo = Some(RefCell::new(combo));
        dialog
    }
}
