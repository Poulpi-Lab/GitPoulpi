//! The Refs sidebar (LOCAL/REMOTE/TAGS/WORKTREES/STASHES): its sections
//! and their list states, grouping and filtering refs, row counts and
//! selection movement, mouse hit-testing of ref rows, and ref drag and drop.

use super::*;
use gitpoulpi_i18n::msg::sidebar as msg;

mod vocab;
pub(crate) use vocab::SidebarSection;

#[cfg(test)]
pub(crate) use gitpoulpi_app::refs_sections::visible_sections;
pub(crate) use gitpoulpi_app::refs_sections::{
    available_sections, can_hide, effective_hidden, hidden_from_names, hidden_to_names,
    linked_worktree_count, set_worktrees_shown, visible_sections_with,
};

/// Writes the hidden set to `config.toml` on a short-lived thread (the input
/// loop never waits on the disk); one process-wide lock keeps successive saves
/// in order.
pub(crate) fn save_hidden_in_background(path: &std::path::Path, hidden: &HashSet<SidebarSection>) {
    static WRITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let names = hidden_to_names(hidden);
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _ = gitpoulpi_config::set_sidebar_sections_hidden_in_file(&path, &names);
    });
}

/// The Refs sidebar's per-section `ListState`s. One struct rather than
/// loose locals so `state_mut` is the single place that maps a section to
/// its state.
#[derive(Default)]
pub(crate) struct SidebarStates {
    pub(crate) local: ListState,
    pub(crate) remote: ListState,
    pub(crate) worktrees: ListState,
    pub(crate) stashes: ListState,
    pub(crate) tags: ListState,
    pub(crate) pulls: ListState,
    pub(crate) issues: ListState,
    pub(crate) submodules: ListState,
    follow: FollowFlags<8>,
}

impl SidebarStates {
    /// The read-only twin of `state_mut`, for callers that only need a
    /// section's scroll offset (`sidebar_ref_at`) and must not hold a mutable
    /// borrow of the whole struct.
    pub(crate) fn state(&self, section: SidebarSection) -> &ListState {
        match section {
            SidebarSection::Local => &self.local,
            SidebarSection::Remote => &self.remote,
            SidebarSection::Worktrees => &self.worktrees,
            SidebarSection::Stashes => &self.stashes,
            SidebarSection::Tags => &self.tags,
            SidebarSection::PullRequests => &self.pulls,
            SidebarSection::Issues => &self.issues,
            SidebarSection::Submodules => &self.submodules,
            SidebarSection::WorktreesEmpty => &self.worktrees,
        }
    }

    pub(crate) fn state_mut(&mut self, section: SidebarSection) -> &mut ListState {
        match section {
            SidebarSection::Local => &mut self.local,
            SidebarSection::Remote => &mut self.remote,
            SidebarSection::Worktrees => &mut self.worktrees,
            SidebarSection::Stashes => &mut self.stashes,
            SidebarSection::Tags => &mut self.tags,
            SidebarSection::PullRequests => &mut self.pulls,
            SidebarSection::Issues => &mut self.issues,
            SidebarSection::Submodules => &mut self.submodules,
            SidebarSection::WorktreesEmpty => &mut self.worktrees,
        }
    }

    pub(crate) fn follow(&self, section: SidebarSection) -> bool {
        self.follow.get(section.index())
    }

    pub(crate) fn follow_mut(&mut self, section: SidebarSection) -> &mut bool {
        self.follow.get_mut(section.index())
    }

    /// Selects like `state_mut(section).select(..)`, but also marks the
    /// section as following again: every keyboard move and click goes through
    /// this, so a wheel-scrolled section snaps back to its selection the next
    /// time something other than the wheel touches it.
    pub(crate) fn select(&mut self, section: SidebarSection, index: Option<usize>) {
        self.state_mut(section).select(index);
        *self.follow_mut(section) = true;
    }
}

/// Jumping to the Refs pane (`Tab` landing on it) should put the keyboard
/// cursor on a usable section: walks LOCAL -> REMOTE -> TAGS -> LOCAL for the
/// first one that is not collapsed, and leaves `active` where it is if all
/// three are, since nothing is selectable either way.
pub(crate) fn land_on_open_ref_section(
    active: &mut SidebarSection,
    collapsed: &std::collections::HashSet<String>,
    visible: &[SidebarSection],
) {
    if visible.is_empty() {
        return;
    }
    // A section elided for being empty is as unusable a landing spot as a
    // collapsed one, so snap onto the visible list first, then walk it for the
    // first expanded section.
    if !visible.contains(active) {
        *active = visible[0];
    }
    if !collapsed.contains(active.title()) {
        return;
    }
    let start = visible.iter().position(|s| s == active).unwrap_or(0);
    for step in 1..=visible.len() {
        let candidate = visible[(start + step) % visible.len()];
        if !collapsed.contains(candidate.title()) {
            *active = candidate;
            return;
        }
    }
    // Every visible section is collapsed: leave `active` where it is.
}

/// Moves the sidebar selection by one row, crossing into the
/// neighboring visible section when it runs off either end.
///
/// `j`/`k` walk the whole sidebar, not just the active section. Collapsed
/// and empty sections are skipped, and a section's own selection is left
/// behind when leaving it so coming back returns you where you were (only
/// the active section paints a highlight).
pub(crate) fn move_sidebar_selection(
    active: &mut SidebarSection,
    states: &mut SidebarStates,
    visible: &[SidebarSection],
    collapsed: &std::collections::HashSet<String>,
    delta: i32,
    row_count: impl Fn(SidebarSection) -> usize,
) {
    if visible.is_empty() || delta == 0 {
        return;
    }
    let rows_of = |s: SidebarSection| -> usize {
        if collapsed.contains(s.title()) {
            0
        } else {
            row_count(s)
        }
    };

    let len = rows_of(*active);
    let current = states.state_mut(*active).selected();
    if let Some(i) = current {
        let next = i as i32 + delta;
        if next >= 0 && (next as usize) < len {
            states.select(*active, Some(next as usize));
            return;
        }
    } else if len > 0 {
        // No selection yet: land on the near end of this section.
        let start = if delta > 0 { 0 } else { len - 1 };
        states.select(*active, Some(start));
        return;
    }

    // Ran off the end (or this section is empty): walk to the next
    // section in that direction that actually has a row to land on.
    let start = visible.iter().position(|s| s == active).unwrap_or(0) as i32;
    let n = visible.len() as i32;
    for step in 1..=visible.len() as i32 {
        let candidate = visible[((start + delta * step).rem_euclid(n)) as usize];
        let candidate_len = rows_of(candidate);
        if candidate_len == 0 {
            continue;
        }
        *active = candidate;
        let landing = if delta > 0 { 0 } else { candidate_len - 1 };
        states.select(candidate, Some(landing));
        return;
    }
}

/// The mouse wheel over a sidebar section scrolls that section's viewport
/// only, never its selection (unlike `move_sidebar_selection`). `area` is the
/// section's own bordered rect (its top+bottom border are not content rows).
pub(crate) fn scroll_sidebar_section(
    states: &mut SidebarStates,
    section: SidebarSection,
    len: usize,
    area: Rect,
    delta: i32,
) {
    let visible = area.height.saturating_sub(2) as usize;
    let state = states.state_mut(section);
    let new_offset = scroll_offset(state.offset(), len, visible, delta);
    *state.offset_mut() = new_offset;
    *states.follow_mut(section) = false;
}

/// Steps `active` to the next/previous *visible* section, wrapping.
pub(crate) fn step_sidebar_section(
    active: &mut SidebarSection,
    visible: &[SidebarSection],
    delta: i32,
) {
    if visible.is_empty() {
        return;
    }
    let start = visible.iter().position(|s| s == active).unwrap_or(0) as i32;
    let len = visible.len() as i32;
    let next = (start + delta).rem_euclid(len) as usize;
    *active = visible[next];
}

pub(crate) use gitpoulpi_app::refs_tree::{
    folder_fully_in, group_refs, group_refs_kind, ref_full_name, refs_title,
    toggle_folder_membership, RefsGroups,
};

/// Row count for one Refs section as currently displayed (flat or
/// hierarchical): the single source of truth every Up/Down/selection bound
/// in the Refs pane calls, so no site drifts from what `ref_display_rows`
/// renders.
#[allow(clippy::too_many_arguments)]
pub(crate) fn ref_section_row_count(
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    refs_filter: &str,
    section: SidebarSection,
    hierarchical: bool,
    collapsed_folders: &CollapsedFolders,
    remotes: &[String],
    worktree_count: usize,
    stash_count: usize,
    hosting_rows: (usize, usize),
    submodule_count: usize,
) -> usize {
    let Some(kind) = section.ref_kind() else {
        return match section {
            SidebarSection::Worktrees => worktree_count,
            SidebarSection::Stashes => stash_count,
            SidebarSection::PullRequests => hosting_rows.0,
            SidebarSection::Issues => hosting_rows.1,
            SidebarSection::Submodules => submodule_count,
            _ => 0,
        };
    };
    refs_result
        .as_ref()
        .map(|refs| {
            let names = group_refs_kind(refs, refs_filter, kind);
            ref_display_rows(&names, kind, hierarchical, collapsed_folders, remotes).len()
        })
        .unwrap_or(0)
}

/// The local branch HEAD points to, per the backend's own `Ref::is_head`,
/// not inferred from log position; `None` on a detached or unborn HEAD.
pub(crate) fn checked_out_branch_name(refs: &[gitpoulpi_core::domain::Ref]) -> Option<&str> {
    refs.iter().find_map(|r| match &r.kind {
        RefKind::LocalBranch { name } if r.is_head => Some(name.as_str()),
        _ => None,
    })
}

/// HEAD's own commit oid (the checked-out branch's target), for actions
/// that create a ref pointing at HEAD (branch/tag create); `None`
/// on a detached or unborn HEAD.
pub(crate) use gitpoulpi_app::ref_rules::head_oid;

/// Full name of the leaf ref selected in the active Refs section, or
/// `None` when the selection sits on a folder row, the section is empty,
/// or the refs failed to load.
///
/// The sidebar's rows are *derived* (filter -> group -> tree), never stored,
/// so "which ref is selected" is recomputed by walking the same pipeline
/// every time. `selected_status_path` is the same idea for the Status pane.
pub(crate) fn selected_ref_leaf(
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    refs_filter: &str,
    section: SidebarSection,
    selected: Option<usize>,
    hierarchical: bool,
    collapsed_folders: &CollapsedFolders,
    remotes: &[String],
) -> Option<String> {
    // Worktrees/Stashes hold no refs at all, so there is nothing to select.
    let kind = section.ref_kind()?;
    let refs = refs_result.as_ref().ok()?;
    let names = group_refs_kind(refs, refs_filter, kind);
    let rows = ref_display_rows(&names, kind, hierarchical, collapsed_folders, remotes);
    rows.get(selected?)?.leaf_name().map(str::to_string)
}

/// The text of a folder row in the Refs sidebar.
///
/// A remote's own node (`origin`, `gh`, `grp/sub`) carries the remote icon
/// and no trailing `/`: it is a remote, not a `/`-derived folder, and its
/// name may itself contain a `/` that must not read as "a folder called grp
/// holding sub". A plain folder keeps the `name/` spelling that says "there
/// is more under this", plus a folder icon between the chevron and the name,
/// closed or open to match `expanded`.
pub(crate) fn ref_folder_label(
    indent: &str,
    chevron: &str,
    label: &str,
    is_remote_root: bool,
    expanded: bool,
) -> String {
    if is_remote_root {
        format!("{indent}{chevron} {REMOTE_ICON} {label}")
    } else {
        let folder_icon = if expanded {
            FOLDER_OPEN_ICON
        } else {
            FOLDER_CLOSED_ICON
        };
        format!("{indent}{chevron} {folder_icon} {label}/")
    }
}

/// The text of a workflow group's folder row (`Feature (3)`): like a plain
/// folder, minus the trailing `/` (it is a group, not a path).
pub(crate) fn group_folder_label(
    indent: &str,
    chevron: &str,
    label: &str,
    expanded: bool,
) -> String {
    let folder_icon = if expanded {
        FOLDER_OPEN_ICON
    } else {
        FOLDER_CLOSED_ICON
    };
    format!("{indent}{chevron} {folder_icon} {label}")
}

thread_local! {
    /// Where the pointer rests while the ordinary mouse path owns it (`None`
    /// under a menu or a popup): the sidebar's paint reads it to light the
    /// row under it, one rule for every section (refs, worktrees, stashes,
    /// submodules), see "Pointer and hover rules" in `where-we-are.md`.
    static ROW_POINTER: std::cell::Cell<Option<(u16, u16)>> = const { std::cell::Cell::new(None) };
}

/// Records the pointer for the sidebar's row hover (`None` clears it).
pub(crate) fn set_row_pointer(pos: Option<(u16, u16)>) {
    ROW_POINTER.with(|p| p.set(pos));
}

/// The row of a sidebar section under the pointer, if any: `area` is the
/// section's bordered rect, `len` its number of real rows. One function for
/// the paint and the tests, so every section answers the same way.
pub(crate) fn hovered_row_of(area: Rect, offset: usize, len: usize) -> Option<(usize, Rect)> {
    let (col, row) = ROW_POINTER.with(|p| p.get())?;
    // The border columns are not the row.
    if col <= area.x || col + 1 >= area.right() {
        return None;
    }
    let index = row_to_index(area, offset, 1, row, len)?;
    Some((
        index,
        Rect::new(area.x + 1, row, area.width.saturating_sub(2), 1),
    ))
}

/// The row of a list without a border (`inner` is its content rect, one row
/// per item) under the pointer: the row index and its full-width band.
pub(crate) fn hovered_inner_row(inner: Rect, offset: usize, len: usize) -> Option<(usize, Rect)> {
    let (col, row) = ROW_POINTER.with(|p| p.get())?;
    if !inner.contains((col, row).into()) {
        return None;
    }
    let index = (row - inner.y) as usize + offset;
    (index < len).then_some((index, Rect::new(inner.x, row, inner.width, 1)))
}

/// Lights a hovered row's band: the one hover look of every list row (the
/// same `ui.hover` wash as the Refs pane). A row already carrying the
/// selection band keeps it.
pub(crate) fn paint_row_hover(buf: &mut ratatui::buffer::Buffer, band: Rect) {
    let selection = crate::th().ui.selection;
    for x in band.x..band.right() {
        if let Some(cell) = buf.cell_mut((x, band.y)) {
            if cell.bg != selection {
                cell.set_bg(crate::th().ui.hover);
            }
        }
    }
}

/// A Refs-sidebar leaf row under a screen position.
pub(crate) struct SidebarRefHit {
    pub(crate) name: String,
    pub(crate) kind: RefRowKind,
    /// The row band *inside* the sub-pane's border, so lighting it can
    /// never paint over the block's own frame.
    pub(crate) rect: Rect,
}

/// The Refs-sidebar ref under `(col, row)`, if the pointer is over one.
///
/// Plain ref rows are deliberately not registered in `hit_targets` (only the
/// section headers are), so this recomputes them from the same `group_refs`
/// and `ref_display_rows` pair the render pass uses (as `log_table` does):
/// the rows a click resolves against cannot drift from the rows drawn.
///
/// It is one function because both the hover path (`Moved`) and the
/// ref-drag path (`Drag`) need it, and crossterm reports a *held* button as
/// `Drag` and never `Moved`: a drag reusing the hover arm's stale answer
/// would believe it was still over the row the press started on.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sidebar_ref_at(
    sections: &[(SidebarSection, Rect)],
    refs: &[gitpoulpi_core::domain::Ref],
    filter: &str,
    hierarchical: bool,
    collapsed: &CollapsedFolders,
    remotes: &[String],
    offset_of: impl Fn(SidebarSection) -> usize,
    col: u16,
    row: u16,
) -> Option<SidebarRefHit> {
    let (section, area) = sections
        .iter()
        .copied()
        .find(|(_, a)| a.contains((col, row).into()))?;
    let kind = section.ref_kind()?;
    let names = group_refs_kind(refs, filter, kind);
    let rows = ref_display_rows(&names, kind, hierarchical, collapsed, remotes);
    let index = row_to_index(area, offset_of(section), 1, row, rows.len())?;
    Some(SidebarRefHit {
        name: rows.get(index)?.leaf_name()?.to_string(),
        kind,
        // `row` itself: `row_to_index` only answered because the pointer is
        // on that row, so finding the band back needs no index-to-row
        // arithmetic (and no second chance to get `top_offset` wrong).
        rect: Rect::new(area.x + 1, row, area.width.saturating_sub(2), 1),
    })
}

/// The folder or remote-root row under `(col, row)`, if the pointer is over
/// one: the folder twin of `sidebar_ref_at`, kept separate because that
/// function's callers (the ref-drag gesture) only want a leaf. Drives the
/// folder row's hover-reveals-the-eye affordance.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sidebar_folder_at(
    sections: &[(SidebarSection, Rect)],
    refs: &[gitpoulpi_core::domain::Ref],
    filter: &str,
    hierarchical: bool,
    collapsed: &CollapsedFolders,
    remotes: &[String],
    offset_of: impl Fn(SidebarSection) -> usize,
    col: u16,
    row: u16,
) -> Option<String> {
    let (section, area) = sections
        .iter()
        .copied()
        .find(|(_, a)| a.contains((col, row).into()))?;
    let kind = section.ref_kind()?;
    let names = group_refs_kind(refs, filter, kind);
    let rows = ref_display_rows(&names, kind, hierarchical, collapsed, remotes);
    let index = row_to_index(area, offset_of(section), 1, row, rows.len())?;
    rows.get(index)?.folder_path().map(str::to_string)
}

/// Whether a ref drag that started on `source` may be dropped on the ref
/// `candidate` of kind `kind`.
///
/// The rule the drop menu (`ref_menu::drop_menu_entries`) is
/// built on: a **local branch** can be dropped on another local branch
/// (merge / rebase / interactive rebase / fast-forward) or on a remote branch
/// (push); a **remote branch** can be dropped on a local branch (merge /
/// fast-forward / rebase / pull); a **tag** can be dropped on a local
/// branch, a remote branch or another tag (move the tag there, and for a
/// local branch the two merges). Nothing but a tag receives a drop on a
/// tag, and nothing can be dropped on itself. Both the lit drop
/// target and the footer banner read this predicate, so "valid" means the
/// same thing on screen as in the message.
pub(crate) fn ref_drop_is_valid(
    source: &str,
    source_kind: RefRowKind,
    candidate: &str,
    kind: RefRowKind,
) -> bool {
    if candidate == source && kind == source_kind {
        return false;
    }
    matches!(
        (source_kind, kind),
        (RefRowKind::Local, RefRowKind::Local | RefRowKind::Remote)
            | (RefRowKind::Remote, RefRowKind::Local)
            | (RefRowKind::Tag, _)
    )
}

/// The ref under the pointer during a drag, and whether it can take the
/// drop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RefDropCandidate {
    pub(crate) name: String,
    pub(crate) kind: RefRowKind,
    pub(crate) valid: bool,
    pub(crate) rect: Rect,
}

/// A ref picked up with the mouse, from the Refs sidebar or from a pill
/// beside the commit graph.
///
/// `started` separates a click from a drag: a press on a ref still selects
/// it and a double press still checks it out, so the gesture only becomes a
/// drag once the pointer leaves the cell it went down on. Without it every
/// click on a branch would pick it up and drop it on itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RefDrag {
    pub(crate) source: String,
    /// Local, remote or tag: what the ref can do when dropped
    /// (`ref_drop_is_valid`).
    pub(crate) source_kind: RefRowKind,
    pub(crate) started: bool,
    pub(crate) origin: (u16, u16),
    /// The pointer's last known cell. `Esc` cancels the drag but carries no
    /// mouse position, and a button release produces no `Moved`, so this is
    /// how either ending restores the shape for where the pointer actually is.
    pub(crate) at: (u16, u16),
    pub(crate) over: Option<RefDropCandidate>,
    /// The cells of the row/pill the ref was picked up from, captured on
    /// the first move so it can be dimmed for as long as the drag lasts.
    pub(crate) source_rect: Option<Rect>,
    /// A sidebar press that should bring the ref's commit into the Log
    /// (`ui.reveal_on_ref_click`): done on the *release*, and only when the
    /// press never turned into a drag, so carrying a ref never moves the
    /// graph away from the row it is about to be dropped on.
    pub(crate) reveal_on_release: Option<(SidebarSection, String)>,
}

impl RefDrag {
    pub(crate) fn armed_at(source: String, source_kind: RefRowKind, col: u16, row: u16) -> Self {
        RefDrag {
            source,
            source_kind,
            started: false,
            origin: (col, row),
            at: (col, row),
            over: None,
            source_rect: None,
            reveal_on_release: None,
        }
    }

    /// The footer line while a ref is being carried: the in-content half of
    /// the affordance and, per `cursor.rs`, the half that works on every
    /// terminal (`OSC 22` is the bonus). It carries what is being carried,
    /// what it is over, whether that can take it, and how to get out, and is
    /// the only "picked up" feedback the graph's ref pills can have without
    /// the Log table's row builder learning about drags.
    ///
    /// The arrow is `\u{25B8}`, the plain triangle the sidebar's folder rows
    /// use, not `\u{2192}`: this segment renders BOLD, and a text-default emoji
    /// codepoint flips to its color presentation under a style.
    pub(crate) fn banner(&self) -> String {
        match &self.over {
            Some(c) if c.valid => format!(
                "\u{2387} {} \u{25B8} {}",
                self.source,
                msg::drag_release(&c.name)
            ),
            // Two different refusals with two different causes: dropping
            // `main` back on `main` is not "merge/rebase land on a local
            // branch", and one wording for both reads as a bug in the rule.
            Some(c) if c.name == self.source => {
                format!("\u{2387} {} \u{25B8} {}", self.source, msg::DRAG_BACK.get())
            }
            Some(c) => format!(
                "\u{2387} {} \u{25B8} {}",
                self.source,
                msg::drag_refused(&c.name, &self.source)
            ),
            None => format!("\u{2387} {}", msg::drag_start(&self.source)),
        }
    }
}

/// Paints the in-flight drag's feedback over an already-drawn frame: the
/// source row dimmed, an invalid hovered target dimmed, and a ghost label
/// following the pointer. It only touches cells of the *current* frame
/// buffer, so nothing survives the drag: the next frame is built from
/// scratch and ratatui's diff repaints whatever the ghost covered.
pub(crate) fn paint_ref_drag(buf: &mut ratatui::buffer::Buffer, drag: &RefDrag) {
    if !drag.started {
        return;
    }
    let area = buf.area;
    if area.width == 0 || area.height == 0 {
        return;
    }
    let dim = |buf: &mut ratatui::buffer::Buffer, r: Rect| {
        for y in r.y..r.y.saturating_add(r.height) {
            for x in r.x..r.x.saturating_add(r.width) {
                if x < area.x + area.width && y < area.y + area.height {
                    buf[(x, y)].set_style(Style::default().add_modifier(Modifier::DIM));
                }
            }
        }
    };
    if let Some(r) = drag.source_rect {
        dim(buf, r);
    }
    let refused = drag.over.as_ref().is_some_and(|c| !c.valid);
    if let Some(c) = drag.over.as_ref().filter(|c| !c.valid) {
        dim(buf, c.rect);
    }
    let (glyph, bg) = if refused {
        ("\u{2298}", th().status.warning)
    } else {
        ("\u{2387}", th().ui.accent)
    };
    let label = format!(" {glyph} {} ", drag.source);
    let width = Line::from(label.as_str()).width() as u16;
    // Just below-right of the pointer so it never hides the cell being
    // aimed at; moved up/left when that would leave the frame.
    let (px, py) = drag.at;
    let mut x = px.saturating_add(2);
    if x.saturating_add(width) > area.x + area.width {
        x = (area.x + area.width).saturating_sub(width).max(area.x);
    }
    let y = if py + 1 < area.y + area.height {
        py + 1
    } else {
        py.saturating_sub(1).max(area.y)
    };
    let max = (area.x + area.width).saturating_sub(x) as usize;
    buf.set_stringn(
        x,
        y,
        label,
        max,
        Style::default()
            .fg(th().chrome.on_bright)
            .bg(bg)
            .add_modifier(Modifier::BOLD),
    );
}

/// Flips the Refs sidebar's branch hierarchy and writes `ui.ref_tree` back
/// to `config.toml`, so `y` and the palette's toggle make the same choice
/// the Preferences screen does rather than a session-only flag.
///
/// Returns the status line. A failed write still leaves the view toggled
/// (the user asked to see it) and says it was not saved.
pub(crate) fn toggle_ref_tree(hierarchical_refs: &mut bool) -> String {
    *hierarchical_refs = !*hierarchical_refs;
    let state = if *hierarchical_refs {
        msg::STATE_ON.get()
    } else {
        msg::STATE_OFF.get()
    };
    let value = if *hierarchical_refs { "true" } else { "false" };
    match gitpoulpi_config::ConfigPaths::discover() {
        Some(paths) => {
            match gitpoulpi_config::set_in_file(&paths.config_file(), "ui.ref_tree", value) {
                Ok(_) => msg::ref_tree_saved(state),
                Err(err) => crate::activity_log::report_problem_clear(
                    crate::activity_log::ProblemSource::Save,
                    &msg::ref_tree_not_saved(state, err),
                ),
            }
        }
        None => msg::ref_tree_no_config(state),
    }
}

/// [`gitpoulpi_app::refs_tree::refs_under_folder`] with the window's
/// workflow view (so a workflow-group folder resolves its members).
pub(crate) fn refs_under_folder(names: &[String], path: &str) -> Vec<String> {
    let flow = crate::flow::view::configured();
    gitpoulpi_app::refs_tree::refs_under_folder(flow.as_deref(), names, path)
}

/// The folder or remote-root row selected in the active Refs section, as
/// `(collapse-key path, is this a remote's own root)`; `None` on a leaf
/// selection, an empty section, or refs that failed to load. The folder twin
/// of `selected_ref_leaf`, for "hide/solo the selected folder".
pub(crate) fn selected_ref_folder(
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    refs_filter: &str,
    section: SidebarSection,
    selected: Option<usize>,
    hierarchical: bool,
    collapsed_folders: &CollapsedFolders,
    remotes: &[String],
) -> Option<(String, bool)> {
    let kind = section.ref_kind()?;
    let refs = refs_result.as_ref().ok()?;
    let names = group_refs_kind(refs, refs_filter, kind);
    let rows = ref_display_rows(&names, kind, hierarchical, collapsed_folders, remotes);
    let row = rows.get(selected?)?;
    let path = row.folder_path()?;
    Some((path.to_string(), row.is_remote_root()))
}

pub(crate) trait SidebarSectionExt: Sized {
    /// The section's icon (a terminal glyph; the section model itself is
    /// `gitpoulpi_app::palette::vocab::refs_sidebar::SidebarSection`).
    fn icon(self) -> &'static str;
}

impl SidebarSectionExt for SidebarSection {
    fn icon(self) -> &'static str {
        match self {
            SidebarSection::Local => LOCAL_ICON,
            SidebarSection::Remote => REMOTE_ICON,
            SidebarSection::Worktrees => WORKTREE_ICON,
            SidebarSection::Stashes => STASH_ICON,
            SidebarSection::Tags => TAG_ICON,
            SidebarSection::PullRequests => PULL_REQUEST_ICON,
            SidebarSection::Issues => ISSUE_ICON,
            SidebarSection::Submodules => SUBMODULE_ICON,
            SidebarSection::WorktreesEmpty => WORKTREE_ICON,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    /// The helper the commit details' Changed files list paints through: the
    /// row under the pointer lights across its whole width (offset applied),
    /// the others do not, and a row carrying the selection band keeps it.
    #[test]
    fn the_file_list_row_under_the_pointer_lights_and_the_others_do_not() {
        use ratatui::buffer::Buffer;
        let inner = Rect::new(2, 5, 20, 4);
        let hover = crate::th().ui.hover;
        let selection = crate::th().ui.selection;
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 12));
        // Scrolled by one: the pointer on screen row 6 is item 2.
        set_row_pointer(Some((9, 6)));
        let (index, band) = hovered_inner_row(inner, 1, 4).expect("a row");
        assert_eq!(index, 2);
        assert_eq!(band, Rect::new(2, 6, 20, 1));
        paint_row_hover(&mut buf, band);
        for x in 2..22 {
            assert_eq!(buf[(x, 6)].bg, hover, "col {x}");
            assert_ne!(buf[(x, 5)].bg, hover);
            assert_ne!(buf[(x, 7)].bg, hover);
        }
        // A selected row keeps its band.
        buf.set_style(
            Rect::new(2, 7, 20, 1),
            ratatui::style::Style::default().bg(selection),
        );
        paint_row_hover(&mut buf, Rect::new(2, 7, 20, 1));
        assert_eq!(buf[(5, 7)].bg, selection);
        // Past the last item, outside the list, and with no pointer: nothing.
        assert!(hovered_inner_row(inner, 1, 2).is_none());
        set_row_pointer(Some((9, 12)));
        assert!(hovered_inner_row(inner, 0, 4).is_none());
        set_row_pointer(None);
        assert!(hovered_inner_row(inner, 0, 4).is_none());
    }

    /// The gesture's rule: merge/rebase/cherry-pick all *write* to a local
    /// branch, so that is the only kind of ref a drop can land on, and
    /// nothing lands on itself. Both the lit drop target and the footer
    /// banner read this one predicate.
    #[test]
    fn a_ref_drag_ghost_follows_the_pointer_and_leaves_no_trace() {
        let area = Rect::new(0, 0, 40, 6);
        let blank = ratatui::buffer::Buffer::empty(area);
        let mut drag = RefDrag::armed_at("feature/x".into(), RefRowKind::Local, 2, 2);
        // Armed but not started: nothing is drawn.
        let mut buf = blank.clone();
        paint_ref_drag(&mut buf, &drag);
        assert_eq!(buf, blank);
        // Started: the ghost sits by the pointer, accent-backed.
        drag.started = true;
        drag.at = (10, 3);
        drag.source_rect = Some(Rect::new(0, 2, 8, 1));
        let mut buf = blank.clone();
        paint_ref_drag(&mut buf, &drag);
        let row: String = (0..40).map(|x| buf[(x, 4)].symbol().to_string()).collect();
        assert!(row.contains("feature/x"), "{row:?}");
        assert_eq!(buf[(13, 4)].bg, th().ui.accent);
        assert!(
            buf[(0, 2)].modifier.contains(Modifier::DIM),
            "source dimmed"
        );
        // Invalid hover: amber ghost, target dimmed.
        drag.over = Some(RefDropCandidate {
            name: "v1".into(),
            kind: RefRowKind::Tag,
            valid: false,
            rect: Rect::new(20, 0, 5, 1),
        });
        let mut buf = blank.clone();
        paint_ref_drag(&mut buf, &drag);
        assert_eq!(buf[(13, 4)].bg, th().status.warning);
        assert!(buf[(21, 0)].modifier.contains(Modifier::DIM));
        // Near the right edge the ghost stays inside the frame.
        drag.at = (39, 5);
        let mut buf = blank.clone();
        paint_ref_drag(&mut buf, &drag);
        let row: String = (0..40).map(|x| buf[(x, 4)].symbol().to_string()).collect();
        assert!(row.contains("feature/x"), "{row:?}");
        // Drag over (drag dropped): nothing paints, frame equals a blank one.
        let mut buf = blank.clone();
        if let Some(d) = None::<&RefDrag>.filter(|d| d.started) {
            paint_ref_drag(&mut buf, d);
        }
        assert_eq!(buf, blank);
    }

    #[test]
    fn a_ref_drops_only_onto_a_different_local_branch() {
        use RefRowKind::{Local, Remote, Tag};
        assert!(ref_drop_is_valid("feature/x", Local, "main", Local));
        assert!(!ref_drop_is_valid("main", Local, "main", Local));
        // Local onto a remote branch is a push; remote onto local a pull/merge.
        assert!(ref_drop_is_valid("feature/x", Local, "origin/main", Remote));
        assert!(ref_drop_is_valid("origin/main", Remote, "main", Local));
        // ...but remote onto remote, and anything with a tag, is nothing.
        assert!(!ref_drop_is_valid("origin/a", Remote, "origin/b", Remote));
        assert!(!ref_drop_is_valid("feature/x", Local, "v1.0", Tag));
        assert!(!ref_drop_is_valid("origin/main", Remote, "v1.0", Tag));
        // A tag goes onto any ref kind, but not onto itself.
        assert!(ref_drop_is_valid("v1.0", Tag, "main", Local));
        assert!(ref_drop_is_valid("v1.0", Tag, "origin/main", Remote));
        assert!(ref_drop_is_valid("v1.0", Tag, "v2.0", Tag));
        assert!(!ref_drop_is_valid("v1.0", Tag, "v1.0", Tag));
        // The same name and kind is the ref itself.
        assert!(!ref_drop_is_valid("main", Remote, "main", Remote));
    }

    /// Arming is not dragging. A press on a ref must leave the click and the
    /// double-click alone; only leaving the press cell turns it into a drag.
    #[test]
    fn a_press_on_a_ref_arms_a_drag_without_starting_one() {
        let drag = RefDrag::armed_at("feature/x".into(), RefRowKind::Local, 7, 12);
        assert!(!drag.started);
        assert_eq!(drag.origin, (7, 12));
        assert_eq!(drag.at, (7, 12));
        assert!(drag.over.is_none());
    }

    /// The in-content half of the drag affordance, which works on a terminal
    /// with no `OSC 22` support (see `cursor.rs`): it names the ref being
    /// carried, what it is over, and the way out, in all three states.
    #[test]
    fn the_drag_banner_names_the_ref_the_target_and_the_way_out() {
        let mut drag = RefDrag::armed_at("feature/x".into(), RefRowKind::Local, 0, 0);
        drag.started = true;

        let carrying = drag.banner();
        assert!(carrying.contains("feature/x"), "{carrying}");
        assert!(carrying.contains("Esc"), "{carrying}");

        drag.over = Some(RefDropCandidate {
            name: "main".into(),
            kind: RefRowKind::Local,
            valid: true,
            rect: Rect::new(0, 0, 1, 1),
        });
        let onto = drag.banner();
        assert!(
            onto.contains("feature/x") && onto.contains("main"),
            "{onto}"
        );
        assert!(onto.contains("release to drop"), "{onto}");

        drag.over = Some(RefDropCandidate {
            name: "v1.0".into(),
            kind: RefRowKind::Tag,
            valid: false,
            rect: Rect::new(0, 0, 1, 1),
        });
        let refused = drag.banner();
        assert!(refused.contains("v1.0"), "{refused}");
        assert!(refused.contains("can't take a drop"), "{refused}");

        // A ref carried back onto itself is refused for a different reason,
        // and says so.
        drag.over = Some(RefDropCandidate {
            name: "feature/x".into(),
            kind: RefRowKind::Local,
            valid: false,
            rect: Rect::new(0, 0, 1, 1),
        });
        let itself = drag.banner();
        assert!(itself.contains("back where it came from"), "{itself}");
        assert!(!itself.contains("can't take a drop"), "{itself}");

        // This segment renders BOLD, and a text-default emoji codepoint
        // flips to its color presentation under a style, as the sidebar's
        // eye icon once did. `\u{2192}` is such a codepoint; the plain
        // triangle the folder rows use is not.
        for banner in [carrying, onto, refused, itself] {
            assert!(!banner.contains('\u{2192}'), "{banner}");
        }
    }

    /// Plain ref rows are not registered hit targets, so the drag resolves
    /// them the same way the render pass builds them. The `1` is
    /// `row_to_index`'s `top_offset` - an absolute offset from `area.y`,
    /// which is the sub-pane's own border here.
    #[test]
    fn sidebar_ref_at_finds_the_row_under_the_pointer_and_its_band() {
        let area = Rect::new(0, 0, 20, 6);
        let sections = vec![(SidebarSection::Local, area)];
        let refs = vec![
            local_branch_at("main", oid(1)),
            local_branch_at("dev", oid(2)),
        ];
        let collapsed = std::collections::HashSet::new();
        let at = |col, row| {
            sidebar_ref_at(
                &sections,
                &refs,
                "",
                false,
                &collapsed,
                &[],
                |_| 0,
                col,
                row,
            )
        };

        // `group_refs` sorts, so row 1 (the first list row under the
        // border) is `dev` and row 2 is `main`.
        let hit = at(5, 1).expect("a ref row");
        assert_eq!(hit.name, "dev");
        assert_eq!(hit.kind, RefRowKind::Local);
        // Inside the border on both sides, so lighting it can never paint
        // over the sub-pane's own frame.
        assert_eq!(hit.rect, Rect::new(1, 1, 18, 1));
        assert_eq!(at(5, 2).expect("a ref row").name, "main");

        // The border row itself, and the empty rows past the end of the
        // list, are not refs.
        assert!(at(5, 0).is_none());
        assert!(at(5, 4).is_none());
        // Outside every section.
        assert!(at(40, 1).is_none());

        // The sidebar's filter is part of what is drawn, so it is part of
        // what a drag resolves against -- otherwise a drag would pick up
        // a ref that is not on screen.
        let filtered = sidebar_ref_at(&sections, &refs, "ma", false, &collapsed, &[], |_| 0, 5, 1)
            .expect("a ref row");
        assert_eq!(filtered.name, "main");
    }

    /// Three remotes - one of them named `grp/sub` - and a folder two
    /// levels down. As `ref_display_rows` lays them out:
    ///
    /// ```text
    ///  0 gh/            (remote)
    ///  1   main          gh/main
    ///  2 grp/sub        (remote)
    ///  3   feature/
    ///  4     x           grp/sub/feature/x
    ///  5 origin         (remote)
    ///  6   feature/
    ///  7     login       origin/feature/login
    ///  8     signup      origin/feature/signup
    ///  9   main          origin/main
    /// ```
    fn nested_remote_refs() -> (Vec<gitpoulpi_core::domain::Ref>, Vec<String>) {
        let refs = vec![
            remote_branch_at("gh", "main", oid(1)),
            remote_branch_at("grp/sub", "feature/x", oid(1)),
            remote_branch_at("origin", "feature/login", oid(2)),
            remote_branch_at("origin", "feature/signup", oid(3)),
            remote_branch_at("origin", "main", oid(1)),
        ];
        let remotes = vec![
            "gh".to_string(),
            "grp/sub".to_string(),
            "origin".to_string(),
        ];
        (refs, remotes)
    }

    /// The sidebar's row -> ref mapping with the tree's extra nesting,
    /// **scrolled** and **collapsed** at once - the two things that
    /// shift every row below them and that a hand-rolled index would get
    /// wrong. Every row of a 4-row viewport is checked, folders included:
    /// a folder must resolve to *no* ref, or a drag or a hover would pick
    /// up whatever leaf sits at that index.
    #[test]
    fn sidebar_ref_at_maps_every_row_of_a_nested_remote_tree_scrolled_and_collapsed() {
        let area = Rect::new(0, 0, 30, 6); // border + 4 rows + border
        let sections = vec![(SidebarSection::Remote, area)];
        let (refs, remotes) = nested_remote_refs();
        let at = |collapsed: &CollapsedFolders, offset: usize, row: u16| {
            sidebar_ref_at(
                &sections,
                &refs,
                "",
                true,
                collapsed,
                &remotes,
                |_| offset,
                5,
                row,
            )
            .map(|hit| {
                assert_eq!(
                    hit.rect,
                    Rect::new(1, row, 28, 1),
                    "the band is the row itself"
                );
                assert_eq!(hit.kind, RefRowKind::Remote);
                hit.name
            })
        };
        let viewport = |collapsed: &CollapsedFolders, offset: usize| -> Vec<Option<String>> {
            (0..6).map(|row| at(collapsed, offset, row)).collect()
        };
        let some = |s: &str| Some(s.to_string());

        let nothing = CollapsedFolders::new();
        assert_eq!(
            viewport(&nothing, 0),
            vec![None, None, some("gh/main"), None, None, None],
            "unscrolled: gh/, main, grp/sub, feature/"
        );
        assert_eq!(
            viewport(&nothing, 3),
            vec![None, None, some("grp/sub/feature/x"), None, None, None],
            "scrolled by 3: feature/, x, origin, feature/"
        );
        assert_eq!(
            viewport(&nothing, 6),
            vec![
                None,
                None,
                some("origin/feature/login"),
                some("origin/feature/signup"),
                some("origin/main"),
                None
            ],
            "scrolled by 6: feature/, login, signup, main"
        );

        // Folding `origin/feature` takes two rows out from under
        // `origin/main`, which moves up to index 7.
        let folded: CollapsedFolders = [(RefRowKind::Remote, "origin/feature".to_string())]
            .into_iter()
            .collect();
        assert_eq!(
            viewport(&folded, 5),
            vec![None, None, None, some("origin/main"), None, None],
            "collapsed + scrolled: origin, feature/ (folded), main, past the end"
        );

        // Folding a remote's own node -- keyed by the remote's name, which
        // here contains a `/`.
        let folded_remote: CollapsedFolders = [(RefRowKind::Remote, "grp/sub".to_string())]
            .into_iter()
            .collect();
        assert_eq!(
            viewport(&folded_remote, 2),
            vec![None, None, None, None, some("origin/feature/login"), None],
            "grp/sub (folded), origin, feature/, login"
        );

        // A LOCAL folder of the same path is a different key: it must not
        // fold anything in REMOTE.
        let local_only: CollapsedFolders = [(RefRowKind::Local, "origin/feature".to_string())]
            .into_iter()
            .collect();
        assert_eq!(viewport(&local_only, 6), viewport(&nothing, 6));
    }

    /// The keyboard half of the same mapping: `Enter`, the palette's
    /// `Refs: Checkout selected ref` and every other "selected ref"
    /// command resolve the selection through `selected_ref_leaf`.
    #[test]
    fn selected_ref_leaf_follows_the_tree_including_folds() {
        let (refs, remotes) = nested_remote_refs();
        let refs = Ok(refs);
        let pick = |i: usize, collapsed: &CollapsedFolders| {
            selected_ref_leaf(
                &refs,
                "",
                SidebarSection::Remote,
                Some(i),
                true,
                collapsed,
                &remotes,
            )
        };
        let nothing = CollapsedFolders::new();
        assert_eq!(pick(4, &nothing).as_deref(), Some("grp/sub/feature/x"));
        assert_eq!(pick(9, &nothing).as_deref(), Some("origin/main"));
        // Folder rows are not refs.
        for folder in [0, 2, 3, 5, 6] {
            assert_eq!(pick(folder, &nothing), None, "row {folder} is a folder");
        }
        let folded: CollapsedFolders = [(RefRowKind::Remote, "origin/feature".to_string())]
            .into_iter()
            .collect();
        assert_eq!(pick(7, &folded).as_deref(), Some("origin/main"));
        assert_eq!(pick(9, &folded), None, "past the end once folded");

        // With the tree off it is the flat, sorted list again.
        assert_eq!(
            selected_ref_leaf(
                &refs,
                "",
                SidebarSection::Remote,
                Some(1),
                false,
                &nothing,
                &remotes
            )
            .as_deref(),
            Some("grp/sub/feature/x")
        );
    }

    /// The REMOTE shape: a remote's own node carries the remote
    /// icon and no trailing `/` (its name may contain one); a `/`-derived
    /// folder keeps the `name/` spelling. A remote root's icon never
    /// changes with fold state - only a plain folder's does.
    #[test]
    fn a_remote_node_gets_the_remote_icon_and_a_folder_does_not() {
        assert_eq!(
            ref_folder_label("", "\u{25BE}", "grp/sub", true, true),
            format!("\u{25BE} {REMOTE_ICON} grp/sub")
        );
        // Folding a remote root does not swap its globe for anything else.
        assert_eq!(
            ref_folder_label("", "\u{25B8}", "grp/sub", true, false),
            format!("\u{25B8} {REMOTE_ICON} grp/sub")
        );
        assert_eq!(
            ref_folder_label("  ", "\u{25B8}", "feature", false, false),
            format!("  \u{25B8} {FOLDER_CLOSED_ICON} feature/")
        );
        assert!(!ref_folder_label("", "\u{25BE}", "feature", false, true).contains(REMOTE_ICON));
    }

    /// An intermediate `/`-level folder's icon reflects its own fold state
    /// (closed while collapsed, open while expanded) and the two are visibly
    /// different glyphs.
    #[test]
    fn a_folders_icon_tracks_its_own_fold_state() {
        let collapsed = ref_folder_label("", "\u{25B8}", "feature", false, false);
        let expanded = ref_folder_label("", "\u{25BE}", "feature", false, true);
        assert!(
            collapsed.contains(FOLDER_CLOSED_ICON),
            "{collapsed:?} should carry the closed-folder icon"
        );
        assert!(
            !collapsed.contains(FOLDER_OPEN_ICON),
            "{collapsed:?} should not carry the open-folder icon"
        );
        assert!(
            expanded.contains(FOLDER_OPEN_ICON),
            "{expanded:?} should carry the open-folder icon"
        );
        assert!(
            !expanded.contains(FOLDER_CLOSED_ICON),
            "{expanded:?} should not carry the closed-folder icon"
        );
        assert_ne!(FOLDER_CLOSED_ICON, FOLDER_OPEN_ICON);
    }

    /// Both folder icons must be a single display cell wide and on the
    /// project's default-emoji-presentation list (no VS16 selector, never a
    /// combining mark), like `REMOTE_ICON`/`TAG_ICON`/`STASH_ICON`, so the
    /// icon cannot flip presentation on a bold/selected row or shear a
    /// neighboring column.
    #[test]
    fn folder_icons_are_one_cell_and_need_no_variation_selector() {
        for icon in [FOLDER_CLOSED_ICON, FOLDER_OPEN_ICON] {
            assert!(!icon.contains('\u{FE0F}'), "{icon:?} should need no VS16");
            assert_eq!(display_width(icon), 2, "{icon:?} should measure 2 cells");
        }
    }

    /// The Refs sidebar's hide mark and the context menu's Hide/Show
    /// entry are one glyph, and not one the solo mode's dots can be
    /// mistaken for.
    #[test]
    fn the_hide_mark_is_the_context_menus_eye_and_not_a_solo_dot() {
        for hidden in [false, true] {
            let actions = ref_menu_entries(
                SidebarSection::Local,
                "main",
                &MenuContext::default(),
                hidden,
                false,
                false,
            );
            let label = actions
                .iter()
                .find_map(|e| match e {
                    context_menu::Entry::Action {
                        label,
                        action: RefContextAction::ToggleHideInGraph,
                        ..
                    } => Some(label),
                    _ => None,
                })
                .expect("a hide entry");
            assert!(
                label.starts_with(HIDE_IN_GRAPH_ICON),
                "{label:?} leads with the sidebar's eye"
            );
        }
        for solo_glyph in [SOLO_FULL_MARK, "\u{25CB}"] {
            assert_ne!(HIDE_IN_GRAPH_ICON, solo_glyph);
            assert_ne!(HIDDEN_IN_GRAPH_MARK, solo_glyph);
        }
        // The hidden *state* is not the hide *action*.
        assert_ne!(HIDDEN_IN_GRAPH_MARK, HIDE_IN_GRAPH_ICON);
        assert_eq!(display_width(HIDDEN_IN_GRAPH_MARK), 1);
        // The gutter slot is one cell; a two-cell eye would push every
        // hidden row's name one column right of its neighbors.
        assert_eq!(display_width(HIDE_IN_GRAPH_ICON), 1);
    }

    fn no_collapse() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn stepping_sections_skips_the_hidden_ones() {
        let visible = visible_sections(0, 0);
        let mut active = SidebarSection::Remote;
        // Worktrees/Stashes sit between Remote and Tags in ORDER, but
        // are not visible here, so `l` must land on Tags.
        step_sidebar_section(&mut active, &visible, 1);
        assert_eq!(active, SidebarSection::Tags);
        // ...and wrap back round to Local.
        step_sidebar_section(&mut active, &visible, 1);
        assert_eq!(active, SidebarSection::Local);
        step_sidebar_section(&mut active, &visible, -1);
        assert_eq!(active, SidebarSection::Tags);
    }

    /// An `active` section that just disappeared (last stash dropped)
    /// must not survive as the landing spot.
    #[test]
    fn landing_snaps_off_a_section_that_is_no_longer_visible() {
        let visible = visible_sections(0, 0);
        let mut active = SidebarSection::Stashes;
        land_on_open_ref_section(&mut active, &no_collapse(), &visible);
        assert_eq!(active, SidebarSection::Local);
    }

    #[test]
    fn landing_skips_collapsed_sections() {
        let visible = visible_sections(0, 0);
        let mut collapsed = HashSet::new();
        collapsed.insert("LOCAL".to_string());
        collapsed.insert("REMOTE".to_string());
        let mut active = SidebarSection::Local;
        land_on_open_ref_section(&mut active, &collapsed, &visible);
        assert_eq!(active, SidebarSection::Tags);
    }

    /// Every visible section collapsed: nothing is selectable either
    /// way, so `active` must be left alone rather than spun forever.
    #[test]
    fn landing_gives_up_when_every_section_is_collapsed() {
        let visible = visible_sections(0, 0);
        let collapsed: HashSet<String> = visible.iter().map(|s| s.title().to_string()).collect();
        let mut active = SidebarSection::Remote;
        land_on_open_ref_section(&mut active, &collapsed, &visible);
        assert_eq!(active, SidebarSection::Remote);
    }

    /// `j` off the last row of a section must cross into the next one rather
    /// than dead-ending.
    #[test]
    fn down_past_the_last_row_crosses_into_the_next_section() {
        let visible = visible_sections(0, 0);
        let mut states = SidebarStates::default();
        let mut active = SidebarSection::Local;
        // LOCAL has 2 rows, REMOTE has 2, TAGS has 2.
        let counts = |_s: SidebarSection| 2usize;

        states.local.select(Some(0));
        move_sidebar_selection(
            &mut active,
            &mut states,
            &visible,
            &no_collapse(),
            1,
            counts,
        );
        assert_eq!(active, SidebarSection::Local);
        assert_eq!(states.local.selected(), Some(1));

        // Now on the last row -- one more `j` crosses over.
        move_sidebar_selection(
            &mut active,
            &mut states,
            &visible,
            &no_collapse(),
            1,
            counts,
        );
        assert_eq!(active, SidebarSection::Remote);
        assert_eq!(states.remote.selected(), Some(0));
    }

    #[test]
    fn up_past_the_first_row_lands_on_the_previous_sections_last_row() {
        let visible = visible_sections(0, 0);
        let mut states = SidebarStates::default();
        let mut active = SidebarSection::Remote;
        let counts = |_s: SidebarSection| 3usize;

        states.remote.select(Some(0));
        move_sidebar_selection(
            &mut active,
            &mut states,
            &visible,
            &no_collapse(),
            -1,
            counts,
        );
        assert_eq!(active, SidebarSection::Local);
        assert_eq!(states.local.selected(), Some(2));
    }

    /// A collapsed (or empty) section in the middle has nothing to land
    /// on and must be stepped over, not selected with a phantom row.
    #[test]
    fn crossing_skips_a_collapsed_section() {
        let visible = visible_sections(0, 0);
        let mut collapsed = HashSet::new();
        collapsed.insert("REMOTE".to_string());
        let mut states = SidebarStates::default();
        let mut active = SidebarSection::Local;
        let counts = |_s: SidebarSection| 1usize;

        states.local.select(Some(0));
        move_sidebar_selection(&mut active, &mut states, &visible, &collapsed, 1, counts);
        assert_eq!(active, SidebarSection::Tags);
        assert_eq!(states.tags.selected(), Some(0));
        assert_eq!(states.remote.selected(), None);
    }

    // The wheel over a Refs section scrolls its viewport, never its
    // selection.

    #[test]
    fn wheel_over_a_section_moves_its_offset_and_leaves_the_selection_alone() {
        let mut states = SidebarStates::default();
        states.local.select(Some(2));
        let area = Rect::new(0, 0, 30, 7); // 5 content rows (border top+bottom)

        scroll_sidebar_section(&mut states, SidebarSection::Local, 20, area, 3);

        assert_eq!(
            states.local.selected(),
            Some(2),
            "the wheel must never move the selection"
        );
        assert_eq!(states.local.offset(), 3, "but it must move the offset");
        assert!(
            !states.follow(SidebarSection::Local),
            "a wheel-scrolled section stops following its selection until \
             something else (keyboard, click) touches it again"
        );
    }

    #[test]
    fn wheel_over_a_section_cannot_scroll_the_offset_past_its_own_content() {
        let mut states = SidebarStates::default();
        let area = Rect::new(0, 0, 30, 10); // 8 content rows, more than len
        scroll_sidebar_section(&mut states, SidebarSection::Tags, 4, area, 50);
        assert_eq!(states.tags.offset(), 0);
    }

    #[test]
    fn a_keyboard_move_after_a_wheel_scroll_snaps_the_section_back_to_following() {
        let visible = visible_sections(0, 0);
        let mut states = SidebarStates::default();
        let area = Rect::new(0, 0, 30, 7);
        scroll_sidebar_section(&mut states, SidebarSection::Local, 20, area, 3);
        assert!(!states.follow(SidebarSection::Local));

        let mut active = SidebarSection::Local;
        let counts = |_s: SidebarSection| 20usize;
        move_sidebar_selection(
            &mut active,
            &mut states,
            &visible,
            &no_collapse(),
            1,
            counts,
        );

        assert!(
            states.follow(SidebarSection::Local),
            "a keyboard move (through move_sidebar_selection, which goes \
             through SidebarStates::select) must re-arm following"
        );
    }

    #[test]
    fn sidebar_folder_at_finds_a_folder_row_and_not_a_leaf() {
        let area = Rect::new(0, 0, 30, 6);
        let sections = vec![(SidebarSection::Remote, area)];
        let (refs, remotes) = nested_remote_refs();
        let collapsed = CollapsedFolders::new();
        let at = |row| {
            sidebar_folder_at(
                &sections,
                &refs,
                "",
                true,
                &collapsed,
                &remotes,
                |_| 0,
                5,
                row,
            )
        };
        // Row 0 is the sub-pane's top border; row 1 is `gh/`, row 2 is
        // the leaf `gh/main`.
        assert_eq!(at(0), None);
        assert_eq!(at(1), Some("gh".to_string()));
        assert_eq!(at(2), None);
    }

    // -- hidden-refs counter, show-all, and folder/remote bulk hide/solo --

    #[test]
    fn refs_under_folder_matches_by_prefix_including_a_remotes_own_root() {
        let names = vec![
            "origin/main".to_string(),
            "origin/feature/login".to_string(),
            "origin/feature/signup".to_string(),
            "gh/main".to_string(),
            // Must not match a *sibling* whose name merely starts with
            // the same characters (`origin2`, not `origin`).
            "origin2/main".to_string(),
        ];
        let mut under_origin = refs_under_folder(&names, "origin");
        under_origin.sort();
        assert_eq!(
            under_origin,
            vec![
                "origin/feature/login".to_string(),
                "origin/feature/signup".to_string(),
                "origin/main".to_string(),
            ]
        );
        assert_eq!(
            refs_under_folder(&names, "origin/feature"),
            vec![
                "origin/feature/login".to_string(),
                "origin/feature/signup".to_string(),
            ]
        );
    }

    #[test]
    fn selected_ref_folder_resolves_a_folder_row_and_not_a_leaf() {
        let (refs, remotes) = nested_remote_refs();
        let refs = Ok(refs);
        let pick = |i: usize| {
            selected_ref_folder(
                &refs,
                "",
                SidebarSection::Remote,
                Some(i),
                true,
                &CollapsedFolders::new(),
                &remotes,
            )
        };
        // Row 0 is `gh/` -- a remote's own root.
        assert_eq!(pick(0), Some(("gh".to_string(), true)));
        // Row 3 is `feature/` under `grp/sub` -- a plain folder.
        assert_eq!(pick(3), Some(("grp/sub/feature".to_string(), false)));
        // Row 1 (`gh/main`) is a leaf, not a folder.
        assert_eq!(pick(1), None);
    }

    #[test]
    fn every_sidebar_section_has_its_own_list_state() {
        // Guards the "new section silently shares another's state" bug:
        // selecting in one must not move any other.
        let mut states = SidebarStates::default();
        for (i, section) in SidebarSection::ORDER.iter().enumerate() {
            states.state_mut(*section).select(Some(i));
        }
        for (i, section) in SidebarSection::ORDER.iter().enumerate() {
            assert_eq!(states.state_mut(*section).selected(), Some(i));
        }
    }
}
