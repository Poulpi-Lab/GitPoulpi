//! The Preferences screen's model types: sections, controls, targets, actions and `Field`.

use gitpoulpi_config::Settings;

use super::ai_page::{self, AiButton, AiTarget};
use super::repo_pages::{RpButton, RpTarget};
use super::workflow_page::{WfButton, WfTarget};

/// One page of the screen: the shared page list (`gitpoulpi_app::settings::Page`),
/// in the sidebar's order.
pub(crate) type PrefSection = gitpoulpi_app::settings::Page;

/// What only the terminal UI says about a page: which page an IPC
/// settings-page id names.
pub(crate) trait PrefSectionExt: Sized {
    /// The page a settings-page id names (the legacy ids included).
    fn from_page(page: gitpoulpi_ipc::ui::SettingsPage) -> Self;
}

impl PrefSectionExt for PrefSection {
    fn from_page(page: gitpoulpi_ipc::ui::SettingsPage) -> Self {
        use gitpoulpi_ipc::ui::SettingsPage as P;
        match page {
            P::General => PrefSection::General,
            P::Launcher => PrefSection::Launcher,
            P::ConflictPreventionGlobal => PrefSection::ConflictGlobal,
            P::Sync => PrefSection::Sync,
            P::Profiles => PrefSection::Profiles,
            P::Ssh => PrefSection::Ssh,
            P::Integrations => PrefSection::Integrations,
            P::Ai => PrefSection::Ai,
            P::AgentsMcp => PrefSection::AgentsMcp,
            P::Appearance | P::UiCustomization => PrefSection::Appearance,
            P::LogGraph => PrefSection::LogGraph,
            P::DiffEditor | P::Editor => PrefSection::DiffEditor,
            P::OpenIn => PrefSection::OpenIn,
            P::Images => PrefSection::Images,
            P::KeyboardMouse | P::KeyboardShortcuts => PrefSection::Keybindings,
            P::Sequences => PrefSection::Sequences,
            P::GitEncoding => PrefSection::GitAndEncoding,
            P::RepoAi => PrefSection::RepoAi,
            P::Workflow => PrefSection::Workflow,
            P::SparseCheckout => PrefSection::SparseCheckout,
            P::Lfs => PrefSection::Lfs,
            P::Commit => PrefSection::Commit,
            P::MergeBehavior => PrefSection::MergeBehavior,
            P::CommitTemplate => PrefSection::CommitTemplate,
            P::ConflictPrevention => PrefSection::ConflictPrevention,
            P::SshCredentials => PrefSection::RepoAuth,
        }
    }
}

/// Which half of the screen the keyboard is driving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PrefFocus {
    Sections,
    /// The master list of a master-detail page (`PrefSection::has_list`).
    List,
    Fields,
}

/// What a row holds and how it is edited: the shared control, with this
/// screen's buttons (`gitpoulpi_app::settings::Control`).
pub(super) type Control = gitpoulpi_app::settings::Control<PrefAction>;
/// One inline button of a [`Control::Buttons`] row.
pub(super) type Btn = gitpoulpi_app::settings::Btn<PrefAction>;
pub(super) use gitpoulpi_app::settings::QuantityUnit;

/// Where a field's value is persisted, and therefore what committing it
/// has to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Target {
    /// A dotted key in `config.toml`.
    Global(&'static str),
    /// A field of this repository's `RepoPreferences`.
    Repo(RepoField),
    /// A field of the profile at this index.
    ProfileField(usize, TextSlot),
    /// A row of the AI page (`ai_page.rs`).
    #[cfg_attr(not(feature = "ai"), allow(dead_code))] // only the AI Settings page builds it
    Ai(AiTarget),
    Action(PrefAction),
    /// A row of the Open in page (`open_in_page.rs`).
    OpenIn(OpenInTarget),
    /// A row of the Images page's editor list (`images_page.rs`); the rows
    /// are the Open in page's, over `[[ui.image_editors]]`.
    ImageEditor(OpenInTarget),
    /// A row of the Workflow page (`workflow_page.rs`).
    Workflow(WfTarget),
    /// A row of the Sequences page (`sequences_page.rs`).
    Sequence(super::sequences_page::SeqTarget),
    /// A row of the Sparse checkout page (`repo_pages.rs`).
    RepoPage(RpTarget),
    /// A row of the SSH & credentials page (`repo_auth_page`).
    RepoAuth(super::repo_auth_page::RaTarget),
    /// Decoration: nothing to commit.
    None,
}

/// The Open in page's rows that are not plain config keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OpenInTarget {
    Label,
    Command,
    /// The variable chips: append one to the command.
    Chips,
    /// "Can open a worktree" (a checkbox).
    Worktree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OpenInButton {
    Test,
    Restore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RepoField {
    Encoding,
    AutoCrlf,
    Eol,
    Lfs,
    /// Commit page: push after each commit.
    PushAfterCommit,
    /// Commit page: skip git hooks by default.
    SkipHooks,
    /// Merge options page: squash.
    SquashMerge,
    /// Merge options page: the fast-forward policy of this repository.
    MergeFastForward,
    /// Message template page: apply to empty commit messages.
    TemplateApply,
    /// Message template page: remove comments from commit messages.
    TemplateStripComments,
    /// Conflict prevention page: detection on for this repository.
    ConflictDetection,
    /// Git page: the remote at this index of the repository's remotes is an
    /// extra push target of new branches (`default_push_targets`).
    PushTarget(usize),
}

impl RepoField {
    /// The git config key this row writes (through the daemon), for the two
    /// rows that are git's own repository config rather than `repo-state.toml`.
    pub(super) fn git_config_key(self) -> Option<&'static str> {
        match self {
            RepoField::AutoCrlf => Some(gitpoulpi_core::repo_config::CORE_AUTOCRLF),
            RepoField::Eol => Some(gitpoulpi_core::repo_config::CORE_EOL),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextSlot {
    Id,
    Name,
    AuthorName,
    AuthorEmail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PrefAction {
    /// Ask the event loop to run `git lfs install --local` here (the
    /// screen has no daemon link of its own; see `take_lfs_init_request`).
    InitializeLfs,
    GenerateSshKey,
    ForgetCredentials,
    /// A button of the AI page.
    Ai(AiButton),
    /// A button of the Open in page.
    OpenIn(OpenInButton),
    /// A button of the Images page's editor list.
    ImageEditor(OpenInButton),
    /// A shortcut button of General's Launcher group.
    Launcher(super::launcher_page::LauncherButton),
    /// A button of the "This installation" rows.
    Installation(super::launcher_install::InstallButton),
    /// A button of General's command alias and PATH rows.
    Tools(super::launcher_alias::ToolsButton),
    /// A button of the File manager menu rows.
    FolderMenu(super::launcher_folder_menu::FolderMenuButton),
    /// A button of the Workflow page.
    Workflow(WfButton),
    /// A button of the Sequences page.
    Sequence(super::sequences_page::SeqButton),
    /// A button of the access token row on Agents & MCP.
    #[cfg(feature = "mcp")]
    McpSecurity(super::mcp_security::McpButton),
    /// A button of the Sparse checkout page.
    RepoPage(RpButton),
    /// A button of the SSH & credentials page.
    RepoAuth(super::repo_auth_page::RaButton),
}

/// A painted chip: (field, chip, line in the field, column, width).
pub(super) type ChipCell = (usize, usize, usize, u16, u16);
/// A row's own control cell: (field, is_input, line, dx, width).
pub(super) type CtrlCell = (usize, bool, usize, u16, u16);

/// One rendered setting.
pub(super) struct Field {
    pub(super) label: String,
    pub(super) control: Control,
    pub(super) help: Option<String>,
    pub(super) warning: Option<String>,
    /// The warning is drawn in the danger color (a setting that exposes
    /// something when it is left on or off the wrong way).
    pub(super) danger: bool,
    pub(super) target: Target,
    /// `false` renders the `not wired yet` note. Derived from
    /// `LIVE_KEYS` for global keys; stated explicitly otherwise.
    pub(super) live: bool,
    /// `Some(reason)` makes the row **not editable at all** - neither
    /// the keyboard nor the mouse can select it (`selectable` returns
    /// `false`), and the reason is rendered in place of a value rather
    /// than under it, so the row cannot be mistaken for a working one.
    /// Guard-tested: every shown row is `live` or `coming`.
    pub(super) coming: Option<String>,
    /// What an empty `Area` shows grayed instead of `(empty)` (the global
    /// text a repository override would inherit).
    pub(super) placeholder: Option<String>,
    /// A read-only value's piece that gives way first when the row is narrow:
    /// cut in the middle with an ellipsis, never dropped (the install path).
    pub(super) ellipsize: Option<String>,
}

impl Field {
    pub(super) fn global(label: &str, key: &'static str, control: Control) -> Self {
        Self {
            label: label.to_string(),
            control,
            help: None,
            warning: None,
            danger: false,
            target: Target::Global(key),
            live: Settings::key_is_implemented(key),
            coming: None,
            placeholder: None,
            ellipsize: None,
        }
    }

    pub(super) fn repo(label: &str, field: RepoField, control: Control) -> Self {
        Self {
            label: label.to_string(),
            control,
            help: None,
            warning: None,
            danger: false,
            target: Target::Repo(field),
            live: false,
            coming: None,
            placeholder: None,
            ellipsize: None,
        }
    }

    pub(super) fn plain(label: &str, control: Control, target: Target) -> Self {
        Self {
            label: label.to_string(),
            control,
            help: None,
            warning: None,
            danger: false,
            target,
            live: true,
            coming: None,
            placeholder: None,
            ellipsize: None,
        }
    }

    /// A row that is one button, labelled by its own text.
    pub(super) fn button(label: &str, action: PrefAction) -> Self {
        Self::plain(
            label,
            Control::Buttons(vec![Btn::new(label, action)]),
            Target::Action(action),
        )
    }

    /// A row with `label` in the label column and these buttons in the
    /// value column.
    pub(super) fn buttons(label: &str, buttons: Vec<Btn>) -> Self {
        let target = buttons
            .first()
            .map_or(Target::None, |b| Target::Action(b.action));
        Self::plain(label, Control::Buttons(buttons), target)
    }

    /// Whether the label is only the text of the row's single button.
    pub(super) fn is_self_labelled(&self) -> bool {
        matches!(&self.control, Control::Buttons(b)
            if self.label.is_empty() || (b.len() == 1 && b[0].label == self.label))
    }

    pub(super) fn heading(label: &str) -> Self {
        Self {
            label: label.to_string(),
            control: Control::Heading,
            help: None,
            warning: None,
            danger: false,
            target: Target::None,
            live: true,
            coming: None,
            placeholder: None,
            ellipsize: None,
        }
    }

    pub(super) fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    /// Marks `piece` of the value as the part that is cut in the middle when
    /// the row is too narrow for the whole text.
    pub(super) fn with_ellipsis_of(mut self, piece: &str) -> Self {
        self.ellipsize = Some(piece.to_string());
        self
    }

    pub(super) fn with_placeholder(mut self, text: &str) -> Self {
        self.placeholder = Some(text.to_string());
        self
    }

    pub(super) fn with_warning(mut self, warning: &str) -> Self {
        self.warning = Some(warning.to_string());
        self
    }

    /// A warning drawn in the danger color.
    pub(super) fn with_danger(mut self, warning: &str) -> Self {
        self.warning = Some(warning.to_string());
        self.danger = true;
        self
    }

    /// Marks the row **coming**: not editable, with `reason` shown in
    /// place of both the "not wired yet" note and the value it would
    /// otherwise commit. See the `coming` field's doc for what this
    /// guarantees.
    /// Marks a repo-scoped row as read by a component (the TUI's LFS
    /// display reads `lfs_enabled` through `lfs_ui::resolved_enabled`).
    pub(super) fn live(mut self) -> Self {
        self.live = true;
        self
    }

    pub(super) fn coming(mut self, reason: &str) -> Self {
        self.coming = Some(reason.to_string());
        self
    }

    pub(super) fn selectable(&self) -> bool {
        self.coming.is_none()
            && !matches!(
                self.control,
                Control::Heading | Control::Info(_) | Control::Swatches(_)
            )
    }

    /// The text shown for the raw choice `raw` of this row: the translated
    /// word for a schema value (`ask`), the raw text otherwise. The stored
    /// value stays `raw`.
    pub(super) fn option_label(&self, raw: &str) -> String {
        match self.target {
            Target::Global(key) => super::choice_labels::label(key, raw),
            _ => raw.to_string(),
        }
    }

    /// The right-hand side, as rendered.
    pub(super) fn value_text(&self) -> String {
        gitpoulpi_app::settings::value_text(
            self.coming.as_deref(),
            &self.control,
            |raw| self.option_label(raw),
            ai_page::first_line_preview,
        )
    }
}

#[cfg(test)]
mod build_tests {
    use super::*;

    #[test]
    fn no_page_is_hidden_in_a_build_with_every_feature() {
        assert!(PrefSection::ALL
            .iter()
            .all(|s| s.is_built() || !cfg!(feature = "full")));
    }

    /// Keyboard & mouse sits right after Appearance; the global Conflict
    /// prevention page (specific) closes the global group, just before the
    /// repository-specific ones.
    #[test]
    fn the_global_pages_are_in_the_owners_order() {
        let at = |p: PrefSection| PrefSection::ALL.iter().position(|s| *s == p).unwrap();
        assert_eq!(
            at(PrefSection::Keybindings),
            at(PrefSection::Appearance) + 1
        );
        assert_eq!(
            at(PrefSection::ConflictGlobal) + 1,
            at(PrefSection::GitAndEncoding)
        );
    }

    #[test]
    fn the_ai_pages_follow_the_ai_feature() {
        for section in [PrefSection::Ai, PrefSection::RepoAi] {
            assert_eq!(section.is_built(), cfg!(feature = "ai"));
        }
        assert!(PrefSection::General.is_built());
    }

    #[test]
    fn stepping_skips_pages_the_build_lacks() {
        let ai = PrefSection::ALL
            .iter()
            .position(|s| *s == PrefSection::Ai)
            .unwrap();
        let hide_ai = |s: PrefSection| s != PrefSection::Ai;
        assert_eq!(PrefSection::step(ai - 1, 1, hide_ai), ai + 1);
        assert_eq!(PrefSection::step(ai + 1, -1, hide_ai), ai - 1);
        assert_eq!(PrefSection::step(0, -1, hide_ai), 0);
        let last = PrefSection::ALL.len() - 1;
        assert_eq!(PrefSection::step(last, 1, hide_ai), last);
        // Nothing hidden: a plain step.
        assert_eq!(PrefSection::step(2, 1, |_| true), 3);
    }
}
