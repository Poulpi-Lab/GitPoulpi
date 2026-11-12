use super::*;
use crate::chrome::HitTarget;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::time::Instant;

/// The lookups, the popup and the view are process-wide: tests take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn turn() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn app(name: &str, command: &str, default: bool) -> OsApp {
    OsApp {
        name: name.into(),
        command: command.into(),
        key: os_apps::command_key(command),
        default,
    }
}

/// A fake machine: `.fakebin` has two apps, `.faketxt` is text, the rest are
/// unknown. Never touches the real desktop configuration.
fn fake(probe: &str) -> Found {
    if probe.ends_with(".fakebin") || probe.ends_with(".fakeslow") {
        Found {
            apps: vec![
                app("Fake Viewer", "fake-viewer ${file}", true),
                app("Other Tool", "other-tool ${file}", false),
            ],
            textual: false,
        }
    } else if probe.ends_with(".fakepng") {
        Found {
            apps: vec![
                app("GNU Image Manipulation Program", "gimp-2.10 ${file}", true),
                app("Fake Krita", "fake-krita ${file}", false),
            ],
            textual: false,
        }
    } else if probe.ends_with(".txt") {
        // A desktop that registers apps for .txt without calling it text
        // (macOS does): the name alone must keep the row out of the menu.
        Found {
            apps: vec![app("Fake Editor", "fake-editor ${file}", true)],
            textual: false,
        }
    } else if probe.ends_with(".faketxt") {
        Found {
            apps: vec![app("Fake Editor", "fake-editor ${file}", true)],
            textual: true,
        }
    } else {
        Found::default()
    }
}

pub(crate) fn install_fake() {
    *TEST_DISCOVER.lock().unwrap() = Some(fake);
}

pub(crate) fn wait_ready(name: &str) -> Arc<Found> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(found) = ready(name) {
            return found;
        }
        assert!(Instant::now() < deadline, "the lookup never finished");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn subject(name: &str) -> Subject {
    Subject {
        path: PathBuf::from("docs").join(name),
        source: Source::WorkingTree,
    }
}

fn paint(subject: &Subject, width: u16, hovered: bool) -> (String, Option<Vec<(Rect, HitTarget)>>) {
    let mut terminal = Terminal::new(TestBackend::new(width, 8)).unwrap();
    let mut hits = None;
    let hover = hovered.then_some(HitTarget::OpenWith);
    view::set_hovered(hover.as_ref());
    terminal
        .draw(|frame| {
            let inner = Rect::new(0, 0, width, 8);
            hits = view::render(
                frame,
                inner,
                subject,
                "binary file (12 bytes) - nothing to show",
            );
        })
        .unwrap();
    let text: Vec<String> = (0..8)
        .map(|y| {
            (0..width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol().to_string())
                .collect()
        })
        .collect();
    (text.join("\n"), hits)
}

#[test]
fn a_lookup_runs_once_per_type_off_the_caller_and_answers_pending_first() {
    static GATE: Mutex<()> = Mutex::new(());
    fn slow(_probe: &str) -> Found {
        let _held = GATE.lock().unwrap_or_else(|e| e.into_inner());
        Found::default()
    }
    let cache = Cache::default();
    let held = GATE.lock().unwrap();
    let first = cache.lookup("Report.PDF", slow);
    assert!(
        matches!(first, Lookup::Pending { .. }),
        "never waits for it"
    );
    assert!(cache.any_pending());
    // The same type under another name shares the one lookup.
    assert!(matches!(
        cache.lookup("other.pdf", slow),
        Lookup::Pending { .. }
    ));
    drop(held);
    let deadline = Instant::now() + Duration::from_secs(10);
    while cache.any_pending() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(matches!(cache.lookup("x.pdf", slow), Lookup::Ready(_)));
}

#[test]
fn the_key_is_the_extension_else_the_file_name() {
    assert_eq!(key_of("a/b/Report.PDF"), "pdf");
    assert_eq!(key_of("Makefile"), "makefile");
    assert_eq!(key_of("dir\\Thing.TAR.GZ"), "gz");
}

#[test]
fn the_menu_offers_open_with_for_binary_types_only() {
    let _t = turn();
    install_fake();
    // Text for sure: no wait, no row.
    assert!(!menu_offered("main.rs"));
    // Unknown while the lookup runs: offered, the popup says it is looking.
    let name = "menu-case.fakebin";
    let first = menu_offered(name);
    assert!(first);
    wait_ready(name);
    assert!(menu_offered(name), "the OS knows two apps");
    let txt = "menu-case.faketxt";
    menu_offered(txt);
    wait_ready(txt);
    assert!(!menu_offered(txt), "the OS says it is text");
    let none = "menu-case.fakeunknown";
    menu_offered(none);
    wait_ready(none);
    assert!(!menu_offered(none), "no app registered");
}

/// The row must not appear once the OS answers for a name that is text for
/// sure: it would insert a row under a click aimed at the rows below.
#[test]
fn a_text_name_never_gains_the_row_when_the_os_answers() {
    let _t = turn();
    install_fake();
    let name = "menu-stable.txt";
    assert!(!menu_offered(name));
    let _ = lookup(name);
    let found = wait_ready(name);
    assert!(!found.apps.is_empty() && !found.textual, "the OS answered");
    assert!(!menu_offered(name), "the answer must not add a row");
}

#[test]
fn the_row_shows_the_button_and_registers_the_rect_it_painted() {
    let _t = turn();
    install_fake();
    popup::close();
    let name = "paint-case.fakebin";
    wait_ready(name);
    let (text, hits) = paint(&subject(name), 80, false);
    let hits = hits.expect("two apps: the row is drawn");
    assert_eq!(hits.len(), 1);
    let (rect, target) = &hits[0];
    assert_eq!(*target, HitTarget::OpenWith);
    assert_eq!(rect.y, 0);
    assert_eq!(rect.width, popup::button_width());
    assert!(text.lines().next().unwrap().contains("Open in"), "{text}");
    assert!(text.contains(name), "{text}");
    assert!(text.contains("binary file (12 bytes)"), "{text}");
    // What the keys and the palette act on.
    assert_eq!(view::shown().unwrap().path.file_name().unwrap(), name);
    // The button is where the hit says: its label sits inside the rect.
    let row: Vec<char> = text.lines().next().unwrap().chars().collect();
    let inside: String = row[rect.x as usize..(rect.x + rect.width) as usize]
        .iter()
        .collect();
    assert!(inside.contains("Open in"), "{inside:?}");
    view::begin_frame();
    assert!(
        view::shown().is_none(),
        "forgotten when a frame does not draw it"
    );
}

#[test]
fn the_row_is_not_drawn_without_apps_for_text_or_in_a_row_too_narrow() {
    let _t = turn();
    install_fake();
    for name in ["none-case.fakeunknown", "txt-case.faketxt"] {
        wait_ready(name);
        let (text, hits) = paint(&subject(name), 80, false);
        assert!(hits.is_none(), "{name}: {text}");
        assert!(!text.contains("Open in"));
    }
    let name = "narrow-case.fakebin";
    wait_ready(name);
    let (_, hits) = paint(&subject(name), popup::button_width() + 1, false);
    assert!(hits.is_none(), "no glyph, no hit target");
}

#[test]
fn the_button_lights_when_the_pointer_is_over_it() {
    let _t = turn();
    install_fake();
    let name = "hover-case.fakebin";
    wait_ready(name);
    let mut cells = Vec::new();
    for hovered in [false, true] {
        let mut terminal = Terminal::new(TestBackend::new(80, 8)).unwrap();
        let mut rect = Rect::default();
        view::set_hovered(hovered.then_some(HitTarget::OpenWith).as_ref());
        terminal
            .draw(|frame| {
                let hits =
                    view::render(frame, Rect::new(0, 0, 80, 8), &subject(name), "n").unwrap();
                rect = hits[0].0;
            })
            .unwrap();
        cells.push(terminal.backend().buffer()[(rect.x + 1, rect.y)].style());
    }
    assert_ne!(cells[0], cells[1], "a hovered button looks different");
}

#[test]
fn a_slow_lookup_shows_looking_up_only_after_150_ms() {
    let _t = turn();
    static OPEN: Mutex<()> = Mutex::new(());
    fn slow(p: &str) -> Found {
        let _held = OPEN.lock().unwrap_or_else(|e| e.into_inner());
        fake(p)
    }
    *TEST_DISCOVER.lock().unwrap() = Some(slow);
    let held = OPEN.lock().unwrap();
    let name = "slow-case.fakeslow";
    let (text, hits) = paint(&subject(name), 80, false);
    assert!(hits.is_none() && !text.contains("looking up"), "{text}");
    std::thread::sleep(LOOKUP_FEEDBACK_AFTER + Duration::from_millis(50));
    let (text, hits) = paint(&subject(name), 80, false);
    assert!(text.contains("looking up apps..."), "{text}");
    assert_eq!(hits.map(|h| h.len()), Some(0), "nothing to click yet");
    drop(held);
    wait_ready(name);
    let (text, hits) = paint(&subject(name), 80, false);
    assert!(hits.is_some() && text.contains("Open in"), "{text}");
    install_fake();
}

#[test]
fn the_popup_lists_the_default_first_and_marked_and_says_when_it_is_looking_or_empty() {
    let _t = turn();
    install_fake();
    let name = "popup-case.fakebin";
    wait_ready(name);
    let found = fake("x.fakebin");
    let options = popup::option_labels(Some(&found));
    assert_eq!(options, ["\u{2022} Fake Viewer", "  Other Tool"]);
    assert_eq!(popup::option_labels(None), ["  looking up apps..."]);
    assert_eq!(
        popup::option_labels(Some(&Found::default())),
        ["  no application found"]
    );
}

/// The list opened by a context-menu row is drawn beside that menu, on the
/// row's line, and not at the screen's corner.
#[test]
fn the_list_opened_from_a_menu_row_hangs_beside_the_menu() {
    let _t = turn();
    install_fake();
    popup::close();
    let name = "flyout-case.fakebin";
    wait_ready(name);
    let parent = Rect::new(20, 4, 24, 9);
    popup::open_for_test(subject(name), Some((parent, 7)));
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut rect = Rect::default();
    terminal
        .draw(|frame| {
            let mut hits = Vec::new();
            popup::render_popup(frame, None, &mut hits);
            rect = hits
                .iter()
                .find(|(_, t)| *t == HitTarget::OpenInPopup)
                .map(|(r, _)| *r)
                .unwrap();
        })
        .unwrap();
    assert_eq!((rect.x, rect.y), (parent.right(), 7), "{rect:?}");
    popup::close();
}

#[test]
fn a_commit_version_is_handed_over_as_a_read_only_copy_and_a_failed_launch_is_reported() {
    let _t = turn();
    let oid = Oid([7u8; 20]);
    let subject = Subject {
        path: PathBuf::from("art/logo.pdf"),
        source: Source::Commit(oid),
    };
    let request = FileInsightRequest {
        path: subject.path.clone(),
        at: oid,
        kind: FileInsightKind::ImageBytes(DiffContentSide::Commit),
        worktree: false,
        staged: false,
    };
    // A program that is not there: the failure goes to the status line.
    let apps = vec![app(
        "Nowhere",
        "gitpoulpi-test-no-such-program ${file}",
        true,
    )];
    *AWAITING.lock().unwrap() = Some(Awaiting {
        request: request.clone(),
        apps,
        index: 0,
        subject: subject.clone(),
    });
    let bytes = gitpoulpi_core::domain::BlobBytes {
        path: subject.path.clone(),
        source: gitpoulpi_core::domain::BlobSource::Commit(oid),
        size: 5,
        body: BlobBody::Bytes(vec![1, 2, 3, 4, 5]),
    };
    // Another request is not ours.
    let other = FileInsightOutcome {
        request: FileInsightRequest {
            path: PathBuf::from("other.bin"),
            ..request.clone()
        },
        result: Ok(FileInsightPayload::ImageBytes(bytes.clone())),
    };
    assert!(offer(other).is_some(), "handed back untouched");
    assert!(AWAITING.lock().unwrap().is_some(), "still waiting");
    let ours = FileInsightOutcome {
        request,
        result: Ok(FileInsightPayload::ImageBytes(bytes)),
    };
    assert!(offer(ours).is_none(), "taken");
    assert!(AWAITING.lock().unwrap().is_none());
    // The Activity entry names the program; the status line stays clear.
    assert_eq!(take_notice().as_deref(), Some(""));
    let card = crate::activity_log::newest_failure_details().expect("a failure entry");
    assert!(card.contains("gitpoulpi-test-no-such-program"), "{card}");
    // The copy was written before the launch: read-only, under the temp root.
    let copy =
        crate::image_editor::materialise(&[1, 2, 3, 4, 5], &subject.rev(), "logo.pdf").unwrap();
    assert!(std::fs::metadata(&copy).unwrap().permissions().readonly());
    assert_eq!(std::fs::read(&copy).unwrap(), [1, 2, 3, 4, 5]);
    assert!(!copy.starts_with("art"));
}

#[test]
fn a_read_that_fails_or_is_too_large_says_so() {
    let _t = turn();
    let oid = Oid([8u8; 20]);
    let subject = Subject {
        path: PathBuf::from("big.iso"),
        source: Source::Commit(oid),
    };
    let request = FileInsightRequest {
        path: subject.path.clone(),
        at: oid,
        kind: FileInsightKind::ImageBytes(DiffContentSide::Commit),
        worktree: false,
        staged: false,
    };
    let wait = |request: &FileInsightRequest| {
        *AWAITING.lock().unwrap() = Some(Awaiting {
            request: request.clone(),
            apps: vec![app("X", "x ${file}", true)],
            index: 0,
            subject: subject.clone(),
        });
    };
    wait(&request);
    let too_large = gitpoulpi_core::domain::BlobBytes {
        path: subject.path.clone(),
        source: gitpoulpi_core::domain::BlobSource::Commit(oid),
        size: 3 * 1024 * 1024,
        body: BlobBody::TooLarge,
    };
    assert!(offer(FileInsightOutcome {
        request: request.clone(),
        result: Ok(FileInsightPayload::ImageBytes(too_large)),
    })
    .is_none());
    let said = take_notice().unwrap();
    assert!(
        said.contains("big.iso") && said.contains("too large"),
        "{said}"
    );
}

#[test]
fn the_working_tree_file_is_launched_in_place_and_a_missing_one_is_reported() {
    let _t = turn();
    let dir = tempfile::tempdir().unwrap();
    let asked = std::cell::RefCell::new(Vec::new());
    let ask = |r: FileInsightRequest| asked.borrow_mut().push(r);
    let apps = vec![app("X", "x ${file}", true)];
    let said = run_with(&apps, 0, &subject("gone.fakebin"), dir.path(), &ask);
    assert!(said.contains("gone.fakebin"), "{said}");
    let said = run_with(&apps, 5, &subject("a.fakebin"), dir.path(), &ask);
    assert!(said.contains("a.fakebin"), "no such entry: {said}");
    assert!(asked.borrow().is_empty(), "the working tree needs no read");

    // A commit's version and the index's are read from the daemon first.
    let oid = Oid([9u8; 20]);
    let commit = Subject {
        path: PathBuf::from("a/b.pdf"),
        source: Source::Commit(oid),
    };
    let said = run_with(&apps, 0, &commit, dir.path(), &ask);
    assert!(said.contains("b.pdf"), "{said}");
    let index = Subject {
        source: Source::Index,
        ..commit.clone()
    };
    run_with(&apps, 0, &index, dir.path(), &ask);
    let asked = asked.borrow();
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[0].at, oid);
    assert_eq!(
        asked[0].kind,
        FileInsightKind::ImageBytes(DiffContentSide::Commit)
    );
    assert_eq!(
        asked[1].kind,
        FileInsightKind::ImageBytes(DiffContentSide::Index)
    );
    assert!(asked.iter().all(|r| r.path == commit.path && !r.worktree));
    *AWAITING.lock().unwrap() = None;
}
