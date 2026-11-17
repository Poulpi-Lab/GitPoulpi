//! The frame around the panes: pane focus and the right panel/central
//! view choices, the toolbar, the `HitTarget` hit-testing registry every
//! click goes through, the mouse-cursor shape, collapsed-pane strips and
//! buttons, the settings gear, the help overlay's text and the command
//! palette popup's geometry.
//!
//! `mod.rs` holds the focus, pane and navigation model; `toolbar`, `hit`,
//! `help_text` and `widgets` hold the rest and are re-exported flat, so
//! callers keep using `chrome::*` paths.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    Refs,
    Log,
    Status,
}

/// Which side pane a collapse/restore affordance or hit target belongs to.
/// Deliberately not `Focus` - the center (`Focus::Log`) can never be
/// collapsed, and folding it into `Focus` would let a match compile while
/// silently never handling `Focus::Log`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneSide {
    Refs,
    Status,
}

/// What the right-hand column is currently showing - deliberately its
/// own piece of state, not derived from `focus`. Moving keyboard focus to
/// the Status pane (`3`/Tab) needs to route j/k there so the user can
/// browse it, but must NOT silently discard a commit's detail just
/// because the keyboard cursor passed through that pane - the user
/// found the old focus-derived version disorienting. `CommitDetail` is
/// entered whenever the Log selection actually changes; `Status` is
/// entered only by a real Status-pane interaction (moving its selection,
/// or clicking a row in it) so simply tabbing past Status doesn't hide an
/// in-progress commit inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RightPanel {
    Status,
    CommitDetail,
}

/// What the *central* column is showing. Same shape as `RightPanel` (its
/// only precedent in this file for a view-mode enum) and for the same
/// reason: a mode the user put the pane into, not something derived from
/// `focus`.
///
/// `Log` is the commit table this pane has always been. `Diff` replaces it
/// wholesale with the selected file's diff -- picking a file in the commit-detail list takes over the main area
/// rather than cramming a diff into the narrow right-hand column. Note that
/// every log-table mouse handler (row hit-testing, column drags, the
/// header's right-click menu) has to be gated on this being `Log`,
/// since all of them assume the central area *is* the table.
///
/// `File` is the File View / Blame / File History
/// reader. It takes the central column exactly as the Diff view does, with
/// the same bar, and it is *derived*: `file_panel::sync_central_view` keeps it equal to
/// "a `FileInsight` is open", so every existing way of opening one
/// (the diff bar, the context menus, the palette) lands here without
/// each having to remember to switch the view.
///
/// `Editor` is Edit in Working Directory, derived the
/// same way from "an `editor::Editor` is open". It sits over the File
/// View or diff it was opened from, which stays open underneath and is
/// what closing the editor goes back to.
///
/// `Merge` is the three-way merge conflict editor (`merge_view`), derived
/// from "a `MergeView` is open" like `Editor`, and on top of everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CentralView {
    Log,
    Diff,
    File,
    Editor,
    Merge,
}

impl Focus {
    fn toggled(self) -> Self {
        match self {
            Focus::Refs => Focus::Log,
            Focus::Log => Focus::Status,
            Focus::Status => Focus::Refs,
        }
    }

    /// Whether this is a side pane and it is currently collapsed - the
    /// center (`Log`) is never collapsible, so it always answers `false`.
    fn is_collapsed(self, left_collapsed: bool, right_collapsed: bool) -> bool {
        match self {
            Focus::Refs => left_collapsed,
            Focus::Status => right_collapsed,
            Focus::Log => false,
        }
    }

    /// The reverse cycle, for `Shift+Tab`.
    fn toggled_back(self) -> Self {
        match self {
            Focus::Refs => Focus::Status,
            Focus::Log => Focus::Refs,
            Focus::Status => Focus::Log,
        }
    }
}

/// `Tab`/`Shift+Tab`'s actual cycle: `toggled`/`toggled_back`, skipping
/// any side pane that is currently collapsed. Bounded to three steps --
/// one full lap of the three-way cycle - so a state with both side panes
/// collapsed cannot loop forever; `Log` is never collapsible, so that lap
/// always lands on it.
pub(crate) fn cycle_focus_skipping_collapsed(
    mut current: Focus,
    backward: bool,
    left_collapsed: bool,
    right_collapsed: bool,
) -> Focus {
    for _ in 0..3 {
        current = if backward {
            current.toggled_back()
        } else {
            current.toggled()
        };
        if !current.is_collapsed(left_collapsed, right_collapsed) {
            return current;
        }
    }
    Focus::Log
}

/// What `Ctrl+N` navigation-mode's next keypress means - a pure mapping
/// from `KeyCode` kept separate from the event loop's giant `match` so the
/// prefix's dispatch table has a test that doesn't need a pty (see
/// `navigation_key_action`'s tests below). The event loop still owns
/// *applying* each variant (it needs a dozen locals: `left_pane_collapsed`,
/// `focus`, `session`, ...) - this only answers "which one did they ask
/// for".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavigationAction {
    /// `Ctrl+N Shift+H` - collapse/restore the Refs pane. Replaces the old bare
    /// `Ctrl+K`.
    CollapseLeft,
    /// `Ctrl+N Shift+L` - collapse/restore the Status pane. Replaces the old
    /// bare `Ctrl+L`.
    CollapseRight,
    /// `Ctrl+N <badge>` - focus whichever sub-pane that badge names in
    /// *this frame*. There is deliberately no fixed "`1` is always Refs"
    /// mapping any more (that died with the bare `1`/`2`/`3` bindings,
    /// see the module doc's "Bare 1/2/3" note) - badges are assigned
    /// fresh every time the prefix opens, in `navigation_badge_targets`,
    /// and this variant only carries *which character was typed*. The
    /// event loop looks it up in the same list it just drew the badges
    /// from, so the glyph on screen and the key that does something can
    /// never disagree.
    FocusByBadge(char),
    /// `Ctrl+N ,` - open settings, the same place bare `Ctrl+,` and the
    /// palette's `App: Settings...` go.
    OpenSettings,
    /// `Ctrl+N a` - open the About window, the 🐙 in the title bar.
    OpenAbout,
    /// `Ctrl+N Shift+A` - open the Activity Log, the status bar's Activity button.
    OpenActivityLog,
    /// `Ctrl+N Shift+G` - the global Activity Log (every repository, clones and
    /// workspace tasks), the Home tab's button.
    OpenGlobalLog,
    /// `Ctrl+N Shift+J` - the background jobs panel, the status bar indicator's label.
    ShowJobs,
    /// `Ctrl+N u` - the update panel, the version chip in the title bar.
    ShowUpdates,
    /// `Ctrl+N ?` - toggle the help overlay, the same place bare `?` goes.
    ToggleHelp,
    /// `Ctrl+N Tab` - open the tab picker: a popup
    /// listing every open repository tab, filtered by typing, that
    /// switches to the one chosen. The same place the palette's
    /// `Repository: Select Tab...` goes (`PaletteCommand::SelectTab`).
    SelectTab,
    /// `Ctrl+N t` - open the theme picker, the palette's `App: Change
    /// Theme...` (`PaletteCommand::ChangeTheme`).
    ChangeTheme,
    /// `Ctrl+N l` - open the language picker, the palette's `App: Change
    /// Language...` (`PaletteCommand::ChangeLanguage`).
    ChangeLanguage,
    /// `Ctrl+N Shift+T` - the graph color theme picker, the palette's
    /// `App: Change Graph Color Theme...`.
    ChangeGraphColorTheme,
    /// `Ctrl+N Shift+Y` - the palette's `App: Toggle Graph Row Tints`.
    ToggleGraphRowTints,
    /// `Ctrl+N +` (or `=`) - grow the active Refs sidebar section by a
    /// row, the keyboard twin of dragging its bottom border down.
    GrowSection,
    /// `Ctrl+N -` - shrink the active Refs sidebar section by a row.
    ShrinkSection,
    /// `Ctrl+N ]` / `Ctrl+N [` - grow / shrink the commit message block
    /// of the right panel by a row (its splitter's keyboard twin).
    GrowMessage,
    ShrinkMessage,
    /// `Ctrl+N Left` / `Right` / `Shift+Left` / `Shift+Right` - move a pane
    /// divider one notch, the palette's `Navigation: Widen Refs Pane` and
    /// its three siblings.
    Resize(crate::pane_layout::PaneResize),
    /// `Ctrl+N Enter` - show the view an AI agent offered (the footer
    /// chip). The twin of clicking the chip.
    AgentShow,
    /// `Ctrl+N Backspace` - dismiss the chip, else go back to the view
    /// before the agent's last change.
    AgentBack,
    /// `Ctrl+N m` - arrange the open dialog: its arrows move it, Shift+arrows
    /// resize it (the keyboard twin of dragging its title bar and borders).
    ArrangeDialog,
    /// `Esc`, or any key this table doesn't name - back to `Normal` with
    /// nothing done.
    Cancel,
}

/// See [`NavigationAction`]. `Esc` and every unrecognized key both cancel -
/// there is no way to type a wrong key here and get stuck. `l`/`,`/
/// `?`/`t` are reserved (never handed out as badges, see
/// `navigation_badge_alphabet`); every other letter or digit is a
/// candidate badge and is looked up against the current frame's targets
/// by the event loop - a key with nothing bound to it becomes
/// `FocusByBadge` too and the lookup simply misses, same as a typo.
#[cfg(test)]
pub(crate) fn navigation_key_action(code: KeyCode) -> NavigationAction {
    navigation_key_action_mods(code, crossterm::event::KeyModifiers::NONE)
}

/// [`navigation_key_action`] for a key that may carry `Shift` (`Shift+Left`).
pub(crate) fn navigation_key_action_mods(
    code: KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> NavigationAction {
    use keymap::Action as Bind;
    // Ctrl and Alt may still be held from the prefix itself.
    let modifiers = modifiers & crossterm::event::KeyModifiers::SHIFT;
    match keymap::current().action_in_code_mods(keymap::Ctx::NavPrefix, code, modifiers) {
        Some(Bind::NavCollapseLeft) => NavigationAction::CollapseLeft,
        Some(Bind::NavCollapseRight) => NavigationAction::CollapseRight,
        Some(Bind::NavOpenSettings) => NavigationAction::OpenSettings,
        Some(Bind::NavOpenAbout) => NavigationAction::OpenAbout,
        Some(Bind::NavOpenActivityLog) => NavigationAction::OpenActivityLog,
        Some(Bind::NavOpenGlobalLog) => NavigationAction::OpenGlobalLog,
        Some(Bind::NavShowJobs) => NavigationAction::ShowJobs,
        Some(Bind::NavShowUpdates) => NavigationAction::ShowUpdates,
        Some(Bind::NavToggleHelp) => NavigationAction::ToggleHelp,
        Some(Bind::NavSelectTab) => NavigationAction::SelectTab,
        Some(Bind::NavChangeTheme) => NavigationAction::ChangeTheme,
        Some(Bind::NavChangeLanguage) => NavigationAction::ChangeLanguage,
        Some(Bind::NavChangeGraphColorTheme) => NavigationAction::ChangeGraphColorTheme,
        Some(Bind::NavToggleGraphRowTints) => NavigationAction::ToggleGraphRowTints,
        Some(Bind::NavGrowSection) => NavigationAction::GrowSection,
        Some(Bind::NavShrinkSection) => NavigationAction::ShrinkSection,
        Some(Bind::NavGrowMessage) => NavigationAction::GrowMessage,
        Some(Bind::NavShrinkMessage) => NavigationAction::ShrinkMessage,
        Some(Bind::NavRefsWider) => {
            NavigationAction::Resize(crate::pane_layout::PaneResize::RefsWider)
        }
        Some(Bind::NavRefsNarrower) => {
            NavigationAction::Resize(crate::pane_layout::PaneResize::RefsNarrower)
        }
        Some(Bind::NavStatusWider) => {
            NavigationAction::Resize(crate::pane_layout::PaneResize::StatusWider)
        }
        Some(Bind::NavStatusNarrower) => {
            NavigationAction::Resize(crate::pane_layout::PaneResize::StatusNarrower)
        }
        Some(Bind::NavAgentShow) => NavigationAction::AgentShow,
        Some(Bind::NavAgentBack) => NavigationAction::AgentBack,
        Some(Bind::NavArrangeDialog) => NavigationAction::ArrangeDialog,
        _ => match code {
            KeyCode::Char(c) if c.is_ascii_alphanumeric() => NavigationAction::FocusByBadge(c),
            _ => NavigationAction::Cancel,
        },
    }
}

/// The footer line while a prefix is pending (`Ctrl+G: s=stage ...`):
/// the effective prefix, then each bound key with its short name. AI keys
/// only appear while AI is on; an unbound action is left out.
pub(crate) fn prefix_hint(km: &keymap::Keymap, ctx: keymap::Ctx, ai_on: bool) -> String {
    use gitpoulpi_i18n::{msg::footer_hint as h, M};
    use keymap::Action as A;
    let (prefix, items): (A, &[(A, M, bool)]) = match ctx {
        keymap::Ctx::NavPrefix => (
            A::GlobalNavPrefix,
            &[
                (A::NavCollapseLeft, h::COLLAPSE_REFS, false),
                (A::NavCollapseRight, h::COLLAPSE_STATUS, false),
            ],
        ),
        _ => (
            A::GlobalGitPrefix,
            &[
                (A::GitStageFile, h::STAGE, false),
                (A::GitUnstageFile, h::UNSTAGE, false),
                (A::GitStageAll, h::STAGE_ALL, false),
                (A::GitDiscardAll, h::DISCARD_ALL, false),
                (A::GitCompose, h::COMMIT, false),
                (A::GitAiCommitMessage, h::AI_MESSAGE, true),
                (A::GitAiExplain, h::AI_EXPLAIN, true),
                (A::GitPull, h::PULL, false),
                (A::GitPush, h::PUSH, false),
                (A::GitPushTags, h::PUSH_ALL_TAGS, false),
                (A::GitOpenLinkedPr, h::OPEN_PR, false),
                (A::GitFetchAll, h::FETCH, false),
                (A::GitNewBranch, h::BRANCH, false),
                (A::GitNewTag, h::TAG, false),
                (A::GitStashPush, h::STASH, false),
                (A::GitStashMessage, h::STASH_WITH_MESSAGE, false),
                (A::GitStashPop, h::POP, false),
                (A::GitDeleteRef, h::DELETE, false),
            ],
        ),
    };
    let mut parts: Vec<String> = Vec::new();
    let mut add = |action: A, name: M, ai: bool| {
        let key = km.shown_key(action);
        if !key.is_empty() && (!ai || ai_on) {
            parts.push(format!("{key}={}", name.get()));
        }
    };
    for (action, name, ai) in items {
        add(*action, *name, *ai);
    }
    let mut line = h::lead(km.shown(prefix));
    if ctx == keymap::Ctx::NavPrefix {
        let mut items = nav_hint_items(km);
        items.extend(Some(nav_help_item(km, false)).filter(|i| !i.is_empty()));
        line.push_str(&items.join("  "));
        line.push_str("  ");
        line.push_str(h::ESC_CANCEL.get());
    } else {
        line.push_str(&parts.join(" "));
        line.push_str("  ");
        line.push_str(h::ESC_CANCEL.get());
    }
    line
}

/// Every action of the Ctrl+N mode that is bound, as `key=name`, the most
/// useful first: the collapse keys, the sub-pane badge, the dialogs (theme,
/// language, Settings, tabs, Activity, jobs, updates, About), the graph looks,
/// then the pane resizing keys. The help key is left to [`nav_help_item`].
pub(crate) fn nav_hint_items(km: &keymap::Keymap) -> Vec<String> {
    use gitpoulpi_i18n::msg::footer_hint as h;
    use keymap::Action as A;
    let items = std::cell::RefCell::new(Vec::new());
    let push = |action: A, name: &str| {
        let key = km.shown_key(action);
        if !key.is_empty() {
            items.borrow_mut().push(format!("{key}={name}"));
        }
    };
    let push_raw = |item: String| items.borrow_mut().push(item);
    push(A::NavCollapseLeft, h::COLLAPSE_REFS.get());
    push(A::NavCollapseRight, h::COLLAPSE_STATUS.get());
    push_raw(format!("<badge>={}", h::FOCUS_SUBPANE.get()));
    push(A::NavChangeTheme, h::THEME.get());
    push(A::NavChangeLanguage, h::LANGUAGE.get());
    push(A::NavOpenSettings, h::SETTINGS.get());
    push(A::NavSelectTab, h::SELECT_TAB.get());
    push(A::NavOpenActivityLog, h::ACTIVITY.get());
    push(A::NavShowJobs, h::JOBS.get());
    push(A::NavShowUpdates, h::UPDATES.get());
    push(A::NavOpenAbout, h::ABOUT.get());
    push(A::NavChangeGraphColorTheme, h::GRAPH_COLORS.get());
    push(A::NavToggleGraphRowTints, h::ROW_TINTS.get());
    push(A::NavOpenGlobalLog, h::GLOBAL_LOG.get());
    push(A::NavGrowSection, h::RESIZE_SECTION.get());
    push(A::NavGrowMessage, h::RESIZE_MESSAGE.get());
    push(A::NavRefsWider, h::RESIZE_REFS.get());
    push(A::NavStatusWider, h::RESIZE_STATUS.get());
    items.into_inner()
}

/// The closing item of the Ctrl+N hint: `?=help`, or with `truncated` the
/// `?=all keys` that says the line left some out. Empty when help is unbound.
pub(crate) fn nav_help_item(km: &keymap::Keymap, truncated: bool) -> String {
    use gitpoulpi_i18n::msg::footer_hint as h;
    let key = km.shown_key(keymap::Action::NavToggleHelp);
    if key.is_empty() {
        String::new()
    } else if truncated {
        h::all_keys(key)
    } else {
        format!("{key}={}", h::HELP.get())
    }
}

/// The cap on a pending vim-style count (`4j`, `10k`, ...) - generous
/// enough that no real motion is ever blocked by it (the Log's own load
/// cap is 5000, see `LogLoadSettings`) while still being a `u32` that
/// can't overflow `saturating_mul`/`saturating_add` into something a
/// selection helper has to defend against.
pub(crate) const MAX_PENDING_COUNT: u32 = 9999;

/// Whether a typed character should extend a pending count rather than
/// falling through to whatever it would otherwise do: a leading `1`-`9`,
/// or any digit once a count is already pending. A bare `0` is never a
/// count start -- mirrors vim, where `0` alone means "start of line", not
/// "the digit 0 of a count".
pub(crate) fn is_count_digit(c: char, pending: Option<u32>) -> bool {
    c.is_ascii_digit() && (c != '0' || pending.is_some())
}

/// Folds one more typed digit into a pending count, capped at
/// `MAX_PENDING_COUNT`. Pure and unit-tested here because the event
/// loop's own accumulation arm needs a terminal to exercise by pty.
pub(crate) fn accumulate_count(pending: Option<u32>, digit: u32) -> Option<u32> {
    Some(
        pending
            .unwrap_or(0)
            .saturating_mul(10)
            .saturating_add(digit)
            .min(MAX_PENDING_COUNT),
    )
}

/// Whether the vim-count arm is even allowed to look at this key, ahead
/// of `is_count_digit` itself - every flag here is a surface where a
/// digit already means something other than "start/continue a count": a
/// menu's own row shortcuts, the commit composer's text fields, and the
/// Preferences screen's own numeric controls. Without the `preferences_open`
/// check every digit typed into
/// a field like "Mouse wheel lines" was swallowed here as a count and
/// never reached `PreferencesState::on_key` - the field showed a cursor
/// but typing anything wrote nothing, and only Backspace (not a digit)
/// worked.
///
/// Pure and unit-tested here for the same reason `is_count_digit` is: the
/// event loop's own arm needs a terminal to exercise by pty, but the
/// *decision* does not depend on one.
pub(crate) fn count_prefix_may_apply(
    in_normal_mode: bool,
    log_context_menu_open: bool,
    ref_context_menu_open: bool,
    composer_focused: bool,
    preferences_open: bool,
    // A popup with text fields that owns the keyboard: the management
    // tab's Browse / Init / Clone dialogs, the Add / Edit remote form,
    // the management tab's search. Item 27's bug again, one surface
    // over: a digit typed into a path or a URL was taken as a count.
    text_popup_open: bool,
) -> bool {
    in_normal_mode
        && !log_context_menu_open
        && !ref_context_menu_open
        && !composer_focused
        && !preferences_open
        && !text_popup_open
}

/// Every sub-pane `Ctrl+N` can send keyboard focus to. One variant per
/// focusable unit, not per outer pane - the point is that each Refs
/// section, each Status half and so on gets its own badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FocusTarget {
    /// The Refs sidebar's filter field (`/`). Focusing it opens
    /// `InputMode::FilteringRefs`, same as pressing `/` while the sidebar
    /// has focus or clicking the field.
    RefsFilter,
    RefsSection(SidebarSection),
    Log,
    /// The commit column left of the Diff / File View / Blame / Edit views:
    /// the arrows then pick a commit.
    CommitColumn,
    StatusSection(StatusSection),
    /// The commit composer's Summary field, under the Status pane --
    /// same as clicking it (`HitTarget::CommitSummaryField`) or the
    /// WIP row's own field, since they're one buffer.
    ComposerSummary,
    /// The Diff view, while it owns the central column. It already
    /// takes every key regardless of `Focus` the moment it's open (its
    /// `KeyCode` arms in `InputMode::Normal` are guarded on
    /// `central_view == CentralView::Diff`, not on `focus`), so this
    /// badge has no state to change - it exists so the badge overlay is
    /// honest about "yes, this is reachable", the same reason the gear
    /// draws even though clicking it and `Ctrl+,` already did the same
    /// thing.
    DiffView,
    /// The File Insight overlay (file content / blame / history), while
    /// it's open. Same "already owns the keyboard" shape as `DiffView`.
    FileView,
    /// The Editor, while it owns the central column. Same "already owns the
    /// keyboard" shape as `DiffView`/`FileView`; without its own badge the
    /// central-view match pushes nothing and `Ctrl+N` never names it.
    Editor,
    /// The selected commit's changed-files list (commit-detail pane):
    /// focusing it is `Enter`/`l` from the Log - the list owns j/k.
    CommitFiles,
    /// The commit-detail pane's `Filter files:` field (same as `/` there
    /// or a click on it).
    CommitFilter,
    /// The commit-detail toolbar's `Path`/`Tree` toggle (the `m` key).
    /// An action badge: pressing it flips the mode.
    CommitPathTree,
    /// The commit-detail toolbar's `View all files` checkbox (the `a`
    /// key). An action badge: pressing it flips the checkbox.
    CommitViewAll,
    /// The commit-detail toolbar's sort toggle. An action badge: pressing
    /// it flips the file list's path order.
    CommitSort,
    /// A pane of the merge conflict editor (A, B, Output, Base): focus it
    /// on the current conflict.
    MergePane(crate::merge_view::MergePane),
    /// The Repository Management tab's search field.
    RepoManagerSearch,
    /// One of the Repository Management tab's four sections (Open,
    /// Favorites, Recent, All) - lands the selection on that section's
    /// header row.
    RepoManagerSection(repo_manager::Section),
    /// A control of the Home tab that a badge presses: a toolbar button
    /// (Browse, Clone, Init, Collapse all, Expand all), a page of the page
    /// switcher, the Workspaces page's New / From open tabs. Pressing the
    /// badge is a click on that control (`repo_manager::click`), so the two
    /// can never disagree.
    HomeControl(repo_manager::ManagerHit),
    /// A part of the Radar tab (filter, comboboxes, buttons, section
    /// headings).
    Radar(crate::radar::Spot),
    /// A control of the Settings screen (search box, section list, sub-tab,
    /// list pane or button, field, close cross): the screen's own paint
    /// walk gave the cell, and pressing the badge is a click there
    /// (`PreferencesState::nav_press`).
    Settings(u16, u16),
    /// A control of the repository dialog on top (Browse, Clone, Init...):
    /// the dialog's own paint walk gave the cell, and pressing the badge is a
    /// click there (`RepoDialogs::nav_press`).
    Dialog(u16, u16),
    /// A control of the Activity Log, the same way
    /// (`activity_log::nav_press`).
    Activity(u16, u16),
    /// A scroll arrow of the tab bar (`true`: the right one), while it is
    /// enabled. An action badge: pressing it scrolls the bar by one tab,
    /// as a click on the arrow does.
    TabScroll(bool),
}

/// Badge characters, in the order they're handed out: `1`-`9` first, then
/// `a`, `b`, `c`, ... - letters if there are more than nine focusable
/// panes in one frame. `l`, `,`, `?`, `q`, `t` are skipped because
/// `navigation_key_action`/Normal mode already gives them a fixed meaning;
/// handing one out as a badge would make `Ctrl+N l` ambiguous between
/// "open the language picker" and "focus whatever got badge l".
pub(crate) fn navigation_badge_char(index: usize) -> char {
    // Never a character some `nav_prefix` binding uses (`Ctrl+N l` must
    // not be ambiguous with badge `l`), computed from the effective
    // keymap; `q` stays reserved for Normal mode. Digits first, then
    // letters; past the end the letters wrap.
    let taken = keymap::current().nav_prefix_chars();
    let free = |c: &char| *c != 'q' && !taken.contains(c);
    let digits: Vec<char> = ('1'..='9').filter(free).collect();
    let letters: Vec<char> = ('a'..='z').filter(free).collect();
    if let Some(c) = digits.get(index) {
        return *c;
    }
    letters[(index - digits.len()) % letters.len()]
}

/// Assigns a badge character to each item of `targets`, in order --
/// `navigation_badge_char` applied positionally. Pure and separate from
/// any `Rect` so it can be unit-tested without a frame.
pub(crate) fn assign_navigation_badges<T: Copy>(targets: &[T]) -> Vec<(char, T)> {
    targets
        .iter()
        .enumerate()
        .map(|(i, t)| (navigation_badge_char(i), *t))
        .collect()
}

pub(crate) mod close_cross;
mod help_text;
mod hit;
mod layout;
mod toolbar;
mod widgets;

pub(crate) use help_text::*;
pub(crate) use hit::*;
pub(crate) use layout::*;
pub(crate) use toolbar::*;
pub(crate) use widgets::*;

#[cfg(test)]
mod tests;
