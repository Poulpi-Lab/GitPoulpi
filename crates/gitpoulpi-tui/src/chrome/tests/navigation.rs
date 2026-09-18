use super::*;

/// Every fixed key `Ctrl+N` navigation mode understands, mapped to the
/// action the event loop is supposed to apply - `h`/`l`/`,`/`?`/`t`,
/// and any other alphanumeric key turning into `FocusByBadge` for the
/// event loop to resolve against the current frame's badge list.
/// Falsified by swapping any two arms in `navigation_key_action` --
/// this fails immediately.
#[test]
fn navigation_key_action_maps_every_prefix_key() {
    assert_eq!(
        navigation_key_action(KeyCode::Char('h')),
        NavigationAction::CollapseLeft
    );
    assert_eq!(
        navigation_key_action(KeyCode::Char('l')),
        NavigationAction::CollapseRight
    );
    assert_eq!(
        navigation_key_action(KeyCode::Char('1')),
        NavigationAction::FocusByBadge('1')
    );
    assert_eq!(
        navigation_key_action(KeyCode::Char('2')),
        NavigationAction::FocusByBadge('2')
    );
    assert_eq!(
        navigation_key_action(KeyCode::Char('b')),
        NavigationAction::FocusByBadge('b')
    );
    assert_eq!(
        navigation_key_action(KeyCode::Char(',')),
        NavigationAction::OpenSettings
    );
    assert_eq!(
        navigation_key_action(KeyCode::Char('?')),
        NavigationAction::ToggleHelp
    );
    assert_eq!(
        navigation_key_action(KeyCode::Char('t')),
        NavigationAction::SelectTab
    );
}

/// `Esc` and a non-alphanumeric unbound key both cancel - there is no
/// way to get the prefix stuck open on a typo.
#[test]
fn navigation_key_action_cancels_on_escape_or_an_unknown_key() {
    assert_eq!(
        navigation_key_action(KeyCode::Esc),
        NavigationAction::Cancel
    );
    assert_eq!(
        navigation_key_action(KeyCode::Char('!')),
        NavigationAction::Cancel
    );
}

/// A leading `1`-`9` starts a count; `0` alone does not. Falsified by
/// deleting the `c != '0'` half of the guard - this fails
/// immediately (`is_count_digit('0', None)` would come back `true`).
#[test]
fn is_count_digit_rejects_a_bare_leading_zero() {
    assert!(!is_count_digit('0', None));
    assert!(is_count_digit('1', None));
    assert!(is_count_digit('9', None));
    assert!(!is_count_digit('a', None));
}

/// Once a count is pending, `0` is an ordinary continuation digit
/// (`10`, `20`, ...) - only a *leading* zero is special.
#[test]
fn is_count_digit_accepts_zero_once_a_count_is_pending() {
    assert!(is_count_digit('0', Some(1)));
}

/// Digits fold left-to-right: `4` then `2` makes `42`, not `24`.
#[test]
fn accumulate_count_folds_digits_in_typed_order() {
    let after_4 = accumulate_count(None, 4);
    assert_eq!(after_4, Some(4));
    let after_42 = accumulate_count(after_4, 2);
    assert_eq!(after_42, Some(42));
}

/// A count is capped rather than allowed to overflow or grow
/// unboundedly - typing a dozen digits in a row must not panic or
/// produce a count large enough to make a selection jump do
/// something surprising.
#[test]
fn accumulate_count_is_capped() {
    let mut count = None;
    for _ in 0..10 {
        count = accumulate_count(count, 9);
    }
    assert_eq!(count, Some(MAX_PENDING_COUNT));
}

/// The all-clear case: nothing is intercepting the key, so a digit is
/// free to start/continue a count.
#[test]
fn count_prefix_may_apply_with_every_guard_clear() {
    assert!(count_prefix_may_apply(
        true, false, false, false, false, false
    ));
}

/// Regression test: without the flag, a digit
/// typed into any Preferences numeric field (e.g. "Mouse wheel
/// lines") was swallowed here as a vim count and never reached
/// `PreferencesState::on_key` - the field showed an edit cursor but
/// typing wrote nothing; only Backspace, not being a digit, ever got
/// through. Falsified by dropping the `!preferences_open` term from
/// `count_prefix_may_apply` - this fails immediately.
#[test]
fn count_prefix_may_not_apply_while_preferences_is_open() {
    assert!(!count_prefix_may_apply(
        true, false, false, false, true, false
    ));
}

/// Every other surface a digit already means something on: outside
/// `InputMode::Normal`, a log or ref context menu open (its own row
/// shortcuts), or the commit composer's text field focused.
#[test]
fn count_prefix_may_not_apply_on_any_other_surface_either() {
    assert!(!count_prefix_may_apply(
        false, false, false, false, false, false
    ));
    assert!(!count_prefix_may_apply(
        true, true, false, false, false, false
    ));
    assert!(!count_prefix_may_apply(
        true, false, true, false, false, false
    ));
    assert!(!count_prefix_may_apply(
        true, false, false, true, false, false
    ));
    // A dialog's path or URL field: digits are text there.
    assert!(!count_prefix_may_apply(
        true, false, false, false, false, true
    ));
}

/// Badges are `1`..`9` and then letters - the exact boundary the
/// badges promise. Falsified by changing `< 9` to `< 10` in
/// `navigation_badge_char` - this fails immediately (index 8 would
/// come back `'a'` instead of `'9'`).
#[test]
fn navigation_badge_char_is_digits_then_letters() {
    assert_eq!(navigation_badge_char(0), '1');
    assert_eq!(navigation_badge_char(8), '9');
    // `a` is the About key, so the letters start at `b`.
    assert_eq!(navigation_badge_char(9), 'b');
    assert_eq!(navigation_badge_char(10), 'c');
}

/// `h`/`l`/`q`/`t` are reserved and must never come back as a badge,
/// at any index - otherwise `Ctrl+N h` would be ambiguous between
/// "collapse the Refs pane" and "focus whatever got badge h", and
/// likewise `Ctrl+N t` between "open the tab picker" and "focus
/// whatever got badge t".
#[test]
fn navigation_badge_char_never_hands_out_a_reserved_letter() {
    for i in 9..80 {
        let c = navigation_badge_char(i);
        assert!(!"ahlqt".contains(c), "index {i} handed out reserved '{c}'");
    }
}

/// `assign_navigation_badges` assigns in the caller's order and is
/// stable - the same input always gets the same badges, which is
/// what lets the render pass and the key-lookup pass share one list
/// without recomputing it twice and risking drift.
#[test]
fn assign_navigation_badges_is_stable_and_positional() {
    let targets = [
        FocusTarget::RefsFilter,
        FocusTarget::RefsSection(SidebarSection::Local),
        FocusTarget::Log,
    ];
    let assigned = assign_navigation_badges(&targets);
    assert_eq!(
        assigned,
        vec![
            ('1', FocusTarget::RefsFilter),
            ('2', FocusTarget::RefsSection(SidebarSection::Local)),
            ('3', FocusTarget::Log),
        ]
    );
}

/// More than nine focusable sub-panes in one frame - a plausible
/// count once every Refs section, Log, both Status halves and the
/// filter field are all counted - rolls over into letters rather
/// than running out of badges.
#[test]
fn assign_navigation_badges_rolls_over_into_letters_past_nine() {
    let targets: Vec<FocusTarget> = (0..11).map(|_| FocusTarget::Log).collect();
    let assigned = assign_navigation_badges(&targets);
    assert_eq!(assigned[8].0, '9');
    assert_eq!(assigned[9].0, 'b');
    assert_eq!(assigned[10].0, 'c');
}
