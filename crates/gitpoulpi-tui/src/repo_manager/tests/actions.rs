use super::*;

/// Enter in the search opens the best match: type, Enter, done.
#[test]
fn enter_in_the_search_opens_the_first_match() {
    let mut m = manager();
    handle_key(&mut m, Key::Char('/'), false);
    for c in "delta".chars() {
        handle_key(&mut m, Key::Char(c), false);
    }
    assert_eq!(
        handle_key(&mut m, Key::Enter, false),
        ManagerAction::Open(PathBuf::from("/work/delta"))
    );
    assert!(!m.search_focused);
}

/// Opening a repository that already has a tab focuses that tab
/// rather than opening it twice; opening one that is gone explains.
#[test]
fn open_focuses_an_existing_tab_opens_a_new_one_and_refuses_a_gone_one() {
    let mut m = manager();
    let rows = m.rows();
    let find = |name: &str, section: Section| {
        rows.iter()
            .position(|r| matches!(r, ManagerRow::Repo { section: s, repo } if *s == section && m.repos[*repo].name == name))
            .unwrap()
    };
    let (alpha, delta, gamma) = (
        find("alpha", Section::Open),
        find("delta", Section::Favorites),
        find("gamma", Section::Recent),
    );
    m.selected = alpha;
    assert_eq!(
        handle_key(&mut m, Key::Enter, false),
        ManagerAction::FocusTab(1)
    );
    m.selected = delta;
    assert_eq!(
        handle_key(&mut m, Key::Enter, false),
        ManagerAction::Open(PathBuf::from("/work/delta"))
    );
    m.selected = gamma;
    assert_eq!(
        handle_key(&mut m, Key::Enter, false),
        ManagerAction::Handled
    );
    assert!(m.message.as_deref().unwrap().contains("gone from disk"));
}

/// `x` means close in Open, remove in Recent, un-star in Favorites.
#[test]
fn x_closes_removes_or_unstars_depending_on_the_section() {
    let mut m = manager();
    let rows = m.rows();
    let at = |section: Section| {
        rows.iter()
            .position(|r| matches!(r, ManagerRow::Repo { section: s, .. } if *s == section))
            .unwrap()
    };
    m.selected = at(Section::Open);
    assert_eq!(
        handle_key(&mut m, Key::Char('x'), false),
        ManagerAction::CloseTab(0)
    );
    m.selected = at(Section::Recent);
    assert_eq!(
        handle_key(&mut m, Key::Delete, false),
        ManagerAction::RemoveRecent(PathBuf::from("/work/alpha"))
    );
    m.selected = at(Section::Favorites);
    assert_eq!(
        handle_key(&mut m, Key::Char('x'), false),
        ManagerAction::ToggleFavorite(PathBuf::from("/work/delta"))
    );
    m.selected = at(Section::All);
    let all_repo = m.selected_repo().unwrap().path.clone();
    assert_eq!(
        handle_key(&mut m, Key::Char('x'), false),
        ManagerAction::RemoveFromList(all_repo)
    );
}

#[test]
fn f_toggles_the_selected_repositorys_favorite() {
    let mut m = manager();
    m.selected = m
        .rows()
        .iter()
        .position(|r| {
            matches!(
                r,
                ManagerRow::Repo {
                    section: Section::Open,
                    ..
                }
            )
        })
        .unwrap();
    assert_eq!(
        handle_key(&mut m, Key::Char('f'), false),
        ManagerAction::ToggleFavorite(PathBuf::from("/work/beta"))
    );
}
