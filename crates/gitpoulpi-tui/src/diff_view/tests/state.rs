use super::*;

// -- Remembering the last-used display toggles -----------------------

#[test]
fn diff_view_mode_and_diff_layout_convert_both_ways_without_losing_the_variant() {
    for mode in [DiffViewMode::Inline, DiffViewMode::Split] {
        let layout: gitpoulpi_config::DiffLayout = mode.into();
        let back: DiffViewMode = layout.into();
        assert_eq!(mode, back, "round trip must be lossless for {mode:?}");
    }
}

#[test]
fn toggle_mode_mutates_state_resets_position_and_reports_the_config_key() {
    let mut state = DiffViewState::for_commit(test_oid(), 0, PathBuf::from("f.rs"));
    state.scroll = 7;
    state.cursor = 3;
    state.selection_anchor = Some(1);
    let (key, value) = DiffViewToggle::Mode(DiffViewMode::Split).apply(&mut state);
    assert_eq!(state.mode, DiffViewMode::Split);
    assert_eq!(key, "ui.diff_layout");
    assert_eq!(value, "split");
    // Falsified: without `reset_position` in `apply`, this failed --
    // scroll/cursor/selection_anchor kept the stale-layout numbering.
    assert_eq!(state.scroll, 0);
    assert_eq!(state.cursor, 0);
    assert_eq!(state.selection_anchor, None);
}

#[test]
fn toggle_word_wrap_and_whitespace_flip_the_bool_and_report_their_key() {
    let mut state = DiffViewState::for_commit(test_oid(), 0, PathBuf::from("f.rs"));
    assert!(!state.word_wrap);
    let (key, value) = DiffViewToggle::ToggleWordWrap.apply(&mut state);
    assert!(state.word_wrap);
    assert_eq!((key, value.as_str()), ("ui.diff_word_wrap", "true"));

    assert!(state.show_whitespace);
    let (key, value) = DiffViewToggle::ToggleWhitespace.apply(&mut state);
    assert!(!state.show_whitespace);
    assert_eq!((key, value.as_str()), ("ui.diff_show_whitespace", "false"));

    assert!(state.hunks_only);
    state.cursor = 4;
    let (key, value) = DiffViewToggle::ToggleHunksOnly.apply(&mut state);
    assert!(!state.hunks_only);
    assert_eq!((key, value.as_str()), ("ui.diff_hunks_only", "false"));
    assert_eq!(state.cursor, 0, "a relayout resets the position");
}

#[test]
fn resume_last_used_overwrites_the_constructors_own_defaults() {
    // See `live_settings::DIFF_VIEW_PREFS_TEST_LOCK`'s doc comment --
    // this test and `lib.rs`'s own `DIFF_VIEW_PREFS` tests share one
    // process-wide static and race without it.
    let _guard = crate::live_settings::DIFF_VIEW_PREFS_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    crate::live_settings::install_diff_view_prefs(crate::live_settings::DiffViewPrefs {
        layout: gitpoulpi_config::DiffLayout::Split,
        hunks_only: false,
        show_whitespace: false,
        word_wrap: true,
    });
    let state = DiffViewState::for_commit(test_oid(), 0, PathBuf::from("f.rs")).resume_last_used();
    assert_eq!(state.mode, DiffViewMode::Split);
    assert!(!state.hunks_only);
    assert!(!state.show_whitespace);
    assert!(state.word_wrap);
    // Leave the process-wide static as every other test in this
    // binary expects to find it -- see `DiffViewPrefs::const_default`.
    crate::live_settings::install_diff_view_prefs(
        crate::live_settings::DiffViewPrefs::const_default(),
    );
}
