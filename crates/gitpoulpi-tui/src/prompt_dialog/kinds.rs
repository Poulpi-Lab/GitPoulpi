//! The questions the dialog asks: one constructor each.

use super::*;
use gitpoulpi_i18n::msg::prompt_dialog as msg;

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
}
