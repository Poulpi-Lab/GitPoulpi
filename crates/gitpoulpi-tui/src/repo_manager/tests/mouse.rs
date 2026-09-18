use super::*;
use crate::th;

#[test]
fn clicking_a_star_the_header_action_and_a_row_twice() {
    let mut m = manager();
    let render = render(&mut m, Rect::new(0, 0, 140, 30));
    let star = render
        .hits
        .iter()
        .find_map(|(_, h)| matches!(h, ManagerHit::Star(_)).then_some(*h))
        .unwrap();
    assert!(matches!(
        click(&mut m, star, false),
        ManagerAction::ToggleFavorite(_)
    ));
    assert_eq!(
        click(&mut m, ManagerHit::SectionAction(Section::Open), false),
        ManagerAction::CloseAllTabs
    );
    assert_eq!(
        click(&mut m, ManagerHit::SectionAction(Section::Recent), false),
        ManagerAction::RemoveAllRecent
    );
    let row = m
        .rows()
        .iter()
        .position(|r| matches!(r, ManagerRow::Repo { repo, .. } if m.repos[*repo].name == "delta"))
        .unwrap();
    assert_eq!(
        click(&mut m, ManagerHit::Row(row), false),
        ManagerAction::Handled
    );
    assert_eq!(m.selected, row);
    assert_eq!(
        click(&mut m, ManagerHit::Row(row), true),
        ManagerAction::Open(PathBuf::from("/work/delta"))
    );
}

/// A click anywhere in this view other than the search
/// box itself releases its typing focus, the query untouched - every
/// `ManagerHit` variant is covered here, including one that already
/// had its own reason to clear it (`Header` also toggles a section;
/// `Row` also selects a repo), so a future variant can't silently
/// reintroduce the bug by being left out of a hand-picked list.
#[test]
fn any_click_other_than_search_releases_its_focus_but_keeps_the_query() {
    let mut m = manager();
    let row = m
        .rows()
        .iter()
        .position(|r| matches!(r, ManagerRow::Repo { .. }))
        .unwrap();
    let star = m
        .rows()
        .iter()
        .position(|r| matches!(r, ManagerRow::Repo { .. }))
        .map(ManagerHit::Star)
        .unwrap();
    let hits = [
        ManagerHit::Button(ManagerButton::CollapseAll),
        ManagerHit::Header(Section::Open),
        ManagerHit::SectionAction(Section::Open),
        ManagerHit::Row(row),
        star,
    ];
    for hit in hits {
        m.query = "alpha".into();
        m.search_focused = true;
        click(&mut m, hit, false);
        assert!(!m.search_focused, "{hit:?} must release the search focus");
        assert_eq!(m.query, "alpha", "{hit:?} must not touch the query");
    }
    // The one exception: clicking the search box itself takes focus,
    // exactly as before.
    m.search_focused = false;
    click(&mut m, ManagerHit::Search, false);
    assert!(m.search_focused);
}

/// The same guarantee holds for a mouse click. It also pins down what
/// made the stray arrow "disappear": collapsing and re-expanding `OPEN REPOSITORIES` --
/// the section header directly *above* the pre-selected row - moves
/// `selected` onto that header itself (`toggle_section`), so the
/// repository row underneath is never the selected one again. Checked
/// through each row's own highlight style, not through the `▸`
/// glyph, because a *collapsed* header legitimately reuses the same
/// code point for its own fold icon and would make a text-only check
/// ambiguous.
#[test]
fn a_click_reveals_the_marker_and_toggling_the_header_moves_selection_off_the_row() {
    let mut m = manager();
    let beta_row = m
        .rows()
        .iter()
        .position(|r| {
            matches!(r, ManagerRow::Repo { section: Section::Open, repo } if m.repos[*repo].name == "beta")
        })
        .unwrap();

    // A click reveals the marker on the pre-selected row, same as a key.
    click(&mut m, ManagerHit::Row(beta_row), false);
    let highlighted_row = |render: &ManagerRender, target: usize| {
        render.hits.iter().any(|(rect, hit)| {
            *hit == ManagerHit::Row(target)
                && render.lines[(rect.y) as usize]
                    .spans
                    .iter()
                    .any(|s| s.style.bg == Some(th().ui.selection))
        })
    };
    let render1 = render(&mut m, Rect::new(0, 0, 140, 30));
    assert!(
        highlighted_row(&render1, beta_row),
        "the clicked row should carry the selection highlight"
    );

    // Collapse then re-expand `OPEN REPOSITORIES`, the header
    // directly above `beta` -- both are the same click target twice.
    click(&mut m, ManagerHit::Header(Section::Open), false);
    click(&mut m, ManagerHit::Header(Section::Open), false);
    let beta_row_after = m
        .rows()
        .iter()
        .position(|r| {
            matches!(r, ManagerRow::Repo { section: Section::Open, repo } if m.repos[*repo].name == "beta")
        })
        .unwrap();
    let render2 = render(&mut m, Rect::new(0, 0, 140, 30));
    assert!(
        !highlighted_row(&render2, beta_row_after),
        "the repository row must no longer carry the marker/highlight \
         after its section header was toggled"
    );
    assert_eq!(
        m.selected_row(),
        Some(ManagerRow::Header(Section::Open)),
        "selection lands on the header itself, not back on the row"
    );
}
