use super::*;

#[test]
fn find_hit_target_picks_the_containing_rect() {
    let targets = vec![
        (
            Rect::new(0, 0, 10, 1),
            HitTarget::Toolbar(ToolbarButton::Pull),
        ),
        (
            Rect::new(10, 0, 10, 1),
            HitTarget::LogRefPill {
                name: "main".into(),
                kind: RefRowKind::Local,
            },
        ),
    ];
    assert_eq!(
        find_hit_target(&targets, 5, 0),
        Some(HitTarget::Toolbar(ToolbarButton::Pull))
    );
    assert_eq!(
        find_hit_target(&targets, 15, 0),
        Some(HitTarget::LogRefPill {
            name: "main".into(),
            kind: RefRowKind::Local,
        })
    );
    assert_eq!(find_hit_target(&targets, 25, 0), None);
    assert_eq!(find_hit_target(&targets, 5, 1), None); // outside the row
}

/// `find_hit_target` takes the FIRST rect containing the point, so a
/// button drawn inside a wider band has to be registered before that
/// band or it can never be clicked.
///
/// Two real ones were: `Stage all changes` sits on the Status
/// section-header row and `Stage folder` on a folder row, and both
/// bands were pushed first - clicking either button collapsed the
/// section or the folder instead of staging anything.
#[test]
fn find_hit_target_takes_the_first_match_so_buttons_precede_their_band() {
    let band = Rect::new(0, 0, 40, 1);
    let button = Rect::new(25, 0, 10, 1);
    let header = HitTarget::StatusSectionHeader(StatusSection::Unstaged);
    let bulk = HitTarget::StatusBulkButton(StatusSection::Unstaged);

    // Button first: both the button and the rest of the band work.
    let good = vec![(button, bulk.clone()), (band, header.clone())];
    assert_eq!(find_hit_target(&good, 30, 0), Some(bulk.clone()));
    assert_eq!(find_hit_target(&good, 2, 0), Some(header.clone()));

    // Band first: the button is unreachable. Pinned so the ordering
    // above reads as deliberate rather than incidental.
    let bad = vec![(band, header.clone()), (button, bulk)];
    assert_eq!(find_hit_target(&bad, 30, 0), Some(header));
}

#[test]
fn desired_cursor_shape_prioritizes_an_active_drag_over_the_current_hit_target() {
    // Mid-drag on a log-boundary, mouse has wandered onto a toolbar
    // button's rect -- must still report the resize shape, not flicker
    // to Pointer just because the cursor is momentarily over a button.
    let hit = HitTarget::Toolbar(ToolbarButton::Pull);
    assert_eq!(
        desired_cursor_shape(Some(&hit), None, None, Some(LogColumn::Author), false, None),
        CursorShape::ColResize
    );
    assert_eq!(
        desired_cursor_shape(Some(&hit), None, Some(1), None, false, None),
        CursorShape::ColResize
    );
}

#[test]
fn desired_cursor_shape_maps_hit_targets_and_falls_back_to_hover_state() {
    assert_eq!(
        desired_cursor_shape(
            Some(&HitTarget::LogColumnBoundary(LogColumn::Author)),
            None,
            None,
            None,
            false,
            Some(ResizeGrab::LogColumn(LogColumn::Author))
        ),
        CursorShape::ColResize
    );
    // Message's own right-hand boundary now resizes Message directly,
    // instead of the old workaround that resized Author from it.
    assert_eq!(
        desired_cursor_shape(
            Some(&HitTarget::LogColumnBoundary(LogColumn::Message)),
            None,
            None,
            None,
            false,
            Some(ResizeGrab::LogColumn(LogColumn::Message))
        ),
        CursorShape::ColResize
    );
    assert_eq!(
        desired_cursor_shape(
            Some(&HitTarget::Toolbar(ToolbarButton::Pull)),
            None,
            None,
            None,
            false,
            None
        ),
        CursorShape::Pointer
    );
    // No hit target, but a Refs-sidebar leaf row is hovered (not
    // registered in `hit_targets` itself) -- a row: the arrow, lit, never
    // the open hand. See the dedicated test below.
    assert_eq!(
        desired_cursor_shape(None, Some("main"), None, None, false, None),
        CursorShape::Default
    );
    // Nothing hit, not near a divider -- plain arrow.
    assert_eq!(
        desired_cursor_shape(None, None, None, None, false, None),
        CursorShape::Default
    );
    // Nothing hit, but the one predicate says the pointer is ON a
    // pane divider -- resize. It no longer says so a cell early: see
    // `resize_grab_at`.
    assert_eq!(
        desired_cursor_shape(
            None,
            None,
            None,
            None,
            false,
            Some(ResizeGrab::PaneDivider(1))
        ),
        CursorShape::ColResize
    );
}

/// The inversion this pins: a *hovered* ref row is a row like every other
/// (the arrow, lit), only the pills take the pointing finger. The open hand is
/// what a drag in progress means everywhere else in UI, and it had
/// been wired to plain hover instead - so the one shape the user
/// reads as "you are carrying something" was showing while nothing
/// was being carried, and the shape that says "this is clickable"
/// never showed on a ref at all.
#[test]
fn hovering_a_ref_shows_the_pointing_finger_not_the_open_hand() {
    assert_eq!(
        desired_cursor_shape(None, Some("main"), None, None, false, None),
        CursorShape::Default
    );
    // The pills beside the graph were already right, and stay right:
    // both pill hit targets sit in the `Pointer` arm.
    for hit in [
        HitTarget::LogRefPill {
            name: "main".into(),
            kind: RefRowKind::Local,
        },
        HitTarget::TooltipRefPill {
            name: "main".into(),
            kind: RefRowKind::Local,
        },
    ] {
        assert_eq!(
            desired_cursor_shape(Some(&hit), None, None, None, false, None),
            CursorShape::Pointer
        );
    }
}

/// The other half of the inversion: the open hand has to mean
/// something, and what it means is "a ref is in the air right now".
/// It outranks whatever the pointer happens to be over, exactly as an
/// active resize drag does - carrying a branch across a toolbar
/// button must not flicker the hand back to a finger.
#[test]
fn the_open_hand_shows_only_while_a_ref_is_actually_being_carried() {
    let pill = HitTarget::LogRefPill {
        name: "main".into(),
        kind: RefRowKind::Local,
    };
    assert_eq!(
        desired_cursor_shape(None, Some("main"), None, None, true, None),
        CursorShape::Grab
    );
    assert_eq!(
        desired_cursor_shape(Some(&pill), None, None, None, true, None),
        CursorShape::Grab
    );
    assert_eq!(
        desired_cursor_shape(
            Some(&HitTarget::Toolbar(ToolbarButton::Pull)),
            None,
            None,
            None,
            true,
            None
        ),
        CursorShape::Grab
    );
    // Over empty chrome, still carrying: still the open hand.
    assert_eq!(
        desired_cursor_shape(None, None, None, None, true, None),
        CursorShape::Grab
    );
}

/// `find_hit_entry` exists to light the pill a drag is over, so it
/// must obey the same first-match rule as `find_hit_target` - the
/// rule that makes "register the inner target first" load-bearing
/// everywhere in this file.
#[test]
fn find_hit_entry_returns_the_first_registered_rect_with_its_bounds() {
    let pill = Rect::new(4, 2, 10, 1);
    let band = Rect::new(0, 2, 40, 1);
    let targets = vec![
        (
            pill,
            HitTarget::LogRefPill {
                name: "main".into(),
                kind: RefRowKind::Local,
            },
        ),
        (band, HitTarget::DiffRow(0)),
    ];
    let (rect, _) = find_hit_entry(&targets, 6, 2).expect("inside the pill");
    assert_eq!(rect, pill);
    let (rect, _) = find_hit_entry(&targets, 30, 2).expect("inside the band only");
    assert_eq!(rect, band);
    assert!(find_hit_entry(&targets, 6, 9).is_none());
}

// ---- the responsive toolbar ----------------------

fn shape_of(hit: Option<HitTarget>) -> CursorShape {
    desired_cursor_shape(hit.as_ref(), None, None, None, false, None)
}

/// Every compact control that acts on its own shows the same pointing hand,
/// whichever surface registered it: tabs (the repository tabs, Home, the
/// Radar, `+`, the crosses), buttons, links, ref pills, folding headers,
/// checkboxes, comboboxes and scrollbars. Rows are not here: they keep the
/// arrow and light on hover (`rows_keep_the_arrow_and_light_on_hover`).
#[test]
fn every_clickable_surface_shows_the_pointing_hand() {
    use crate::scrollbar::ScrollBar;
    let clickable = [
        ("a repository tab", HitTarget::RepoTab(0)),
        ("its close cross", HitTarget::RepoTab(1)),
        ("Home", HitTarget::OpenRepositoryButton),
        ("the Radar tab", HitTarget::RadarTab),
        ("the Radar close cross", HitTarget::RadarClose),
        ("the + button", HitTarget::NewTabButton),
        ("the dialog cross", HitTarget::CloseDiff),
        ("a toolbar button", HitTarget::Toolbar(ToolbarButton::Pull)),
        ("a link", HitTarget::LfsInstallLink),
        ("an issue reference", HitTarget::CommitReference(0)),
        (
            "a ref pill",
            HitTarget::LogRefPill {
                name: "main".into(),
                kind: RefRowKind::Local,
            },
        ),
        ("a control found by geometry", HitTarget::VirtualHand),
        ("a checkbox", HitTarget::AmendCheckbox),
        ("a combobox field", HitTarget::ProfileField),
        (
            "a scrollbar arrow or track",
            HitTarget::ScrollbarTrack(ScrollBar::DiffVertical),
        ),
        (
            "a scrollbar thumb",
            HitTarget::ScrollbarThumb(ScrollBar::DiffVertical),
        ),
        (
            "a button of the find bar",
            HitTarget::SearchBarButton(SearchBarButton::Next),
        ),
    ];
    for (what, hit) in clickable {
        assert_eq!(shape_of(Some(hit)), CursorShape::Pointer, "{what}");
    }
}

/// Pane list rows, menu rows and combobox rows are lit on hover and keep the
/// arrow: the one rule of "Pointer and hover rules" in where-we-are.md.
#[test]
fn rows_keep_the_arrow_and_light_on_hover() {
    use crate::hosting_ui::{HostingHit, HostingList};
    let rows = [
        ("a menu row", HitTarget::LogContextMenuItem(0)),
        ("a ref menu row", HitTarget::RefContextMenuItem(0)),
        ("a combobox row", HitTarget::ProfileItem(0)),
        ("a changed-files row by geometry", HitTarget::ListRow),
        ("a compare card", HitTarget::CompareCard(0)),
        (
            "a hosting row",
            HitTarget::Hosting(HostingHit::Row(HostingList::Issues, 0)),
        ),
        (
            "a history row",
            HitTarget::History(crate::history_view::HistoryHit::Row(0)),
        ),
    ];
    for (what, hit) in rows {
        assert_eq!(shape_of(Some(hit)), CursorShape::Default, "{what}");
    }
    // A hovered ref row of the Refs sidebar: the arrow too (the open hand is
    // only for a ref in the air).
    assert_eq!(
        desired_cursor_shape(None, Some("main"), None, None, false, None),
        CursorShape::Default
    );
}

/// The merge editor's Output is the editable result (I-beam); its buttons,
/// chunk checkboxes and tabs keep the hand.
#[test]
fn the_merge_output_is_an_i_beam_and_its_controls_keep_the_hand() {
    use crate::merge_view::MergeHit;
    use gitpoulpi_core::domain::ConflictSide;
    assert_eq!(
        shape_of(Some(HitTarget::Merge(MergeHit::OutputLine(3)))),
        CursorShape::Text
    );
    for hit in [
        MergeHit::Save,
        MergeHit::Close,
        MergeHit::Tab(ConflictSide::Ours),
        MergeHit::Chunk {
            conflict: 0,
            side: ConflictSide::Ours,
        },
        MergeHit::Placeholder { conflict: 0 },
    ] {
        assert_eq!(
            shape_of(Some(HitTarget::Merge(hit))),
            CursorShape::Pointer,
            "{hit:?}"
        );
    }
    assert_eq!(
        shape_of(Some(HitTarget::Editor(crate::editor::EditorHit::Save))),
        CursorShape::Pointer
    );
}

/// A text field takes the I-beam; a diff line is text to read, a Log or
/// commit-column row a plain table row, and an empty area nothing: all of
/// those keep the arrow (only a ref pill in a row takes the hand).
#[test]
fn text_fields_take_the_i_beam_and_plain_rows_keep_the_arrow() {
    for hit in [
        HitTarget::CommitSummaryField,
        HitTarget::CommitDescriptionField,
        HitTarget::WipMessageField,
        HitTarget::CommitFilterField,
    ] {
        assert_eq!(shape_of(Some(hit)), CursorShape::Text);
    }
    for (what, hit) in [
        ("a diff row", Some(HitTarget::DiffRow(0))),
        (
            "a commit column row",
            Some(HitTarget::CommitColumn(
                crate::commit_column::ColumnHit::Row(gitpoulpi_core::domain::Oid([7; 20])),
            )),
        ),
        ("an empty area", None),
    ] {
        assert_eq!(shape_of(hit), CursorShape::Default, "{what}");
    }
}

/// The resize and carry shapes stay what they were: they are answered ahead
/// of the hand.
#[test]
fn resize_and_carry_shapes_outrank_the_hand() {
    let tab = HitTarget::RepoTab(0);
    assert_eq!(
        desired_cursor_shape(Some(&tab), None, None, None, true, None),
        CursorShape::Grab
    );
    assert_eq!(
        desired_cursor_shape(
            Some(&tab),
            None,
            None,
            None,
            false,
            Some(ResizeGrab::PaneDivider(1))
        ),
        CursorShape::ColResize
    );
}
