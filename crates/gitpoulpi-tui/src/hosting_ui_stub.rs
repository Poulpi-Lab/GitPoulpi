//! `hosting_ui` for a build with no hosting provider (custom builds, see
//! the custom-builds design section 3.2): the same
//! `pub(crate)` surface the event loop calls, all of it inert. There is no
//! PULL REQUESTS / ISSUES section, no toolbar chip, no hosting menu.
#![allow(dead_code)]

use super::*;

/// What [`HostingUi::reveal`] found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RevealOutcome {
    Row(usize),
    NotLinked,
    NotListed,
}

/// Which of the two lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HostingList {
    Pulls,
    Issues,
}

impl HostingList {
    pub(crate) fn section(self) -> SidebarSection {
        match self {
            HostingList::Pulls => SidebarSection::PullRequests,
            HostingList::Issues => SidebarSection::Issues,
        }
    }

    pub(crate) fn from_section(section: SidebarSection) -> Option<Self> {
        match section {
            SidebarSection::PullRequests => Some(HostingList::Pulls),
            SidebarSection::Issues => Some(HostingList::Issues),
            _ => None,
        }
    }
}

pub(crate) fn request_issues_refresh() {}

/// A click target inside the hosting UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HostingHit {
    Row(HostingList, usize),
    NewPr,
    NewIssue,
    Board,
    EditRow(usize),
    MenuItem(usize),
    PickerItem(usize),
    LinkedPrChip,
    NewPrChip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PrGroup {
    Mine,
    Assigned,
    Awaiting,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HostingRow {
    Reconnect,
}

impl HostingRow {
    pub(crate) fn is_control(&self) -> bool {
        true
    }
}

/// Never built: a menu that never opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HostingAction {}

/// What the event loop must do after `HostingUi` handled something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HostingEffect {
    None,
    Message(String),
    Preferences(Option<String>),
    RadarOptions,
    Checkout {
        branch: String,
        remote: String,
    },
    CheckoutPullRequestHead {
        remote: String,
        remote_ref: String,
        branch: String,
    },
    ViewBranch {
        branch: String,
        remote: String,
    },
    NewBranch(String),
    StartPullRequest(String),
    CreatePullRequest,
    Focus(HostingList),
}

#[allow(unused_imports)]
pub(crate) use crate::hosting_command::{
    command_unavailable_from, HostingAvailability, HostingCommand,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ToolbarChip {
    pub(crate) text: String,
    pub(crate) short: String,
    pub(crate) icon: String,
    pub(crate) fg: Color,
    pub(crate) bg: Option<Color>,
    pub(crate) hit: HostingHit,
}

pub(crate) struct HostingPicker;

pub(crate) fn branch_target(
    refs: &[gitpoulpi_core::domain::Ref],
    remote: &str,
    branch: &str,
) -> Option<String> {
    let local = refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == branch));
    if local {
        return Some(branch.to_string());
    }
    refs.iter()
        .any(|r| {
            matches!(&r.kind, RefKind::RemoteBranch { remote: rm, name }
                if rm == remote && name == branch)
        })
        .then(|| format!("{remote}/{branch}"))
}

pub(crate) struct HostingUi {
    pub(crate) menu: Option<ContextMenu<HostingAction>>,
    pub(crate) picker: Option<HostingPicker>,
    pub(crate) editing: Option<HostingList>,
}

impl HostingUi {
    pub(crate) fn new(_repo_root: &Path, _interval: Option<Duration>) -> Self {
        Self {
            menu: None,
            picker: None,
            editing: None,
        }
    }

    pub(crate) fn rewrite_host(&self) -> Option<Option<crate::rewrite_view::HostHandle>> {
        Some(None)
    }
    pub(crate) fn folded_groups(&self) -> Vec<String> {
        Vec::new()
    }
    pub(crate) fn restore_folded_groups(&mut self, _keys: Option<Vec<String>>) {}
    pub(crate) fn on_refresh(
        &mut self,
        _now: Instant,
        _remotes: &[gitpoulpi_core::domain::Remote],
        _refs: &[gitpoulpi_core::domain::Ref],
    ) {
    }
    pub(crate) fn animating(&self) -> bool {
        false
    }
    pub(crate) fn next_due(&self) -> Option<Instant> {
        None
    }
    pub(crate) fn tick(&mut self, _now: Instant) {}
    pub(crate) fn demand(&mut self) {}
    pub(crate) fn set_interval(&mut self, _interval: Option<Duration>) {}
    pub(crate) fn poll(&mut self) -> bool {
        false
    }
    pub(crate) fn linked_is_gitlab(&self) -> bool {
        false
    }
    pub(crate) fn edit_row(
        &mut self,
        _index: usize,
        _clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        HostingEffect::None
    }
    pub(crate) fn sections_visible(&self) -> (bool, bool) {
        (false, false)
    }
    pub(crate) fn issue_choices(&self) -> Vec<(String, String)> {
        Vec::new()
    }
    pub(crate) fn rows(&self, _list: HostingList) -> Vec<HostingRow> {
        Vec::new()
    }
    pub(crate) fn row_counts(&self) -> (usize, usize) {
        (0, 0)
    }
    pub(crate) fn item_count(&self, _list: HostingList) -> usize {
        0
    }
    pub(crate) fn chip(&self) -> Option<ToolbarChip> {
        None
    }
    pub(crate) fn chip_tooltip(&self) -> Vec<String> {
        Vec::new()
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_section(
        &self,
        _frame: &mut Frame,
        _area: Rect,
        _list: HostingList,
        _expanded: bool,
        _active: bool,
        _hovered: Option<&HitTarget>,
        _states: &mut SidebarStates,
        _hits: &mut Vec<(Rect, HitTarget)>,
    ) {
    }
    pub(crate) fn token_bases(&self) -> crate::issue_tokens::TokenBases {
        crate::issue_tokens::TokenBases
    }
    pub(crate) fn set_commit_references(&mut self, _refs: Vec<(String, String)>) {}
    pub(crate) fn take_notice(&mut self) -> Option<String> {
        None
    }
    pub(crate) fn issue_ref_available(&self) -> bool {
        false
    }
    pub(crate) fn modal_open(&self) -> bool {
        false
    }
    pub(crate) fn reveal(&mut self, _list: HostingList, _number: u64) -> RevealOutcome {
        RevealOutcome::NotLinked
    }
    pub(crate) fn item_effect(&self, _number: u64, _pr: bool) -> HostingEffect {
        HostingEffect::None
    }
    pub(crate) fn focus_row(&self, _list: HostingList) -> Option<usize> {
        None
    }
    pub(crate) fn render_picker(
        &mut self,
        _frame: &mut Frame,
        _hovered_row: Option<usize>,
        _hits: &mut Vec<(Rect, HitTarget)>,
    ) {
    }
    pub(crate) fn on_picker_key(&mut self, _key: KeyEvent) -> HostingEffect {
        HostingEffect::None
    }
    pub(crate) fn picker_click(&mut self, _row: usize) -> HostingEffect {
        HostingEffect::None
    }
    pub(crate) fn picker_wants_mouse(&self, _m: &crossterm::event::MouseEvent) -> bool {
        false
    }
    pub(crate) fn picker_mouse(&mut self, _m: crossterm::event::MouseEvent) {}
    pub(crate) fn close_picker(&mut self) {}
    pub(crate) fn paste(&mut self, _text: &str) {}
    pub(crate) fn on_search_key(&mut self, _key: KeyEvent) -> bool {
        false
    }
    pub(crate) fn render_chip_tooltip(&self, _frame: &mut Frame, chip: Rect) -> Rect {
        chip
    }
    pub(crate) fn render_menu(
        &self,
        _frame: &mut Frame,
        _hovered_row: Option<usize>,
        _hits: &mut Vec<(Rect, HitTarget)>,
    ) {
    }
    pub(crate) fn open_row_menu(
        &mut self,
        _list: HostingList,
        _index: usize,
        _col: u16,
        _row: u16,
    ) -> bool {
        false
    }
    pub(crate) fn open_chip_menu(&mut self, _col: u16, _row: u16) -> bool {
        false
    }
    pub(crate) fn open_row_in_browser(
        &mut self,
        _list: HostingList,
        _index: usize,
        _clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        HostingEffect::None
    }
    pub(crate) fn activate_row(
        &mut self,
        _list: HostingList,
        _index: usize,
        _clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        HostingEffect::None
    }
    pub(crate) fn run(
        &mut self,
        action: HostingAction,
        _clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        match action {}
    }
    pub(crate) fn click_chip(
        &mut self,
        _hit: &HostingHit,
        _clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        HostingEffect::None
    }
    pub(crate) fn availability(&self) -> HostingAvailability {
        HostingAvailability::default()
    }
    pub(crate) fn run_command(
        &mut self,
        _cmd: HostingCommand,
        _anchor: (u16, u16),
        _clipboard: &mut crate::clipboard::Clipboard,
    ) -> HostingEffect {
        HostingEffect::None
    }
}
