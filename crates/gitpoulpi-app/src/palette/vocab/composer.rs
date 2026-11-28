//! The palette's route to the AI writing help of a Markdown text box.
//!
//! The text boxes that carry the help (the conversation's composer, the new
//! pull request and issue forms, the review's comment box and Finish dialog)
//! are modal: while one has the keyboard, the palette used to stay closed, so
//! the help's two actions were reachable by key (`Alt+E`) and button only.
//! A host that holds a text box with the help lets `Ctrl+P` open the palette
//! over it; the palette then offers these two rows, and nothing else (the rest
//! is grayed out with [`ComposerAvail`]'s `modal` reason, `palette::availability`).

use gitpoulpi_i18n::msg::md_editor as msg;

/// What the help can be asked to do from the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerCmd {
    /// Open or close the Write with AI panel (the button, `Alt+E`).
    WriteWithAi,
    /// Forget the repository context kept for the next draft (the panel's
    /// Refresh context button).
    RefreshContext,
}

/// What the open text box allows (`None` in the palette context: none is
/// open). The reasons are resolved by the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComposerAvail {
    /// Why Refresh context does nothing (`None`: it can run).
    pub refresh: Option<&'static str>,
    /// The text box is a modal form or popup (not a view that hosts the
    /// palette itself): every other command waits until it is closed.
    pub modal: bool,
}

impl ComposerCmd {
    /// Why the command does nothing right now (`None`: it can run).
    pub fn unavailable(self, avail: Option<ComposerAvail>) -> Option<&'static str> {
        let Some(a) = avail else {
            return Some(msg::WHY_NO_COMPOSER.get());
        };
        match self {
            Self::WriteWithAi => None,
            Self::RefreshContext => a.refresh,
        }
    }
}
