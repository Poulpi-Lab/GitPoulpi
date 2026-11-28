//! The settings files through the real window: what `desktop.toml` says at start (theme, geometry,
//! diff layout), and an edit of the files made from outside while the window runs (the theme, the
//! language, the date patterns, a key binding and the diff layout follow, without a restart).
//!
//! The language and the keymap are process-wide, so this binary holds one test.

use std::time::{Duration, Instant};

use gitpoulpi_config::{ConfigPaths, DesktopConfig};
use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::prefs::PrefsStore;
use gitpoulpi_desktop::theme::CurrentTheme;
use gitpoulpi_i18n::Lang;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepoBuilder};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, Keystroke, TestAppContext};

fn press(cx: &mut TestAppContext, window: AnyWindowHandle, key: &'static str) {
    cx.update_window(window, |_, w, cx| {
        w.dispatch_keystroke(Keystroke::parse(key).unwrap(), cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn wait_until(
    cx: &mut TestAppContext,
    what: &str,
    mut done: impl FnMut(&mut TestAppContext) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn theme(cx: &mut TestAppContext) -> String {
    cx.update(|cx| cx.global::<CurrentTheme>().0.clone())
}

fn split(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> bool {
    ws.read_with(cx, |ws, cx| ws.diff.read(cx).is_split())
}

#[gpui_kit::test]
fn the_settings_files_are_read_at_start_and_an_outside_edit_is_applied_live(
    cx: &mut TestAppContext,
) {
    isolate_process_env();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let dir = tempfile::tempdir().unwrap();
    let paths = ConfigPaths::for_dir(dir.path());
    DesktopConfig::set_in_file(&paths, "ui.theme", "dark").unwrap();
    DesktopConfig::set_in_file(&paths, "ui.language", "en").unwrap();
    DesktopConfig::set_in_file(&paths, "ui.diff_layout", "split").unwrap();
    DesktopConfig::set_window(&paths, (1300, 700), Some((40, 50)), false).unwrap();

    let backend = Backend::start(repo.root()).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(paths.clone()), &Overrides::default()).unwrap());

    // The remembered geometry is the window's.
    let options = cx.update(|cx| app::window_options(cx));
    let bounds = match options.window_bounds {
        Some(gpui_kit::WindowBounds::Windowed(b)) => b,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        (bounds.size.width.as_f32(), bounds.size.height.as_f32()),
        (1300.0, 700.0)
    );
    assert_eq!(
        (bounds.origin.x.as_f32(), bounds.origin.y.as_f32()),
        (40.0, 50.0)
    );

    let (window, ws) = cx.update(|cx| app::open_main_window(backend, cx).unwrap());
    cx.run_until_parked();
    assert_eq!(theme(cx), "dark");
    assert!(split(cx, &ws), "diff_layout = split");
    assert_eq!(gitpoulpi_i18n::current(), Lang::En);

    // An edit made from outside while the window runs.
    DesktopConfig::set_in_file(&paths, "ui.theme", "light").unwrap();
    DesktopConfig::set_in_file(&paths, "ui.language", "fr").unwrap();
    DesktopConfig::set_in_file(&paths, "ui.date_time_format", "DD/MM/YYYY").unwrap();
    DesktopConfig::set_in_file(&paths, "ui.diff_layout", "inline").unwrap();
    DesktopConfig::set_keybinding(&paths, "global", "palette", &["Ctrl+K".to_string()]).unwrap();
    cx.update(PrefsStore::check);
    wait_until(cx, "the new theme", |cx| theme(cx) == "light");

    assert_eq!(gitpoulpi_i18n::current(), Lang::Fr, "the language follows");
    assert_eq!(
        cx.update(|cx| PrefsStore::get(cx).dates().format_time(86_400 * 365)),
        "01/01/1971",
        "the date patterns follow"
    );
    assert!(!split(cx, &ws), "the diff layout follows");
    press(cx, window, "ctrl-p");
    assert!(
        !cx.update_window(window, |_, w, cx| w.has_active_dialog(cx))
            .unwrap(),
        "the old key no longer opens the palette"
    );
    press(cx, window, "ctrl-k");
    assert!(
        cx.update_window(window, |_, w, cx| w.has_active_dialog(cx))
            .unwrap(),
        "the new one does"
    );

    // What this window writes itself is not mistaken for an outside edit, and lands in the files.
    cx.update(|cx| PrefsStore::set_theme(cx, "dark"));
    cx.update(PrefsStore::check);
    cx.run_until_parked();
    assert_eq!(DesktopConfig::load(&paths).unwrap().ui.theme, "dark");
    cx.update(|cx| PrefsStore::remember_recent(cx, "/some/repo"));
    assert_eq!(
        cx.update(|cx| PrefsStore::get(cx).recent().first().cloned()),
        Some("/some/repo".to_string())
    );
    let recent =
        gitpoulpi_session::RecentRepos::load(&gitpoulpi_session::RecentRepos::file_in(dir.path()));
    assert_eq!(
        recent
            .iter()
            .next()
            .map(|r| r.path.to_string_lossy().into_owned()),
        Some("/some/repo".to_string()),
        "the recent list is the shared one"
    );
}
