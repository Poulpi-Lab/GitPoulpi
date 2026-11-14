//! `ui.graph_ghost_ref`: a faded pill naming the deduced ref in the Ref cell
//! of a hovered commit that has none of its own; nothing without a ghost,
//! when the commit has a ref, and truncated like a real pill when narrow.

use super::*;
use crate::test_support::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const ORDER: [LogColumn; 2] = [LogColumn::Ref, LogColumn::Message];

/// The Ref cells of the commit's row.
fn ref_cells(
    ghost: Option<&crate::graph::GhostRef>,
    ref_w: u16,
    with_ref: bool,
) -> Vec<ratatui::buffer::Cell> {
    let c = commit(1, &[]);
    let refs = if with_ref {
        vec![local_branch_at("main", c.id)]
    } else {
        Vec::new()
    };
    let widths = LogColumnWidths {
        ref_w,
        message_w: 6,
        ..Default::default()
    };
    let row = commit_row_with_changes(RowCtx {
        commit: &c,
        pills: &ref_pills_for_commit(c.id, &refs),
        graph: Line::default(),
        graph_node: None,
        lane_color: Color::Green,
        widths: &widths,
        visible_order: &ORDER,
        is_selected: false,
        is_hovered: true,
        popup_fill_width: None,
        is_dimmed: false,
        avatar_mode: avatar::LogAvatarMode::Off,
        avatar_fallback: gitpoulpi_config::AvatarFallback::Initials,
        ref_connector: None,
        changes: None,
        ghost,
    });
    let area = Rect::new(0, 0, ref_w + 6, 4);
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    let mut state = TableState::default();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = ORDER.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(vec![row], header, &ORDER, &widths, Style::default()),
                area,
                &mut state,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let y = log_top_y(area);
    (0..ref_w).map(|x| buf[(x, y)].clone()).collect()
}

/// The Ref cell's text of the commit's row.
fn ref_cell(ghost: Option<&crate::graph::GhostRef>, ref_w: u16, with_ref: bool) -> String {
    ref_cells(ghost, ref_w, with_ref)
        .iter()
        .map(|c| c.symbol().to_string())
        .collect()
}

/// The modifiers of the drawn (non-blank) Ref cells.
fn drawn_modifiers(ghost: Option<&crate::graph::GhostRef>, with_ref: bool) -> Modifier {
    ref_cells(ghost, 16, with_ref)
        .iter()
        .filter(|c| c.symbol().trim() != "")
        .fold(Modifier::empty(), |m, c| m | c.modifier)
}

fn ghost(name: &str) -> crate::graph::GhostRef {
    crate::graph::GhostRef {
        pill: RefPill {
            name: name.into(),
            kind: RefRowKind::Local,
            checked_out: false,
            combined_remote: false,
            tag_where: None,
            multi_remote: None,
        },
        remote: None,
        deleted_in: None,
    }
}

#[test]
fn the_ghost_pill_names_the_deduced_ref_in_a_row_without_refs() {
    let g = ghost("feature");
    let cell = ref_cell(Some(&g), 16, false);
    assert!(cell.contains("feature"), "{cell:?}");
    assert!(!cell.contains('+'), "no create-branch glyph: {cell:?}");
}

#[test]
fn the_ghost_pill_is_faded_and_a_real_pill_is_not() {
    let g = ghost("feature");
    assert!(drawn_modifiers(Some(&g), false).contains(Modifier::ITALIC));
    assert!(!drawn_modifiers(None, true).contains(Modifier::ITALIC));
}

#[test]
fn no_ghost_without_one_or_when_the_commit_has_a_ref_and_a_narrow_cell_truncates() {
    let g = ghost("feature-with-a-long-name");
    assert!(!ref_cell(None, 16, false).contains("feature"));
    let with_ref = ref_cell(Some(&g), 16, true);
    assert!(
        with_ref.contains("main") && !with_ref.contains("feature"),
        "{with_ref:?}"
    );
    let narrow = ref_cell(Some(&g), 12, false);
    assert!(narrow.contains('\u{2026}'), "{narrow:?}");
}
