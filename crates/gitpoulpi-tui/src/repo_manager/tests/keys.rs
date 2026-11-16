use super::*;

/// The control chords belong to the window: the palette must stay
/// reachable from this view.
#[test]
fn control_chords_other_than_find_are_passed_through() {
    let mut m = manager();
    assert_eq!(
        handle_key(&mut m, Key::Char('p'), true),
        ManagerAction::NotHandled
    );
    assert_eq!(
        handle_key(&mut m, Key::Char('f'), true),
        ManagerAction::Handled
    );
    assert!(m.search_focused);
}

/// `.` opens the selected row's menu, as do its two standard aliases. Falsified by reverting the `Key::Char('.') |
/// Key::Menu` arm to `Key::Char('.')` alone: the `Key::Menu` cases
/// below then fall through to the default arm and return
/// `ManagerAction::Handled` instead of `Menu { .. }`.
#[test]
fn dot_and_its_aliases_open_the_selected_rows_menu() {
    let mut m = manager();
    for key in [Key::Char('.'), Key::Menu] {
        m.selected = m
            .rows()
            .iter()
            .position(
                |r| matches!(r, ManagerRow::Repo { repo, .. } if m.repos[*repo].name == "alpha"),
            )
            .unwrap();
        assert_eq!(
            handle_key(&mut m, key, false),
            ManagerAction::Menu {
                repo: m.repos.iter().position(|r| r.name == "alpha").unwrap()
            },
            "{key:?} must open the row's menu exactly like `.` does"
        );
    }
}

/// `from_crossterm` is the only place `Shift+F10` and the Menu/Apps
/// key become `Key::Menu` - see `context_menu::is_menu_key`'s doc
/// comment for why `KeyCode::Menu` itself is close to unreachable on
/// Windows, and why `Shift+F10` is the one that actually works there.
/// A plain, unmodified `F10` must NOT decode to `Key::Menu` - it is
/// unbound today, and silently swallowing it as a menu key would be a
/// keybinding regression nobody asked for.
#[test]
fn from_crossterm_maps_shift_f10_and_the_menu_key_to_key_menu_plain_f10_falls_through() {
    assert_eq!(
        Key::from_crossterm(KeyCode::F(10), KeyModifiers::SHIFT),
        Key::Menu
    );
    assert_eq!(
        Key::from_crossterm(KeyCode::Menu, KeyModifiers::NONE),
        Key::Menu
    );
    assert_eq!(
        Key::from_crossterm(KeyCode::F(10), KeyModifiers::NONE),
        Key::Other
    );
    // `.` keeps decoding to `Key::Char('.')`, not `Key::Menu` --
    // `handle_key`'s `Key::Char('.') | Key::Menu` arm only works
    // because both spellings reach it, and a caller matching on
    // `Key::Char('.')` alone elsewhere must still see it.
    assert_eq!(
        Key::from_crossterm(KeyCode::Char('.'), KeyModifiers::NONE),
        Key::Char('.')
    );
}

/// `y` and `o` ask for the selected repository's path to be copied or
/// opened; a folder that is gone refuses to open and says why.
#[test]
fn y_copies_and_o_opens_the_selected_path() {
    let mut m = manager();
    m.selected = m
        .rows()
        .iter()
        .position(|r| matches!(r, ManagerRow::Repo { repo, .. } if m.repos[*repo].name == "alpha"))
        .unwrap();
    assert_eq!(
        handle_key(&mut m, Key::Char('y'), false),
        ManagerAction::CopyPath(PathBuf::from("/work/alpha"))
    );
    assert_eq!(
        handle_key(&mut m, Key::Char('o'), false),
        ManagerAction::OpenIn(PathBuf::from("/work/alpha"))
    );
    m.selected = m
        .rows()
        .iter()
        .position(|r| matches!(r, ManagerRow::Repo { repo, .. } if m.repos[*repo].name == "gamma"))
        .unwrap();
    assert_eq!(
        handle_key(&mut m, Key::Char('o'), false),
        ManagerAction::Handled
    );
    assert!(m
        .message
        .as_deref()
        .is_some_and(|s| s.contains("/work/gamma")));
}
