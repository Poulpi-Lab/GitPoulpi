use super::*;
use crossterm::event::KeyModifiers;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn lock() -> std::sync::MutexGuard<'static, ()> {
    super::super::TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn draw(terminal: &mut Terminal<TestBackend>) -> String {
    terminal.draw(render).unwrap();
    dialog_kit::buffer_text(terminal.backend().buffer())
}

fn key(code: KeyCode) {
    on_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn type_text(text: &str) {
    for c in text.chars() {
        key(KeyCode::Char(c));
    }
}

fn mouse(kind: MouseEventKind, col: u16, row: u16) {
    on_mouse(MouseEvent {
        kind,
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    });
}

fn click(col: u16, row: u16) {
    mouse(MouseEventKind::Down(MouseButton::Left), col, row);
    mouse(MouseEventKind::Up(MouseButton::Left), col, row);
}

fn hit_rect(hit: Hit) -> Rect {
    with(|s| s.hits.iter().find(|(_, h)| *h == hit).map(|(r, _)| *r)).expect("hit painted")
}

fn error(title: &str) -> feed::Entry {
    let mut e = feed::Entry::new(Severity::Error, title.into(), "boom".into());
    e.detail = Some(gitpoulpi_core::ErrorDetail::from_git(
        &["push", "origin"],
        Some(1),
        "! [rejected] main -> main (non-fast-forward)\n",
    ));
    e
}

fn terminal() -> Terminal<TestBackend> {
    close();
    with(|s| {
        s.expanded.clear();
        s.filter.clear();
        // A scope test before this one may leave its choice behind.
        s.remembered = [None, None];
        s.ws_cache.clear();
        s.home = false;
    });
    open();
    Terminal::new(TestBackend::new(120, 36)).unwrap()
}

/// The process-wide log, started once for every test that needs one.
fn logger() -> std::sync::Arc<gitpoulpi_log::Logger> {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    let dir = DIR.get_or_init(|| tempfile::tempdir().unwrap());
    gitpoulpi_log::init(gitpoulpi_log::Options {
        dir: Some(dir.path().to_path_buf()),
        level: Level::Info,
        retention_days: 1,
        max_file_bytes: 1024 * 1024,
        process: "test",
    })
}

fn wait_for(terminal: &mut Terminal<TestBackend>, needle: &str) -> String {
    for _ in 0..200 {
        let screen = draw(terminal);
        if screen.contains(needle) {
            return screen;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    draw(terminal)
}

#[test]
fn the_activity_tab_lists_entries_newest_first_and_expands_them() {
    let _lock = lock();
    feed::push(error("Pull failed"));
    feed::push(error("Push failed"));
    let mut terminal = terminal();
    let screen = draw(&mut terminal);
    let push = screen.find("Push failed").expect("newest listed");
    let pull = screen.find("Pull failed").expect("older listed");
    assert!(push < pull, "newest first\n{screen}");
    assert!(!screen.contains("exit code"), "collapsed");

    key(KeyCode::Enter);
    let screen = draw(&mut terminal);
    assert!(screen.contains("command: git push origin"), "{screen}");
    assert!(screen.contains("exit code: 1"), "{screen}");
    assert!(screen.contains("non-fast-forward"), "{screen}");

    // Down selects the older one; Enter opens it too; Enter again closes.
    key(KeyCode::Down);
    key(KeyCode::Enter);
    assert_eq!(with(|s| s.expanded.len()), 2);
    key(KeyCode::Enter);
    assert_eq!(with(|s| s.expanded.len()), 1);
    close();
}

#[test]
fn copying_an_entry_reports_what_was_copied() {
    let _lock = lock();
    feed::push(error("Push failed"));
    let mut terminal = terminal();
    draw(&mut terminal);
    key(KeyCode::Enter);
    draw(&mut terminal);
    // The Copy button under the opened entry, by mouse.
    let copy = hit_rect(Hit::CopyEntry(0));
    click(copy.x + 1, copy.y);
    assert!(
        with(|s| s.notice.clone())
            .unwrap_or_default()
            .contains("error details"),
        "{:?}",
        with(|s| s.notice.clone())
    );
    // And by key.
    with(|s| s.notice = None);
    key(KeyCode::Char('c'));
    assert!(with(|s| s.notice.is_some()));
    close();
}

#[test]
fn opening_the_log_marks_errors_read_and_the_button_counts_unread_ones() {
    let _lock = lock();
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 1));
    feed::push(error("Push failed"));
    feed::push(error("Pull failed"));
    paint_status_button(&mut buf, Rect::new(0, 0, 100, 1));
    let row = dialog_kit::buffer_text(&buf);
    assert!(row.contains("Activity (2)"), "{row}");
    assert!(status_button_hit(99, 0), "the button is the click target");
    assert!(!status_button_hit(0, 0));
    open();
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 1));
    paint_status_button(&mut buf, Rect::new(0, 0, 100, 1));
    let row = dialog_kit::buffer_text(&buf);
    assert!(row.contains("Activity") && !row.contains("(2)"), "{row}");
    close();
}

#[test]
fn escape_closes_and_a_click_outside_closes() {
    let _lock = lock();
    let mut terminal = terminal();
    draw(&mut terminal);
    key(KeyCode::Esc);
    assert!(!is_open());
    open();
    draw(&mut terminal);
    click(0, 0);
    assert!(!is_open(), "a click outside the window closes it");
    open();
    draw(&mut terminal);
    let close_button = with(|s| s.close);
    click(close_button.x + 1, close_button.y);
    assert!(!is_open(), "the close control closes it");
}

#[test]
fn tabs_switch_by_key_and_by_click() {
    let _lock = lock();
    logger();
    let mut terminal = terminal();
    let screen = draw(&mut terminal);
    let (git, application) = (screen.find("Git operations"), screen.find("Application"));
    assert!(
        git.is_some() && application > git,
        "Git operations comes first, then Application\n{screen}"
    );
    assert!(!screen.contains(" Global ") && !screen.contains(" Repository "));
    assert_eq!(with(|s| s.tab), Tab::Git, "it opens on Git operations");
    key(KeyCode::Tab);
    assert_eq!(with(|s| s.tab), Tab::Application);
    key(KeyCode::Tab);
    assert_eq!(with(|s| s.tab), Tab::Git);
    // The letters are twins of the tabs.
    key(KeyCode::Char('p'));
    assert_eq!(with(|s| s.tab), Tab::Application);
    key(KeyCode::Char('g'));
    assert_eq!(with(|s| s.tab), Tab::Git);
    draw(&mut terminal);
    let tab = hit_rect(Hit::Tab(Tab::Application));
    click(tab.x + 1, tab.y);
    assert_eq!(with(|s| s.tab), Tab::Application);
    close();
}

#[test]
fn the_application_log_follows_the_level_box_and_the_filter() {
    let _lock = lock();
    let log = logger();
    log.set_level(Level::Info);
    log.emit(Level::Error, "daemon", "marker-error-line");
    log.emit(Level::Warn, "tui", "marker-warn-line");
    log.emit(Level::Info, "tui", "marker-info-line");
    log.flush();
    let mut terminal = terminal();
    key(KeyCode::Tab);
    // Other tests log to the same process-wide log while this one runs: keep
    // to this test's own lines.
    key(KeyCode::Char('/'));
    type_text("marker-");
    key(KeyCode::Esc);
    let screen = wait_for(&mut terminal, "marker-info-line");
    for line in ["marker-error-line", "marker-warn-line", "marker-info-line"] {
        assert!(screen.contains(line), "{line}\n{screen}");
    }
    assert!(
        screen.contains("ERROR") && screen.contains("daemon"),
        "{screen}"
    );

    // The level box: `l` opens the list, Up/Up to Error... choose by the list.
    key(KeyCode::Char('l'));
    draw(&mut terminal);
    key(KeyCode::Home);
    key(KeyCode::Enter);
    let screen = draw(&mut terminal);
    assert!(screen.contains("marker-error-line"), "{screen}");
    assert!(!screen.contains("marker-warn-line"), "{screen}");
    assert!(!screen.contains("marker-info-line"), "{screen}");

    // The filter: `/` then text, on top of the level.
    key(KeyCode::Char('l'));
    draw(&mut terminal);
    key(KeyCode::Down);
    key(KeyCode::Down);
    key(KeyCode::Enter);
    key(KeyCode::Char('/'));
    type_text("warn-line");
    let screen = draw(&mut terminal);
    assert!(screen.contains("marker-warn-line"), "{screen}");
    assert!(!screen.contains("marker-error-line"), "{screen}");
    key(KeyCode::Esc);
    close();
}

#[test]
fn asking_for_more_detail_than_is_recorded_raises_the_recording_level() {
    let _lock = lock();
    let log = logger();
    log.set_level(Level::Info);
    // `choose_level` also tries to save `log.level`; keep it off the real
    // config by pointing the ambient config at a scratch folder.
    let scratch = tempfile::tempdir().unwrap();
    std::env::set_var(gitpoulpi_config::CONFIG_DIR_ENV, scratch.path());
    choose_level(Level::Debug);
    std::env::remove_var(gitpoulpi_config::CONFIG_DIR_ENV);
    assert_eq!(log.level(), Level::Debug);
    assert!(with(|s| s.notice.clone())
        .unwrap_or_default()
        .contains("Debug"));
    log.set_level(Level::Info);
    close();
}

#[test]
fn a_long_list_has_a_scrollbar_that_the_track_press_and_the_wheel_move() {
    let _lock = lock();
    for i in 0..60 {
        feed::push(error(&format!("Failure number {i}")));
    }
    let mut terminal = terminal();
    let screen = draw(&mut terminal);
    assert!(
        screen.contains("Failure number 59"),
        "newest on top\n{screen}"
    );
    assert!(
        !screen.contains("Failure number 0\n"),
        "oldest is far below"
    );
    let (body, list) = with(|s| (s.body, s.list));
    assert!(list.height > 5);

    // The wheel.
    mouse(MouseEventKind::ScrollDown, list.x + 3, list.y + 1);
    assert_eq!(with(|s| s.offset[0]), 3);
    mouse(MouseEventKind::ScrollUp, list.x + 3, list.y + 1);
    assert_eq!(with(|s| s.offset[0]), 0);

    // A press on the bare track brings the thumb under the pointer.
    let track_x = body.right() - 1;
    let low = list.bottom() - 3;
    mouse(MouseEventKind::Down(MouseButton::Left), track_x, low);
    let jumped = with(|s| s.offset[0]);
    assert!(jumped > 20, "the track press jumped to {jumped}");
    // Dragging off the track keeps following; releasing stops.
    mouse(MouseEventKind::Drag(MouseButton::Left), track_x, list.y);
    assert!(with(|s| s.offset[0]) < jumped, "dragging up moves back");
    mouse(MouseEventKind::Up(MouseButton::Left), track_x, list.y);
    assert!(with(|s| s.grab.is_none()));
    close();
}

#[test]
fn copy_diagnostics_has_the_environment_the_latest_problem_and_the_log_tail() {
    let _lock = lock();
    let log = logger();
    log.emit(
        Level::Warn,
        "tui",
        "marker-diagnostic-line token=abcdef123456",
    );
    feed::push(error("Push failed"));
    let text = diagnostics_text();
    assert!(text.starts_with("GitPoulpi diagnostics"), "{text}");
    assert!(text.contains("latest problem:"), "{text}");
    assert!(text.contains("title: Push failed"), "{text}");
    assert!(text.contains("command: git push origin"), "{text}");
    assert!(text.contains("marker-diagnostic-line"), "{text}");
    assert!(!text.contains("abcdef123456"), "secrets stay out\n{text}");
    close();
}

fn long_line_marker() -> String {
    format!("start-marker {} end-marker", "x".repeat(300))
}

#[test]
fn a_long_application_line_scrolls_sideways_by_keys_wheel_and_bar() {
    let _lock = lock();
    let log = logger();
    log.emit(Level::Info, "tui", &long_line_marker());
    log.flush();
    let mut terminal = terminal();
    // The log is process-wide and other tests write to it at the same
    // time: narrow the list to this test's line so their lines cannot
    // move it or change the widest line.
    with(|s| s.filter = "start-marker".into());
    key(KeyCode::Tab);
    let screen = wait_for(&mut terminal, "start-marker");
    assert!(!screen.contains("end-marker"), "cut at the edge\n{screen}");
    let list = with(|s| s.list);
    assert_eq!(with(|s| s.hoffset), 0);

    // Right/Left and Home.
    key(KeyCode::Right);
    assert_eq!(with(|s| s.hoffset), 8);
    key(KeyCode::Left);
    assert_eq!(with(|s| s.hoffset), 0);
    key(KeyCode::Right);
    key(KeyCode::Right);
    key(KeyCode::Home);
    assert_eq!(with(|s| s.hoffset), 0, "Home resets");

    // Shift+wheel.
    let shifted = |kind| {
        on_mouse(MouseEvent {
            kind,
            column: list.x + 3,
            row: list.y,
            modifiers: KeyModifiers::SHIFT,
        })
    };
    shifted(MouseEventKind::ScrollDown);
    assert_eq!(with(|s| s.hoffset), 9);
    shifted(MouseEventKind::ScrollUp);
    assert_eq!(with(|s| s.hoffset), 0);

    // The bar: a press on the bare track brings the thumb under the pointer,
    // dragging follows, and the far end shows the end of the line.
    // The view keeps only the newest records of a log every test of the
    // process writes to: pin it to this line, then read the bar where this
    // frame put it.
    show_only(vec![gitpoulpi_log::Record {
        ts_ms: 0,
        level: Level::Info,
        process: "test".into(),
        pid: 1,
        target: "tui".into(),
        message: long_line_marker(),
        repo: None,
    }]);
    let pinned = wait_for(&mut terminal, "start-marker");
    let hbody = with(|s| s.hbody);
    let bar_y = hbody.bottom() - 1;
    let right_end = hbody.right() - 3;
    mouse(MouseEventKind::Down(MouseButton::Left), right_end, bar_y);
    let jumped = with(|s| s.hoffset);
    assert!(jumped > 100, "the track press jumped to {jumped}");
    let screen = draw(&mut terminal);
    assert!(screen.contains("end-marker"), "{screen}");
    mouse(MouseEventKind::Drag(MouseButton::Left), hbody.x + 1, bar_y);
    assert!(with(|s| s.hoffset) < jumped, "dragging left moves back");
    mouse(MouseEventKind::Up(MouseButton::Left), hbody.x + 1, bar_y);
    assert!(with(|s| s.hgrab.is_none()));
    unfreeze();
    assert!(pinned.contains("start-marker"), "{pinned}");
    close();
}

#[test]
fn the_git_operations_tab_lists_the_active_repositorys_operations_with_their_duration() {
    let _lock = lock();
    crate::activity_log::timing::install(&gitpoulpi_config::LogConfig::default());
    let here = std::path::Path::new("/tmp/panel-repo-here");
    let other = std::path::Path::new("/tmp/panel-repo-other");
    set_active_repo(here);
    feed::push(feed::Entry::operation(here, "Push".into(), 123, false));
    feed::push(feed::Entry::operation(other, "Fetch".into(), 77, false));
    let mut terminal = terminal();
    let screen = draw(&mut terminal);
    assert!(screen.contains("Push - 123 ms"), "{screen}");
    assert!(!screen.contains("77 ms"), "other repository\n{screen}");

    // Timings off hides the figure.
    crate::activity_log::timing::install(&gitpoulpi_config::LogConfig {
        timings: false,
        ..Default::default()
    });
    let screen = draw(&mut terminal);
    assert!(!screen.contains("123 ms"), "{screen}");
    crate::activity_log::timing::install(&gitpoulpi_config::LogConfig::default());
    close();
}

fn record(level: Level, message: &str) -> Record {
    Record {
        ts_ms: 0,
        level,
        process: "tui".into(),
        pid: 1,
        target: "t".into(),
        message: message.into(),
        repo: None,
    }
}

#[test]
fn the_display_toggles_hide_levels_and_the_filter_still_applies() {
    let _lock = lock();
    close();
    let records = vec![
        record(Level::Error, "e boom"),
        record(Level::Warn, "w careful"),
        record(Level::Info, "i hello"),
        record(Level::Debug, "d detail"),
    ];
    APP.lock().unwrap().records = records;
    let everything = Scoping {
        scope: Scope::All,
        active: None,
        cache: Vec::new(),
    };
    let texts = |level, shown: [bool; 5], filter: &str| -> Vec<String> {
        visible_records(&everything, level, &shown, filter)
            .into_iter()
            .map(|r| r.message)
            .collect()
    };
    let all = [true; 5];
    // Newest first; the capture level still decides what exists.
    assert_eq!(
        texts(Level::Info, all, ""),
        ["i hello", "w careful", "e boom"]
    );
    // Unticking Info leaves the warning and the error.
    let mut no_info = all;
    no_info[Level::Info as usize] = false;
    assert_eq!(texts(Level::Info, no_info, ""), ["w careful", "e boom"]);
    // A ticked level above the capture level shows nothing.
    assert!(!texts(Level::Info, all, "").contains(&"d detail".to_string()));
    // None shows nothing; the text filter narrows what the ticks keep.
    assert!(texts(Level::Trace, [false; 5], "").is_empty());
    assert_eq!(texts(Level::Trace, all, "careful"), ["w careful"]);
    assert!(texts(Level::Trace, no_info, "hello").is_empty());
    // The counts follow the filter, not the ticks.
    let counts = |level, filter: &str| level_counts(&everything, level, filter);
    assert_eq!(counts(Level::Trace, "")[Level::Info as usize], 1);
    assert_eq!(counts(Level::Info, "")[Level::Debug as usize], 0);
    assert_eq!(counts(Level::Trace, "boom")[Level::Warn as usize], 0);
    APP.lock().unwrap().records.clear();
}

#[test]
fn the_level_toggles_answer_to_keys_clicks_and_the_palette_commands() {
    let _lock = lock();
    let mut terminal = terminal();
    with(|s| {
        s.tab = Tab::Application;
        s.view_level = Level::Info;
        s.shown = [true; 5];
        s.notice = None;
    });
    let screen = draw(&mut terminal);
    assert!(screen.contains("[x] Info"), "{screen}");
    assert!(
        screen.contains("[ ] Debug"),
        "grayed above the capture level\n{screen}"
    );

    // Keys: 3 is Info; n none; a all; 4 (Debug) is above the capture level.
    key(KeyCode::Char('3'));
    assert_eq!(with(|s| s.shown), [true, true, false, true, true]);
    key(KeyCode::Char('n'));
    assert_eq!(with(|s| s.shown), [false; 5]);
    key(KeyCode::Char('a'));
    assert_eq!(with(|s| s.shown), [true; 5]);
    key(KeyCode::Char('4'));
    assert_eq!(with(|s| s.shown), [true; 5], "a grayed box does not toggle");
    assert!(with(|s| s.notice.clone()).is_some_and(|n| n.contains("Info")));

    // Clicks on the painted boxes and buttons.
    draw(&mut terminal);
    let info = hit_rect(Hit::Toggle(Level::Info));
    click(info.x + 1, info.y);
    assert!(!with(|s| s.shown[Level::Info as usize]));
    let screen = draw(&mut terminal);
    assert!(screen.contains("[ ] Info"), "{screen}");
    let none = hit_rect(Hit::ShowNone);
    click(none.x + 1, none.y);
    assert_eq!(with(|s| s.shown), [false; 5]);
    let all = hit_rect(Hit::ShowAll);
    click(all.x + 1, all.y);
    assert_eq!(with(|s| s.shown), [true; 5]);

    // The palette's commands open the window on the Application tab.
    close();
    level_cmd_open(LevelCmd::Toggle(Level::Warn));
    assert!(is_open());
    assert_eq!(
        with(|s| (s.tab, s.shown[Level::Warn as usize])),
        (Tab::Application, false)
    );
    level_cmd(LevelCmd::All);
    assert_eq!(with(|s| s.shown), [true; 5]);
    close();
}

// ---- clearing the application log ----

fn app_lines(log: &gitpoulpi_log::Logger) -> usize {
    log.flush();
    gitpoulpi_log::read_records(log.dir().unwrap(), 10_000)
        .iter()
        .filter(|r| r.message.contains("clear-marker"))
        .count()
}

fn seed_for_clear(terminal: &mut Terminal<TestBackend>) -> std::sync::Arc<gitpoulpi_log::Logger> {
    let log = logger();
    log.set_level(Level::Info);
    log.emit(Level::Info, "tui", "clear-marker-one");
    log.emit(Level::Warn, "tui", "clear-marker-two");
    log.flush();
    key(KeyCode::Tab);
    key(KeyCode::Char('/'));
    type_text("clear-marker");
    key(KeyCode::Esc);
    let screen = wait_for(terminal, "clear-marker-two");
    assert!(screen.contains("clear-marker-one"), "{screen}");
    log
}

#[test]
fn the_clear_button_asks_first_cancel_keeps_the_lines_and_confirm_clears_them() {
    let _lock = lock();
    let mut terminal = terminal();
    let log = seed_for_clear(&mut terminal);
    assert!(app_lines(&log) >= 2);

    // By mouse: the button opens the dialog, which says what goes.
    let button = hit_rect(Hit::ClearLog);
    click(button.x + 1, button.y);
    let screen = draw(&mut terminal);
    assert!(
        screen.contains("Clear the application log")
            && screen.contains("all kept days")
            && screen.contains("cannot be undone"),
        "{screen}"
    );

    // Nothing behind the dialog hovers or answers: a click on the Open
    // folder button is swallowed, the dialog stays.
    let hover = hit_rect(Hit::ClearCancel);
    mouse(MouseEventKind::Moved, hover.x, hover.y);
    let behind = with(|s| {
        s.hits
            .iter()
            .find(|(_, h)| *h == Hit::OpenFolder)
            .map(|(r, _)| *r)
    });
    if let Some(r) = behind {
        click(r.x + 1, r.y);
    }
    assert!(with(|s| s.clear_open));

    // Cancel keeps everything.
    click(hover.x + 1, hover.y);
    assert!(!with(|s| s.clear_open));
    let screen = draw(&mut terminal);
    assert!(screen.contains("clear-marker-two"), "{screen}");
    assert!(app_lines(&log) >= 2);

    // By key: x opens, Esc cancels, x then y confirms.
    key(KeyCode::Char('x'));
    assert!(with(|s| s.clear_open));
    draw(&mut terminal);
    key(KeyCode::Esc);
    assert!(
        !with(|s| s.clear_open) && is_open(),
        "Esc only closes the dialog"
    );
    key(KeyCode::Char('x'));
    key(KeyCode::Enter);
    assert!(
        !with(|s| s.clear_open),
        "Enter on the default Cancel cancels"
    );
    assert!(app_lines(&log) >= 2);
    key(KeyCode::Char('x'));
    key(KeyCode::Char('y'));
    let screen = wait_for(&mut terminal, "Application log cleared.");
    assert!(screen.contains("Application log cleared."), "{screen}");
    assert!(!screen.contains("clear-marker-two"), "{screen}");
    assert_eq!(app_lines(&log), 0, "the files are emptied");

    // The log keeps working afterwards.
    log.emit(Level::Warn, "tui", "clear-marker-after");
    let screen = wait_for(&mut terminal, "clear-marker-after");
    assert!(screen.contains("clear-marker-after"), "{screen}");
    close();
}

#[test]
fn the_confirm_button_and_the_palette_command_clear_too() {
    let _lock = lock();
    let mut terminal = terminal();
    let log = seed_for_clear(&mut terminal);
    draw(&mut terminal);
    click(hit_rect(Hit::ClearLog).x + 1, hit_rect(Hit::ClearLog).y);
    draw(&mut terminal);
    let confirm = hit_rect(Hit::ClearConfirm);
    click(confirm.x + 1, confirm.y);
    wait_for(&mut terminal, "Application log cleared.");
    assert_eq!(app_lines(&log), 0);

    // The palette's command opens the window on the tab and asks first.
    close();
    level_cmd_open(LevelCmd::ClearLog);
    assert!(is_open() && with(|s| s.tab == Tab::Application && s.clear_open));
    close();
    assert!(
        !with(|s| s.clear_open),
        "closing the window drops the dialog"
    );
}

// ---- the Scope box ----

fn entry_in(title: &str, repo: Option<&str>) -> feed::Entry {
    let mut e = feed::Entry::new(Severity::Error, title.into(), "boom".into());
    e.repo = repo.map(std::path::PathBuf::from);
    e
}

fn titles() -> Vec<String> {
    entries().into_iter().map(|e| e.title).collect()
}

fn set_scope(scope: Scope) {
    with(|s| s.scope = scope);
}

#[test]
fn the_scope_narrows_the_git_operations_and_app_wide_entries_stay_marked() {
    let _lock = lock();
    feed::push(entry_in("in alpha", Some("/work/alpha")));
    feed::push(entry_in("in beta", Some("/work/beta")));
    feed::push(entry_in("failed save", None));
    let mut walk = entry_in("fetch all", None);
    walk.workspace = Some("Acme".into());
    feed::push(walk);
    close();
    open();
    set_active_repo(std::path::Path::new("/work/alpha"));
    with(|s| {
        s.ws_cache = vec![
            ("Acme".into(), vec![std::path::PathBuf::from("/work/alpha")]),
            ("Other".into(), vec![std::path::PathBuf::from("/work/beta")]),
        ]
    });

    set_scope(Scope::All);
    assert_eq!(
        titles(),
        ["fetch all", "failed save", "in beta", "in alpha"],
        "all of them, newest first"
    );
    set_scope(Scope::This);
    assert_eq!(
        titles(),
        ["failed save", "in alpha"],
        "this repository's, and what belongs to none (a workspace task is not)"
    );
    set_scope(Scope::Repo("/work/beta".into()));
    assert_eq!(titles(), ["failed save", "in beta"]);
    set_scope(Scope::Workspace("Acme".into()));
    assert_eq!(
        titles(),
        ["fetch all", "failed save", "in alpha"],
        "its repositories' entries and its own tasks"
    );
    set_scope(Scope::Workspace("Other".into()));
    assert_eq!(titles(), ["failed save", "in beta"]);
    close();
}

#[test]
fn it_opens_on_this_repository_from_a_repository_and_on_all_from_home() {
    let _lock = lock();
    let mut terminal = terminal();
    set_active_repo(std::path::Path::new("/work/alpha"));
    assert_eq!(with(|s| s.scope.clone()), Scope::This);
    let screen = draw(&mut terminal);
    assert!(screen.contains("This repository (alpha)"), "{screen}");
    close();
    set_home(true);
    open();
    assert_eq!(with(|s| s.scope.clone()), Scope::All);
    let screen = draw(&mut terminal);
    assert!(screen.contains("Scope:") && screen.contains("All repositories"));
    // Home has no repository of its own: the row is not offered.
    assert!(scope_options().iter().all(|o| o.value != "this"));
    // The palette's and the Home button's "global" entry is always All.
    close();
    set_home(false);
    open_global();
    assert_eq!(with(|s| s.scope.clone()), Scope::All);
    close();
}

#[test]
fn an_explicit_choice_is_remembered_per_entry_point() {
    let _lock = lock();
    let _ = terminal();
    set_active_repo(std::path::Path::new("/work/alpha"));
    box_picked(Box3::Scope, "all");
    close();
    open();
    assert_eq!(
        with(|s| s.scope.clone()),
        Scope::All,
        "the repository entry remembered it"
    );
    close();
    set_home(true);
    open();
    assert_eq!(
        with(|s| s.scope.clone()),
        Scope::All,
        "Home keeps its own default"
    );
    box_picked(Box3::Scope, "repo:/work/beta");
    close();
    open();
    assert_eq!(with(|s| s.scope.clone()), Scope::Repo("/work/beta".into()));
    set_home(false);
    close();
    open();
    assert_eq!(with(|s| s.scope.clone()), Scope::All, "not the Home choice");
    set_home(false);
    with(|s| s.remembered = [None, None]);
    close();
}

#[test]
fn the_scope_box_opens_by_key_and_by_click_and_picks_for_both_tabs() {
    let _lock = lock();
    feed::push(entry_in("in alpha", Some("/work/alpha")));
    feed::push(entry_in("in beta", Some("/work/beta")));
    let mut terminal = terminal();
    set_active_repo(std::path::Path::new("/work/alpha"));
    let screen = draw(&mut terminal);
    assert!(screen.contains("This repository (alpha)"), "{screen}");
    assert!(
        screen.contains("in alpha") && !screen.contains("in beta"),
        "{screen}"
    );

    // A click on the box opens its list (never a cycling click): the rows
    // are this repository, all, then the others.
    let field = with(|s| s.scope_field);
    assert!(field.width > 0);
    click(field.x + 1, field.y);
    assert!(with(|s| s.scope_combo.as_ref().is_some_and(Combo::is_open)));
    let screen = draw(&mut terminal);
    assert!(
        screen.contains("All repositories") && screen.contains("beta"),
        "{screen}"
    );
    assert_eq!(with(|s| s.scope.clone()), Scope::This, "a click only opens");
    // Down to "All repositories", Enter picks it.
    key(KeyCode::Down);
    key(KeyCode::Enter);
    assert_eq!(with(|s| s.scope.clone()), Scope::All);
    let screen = draw(&mut terminal);
    assert!(
        screen.contains("in alpha") && screen.contains("in beta"),
        "{screen}"
    );

    // `s` is the keyboard twin of the click; Esc closes the list.
    key(KeyCode::Char('s'));
    assert!(with(|s| s.scope_combo.as_ref().is_some_and(Combo::is_open)));
    key(KeyCode::Esc);
    assert!(is_open(), "Esc closes only the list");
    assert!(!with(|s| s
        .scope_combo
        .as_ref()
        .is_some_and(Combo::is_open)));

    // The box is on the Application tab too.
    key(KeyCode::Tab);
    let screen = draw(&mut terminal);
    assert!(
        screen.contains("Scope:") && screen.contains("All repositories"),
        "{screen}"
    );
    key(KeyCode::Char('s'));
    assert!(with(|s| s.scope_combo.as_ref().is_some_and(Combo::is_open)));
    key(KeyCode::Esc);
    close();
}

fn scoped_record(message: &str, repo: Option<&str>) -> Record {
    Record {
        repo: repo.map(str::to_string),
        ..record(Level::Info, message)
    }
}

#[test]
fn the_application_tab_follows_the_scope_and_marks_app_wide_lines() {
    let _lock = lock();
    let mut terminal = terminal();
    set_active_repo(std::path::Path::new("/work/alpha"));
    show_only(vec![
        scoped_record("line-of-alpha", Some("/work/alpha")),
        scoped_record("line-of-beta", Some("/work/beta")),
        scoped_record("line-of-none", None),
    ]);
    key(KeyCode::Tab);
    // This repository: its lines, plus the app-wide one, marked.
    let screen = draw(&mut terminal);
    assert!(screen.contains("line-of-alpha"), "{screen}");
    assert!(!screen.contains("line-of-beta"), "{screen}");
    assert!(screen.contains("[app-wide] line-of-none"), "{screen}");
    assert!(!screen.contains("[app-wide] line-of-alpha"), "{screen}");
    // All repositories: everything, each line says whose it is.
    set_scope(Scope::All);
    let screen = draw(&mut terminal);
    assert!(
        screen.contains("[alpha] line-of-alpha") && screen.contains("[beta] line-of-beta"),
        "{screen}"
    );
    assert!(
        screen.contains("line-of-none") && !screen.contains("[app-wide]"),
        "{screen}"
    );
    // Another repository: its lines and the app-wide one.
    set_scope(Scope::Repo("/work/beta".into()));
    let screen = draw(&mut terminal);
    assert!(
        screen.contains("line-of-beta") && screen.contains("line-of-none"),
        "{screen}"
    );
    assert!(!screen.contains("line-of-alpha"), "{screen}");
    // The level counts and Copy all follow it.
    let sc = scoping();
    assert_eq!(level_counts(&sc, Level::Info, "")[Level::Info as usize], 2);
    unfreeze();
    close();
}

#[test]
fn a_clone_failure_is_listed_under_its_destination_and_opens_on_its_details() {
    let _lock = lock();
    let dest = std::path::Path::new("/work/never-cloned");
    let id = super::super::problem::report_in(
        super::super::problem::Source::Clone,
        "fatal: could not read",
        dest,
    );
    set_active_repo(std::path::Path::new("/work/home"));
    let entry = feed::get(id).unwrap();
    assert!(belongs_elsewhere(&entry), "not the tab in front");
    close();
    open_global_on(id);
    assert_eq!(with(|s| s.scope.clone()), Scope::All);
    assert_eq!(entries()[with(|s| s.selected)].id, id);
    assert!(with(|s| s.expanded.contains(&id)), "opened on the entry");
    // The scope of the tab in front does not carry it.
    set_scope(Scope::This);
    assert!(entries().iter().all(|e| e.id != id));
    set_active_repo(dest);
    assert!(
        !belongs_elsewhere(&entry),
        "its own repository is the tab in front"
    );
    close();
}

/// A failed fetch that could not reach the remote: it offers Retry.
fn unreachable_fetch() -> feed::Entry {
    let mut e = feed::Entry::new(Severity::Error, "Fetch failed".into(), "boom".into());
    e.kind = gitpoulpi_core::FailureKind::NetworkUnreachable;
    e.retry = Some(crate::daemon_link::Action::FetchAll);
    e
}

#[test]
fn an_entry_with_a_remedy_shows_its_button_and_the_key_and_the_click_hand_over_the_command() {
    let _lock = lock();
    let _ = take_command();
    feed::push(unreachable_fetch());
    let mut terminal = terminal();
    key(KeyCode::Enter);
    let screen = draw(&mut terminal);
    assert!(screen.contains(" Retry "), "{screen}");

    // The click on the button.
    let button = hit_rect(Hit::Remedy(0));
    click(button.x + 1, button.y);
    assert!(!is_open(), "the window closes");
    assert_eq!(
        take_command(),
        Some(crate::activity_log::Command::Submit(
            crate::daemon_link::Action::FetchAll
        ))
    );

    // The key on the selected entry.
    let mut terminal = self::terminal();
    key(KeyCode::Enter);
    draw(&mut terminal);
    key(KeyCode::Char('r'));
    assert!(!is_open());
    assert!(take_command().is_some());
}

#[test]
fn an_entry_without_a_remedy_has_no_button_and_r_does_nothing() {
    let _lock = lock();
    let _ = take_command();
    feed::push(error("Push failed"));
    let mut terminal = terminal();
    key(KeyCode::Enter);
    let screen = draw(&mut terminal);
    assert!(!screen.contains(" Retry "), "{screen}");
    key(KeyCode::Char('r'));
    assert!(is_open(), "nothing to run: the window stays");
    assert!(take_command().is_none());
    close();
}

#[test]
fn the_window_shows_the_pointing_hand_on_controls_and_the_arrow_on_the_filter() {
    let _lock = lock();
    let mut terminal = terminal();
    draw(&mut terminal);
    let tab = hit_rect(Hit::Tab(Tab::Application));
    assert!(pointer_hand(tab.x + 1, tab.y), "a tab");
    let close_button = with(|s| s.close);
    assert!(
        pointer_hand(close_button.x + 1, close_button.y),
        "the cross"
    );
    let copy = hit_rect(Hit::CopyDiagnostics);
    assert!(pointer_hand(copy.x + 1, copy.y), "a button");
    click(tab.x + 1, tab.y);
    draw(&mut terminal);
    let filter = hit_rect(Hit::Filter);
    assert!(
        !pointer_hand(filter.x + 1, filter.y),
        "the filter is a text field"
    );
    let level = with(|s| s.level_field);
    assert!(pointer_hand(level.x + 1, level.y), "the Level box");
    assert!(!pointer_hand(0, 0), "outside the window");
    close();
}
