//! The remotes badge of a Refs row: its forms by width, its colour and its hit
//! target (registered before the section header's band).

use super::*;
use gitpoulpi_core::domain::{
    Oid, Ref, RefKind, RefVisibility, RemoteRelation, RemoteTargetState, TargetRole,
};
use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

fn target(remote: &str, relation: RemoteRelation) -> RemoteTargetState {
    RemoteTargetState {
        remote: remote.into(),
        branch: "feat".into(),
        role: TargetRole::PushTo,
        pull: false,
        tracking: None,
        relation,
        missing: false,
    }
}

fn local(name: &str, multi: Option<Vec<RemoteTargetState>>) -> Ref {
    Ref {
        kind: RefKind::LocalBranch { name: name.into() },
        target: Oid([0; 20]),
        visibility: RefVisibility::Shown,
        is_head: false,
        upstream: None,
        tag_sync: Default::default(),
        multi_remote: multi,
    }
}

fn draw(refs: Vec<Ref>, width: u16) -> (Buffer, Vec<(Rect, HitTarget)>) {
    let dir = tempfile::tempdir().unwrap();
    #[cfg(feature = "hosting")]
    let hosting = HostingUi::with_parts(dir.path(), None, None, Vec::new, None);
    #[cfg(not(feature = "hosting"))]
    let hosting = HostingUi::new(dir.path(), None);
    let refs_result = Ok(refs);
    let worktrees_result = Ok(Vec::new());
    let stashes_result = Ok(Vec::new());
    let (hidden, sections, solo, unpushed) = (
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    );
    let (hovered, hovered_row, hovered_folder, mode, drag, filter) =
        (None, None, None, InputMode::Normal, None, String::new());
    let (remote_names, submodules, notice, heights) = (Vec::new(), Vec::new(), None, Vec::new());
    let mut states = SidebarStates::default();
    let (mut hits, mut editor, mut filter_area, mut avail, mut specs, mut areas) = (
        Vec::new(),
        Default::default(),
        Rect::default(),
        0,
        Vec::new(),
        Vec::new(),
    );
    let collapsed = Default::default();
    let mut terminal = Terminal::new(TestBackend::new(width, 12)).unwrap();
    terminal
        .draw(|frame| {
            draw_sidebar(
                frame,
                &mut SidebarDrawCtx {
                    focus: Focus::Refs,
                    left_pane_collapsed: false,
                    hierarchical_refs: false,
                    active_ref_section: SidebarSection::Local,
                    refs_area: Rect::new(0, 0, width, 12),
                    hovered_target: &hovered,
                    hidden_from_graph: &hidden,
                    collapsed_ref_sections: &sections,
                    collapsed_ref_folders: &collapsed,
                    hovered_ref_row: &hovered_row,
                    hovered_ref_folder: &hovered_folder,
                    input_mode: &mode,
                    ref_drag: &drag,
                    ref_drop_mark: &None,
                    refs_filter: &filter,
                    refs_result: &refs_result,
                    remote_names: &remote_names,
                    leftover_remotes: &Default::default(),
                    sidebar_states: &mut states,
                    solo_refs: &solo,
                    submodules_result: &submodules,
                    unpushed_tags: &unpushed,
                    worktrees_result: &worktrees_result,
                    stashes_result: &stashes_result,
                    pane_notice: &notice,
                    sidebar_heights: &heights,
                    hidden_sidebar_sections: &Default::default(),
                    hosting: &hosting,
                    frame_hits: &mut hits,
                    field_editor: &mut editor,
                    refs_filter_area: &mut filter_area,
                    sidebar_layout_avail: &mut avail,
                    sidebar_layout_specs: &mut specs,
                    sidebar_section_areas: &mut areas,
                    ref_name_error: &None,
                    name_slot: &mut NameSlot::default(),
                },
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    (buf, hits)
}

fn row_text(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

/// The text of the row holding `name`.
fn row_with(buf: &Buffer, name: &str) -> String {
    (0..buf.area.height)
        .map(|y| row_text(buf, y))
        .find(|t| t.contains(name))
        .unwrap_or_default()
}

/// The row of `name` and the column of its badge glyph.
fn badge_cell(buf: &Buffer, name: &str) -> Option<(u16, u16)> {
    (0..buf.area.height).find_map(|y| {
        let text = row_text(buf, y);
        text.contains(name)
            .then(|| (0..buf.area.width).find(|x| buf[(*x, y)].symbol() == GLYPH))
            .flatten()
            .map(|x| (x, y))
    })
}

const GLYPH: &str = "\u{21c9}";

fn three(a: RemoteRelation, b: RemoteRelation) -> Vec<RemoteTargetState> {
    vec![
        target("origin", a),
        target("gitlab", b),
        target("backup", RemoteRelation::UpToDate),
    ]
}

#[test]
fn a_wide_row_lists_the_remotes_and_a_narrow_one_keeps_the_count() {
    let multi = three(RemoteRelation::LocalAhead(2), RemoteRelation::UpToDate);
    let refs = vec![local("feat", Some(multi)), local("main", None)];
    let (wide, _) = draw(refs.clone(), 70);
    let text = row_with(&wide, "feat");
    assert!(text.contains("feat"), "{text}");
    assert!(text.contains("origin \u{2191}2 \u{b7} gitlab = \u{b7} backup ="), "{text}");
    let main = row_with(&wide, "main");
    assert!(!main.contains(GLYPH), "a plain branch has no badge: {main}");

    let (narrow, _) = draw(refs.clone(), 24);
    let text = row_with(&narrow, "feat");
    assert!(text.contains("\u{21c9}3 \u{2191}2"), "{text}");
    assert!(!text.contains("origin"), "{text}");

    let (tiny, _) = draw(refs, 16);
    let text = row_with(&tiny, "feat");
    assert!(text.contains("\u{21c9}3"), "the compact count is the last form: {text}");
}

#[test]
fn the_badge_is_green_yellow_or_red_by_state() {
    let cases = [
        (
            three(RemoteRelation::UpToDate, RemoteRelation::UpToDate),
            th().status.success,
        ),
        (
            three(RemoteRelation::LocalAhead(1), RemoteRelation::RemoteAhead(1)),
            th().status.warning,
        ),
        (
            three(
                RemoteRelation::UpToDate,
                RemoteRelation::Diverged { ahead: 1, behind: 2 },
            ),
            th().status.danger,
        ),
    ];
    for (multi, want) in cases {
        let (buf, _) = draw(vec![local("feat", Some(multi))], 24);
        let (x, y) = badge_cell(&buf, "feat").expect("badge drawn");
        assert_eq!(buf[(x, y)].fg, want);
    }
}

#[test]
fn the_badge_is_a_hit_target_over_exactly_its_glyphs_and_before_the_header_band() {
    let multi = three(RemoteRelation::LocalAhead(2), RemoteRelation::UpToDate);
    let (buf, hits) = draw(vec![local("feat", Some(multi))], 24);
    let (x, y) = badge_cell(&buf, "feat").expect("badge drawn");
    let (rect, _) = hits
        .iter()
        .find(|(_, t)| *t == HitTarget::RefRemotesBadge("feat".into()))
        .expect("the badge registers a hit");
    assert_eq!((rect.x, rect.y, rect.height), (x, y, 1));
    let text = row_text(&buf, y);
    let drawn: String = (rect.x..rect.x + rect.width)
        .map(|c| buf[(c, y)].symbol().to_string())
        .collect();
    assert_eq!(drawn, "\u{21c9}3 \u{2191}2", "{text}");
    // Inner first: the section header's band comes after it.
    let badge_at = hits
        .iter()
        .position(|(_, t)| matches!(t, HitTarget::RefRemotesBadge(_)))
        .unwrap();
    let header_at = hits
        .iter()
        .position(|(_, t)| *t == HitTarget::RefsSectionHeader(SidebarSection::Local))
        .unwrap();
    assert!(badge_at < header_at);
}

#[test]
fn a_long_name_gives_way_so_the_badge_still_fits() {
    let multi = three(RemoteRelation::UpToDate, RemoteRelation::UpToDate);
    let (buf, hits) = draw(
        vec![local("feature/a-very-long-branch-name-indeed", Some(multi))],
        24,
    );
    assert!(badge_cell(&buf, "feature").is_some(), "badge drawn");
    assert!(hits
        .iter()
        .any(|(_, t)| matches!(t, HitTarget::RefRemotesBadge(_))));
}
