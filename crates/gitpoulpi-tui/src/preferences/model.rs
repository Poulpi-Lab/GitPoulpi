//! The Preferences screen's model types: sections, controls, targets, actions and `Field`.

use gitpoulpi_config::Settings;

use super::ai_page::{self, AiButton, AiTarget};
use super::repo_pages::{RpButton, RpTarget};
use super::workflow_page::{WfButton, WfTarget};
use gitpoulpi_i18n::msg::{settings_general, settings_open_in, settings_screen as screen};

/// One page of the screen. The order is the sidebar's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrefSection {
    General,
    Sync,
    Profiles,
    Ssh,
    Integrations,
    Ai,
    AgentsMcp,
    Appearance,
    LogGraph,
    DiffEditor,
    OpenIn,
    Images,
    Keybindings,
    GitAndEncoding,
    RepoAi,
    Workflow,
    SparseCheckout,
    Lfs,
    Commit,
    MergeBehavior,
    CommitTemplate,
    ConflictPrevention,
    RepoAuth,
}

impl PrefSection {
    /// A master-detail page: a `ListEditor` on the left (stacked above on a
    /// narrow body) and the selected item's rows on the right. Profiles, AI,
    /// Open in and Images so far; Workflow follows (plan M5).
    pub(crate) fn has_list(self) -> bool {
        matches!(
            self,
            PrefSection::Profiles
                | PrefSection::Ai
                | PrefSection::OpenIn
                | PrefSection::Images
                | PrefSection::Workflow
        )
    }

    pub(crate) const ALL: &'static [PrefSection] = &[
        PrefSection::General,
        PrefSection::Sync,
        PrefSection::Profiles,
        PrefSection::Ssh,
        PrefSection::Integrations,
        PrefSection::Ai,
        PrefSection::AgentsMcp,
        PrefSection::Appearance,
        PrefSection::LogGraph,
        PrefSection::DiffEditor,
        PrefSection::OpenIn,
        PrefSection::Images,
        PrefSection::Keybindings,
        PrefSection::GitAndEncoding,
        PrefSection::RepoAuth,
        PrefSection::RepoAi,
        PrefSection::Workflow,
        PrefSection::SparseCheckout,
        PrefSection::Lfs,
        PrefSection::Commit,
        PrefSection::MergeBehavior,
        PrefSection::CommitTemplate,
        PrefSection::ConflictPrevention,
    ];

    pub(crate) fn title(self) -> &'static str {
        match self {
            PrefSection::General => screen::SECTION_GENERAL.get(),
            PrefSection::Sync => screen::SECTION_SYNC.get(),
            PrefSection::Profiles => screen::SECTION_PROFILES.get(),
            PrefSection::Ssh => screen::SECTION_SSH.get(),
            PrefSection::Integrations => screen::SECTION_INTEGRATIONS.get(),
            PrefSection::Ai => screen::SECTION_AI.get(),
            PrefSection::AgentsMcp => screen::SECTION_AGENTS.get(),
            PrefSection::Appearance => screen::SECTION_UI.get(),
            PrefSection::LogGraph => screen::SECTION_LOG.get(),
            PrefSection::Keybindings => screen::SECTION_KEYBINDINGS.get(),
            PrefSection::DiffEditor => screen::SECTION_EDITOR.get(),
            PrefSection::OpenIn => settings_open_in::SECTION_TITLE.get(),
            PrefSection::Images => gitpoulpi_i18n::msg::settings_images::SECTION_TITLE.get(),
            PrefSection::GitAndEncoding => screen::SECTION_GIT.get(),
            PrefSection::RepoAi => screen::SECTION_AI.get(),
            PrefSection::Workflow => gitpoulpi_i18n::msg::settings_workflow::SECTION_TITLE.get(),
            PrefSection::SparseCheckout => gitpoulpi_i18n::msg::settings_repo::SECTION_SPARSE.get(),
            PrefSection::Lfs => gitpoulpi_i18n::msg::settings_repo::SECTION_LFS.get(),
            PrefSection::Commit => gitpoulpi_i18n::msg::settings_repo::SECTION_COMMIT.get(),
            PrefSection::MergeBehavior => gitpoulpi_i18n::msg::settings_repo::SECTION_MERGE.get(),
            PrefSection::CommitTemplate => {
                gitpoulpi_i18n::msg::settings_repo::SECTION_TEMPLATE.get()
            }
            PrefSection::ConflictPrevention => {
                gitpoulpi_i18n::msg::conflict_prevention::SECTION.get()
            }
            PrefSection::RepoAuth => gitpoulpi_i18n::msg::repo_auth::SECTION.get(),
        }
    }

    /// The stable id the IPC/MCP `ui show settings` surface names this page
    /// by (`gitpoulpi_ipc::ui::SettingsPage::IDS`).
    pub(super) const fn id(self) -> &'static str {
        match self {
            PrefSection::General => "general",
            PrefSection::Sync => "sync",
            PrefSection::Profiles => "profiles",
            PrefSection::Ssh => "ssh",
            PrefSection::Integrations => "integrations",
            PrefSection::Ai => "ai",
            PrefSection::AgentsMcp => "agents-mcp",
            PrefSection::Appearance => "appearance",
            PrefSection::LogGraph => "log-graph",
            PrefSection::DiffEditor => "diff-editor",
            PrefSection::OpenIn => "open-in",
            PrefSection::Images => "images",
            PrefSection::Keybindings => "keyboard-mouse",
            PrefSection::GitAndEncoding => "git-encoding",
            PrefSection::RepoAi => "repo-ai",
            PrefSection::Workflow => "workflow",
            PrefSection::SparseCheckout => "sparse-checkout",
            PrefSection::Lfs => "lfs",
            PrefSection::Commit => "commit",
            PrefSection::MergeBehavior => "merge-behavior",
            PrefSection::CommitTemplate => "commit-template",
            PrefSection::ConflictPrevention => "conflict-prevention",
            PrefSection::RepoAuth => "ssh-credentials",
        }
    }

    /// The page a settings-page id names (the legacy ids included).
    pub(super) fn from_page(page: gitpoulpi_ipc::ui::SettingsPage) -> Self {
        use gitpoulpi_ipc::ui::SettingsPage as P;
        match page {
            P::General => PrefSection::General,
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

    /// The cargo feature this page belongs to; `None` = always built.
    pub(super) const fn requires(self) -> Option<crate::build_features::Feature> {
        match self {
            PrefSection::Ai | PrefSection::RepoAi => Some(crate::build_features::Feature::Ai),
            PrefSection::Integrations => Some(crate::build_features::Feature::Hosting),
            PrefSection::Lfs => Some(crate::build_features::Feature::Lfs),
            _ => None,
        }
    }

    /// Whether this build shows the page.
    pub(super) fn is_built(self) -> bool {
        crate::build_features::offered(self.requires())
    }

    /// The page index reached from `from` by `delta` steps over the pages
    /// `built` accepts (stops at the first or last such page).
    pub(super) fn step(from: usize, delta: isize, built: impl Fn(PrefSection) -> bool) -> usize {
        let len = Self::ALL.len() as isize;
        let step = delta.signum();
        let mut at = from as isize;
        for _ in 0..delta.abs() {
            let mut probe = at + step;
            while (0..len).contains(&probe) && !built(Self::ALL[probe as usize]) {
                probe += step;
            }
            if !(0..len).contains(&probe) {
                break;
            }
            at = probe;
        }
        at as usize
    }

    /// `true` for the one page under `Repo-Specific Settings`.
    pub(super) fn is_repo_scoped(self) -> bool {
        matches!(
            self,
            PrefSection::GitAndEncoding
                | PrefSection::RepoAi
                | PrefSection::Workflow
                | PrefSection::SparseCheckout
                | PrefSection::Lfs
                | PrefSection::Commit
                | PrefSection::MergeBehavior
                | PrefSection::CommitTemplate
                | PrefSection::ConflictPrevention
                | PrefSection::RepoAuth
        )
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

/// What a row does when it is activated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Control {
    Toggle(bool),
    Number {
        value: i64,
        min: i64,
        max: i64,
    },
    Text(String),
    /// A number and a unit on one line, `[ 2 ] [weeks v]`: a whole number
    /// typed into a field and a unit picked from a dropdown. The row's
    /// target stores `value x unit.weight` (a base unit such as days);
    /// `unit_key` stores the unit's own key. Part 0 is the number, part 1
    /// the unit (`PreferencesState::quantity_part`).
    Quantity {
        value: i64,
        /// Index into `units` of the unit shown.
        unit: usize,
        units: Vec<QuantityUnit>,
        /// Largest stored total (in base units); the number may not exceed
        /// `max_total / weight` of the chosen unit.
        max_total: i64,
        /// Whether `0` is a valid number (a timer that can be turned off),
        /// else the number is at least 1.
        allow_zero: bool,
        /// The config key the unit's `key` is written to.
        unit_key: &'static str,
    },
    Choice {
        options: Vec<String>,
        index: usize,
    },
    /// A dropdown of known values (label, value) that is also free text
    /// when `custom`: the value is a string, not an index. Opens the
    /// shared combobox like `Choice`.
    #[cfg_attr(not(feature = "ai"), allow(dead_code))] // only the AI Settings page builds it
    Combo {
        options: Vec<(String, String)>,
        value: String,
        custom: bool,
    },
    /// One or more inline buttons in the value column (`Btn`). Left/Right
    /// move between them, Enter presses the focused one, a click presses
    /// the one under it.
    Buttons(Vec<Btn>),
    /// A sub-heading inside a page. Not selectable.
    Heading,
    /// Read-only text on the right (a live preview, a derived value).
    Info(String),
    /// Read-only color chips on the right (the theme swatch strip).
    Swatches(Vec<ratatui::style::Color>),
    /// A row of quick-insert buttons (the AI page's chips). One selectable
    /// row; Left/Right pick the chip, Enter inserts it, a click on one
    /// inserts it.
    Chips(Vec<String>),
    /// A multi-line text, drawn as a box under its label; Enter opens the
    /// multi-line editor.
    #[cfg_attr(not(feature = "ai"), allow(dead_code))] // only the AI Settings page builds it
    Area(String),
}

/// One unit of a [`Control::Quantity`] row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct QuantityUnit {
    /// The unit's name for the current number ("week" or "weeks").
    pub(super) label: String,
    /// What the unit key stores for it.
    pub(super) key: &'static str,
    /// How many base units (days, minutes, MiB...) one of it is.
    pub(super) weight: i64,
}

/// One inline button of a [`Control::Buttons`] row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Btn {
    pub(super) label: String,
    pub(super) action: PrefAction,
    pub(super) enabled: bool,
    /// A text value: takes all the room the row's other buttons leave.
    pub(super) fill: bool,
}

impl Btn {
    pub(super) fn new(label: &str, action: PrefAction) -> Self {
        Self {
            label: label.to_string(),
            action,
            enabled: true,
            fill: false,
        }
    }

    /// This button is a long-text value: it fills the row.
    #[cfg_attr(not(feature = "ai"), allow(dead_code))] // only the AI Settings page uses it
    pub(super) fn filling(mut self) -> Self {
        self.fill = true;
        self
    }
}

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
    /// Message template page: apply to empty commit messages.
    TemplateApply,
    /// Message template page: remove comments from commit messages.
    TemplateStripComments,
    /// Conflict prevention page: detection on for this repository.
    ConflictDetection,
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
    /// A button of General's command alias and PATH rows.
    Tools(super::launcher_alias::ToolsButton),
    /// A button of the Workflow page.
    Workflow(WfButton),
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
        }
    }

    pub(super) fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
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
    #[cfg(any(feature = "mcp", feature = "ai"))]
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
        if let Some(reason) = &self.coming {
            if reason == settings_general::UNDO_OFF_NOTE.get()
                || super::launcher_page::is_note(reason)
                || super::launcher_alias::is_note(reason)
                || super::repo_pages::is_note(reason)
            {
                return reason.clone();
            }
            return screen::coming_value(reason);
        }
        match &self.control {
            Control::Toggle(on) => (if *on { "[x]" } else { "[ ]" }).to_string(),
            Control::Number { value, .. } => value.to_string(),
            Control::Text(value) if value.is_empty() => screen::NOT_SET.get().to_string(),
            Control::Text(value) => value.clone(),
            Control::Quantity {
                value, unit, units, ..
            } => match units.get(*unit) {
                Some(u) => format!("{value} {}", u.label),
                None => value.to_string(),
            },
            Control::Choice { options, index } => options
                .get(*index)
                .map(|o| self.option_label(o))
                .unwrap_or_else(|| screen::NONE.get().to_string()),
            Control::Combo { options, value, .. } => options
                .iter()
                .find(|(_, v)| v == value)
                .map(|(l, _)| l.clone())
                .filter(|_| !value.is_empty() || options.iter().any(|(_, v)| v.is_empty()))
                .unwrap_or_else(|| {
                    if value.is_empty() {
                        screen::NOT_SET.get().to_string()
                    } else {
                        value.clone()
                    }
                }),
            Control::Buttons(buttons) => buttons
                .iter()
                .map(|b| b.label.as_str())
                .collect::<Vec<_>>()
                .join(" | "),
            Control::Heading => String::new(),
            Control::Info(text) => text.clone(),
            Control::Swatches(_) => String::new(),
            Control::Chips(labels) => labels.join(" | "),
            Control::Area(text) => ai_page::first_line_preview(text),
        }
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
