use super::*;
use crate::th;

/// Every hit rect has its own glyphs painted under it, and the
/// buttons of a row are reachable - registered before the row band
/// that covers them (the inner-first rule).
#[test]
fn every_button_is_painted_where_it_is_hit_and_reachable() {
    let mut m = manager();
    let area = Rect::new(0, 1, 140, 30);
    let render = render(&mut m, area);
    let mut seen_star = false;
    let mut seen_open = false;
    let mut seen_close = false;
    let mut seen_menu = false;
    let mut seen_section = false;
    for (rect, hit) in &render.hits {
        if matches!(hit, ManagerHit::Row(_) | ManagerHit::Header(_)) {
            continue;
        }
        let line = &render.lines[(rect.y - area.y) as usize];
        let painted: String = text(line)
            .chars()
            .skip((rect.x - area.x) as usize)
            .take(rect.width as usize)
            .collect();
        assert_eq!(
            render.hit(rect.x, rect.y),
            Some(*hit),
            "{hit:?} is covered by something registered before it"
        );
        match hit {
            ManagerHit::Star(_) => {
                seen_star = true;
                assert!(painted == STAR_ON || painted == STAR_OFF, "{painted:?}");
            }
            ManagerHit::RowOpen(_) => {
                seen_open = true;
                assert!(painted.contains("Open") || painted.contains("Switch"));
            }
            ManagerHit::RowClose(_) => {
                seen_close = true;
                assert!(painted.contains("Close") || painted.contains("Remove"));
            }
            ManagerHit::RowMenu(_) => {
                seen_menu = true;
                assert!(painted.contains(MORE));
            }
            ManagerHit::SectionAction(_) => {
                seen_section = true;
                assert!(painted.contains("Close all") || painted.contains("Remove all"));
            }
            ManagerHit::Button(b) => assert!(painted.contains(b.label()), "{painted:?}"),
            _ => {}
        }
    }
    assert!(seen_star && seen_open && seen_close && seen_menu && seen_section);
}

#[test]
fn a_row_shows_owner_branch_ahead_behind_and_wip_counters() {
    let mut m = manager();
    m.learn(
        Path::new("/work/beta"),
        RepoFacts {
            branch: Some("feature/x".into()),
            ahead: 2,
            behind: 3,
            modified: 4,
            added: 5,
            owner: Some("Poulpi-Lab".into()),
            dirty: true,
        },
    );
    let render = render(&mut m, Rect::new(0, 0, 160, 30));
    let beta = render
        .lines
        .iter()
        .map(text)
        .find(|l| l.contains("beta"))
        .unwrap();
    for part in [
        "Poulpi-Lab",
        "feature/x",
        "\u{2191}2",
        "\u{2193}3",
        "\u{270E} 4",
        "+ 5",
    ] {
        assert!(beta.contains(part), "{part:?} missing from {beta:?}");
    }
    // Before the probe answers: the parent folder stands in for the
    // owner, and an ellipsis for the status.
    let alpha = render
        .lines
        .iter()
        .map(text)
        .find(|l| l.contains("alpha"))
        .unwrap();
    assert!(alpha.contains("work"), "{alpha:?}");
    assert!(alpha.contains('\u{2026}'), "{alpha:?}");
}

/// The keyboard selection is kept on screen, however long the list.
#[test]
fn the_selection_scrolls_into_view() {
    let repos: Vec<KnownRepo> = (0..60)
        .map(|i| {
            let mut r = repo(&format!("r{i:02}"));
            r.recent = Some(i);
            r
        })
        .collect();
    let mut m = RepoManager::new(repos);
    let area = Rect::new(0, 0, 120, 20);
    render(&mut m, area);
    handle_key(&mut m, Key::End, false);
    let render = render(&mut m, area);
    let last = m.rows().len() - 1;
    assert!(m.scroll <= last && last < m.scroll + m.list_height);
    assert!(render.lines.iter().any(|l| text(l).contains("r59")));
}

/// Home tab selection: `new()` pre-selects the first repository row (what
/// `Enter` opens), shown selected from the very first draw by its row
/// background alone. The left gutter carries no arrow (owner, 2026-10-28):
/// it was mistaken for a section header's fold arrow. The row text keeps
/// the column it has when unselected.
#[test]
fn the_preselected_row_is_highlighted_without_an_arrow_marker() {
    let mut m = manager();
    let first_draw = render(&mut m, Rect::new(0, 0, 140, 30));
    let selected = m.selected;
    let (rect, _) = first_draw
        .hits
        .iter()
        .find(|(_, hit)| *hit == ManagerHit::Row(selected))
        .expect("the selected row is drawn");
    let line = &first_draw.lines[rect.y as usize];
    assert!(
        line.spans
            .iter()
            .any(|s| s.style.bg == Some(th().ui.selection)),
        "the selected row keeps its background highlight"
    );
    let gutter: String = text(line).chars().take(3).collect();
    assert_eq!(
        gutter, "   ",
        "no arrow in the selection gutter: {gutter:?}"
    );
    // The star sits where it does on an unselected row.
    let star_at = text(line)
        .chars()
        .position(|c| c.to_string() == STAR_ON || c.to_string() == STAR_OFF)
        .unwrap();
    assert_eq!(star_at, 3);
    // Only the fold arrows of section headers remain.
    for l in &first_draw.lines {
        let t = text(l);
        if t.contains('\u{25B8}') || t.contains('\u{25BE}') {
            assert!(
                t.contains("REPOSITORIES") || t.contains("FAVORITES"),
                "a stray arrow outside a section header: {t:?}"
            );
        }
    }
}

/// A list longer than its room keeps its last column for the scrollbar and
/// shows none when everything fits; a drag on the thumb moves the list.
#[test]
fn a_long_list_has_a_scrollbar_the_thumb_drags() {
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    let repos: Vec<KnownRepo> = (0..60)
        .map(|i| {
            let mut r = repo(&format!("r{i:02}"));
            r.recent = Some(i);
            r
        })
        .collect();
    let mut m = RepoManager::new(repos);
    let area = Rect::new(0, 0, 120, 20);
    let drawn = render(&mut m, area);
    let track = m.bar.expect("the list overflows");
    assert_eq!(track.rect.x, 119);
    assert!(
        drawn.hits.iter().all(|(r, _)| r.right() <= track.rect.x),
        "no row or button runs under the bar"
    );
    let (len, top) = m.list_bar_state();
    assert_eq!(len, m.rows().len());
    let mouse = |kind, column, row| MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    };
    let down = mouse(
        MouseEventKind::Down(MouseButton::Left),
        119,
        track.rect.y + 1,
    );
    let top = m
        .bar_drag
        .on_mouse(m.bar, down, len, top)
        .expect("the thumb is held");
    m.set_list_scroll(top);
    let drag = mouse(
        MouseEventKind::Drag(MouseButton::Left),
        0,
        track.rect.bottom() + 9,
    );
    let top = m.bar_drag.on_mouse(m.bar, drag, len, top).expect("dragged");
    m.set_list_scroll(top);
    render(&mut m, area);
    assert_eq!(m.scroll, len - m.list_height, "dragged to the end");
    // A page with few rows has no bar.
    let mut few = manager();
    render(&mut few, area);
    assert!(few.bar.is_none());
}

/// The path cell: `~` for the home folder, cut in the middle, absent when
/// there is no room - and never at the name's expense at any width.
#[test]
fn paths_are_shortened_and_never_push_the_name_out() {
    use super::super::render::{home_shortened, mid_fit};
    assert_eq!(
        home_shortened(Path::new("/home/me/code/app"), Some(Path::new("/home/me"))),
        if cfg!(windows) {
            "~\\code\\app"
        } else {
            "~/code/app"
        }
    );
    assert_eq!(home_shortened(Path::new("/srv/app"), None), "/srv/app");
    let cut = mid_fit("~/work/some/deep/folder/app", 14);
    assert_eq!(display_width(&cut), 14);
    assert!(cut.contains('\u{2026}') && cut.starts_with("~/") && cut.ends_with("app"));
    assert_eq!(mid_fit("short", 14), "short");

    let mut long = repo("alpha");
    long.path = PathBuf::from("/very/long/parent/folders/that/go/on/alpha");
    for width in [60u16, 80, 100, 140] {
        let mut m = RepoManager::new(vec![long.clone()]);
        let area = Rect::new(0, 0, width, 12);
        let r = render(&mut m, area);
        let row = r
            .lines
            .iter()
            .map(text)
            .find(|l| l.contains("alpha"))
            .unwrap_or_else(|| panic!("name missing at {width}"));
        assert!(display_width(&row) <= width as usize, "{width}: {row:?}");
        if width >= 100 {
            assert!(row.contains("/very"), "{width}: {row:?}");
        }
    }
}
