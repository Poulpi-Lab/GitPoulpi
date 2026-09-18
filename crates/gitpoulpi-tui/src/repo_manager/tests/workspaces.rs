//! The Home tab's Workspaces page: its rows, its buttons (painted where they
//! are hit, reachable through the row band that covers them), its keys.

use super::*;

fn entry(id: &str, name: &str, repos: &[(&str, bool, Option<&str>)]) -> WsEntry {
    WsEntry {
        id: id.into(),
        name: name.into(),
        color: Some("green".into()),
        description: String::new(),
        profile: Some("Work".into()),
        repos: repos
            .iter()
            .map(|(n, present, remote)| WsRepo {
                path: PathBuf::from(format!("/work/{n}")),
                name: (*n).into(),
                present: *present,
                remote: remote.map(str::to_string),
            })
            .collect(),
    }
}

/// Acme (api present, web missing with a remote, docs missing without one)
/// and Home lab (empty); Acme is the active workspace.
fn page() -> RepoManager {
    let mut m = manager();
    m.ws.set_entries(
        vec![
            entry(
                "acme",
                "Acme",
                &[
                    ("api", true, Some("git@h:acme/api.git")),
                    ("web", false, Some("git@h:acme/web.git")),
                    ("docs", false, None),
                ],
            ),
            entry("lab", "Home lab", &[]),
        ],
        Some("acme".into()),
    );
    m.show_page(HomePage::Workspaces);
    m
}

fn screen(m: &mut RepoManager, width: u16) -> (Area, ManagerRender, Vec<String>) {
    let area = Rect::new(0, 1, width, 24);
    let render = render(m, area);
    let lines = render.lines.iter().map(text).collect();
    (area, render, lines)
}

type Area = Rect;

fn key(m: &mut RepoManager, c: char) -> ManagerAction {
    handle_key(m, Key::Char(c), false)
}

#[test]
fn the_page_lists_each_workspace_with_its_count_profile_and_marks_the_active_one() {
    let mut m = page();
    let (_, _, lines) = screen(&mut m, 140);
    let acme = lines.iter().find(|l| l.contains("Acme")).expect("Acme row");
    assert!(acme.contains("3 repositories"), "{acme}");
    assert!(acme.contains("2 missing"), "{acme}");
    assert!(acme.contains("profile Work"), "{acme}");
    assert!(acme.contains("active"), "{acme}");
    assert!(acme.contains(crate::workspace_ui::DOT), "{acme}");
    let lab = lines.iter().find(|l| l.contains("Home lab")).unwrap();
    assert!(
        lab.contains("0 repositories") || lab.contains("repositories"),
        "{lab}"
    );
    assert!(!lab.contains("active"), "{lab}");
    // The title row names the tab; there is no page switcher.
    assert!(lines[0].contains("Workspaces"), "{}", lines[0]);
    assert!(!lines[0].contains("Repositories"), "{}", lines[0]);
}

#[test]
fn expanding_shows_the_repositories_with_missing_ones_marked_and_cloneable_only_with_a_remote() {
    let mut m = page();
    assert_eq!(key(&mut m, ' '), ManagerAction::Handled);
    let (_, render, lines) = screen(&mut m, 160);
    let web = lines.iter().find(|l| l.contains("web")).expect("web row");
    assert!(web.contains("missing"), "{web}");
    assert!(web.contains("Locate...") && web.contains("Clone"), "{web}");
    let docs = lines.iter().find(|l| l.contains("docs")).expect("docs row");
    assert!(docs.contains("Locate..."), "{docs}");
    assert!(
        !docs.contains("Clone"),
        "no recorded remote, nothing to clone: {docs}"
    );
    let api = lines.iter().find(|l| l.contains("api")).unwrap();
    assert!(!api.contains("missing") && api.contains("Open"), "{api}");
    assert_eq!(
        render
            .hits
            .iter()
            .filter(|(_, h)| matches!(h, ManagerHit::Ws(WsHit::Clone(_))))
            .count(),
        1
    );
}

#[test]
fn every_button_is_painted_where_it_is_hit_and_reachable() {
    let mut m = page();
    m.ws.set_entries(m.ws.entries.clone(), Some("acme".into()));
    key(&mut m, '+');
    let (area, render, _) = screen(&mut m, 160);
    let mut seen = std::collections::HashSet::new();
    for (rect, hit) in &render.hits {
        let ManagerHit::Ws(ws) = hit else { continue };
        if matches!(ws, WsHit::Row(_)) {
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
        let expected = match ws {
            WsHit::New => "New workspace",
            WsHit::FromTabs => "From open tabs",
            WsHit::Open(_) => "Open",
            WsHit::Edit(_) => "Edit...",
            WsHit::Menu(_) => MORE,
            WsHit::Locate(_) => "Locate...",
            WsHit::Clone(_) => "Clone",
            WsHit::Remove(_) => "Remove",
            WsHit::Row(_) => unreachable!(),
        };
        assert!(painted.contains(expected), "{hit:?}: {painted:?}");
        seen.insert(std::mem::discriminant(ws));
    }
    assert_eq!(seen.len(), 8, "every control is on screen: {seen:?}");
    // Home has no page switcher any more: Workspaces is a tab of its own.
    let top = text(&render.lines[0]);
    assert!(!top.contains("Repositories"), "{top:?}");
    assert!(top.contains("Workspaces"), "the tab's own title: {top:?}");
}

#[test]
fn keys_open_edit_fetch_and_delete_the_workspace_under_the_cursor_and_x_removes_a_repository() {
    let mut m = page();
    assert_eq!(
        handle_key(&mut m, Key::Enter, false),
        ManagerAction::Workspace(WsAction::Open("acme".into()))
    );
    assert_eq!(
        key(&mut m, 'e'),
        ManagerAction::Workspace(WsAction::Edit("acme".into()))
    );
    assert_eq!(key(&mut m, 'n'), ManagerAction::Workspace(WsAction::New));
    assert_eq!(
        key(&mut m, 'a'),
        ManagerAction::Workspace(WsAction::FromTabs)
    );
    assert_eq!(
        key(&mut m, 'F'),
        ManagerAction::Workspace(WsAction::FetchAll("acme".into()))
    );
    assert_eq!(
        key(&mut m, 'P'),
        ManagerAction::Workspace(WsAction::PullAll("acme".into()))
    );
    assert_eq!(
        key(&mut m, 'm'),
        ManagerAction::Workspace(WsAction::CloneMissing("acme".into()))
    );
    // `x` on a workspace asks to delete it; on a repository it leaves the workspace.
    assert_eq!(
        key(&mut m, 'x'),
        ManagerAction::Workspace(WsAction::Delete("acme".into()))
    );
    key(&mut m, ' ');
    key(&mut m, 'j');
    key(&mut m, 'j');
    assert_eq!(
        key(&mut m, 'x'),
        ManagerAction::Workspace(WsAction::RemoveRepo {
            id: "acme".into(),
            path: PathBuf::from("/work/web"),
        })
    );
    // Enter on the missing repository locates it; `l` does the same.
    assert_eq!(
        handle_key(&mut m, Key::Enter, false),
        ManagerAction::Workspace(WsAction::Locate {
            id: "acme".into(),
            path: PathBuf::from("/work/web"),
        })
    );
    assert_eq!(key(&mut m, 'l'), handle_key(&mut m, Key::Enter, false));
    // The menu key opens the row's menu.
    assert_eq!(
        handle_key(&mut m, Key::Menu, false),
        ManagerAction::Workspace(WsAction::Menu {
            id: "acme".into(),
            repo: Some(PathBuf::from("/work/web")),
        })
    );
}

#[test]
fn the_number_keys_no_longer_switch_pages_and_the_search_belongs_to_the_repositories_page() {
    let mut m = manager();
    assert_eq!(m.page, HomePage::Repositories);
    key(&mut m, '2');
    assert_eq!(
        m.page,
        HomePage::Repositories,
        "Workspaces is a tab, not `2`"
    );
    m.show_page(HomePage::Workspaces);
    key(&mut m, '1');
    assert_eq!(m.page, HomePage::Workspaces, "`1` is not a page key either");
    // `/` is not the Workspaces page's: the palette's search goes back first.
    m.focus_search();
    assert_eq!(m.page, HomePage::Repositories);
    assert!(m.search_focused);
}

#[test]
fn clicks_fold_rows_and_buttons_return_their_actions() {
    let mut m = page();
    let (_, render, _) = screen(&mut m, 160);
    let row = render
        .hits
        .iter()
        .find_map(|(_, h)| matches!(h, ManagerHit::Ws(WsHit::Row(0))).then_some(*h))
        .unwrap();
    assert_eq!(click(&mut m, row, false), ManagerAction::Handled);
    assert_eq!(
        m.ws.rows().len(),
        5,
        "the click unfolded Acme: 2 workspaces + 3 repositories"
    );
    assert_eq!(
        click(&mut m, row, true),
        ManagerAction::Workspace(WsAction::Open("acme".into()))
    );
    assert_eq!(
        click(&mut m, ManagerHit::Ws(WsHit::New), false),
        ManagerAction::Workspace(WsAction::New)
    );
    assert_eq!(
        click(&mut m, ManagerHit::Ws(WsHit::Edit(0)), false),
        ManagerAction::Workspace(WsAction::Edit("acme".into()))
    );
    m.show_page(HomePage::Repositories);
    assert_eq!(m.page, HomePage::Repositories);
}

#[test]
fn a_narrow_window_still_draws_the_buttons_it_registers() {
    let mut m = page();
    let (area, render, _) = screen(&mut m, 60);
    for (rect, hit) in &render.hits {
        if matches!(hit, ManagerHit::Ws(WsHit::Row(_))) {
            continue;
        }
        let line = text(&render.lines[(rect.y - area.y) as usize]);
        let painted: String = line
            .chars()
            .skip((rect.x - area.x) as usize)
            .take(rect.width as usize)
            .collect();
        assert!(
            !painted.trim().is_empty(),
            "{hit:?} registered over blank cells at {rect:?}"
        );
    }
}
