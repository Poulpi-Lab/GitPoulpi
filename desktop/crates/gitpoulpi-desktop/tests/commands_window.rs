//! Headless tests of the command layer on the real main window: keys resolved with the shared keymap,
//! the command palette, the menu bar, the keybindings view and the "not available yet" answer.

use std::path::Path;

use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::commands::context::WindowState;
use gitpoulpi_desktop::commands::{self, palette, status, Outcome};
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::prefs::PrefsStore;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, Keystroke, TestAppContext};

fn fixture() -> FixtureRepo {
    isolate_process_env();
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "a", "second")
        .unwrap()
        .build()
}

fn open(
    cx: &mut TestAppContext,
    repo: &Path,
    config_dir: &Path,
) -> (AnyWindowHandle, Entity<Workspace>) {
    let backend = Backend::start(repo).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(config_dir.to_path_buf()), &Overrides::default()).unwrap());
    let opened = cx.update(|cx| app::open_main_window(backend, cx).unwrap());
    cx.run_until_parked();
    opened
}

fn press(cx: &mut TestAppContext, window: AnyWindowHandle, keys: &'static str) {
    for key in keys.split(' ') {
        cx.update_window(window, |_, w, cx| {
            w.dispatch_keystroke(Keystroke::parse(key).unwrap(), cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

fn dialog_open(cx: &mut TestAppContext, window: AnyWindowHandle) -> bool {
    cx.update_window(window, |_, w, cx| w.has_active_dialog(cx))
        .unwrap()
}

fn status_text(cx: &mut TestAppContext) -> Option<String> {
    cx.update(|cx| status::current(cx).map(|s| s.to_string()))
}

#[gpui_kit::test]
fn ctrl_p_opens_the_command_palette_and_escape_closes_it(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    assert!(!dialog_open(cx, window));
    press(cx, window, "ctrl-p");
    assert!(dialog_open(cx, window), "Ctrl+P opens the palette");
    press(cx, window, "escape");
    assert!(!dialog_open(cx, window), "Escape closes it");
}

#[gpui_kit::test]
fn a_command_that_is_not_implemented_answers_in_the_status_line(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    let outcome = cx
        .update_window(window, |_, w, cx| {
            commands::run_id("git_prefix.fetch_all", w, cx)
        })
        .unwrap();
    assert_eq!(outcome, Outcome::NotAvailable);
    let text = status_text(cx).expect("a message");
    assert!(text.contains("not available yet"), "{text}");
    // An unknown id never panics either.
    cx.update_window(window, |_, w, cx| commands::run_id("no.such_id", w, cx))
        .unwrap();
}

#[gpui_kit::test]
fn the_git_prefix_chord_reaches_the_command_and_says_it_is_not_there_yet(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    press(cx, window, "ctrl-g");
    assert!(status_text(cx).is_some(), "the prefix key shows a hint");
    press(cx, window, "f");
    let text = status_text(cx).expect("a message");
    assert!(text.contains("not available yet"), "{text}");
}

#[gpui_kit::test]
fn the_navigation_prefix_toggles_the_refs_pane(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    let shown = |cx: &mut TestAppContext| cx.update(|cx| PrefsStore::get(cx).layout.show_left);
    let before = shown(cx);
    press(cx, window, "ctrl-n");
    press(cx, window, "shift-h");
    assert_ne!(shown(cx), before);
    press(cx, window, "ctrl-n shift-h");
    assert_eq!(shown(cx), before);
}

#[gpui_kit::test]
fn f10_and_alt_letters_open_the_menus(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, ws) = open(cx, repo.root(), dir.path());
    let open_menu = |cx: &mut TestAppContext| {
        ws.read_with(cx, |ws, cx| {
            ws.menu_bar().read(cx).open_state().map(|o| o.heading)
        })
    };
    assert_eq!(open_menu(cx), None);
    press(cx, window, "f10");
    assert_eq!(open_menu(cx), Some(0), "F10 opens the first menu");
    press(cx, window, "right");
    assert_eq!(open_menu(cx), Some(0), "Right first enters the submenu");
    press(cx, window, "right");
    assert_eq!(open_menu(cx), Some(1), "then moves to the next menu");
    press(cx, window, "escape");
    assert_eq!(open_menu(cx), None, "Escape closes the menu");
    press(cx, window, "alt-f");
    assert_eq!(open_menu(cx), Some(0), "Alt+F opens File");
    press(cx, window, "escape");
}

#[gpui_kit::test]
fn a_disabled_menu_item_answers_with_its_reason_instead_of_running(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, ws) = open(cx, repo.root(), dir.path());
    press(cx, window, "alt-g");
    let item_state = ws.read_with(cx, |ws, cx| {
        let bar = ws.menu_bar().read(cx);
        bar.open_state().map(|_| bar.menus().len())
    });
    assert!(item_state.is_some(), "the Git menu opened");
    // Down into the first submenu's first item, then Enter on an item that is not implemented.
    press(cx, window, "enter down");
    press(cx, window, "enter");
    let text = status_text(cx).expect("a message");
    assert!(text.contains("not available yet"), "{text}");
    let still_open = ws.read_with(cx, |ws, cx| ws.menu_bar().read(cx).open_state().is_some());
    assert!(still_open, "a disabled item does not close the menu");
}

#[gpui_kit::test]
fn the_keybindings_view_opens_from_help_and_from_the_command_id(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    cx.update_window(window, |_, w, cx| {
        assert_eq!(
            commands::run_id("global.help", w, cx),
            Outcome::Done,
            "the help command is implemented"
        );
    })
    .unwrap();
    cx.run_until_parked();
    assert!(dialog_open(cx, window));
}

#[gpui_kit::test]
fn a_dialog_owns_the_keyboard(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    press(cx, window, "ctrl-p");
    assert!(dialog_open(cx, window));
    // Ctrl+N would start the navigation prefix on the window: inside the palette it is left alone.
    press(cx, window, "ctrl-n");
    assert_eq!(status_text(cx), None);
    press(cx, window, "escape");
}

#[gpui_kit::test]
fn the_palette_filters_moves_and_runs_a_row(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    let picker = cx
        .update_window(window, |_, w, cx| {
            palette::open_with_state(WindowState::default(), w, cx)
        })
        .unwrap();
    cx.run_until_parked();
    assert!(dialog_open(cx, window));
    cx.update_window(window, |_, w, cx| {
        picker.update(cx, |p, cx| p.set_query("fetch", w, cx))
    })
    .unwrap();
    let (first, dim, reason) = picker.read_with(cx, |p, _| {
        let r = &p.rows()[0];
        (r.label.to_string(), r.dim, r.reason.clone())
    });
    assert!(first.to_lowercase().contains("fetch"), "{first}");
    assert!(dim, "not implemented yet: dimmed");
    assert_eq!(
        reason.as_deref(),
        Some(gitpoulpi_desktop::strings::NOT_AVAILABLE_YET)
    );
    press(cx, window, "down");
    assert_eq!(picker.read_with(cx, |p, _| p.selected()), 1, "Down moves");
    press(cx, window, "up");
    assert_eq!(
        picker.read_with(cx, |p, _| p.selected()),
        0,
        "Up moves back"
    );
    press(cx, window, "enter");
    assert!(!dialog_open(cx, window), "the palette closed");
    let text = status_text(cx).expect("the status line answers");
    assert!(text.contains("not available yet"), "{text}");
}

#[gpui_kit::test]
fn an_implemented_palette_row_runs_and_opens_its_view(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    let picker = cx
        .update_window(window, |_, w, cx| {
            palette::open_with_state(WindowState::default(), w, cx)
        })
        .unwrap();
    cx.run_until_parked();
    cx.update_window(window, |_, w, cx| {
        picker.update(cx, |p, cx| p.set_query("change language", w, cx))
    })
    .unwrap();
    let (label, dim) =
        picker.read_with(cx, |p, _| (p.rows()[0].label.to_string(), p.rows()[0].dim));
    assert!(label.to_lowercase().contains("language"), "{label}");
    assert!(!dim);
    press(cx, window, "enter");
    cx.run_until_parked();
    assert!(dialog_open(cx, window), "the language picker is open");
    assert_eq!(status_text(cx), None);
}

#[gpui_kit::test]
fn a_narrow_window_folds_the_last_menus_but_loses_no_command(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), dir.path());
    let bar = ws.read_with(cx, |ws, _| ws.menu_bar().clone());
    bar.update(cx, |b, _| b.set_width(320.0));
    let (shown, all, last, items_shown, items_all) = bar.read_with(cx, |b, _| {
        (
            b.shown_menus().len(),
            b.menus().len(),
            b.shown_menus().last().map(|m| m.title.clone()),
            b.shown_menus().iter().flat_map(|m| m.items()).count(),
            b.menus().iter().flat_map(|m| m.items()).count(),
        )
    });
    assert!(shown < all, "{shown} of {all}");
    assert_eq!(last.as_deref(), Some("..."));
    assert_eq!(items_shown, items_all, "every command stays reachable");
    bar.update(cx, |b, _| b.set_width(1600.0));
    assert_eq!(bar.read_with(cx, |b, _| b.shown_menus().len()), all);
}
