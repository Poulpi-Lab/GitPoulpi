//! Headless tests of staging and committing on the real main window: the working-changes lists, hunk
//! and line staging from the diff, the commit form, and the two dialogs (discard, sensitive files),
//! against fixture repositories and a temporary config directory.

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_app::palette::vocab::status_pane::StatusSection;
use gitpoulpi_core::domain::Direction;
use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::commands;
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::dialogs;
use gitpoulpi_desktop::panels::diff::DiffSource;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, Modifiers, TestAppContext};

const BODY: &str = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\neleven\ntwelve\nthirteen\nfourteen\nfifteen\nsixteen\n";

fn base() -> FixtureRepoBuilder {
    isolate_process_env();
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", BODY, "add a")
        .unwrap()
        .commit_file("b.txt", "b\n", "add b")
        .unwrap()
}

/// a.txt edited in two places far apart, b.txt edited, n.txt new and untracked.
fn dirty() -> FixtureRepo {
    let repo = base().build();
    let edited = BODY
        .replace("two\n", "TWO\n")
        .replace("fifteen\n", "FIFTEEN\n");
    std::fs::write(repo.root().join("a.txt"), edited).unwrap();
    std::fs::write(repo.root().join("b.txt"), "b changed\n").unwrap();
    std::fs::write(repo.root().join("n.txt"), "new\n").unwrap();
    repo
}

type Opened = (AnyWindowHandle, Entity<Workspace>);

fn open(cx: &mut TestAppContext, repo: &Path, prefs_dir: &Path) -> Opened {
    let backend = Backend::start(repo).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| {
        app::init(
            cx,
            Some(gitpoulpi_config::ConfigPaths::for_dir(prefs_dir)),
            &Overrides::default(),
        )
        .unwrap()
    });
    cx.update(|cx| app::open_main_window(backend, cx).unwrap())
}

fn wait_until(
    cx: &mut TestAppContext,
    what: &str,
    mut done: impl FnMut(&mut TestAppContext) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn len(cx: &mut TestAppContext, ws: &Entity<Workspace>, s: StatusSection) -> usize {
    ws.read_with(cx, |ws, cx| ws.right.read(cx).section_len(s))
}

/// Opens the window on `repo` and waits for the WIP row's lists.
fn open_dirty(
    cx: &mut TestAppContext,
    repo: &FixtureRepo,
    unstaged: usize,
    staged: usize,
) -> Opened {
    let prefs = tempfile::tempdir().unwrap();
    let opened = open(cx, repo.root(), prefs.path());
    // The preferences directory must outlive the window: leak it for the test's lifetime.
    std::mem::forget(prefs);
    let ws = opened.1.clone();
    wait_until(cx, "the working changes", |cx| {
        len(cx, &ws, StatusSection::Unstaged) == unstaged
            && len(cx, &ws, StatusSection::Staged) == staged
            && ws.read_with(cx, |ws, cx| ws.right.read(cx).shows_working_changes())
    });
    opened
}

fn click(
    cx: &mut TestAppContext,
    ws: &Entity<Workspace>,
    s: StatusSection,
    ix: usize,
    m: Modifiers,
) {
    ws.update(cx, |ws, cx| {
        ws.right.update(cx, |r, cx| r.click_row(s, ix, m, cx))
    });
}

fn row_of(cx: &mut TestAppContext, ws: &Entity<Workspace>, s: StatusSection, path: &str) -> usize {
    ws.read_with(cx, |ws, cx| ws.right.read(cx).row_paths(s))
        .iter()
        .position(|p| p == path)
        .unwrap_or_else(|| panic!("{path} is not listed"))
}

fn run(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) {
    cx.update_window(window, |_, w, cx| commands::run_id(id, w, cx))
        .unwrap();
    cx.run_until_parked();
}

fn undo(cx: &mut TestAppContext, window: AnyWindowHandle, ws: &Entity<Workspace>) {
    wait_until(cx, "an entry to undo", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(Direction::Undo))
    });
    run(cx, window, "global.undo");
}

fn dialog_open(cx: &mut TestAppContext, window: AnyWindowHandle) -> bool {
    cx.update_window(window, |_, w, cx| w.has_active_dialog(cx))
        .unwrap()
}

fn answer(cx: &mut TestAppContext, window: AnyWindowHandle, ix: usize) {
    let done = cx
        .update_window(window, |_, w, cx| dialogs::answer_open(ix, w, cx))
        .unwrap();
    assert!(done, "a question is open");
    cx.run_until_parked();
}

fn notice(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> String {
    ws.read_with(cx, |ws, cx| {
        ws.notice(cx).map(|n| n.text).unwrap_or_default()
    })
}

const NONE: Modifiers = Modifiers {
    control: false,
    alt: false,
    shift: false,
    platform: false,
    function: false,
};
const CTRL: Modifiers = Modifiers {
    control: true,
    ..NONE
};

fn staged_names(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["diff", "--cached", "--name-only"])
        .lines()
        .map(str::to_string)
        .collect()
}

#[gpui_kit::test]
fn the_sections_list_the_files_with_counts_and_stage_and_unstage_undo_as_one_entry(
    cx: &mut TestAppContext,
) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    // The title counts the files; the untracked file is in Unstaged.
    let paths = ws.read_with(cx, |ws, cx| {
        ws.right.read(cx).row_paths(StatusSection::Unstaged)
    });
    assert_eq!(paths, vec!["a.txt", "b.txt", "n.txt"]);

    let ix = row_of(cx, &ws, StatusSection::Unstaged, "b.txt");
    click(cx, &ws, StatusSection::Unstaged, ix, NONE);
    run(cx, window, "git_prefix.stage_file");
    wait_until(cx, "b.txt staged", |_| staged_names(&repo) == ["b.txt"]);
    wait_until(cx, "the lists to follow", |cx| {
        len(cx, &ws, StatusSection::Staged) == 1
    });

    undo(cx, window, &ws);
    wait_until(cx, "b.txt unstaged again", |_| {
        staged_names(&repo).is_empty()
    });

    // Stage all, then unstage all: each is one action.
    ws.update(cx, |ws, cx| ws.right.update(cx, |r, cx| r.stage_all(cx)));
    wait_until(cx, "everything staged", |_| staged_names(&repo).len() == 3);
    wait_until(cx, "the lists to follow", |cx| {
        len(cx, &ws, StatusSection::Staged) == 3
    });
    ws.update(cx, |ws, cx| ws.right.update(cx, |r, cx| r.unstage_all(cx)));
    wait_until(cx, "everything unstaged", |_| {
        staged_names(&repo).is_empty()
    });
}

#[gpui_kit::test]
fn a_plain_click_opens_the_diff_while_ctrl_and_shift_clicks_only_select(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    let diff_open = |cx: &mut TestAppContext| ws.read_with(cx, |ws, _| ws.diff_is_open());
    assert!(!diff_open(cx));
    // Ctrl and Shift clicks build a selection without opening anything.
    click(cx, &ws, StatusSection::Unstaged, 0, CTRL);
    click(cx, &ws, StatusSection::Unstaged, 1, CTRL);
    cx.run_until_parked();
    std::thread::sleep(Duration::from_millis(300));
    cx.run_until_parked();
    assert!(!diff_open(cx), "Ctrl+click does not open a diff");
    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };
    click(cx, &ws, StatusSection::Unstaged, 2, shift);
    std::thread::sleep(Duration::from_millis(300));
    cx.run_until_parked();
    assert!(!diff_open(cx), "Shift+click does not open a diff");
    assert!(ws.read_with(cx, |ws, cx| ws.right.read(cx).selection_len()) >= 2);
    // A plain click selects that one row and opens its diff.
    click(cx, &ws, StatusSection::Unstaged, 1, NONE);
    wait_until(cx, "the diff", |cx| diff_open(cx));
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).selection_len()),
        1
    );
    let _ = window;
}

#[gpui_kit::test]
fn a_multi_selection_stages_as_one_entry_that_one_undo_takes_back(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    click(cx, &ws, StatusSection::Unstaged, 0, NONE);
    click(cx, &ws, StatusSection::Unstaged, 2, CTRL);
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).selection_len()),
        2
    );
    run(cx, window, "status.stage_toggle");
    wait_until(cx, "both staged", |_| {
        let mut n = staged_names(&repo);
        n.sort();
        n == ["a.txt", "n.txt"]
    });
    undo(cx, window, &ws);
    wait_until(cx, "one undo took both back", |_| {
        staged_names(&repo).is_empty()
    });
}

#[gpui_kit::test]
fn dragging_a_file_onto_the_other_list_moves_it(cx: &mut TestAppContext) {
    let repo = dirty();
    let (_window, ws) = open_dirty(cx, &repo, 3, 0);
    let dragged = gitpoulpi_desktop::panels::right::DraggedFiles {
        from: StatusSection::Unstaged,
        paths: vec!["b.txt".into()],
    };
    ws.update(cx, |ws, cx| {
        ws.right.update(cx, |r, cx| {
            r.drop_files(StatusSection::Staged, &dragged, cx)
        })
    });
    wait_until(cx, "b.txt staged by the drop", |_| {
        staged_names(&repo) == ["b.txt"]
    });
    let back = gitpoulpi_desktop::panels::right::DraggedFiles {
        from: StatusSection::Staged,
        paths: vec!["b.txt".into()],
    };
    ws.update(cx, |ws, cx| {
        ws.right
            .update(cx, |r, cx| r.drop_files(StatusSection::Unstaged, &back, cx))
    });
    wait_until(cx, "b.txt unstaged by the drop", |_| {
        staged_names(&repo).is_empty()
    });
}

#[gpui_kit::test]
fn discard_asks_first_and_one_undo_brings_the_changes_back(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    let ix = row_of(cx, &ws, StatusSection::Unstaged, "b.txt");
    click(cx, &ws, StatusSection::Unstaged, ix, NONE);

    run(cx, window, "palette.file_menu_discard");
    assert!(dialog_open(cx, window), "discarding asks first");
    answer(cx, window, 1);
    assert!(!dialog_open(cx, window));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("b.txt")).unwrap(),
        "b changed\n",
        "Cancel keeps the changes"
    );

    run(cx, window, "palette.file_menu_discard");
    assert!(dialog_open(cx, window));
    answer(cx, window, 0);
    wait_until(cx, "b.txt discarded", |_| {
        std::fs::read_to_string(repo.root().join("b.txt")).unwrap() == "b\n"
    });
    undo(cx, window, &ws);
    wait_until(cx, "the discard undone", |_| {
        std::fs::read_to_string(repo.root().join("b.txt")).unwrap() == "b changed\n"
    });
}

#[gpui_kit::test]
fn discard_all_says_how_many_are_untracked(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, _ws) = open_dirty(cx, &repo, 3, 0);
    run(cx, window, "git_prefix.discard_all");
    let title = cx
        .update(|cx| dialogs::current(cx).map(|d| d.read(cx).spec().title.clone()))
        .expect("a question");
    assert!(
        title.contains("3 files") && title.contains("1 untracked"),
        "{title}"
    );
    answer(cx, window, 1);
    assert!(repo.root().join("n.txt").exists(), "nothing was discarded");
}

/// Opens the diff of `path` from the Unstaged or Staged list and waits for its rows.
fn open_diff(
    cx: &mut TestAppContext,
    ws: &Entity<Workspace>,
    window: AnyWindowHandle,
    section: StatusSection,
    path: &str,
) {
    let ix = row_of(cx, ws, section, path);
    click(cx, ws, section, ix, NONE);
    run(cx, window, "status.open_diff");
    wait_until(cx, "the diff", |cx| {
        ws.read_with(cx, |ws, cx| {
            let d = ws.diff.read(cx);
            ws.diff_is_open() && !d.hunk_rows().is_empty()
        })
    });
}

#[gpui_kit::test]
fn a_hunk_stages_and_unstages_from_the_diff_and_the_diff_follows(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    open_diff(cx, &ws, window, StatusSection::Unstaged, "a.txt");
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.diff.read(cx).source()),
        DiffSource::Unstaged
    );
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.diff.read(cx).hunk_rows().len()),
        2
    );

    run(cx, window, "diff.stage_hunk");
    wait_until(cx, "the first hunk staged", |_| {
        let cached = repo.git(&["diff", "--cached"]);
        cached.contains("+TWO") && !cached.contains("+FIFTEEN")
    });
    wait_until(cx, "the diff to keep only the second hunk", |cx| {
        ws.read_with(cx, |ws, cx| ws.diff.read(cx).hunk_rows().len()) == 1
    });

    undo(cx, window, &ws);
    wait_until(cx, "the hunk unstaged by undo", |_| {
        repo.git(&["diff", "--cached"]).is_empty()
    });
}

#[gpui_kit::test]
fn selected_lines_stage_alone(cx: &mut TestAppContext) {
    let repo = base().build();
    std::fs::write(repo.root().join("b.txt"), "b\nadded 1\nadded 2\nadded 3\n").unwrap();
    let (window, ws) = open_dirty(cx, &repo, 1, 0);
    open_diff(cx, &ws, window, StatusSection::Unstaged, "b.txt");
    let rows = ws.read_with(cx, |ws, cx| ws.diff.read(cx).changed_rows());
    assert_eq!(rows.len(), 3, "three added lines");
    ws.update(cx, |ws, cx| {
        ws.diff.update(cx, |d, cx| {
            d.set_cursor(rows[0], false, cx);
            d.toggle_selection(cx);
            d.set_cursor(rows[1], false, cx);
        })
    });
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.diff.read(cx).selection()),
        Some((rows[0], rows[1]))
    );
    run(cx, window, "diff.stage_lines");
    wait_until(cx, "two lines staged", |_| {
        let cached = repo.git(&["diff", "--cached"]);
        cached.contains("+added 1") && cached.contains("+added 2") && !cached.contains("+added 3")
    });
}

#[gpui_kit::test]
fn discarding_a_hunk_asks_and_a_refusal_changes_nothing(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    open_diff(cx, &ws, window, StatusSection::Unstaged, "a.txt");
    run(cx, window, "diff.discard_hunk");
    assert!(dialog_open(cx, window), "discarding a hunk asks first");
    answer(cx, window, 1);
    assert!(std::fs::read_to_string(repo.root().join("a.txt"))
        .unwrap()
        .contains("TWO"));
    run(cx, window, "diff.discard_hunk");
    answer(cx, window, 0);
    wait_until(cx, "the first hunk discarded", |_| {
        let text = std::fs::read_to_string(repo.root().join("a.txt")).unwrap();
        text.contains("\ntwo\n") && text.contains("FIFTEEN")
    });
    undo(cx, window, &ws);
    wait_until(cx, "the discard undone", |_| {
        std::fs::read_to_string(repo.root().join("a.txt"))
            .unwrap()
            .contains("TWO")
    });
}

#[gpui_kit::test]
fn a_commit_diff_is_read_only(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    open_diff(cx, &ws, window, StatusSection::Unstaged, "a.txt");
    ws.update(cx, |ws, cx| {
        ws.diff.update(cx, |d, cx| {
            let files = std::sync::Arc::new(Vec::new());
            d.show(files, 0, DiffSource::Commit, cx)
        })
    });
    run(cx, window, "diff.stage_hunk");
    assert!(
        notice(cx, &ws).contains("working tree"),
        "{}",
        notice(cx, &ws)
    );
    assert!(staged_names(&repo).is_empty());
}

fn stage_and_message(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    ws: &Entity<Workspace>,
    summary: &str,
    body: &str,
) {
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| {
            ws.right
                .update(cx, |r, cx| r.set_message(summary, body, w, cx))
        })
    })
    .unwrap();
}

#[gpui_kit::test]
fn committing_clears_the_form_and_the_log_shows_the_commit(cx: &mut TestAppContext) {
    let repo = dirty();
    repo.git(&["add", "b.txt"]);
    let (window, ws) = open_dirty(cx, &repo, 2, 1);
    let before = ws.read_with(cx, |ws, cx| ws.log.read(cx).loaded_commits());
    stage_and_message(cx, window, &ws, "Change b", "because it needed it");
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).commit_blocker(cx)),
        None
    );
    run(cx, window, "composer.commit");
    wait_until(cx, "the commit", |_| {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "Change b"
    });
    assert_eq!(
        repo.git(&["log", "-1", "--format=%b"]).trim(),
        "because it needed it"
    );
    wait_until(cx, "the form to clear", |cx| {
        ws.read_with(cx, |ws, cx| ws.right.read(cx).summary_text(cx).is_empty())
    });
    wait_until(cx, "the Log to show the new commit", |cx| {
        ws.read_with(cx, |ws, cx| ws.log.read(cx).loaded_commits()) == before + 1
    });
}

#[gpui_kit::test]
fn amend_replaces_the_last_commit_and_the_toggles_are_commands(cx: &mut TestAppContext) {
    let repo = dirty();
    repo.git(&["add", "b.txt"]);
    let (window, ws) = open_dirty(cx, &repo, 2, 1);
    let count = |repo: &FixtureRepo| {
        repo.git(&["rev-list", "--count", "HEAD"])
            .trim()
            .to_string()
    };
    let before = count(&repo);
    run(cx, window, "status.amend");
    assert!(ws.read_with(cx, |ws, cx| ws.right.read(cx).is_amending()));
    stage_and_message(cx, window, &ws, "Reworded b", "");
    run(cx, window, "status.skip_hooks");
    assert!(ws.read_with(cx, |ws, cx| ws.right.read(cx).skips_hooks()));
    run(cx, window, "composer.commit");
    wait_until(cx, "the amended commit", |_| {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "Reworded b"
    });
    assert_eq!(count(&repo), before, "amend adds no commit");
    wait_until(cx, "the toggles to reset", |cx| {
        ws.read_with(cx, |ws, cx| {
            let r = ws.right.read(cx);
            !r.is_amending() && !r.skips_hooks()
        })
    });
}

#[gpui_kit::test]
fn the_button_says_why_it_cannot_commit_and_nothing_is_sent(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    let why = ws.read_with(cx, |ws, cx| ws.right.read(cx).commit_blocker(cx));
    assert_eq!(why, Some(gitpoulpi_i18n::msg::composer::NEED_STAGED.get()));
    stage_and_message(cx, window, &ws, "Nothing staged", "");
    run(cx, window, "composer.commit");
    assert_eq!(
        notice(cx, &ws),
        gitpoulpi_i18n::msg::composer::NEED_STAGED.get()
    );
    cx.run_until_parked();
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]).trim(), "add b");
}

#[cfg(unix)]
#[gpui_kit::test]
fn a_failing_hook_keeps_the_message_and_shows_the_daemons_text(cx: &mut TestAppContext) {
    use std::os::unix::fs::PermissionsExt;
    let repo = dirty();
    repo.git(&["add", "b.txt"]);
    let hook = repo.root().join(".git/hooks/pre-commit");
    std::fs::write(&hook, "#!/bin/sh\necho 'lint says no' >&2\nexit 1\n").unwrap();
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    let (window, ws) = open_dirty(cx, &repo, 2, 1);
    stage_and_message(cx, window, &ws, "Blocked by hook", "");
    run(cx, window, "composer.commit");
    wait_until(cx, "the hook's message", |cx| {
        notice(cx, &ws).contains("lint says no")
    });
    assert_eq!(repo.git(&["log", "-1", "--format=%s"]).trim(), "add b");
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).summary_text(cx)),
        "Blocked by hook",
        "the text stays for another try"
    );
    // Skip hooks commits it.
    run(cx, window, "status.skip_hooks");
    run(cx, window, "composer.commit");
    wait_until(cx, "the commit without hooks", |_| {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "Blocked by hook"
    });
}

#[gpui_kit::test]
fn a_sensitive_file_asks_and_cancel_stages_nothing_while_stage_anyway_stages_it(
    cx: &mut TestAppContext,
) {
    let repo = base().build();
    std::fs::write(repo.root().join(".env"), "TOKEN=abc\n").unwrap();
    std::fs::write(repo.root().join("ok.txt"), "fine\n").unwrap();
    let (window, ws) = open_dirty(cx, &repo, 2, 0);
    ws.update(cx, |ws, cx| {
        ws.right.update(cx, |r, cx| {
            r.click_row(StatusSection::Unstaged, 0, NONE, cx);
            r.select_all(cx);
        })
    });
    run(cx, window, "git_prefix.stage_file");
    wait_until(cx, "the question", |cx| dialog_open(cx, window));
    let lines = cx
        .update(|cx| dialogs::current(cx).map(|d| d.read(cx).spec().lines.clone()))
        .unwrap();
    assert!(lines.iter().any(|l| l.starts_with(".env")), "{lines:?}");

    // A stray Enter answers nothing.
    cx.update_window(window, |_, w, cx| {
        w.dispatch_keystroke(gpui_kit::Keystroke::parse("enter").unwrap(), cx)
    })
    .unwrap();
    cx.run_until_parked();
    assert!(dialog_open(cx, window), "no answer is highlighted yet");

    answer(cx, window, 1);
    assert!(!dialog_open(cx, window));
    cx.run_until_parked();
    assert!(
        staged_names(&repo).is_empty(),
        "cancelling stages nothing, not even the ordinary file"
    );

    run(cx, window, "git_prefix.stage_file");
    wait_until(cx, "the question again", |cx| dialog_open(cx, window));
    answer(cx, window, 0);
    wait_until(cx, "both staged", |_| {
        let mut n = staged_names(&repo);
        n.sort();
        n == [".env", "ok.txt"]
    });
}

#[gpui_kit::test]
fn closing_the_question_without_answering_stages_nothing(cx: &mut TestAppContext) {
    let repo = base().build();
    std::fs::write(repo.root().join(".env"), "TOKEN=abc\n").unwrap();
    let (window, ws) = open_dirty(cx, &repo, 1, 0);
    click(cx, &ws, StatusSection::Unstaged, 0, NONE);
    run(cx, window, "git_prefix.stage_file");
    wait_until(cx, "the question", |cx| dialog_open(cx, window));
    cx.update_window(window, |_, w, cx| w.close_dialog(cx))
        .unwrap();
    cx.run_until_parked();
    assert!(!dialog_open(cx, window));
    assert!(staged_names(&repo).is_empty());
}

#[gpui_kit::test]
fn the_commands_are_available_where_the_terminal_ui_offers_them(cx: &mut TestAppContext) {
    use gitpoulpi_desktop::commands::context::window_state;
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    let state = |cx: &mut TestAppContext| {
        cx.update_window(window, |_, w, cx| window_state(w, cx))
            .unwrap()
    };
    assert!(!state(cx).unstaged_selected, "nothing selected yet");
    click(cx, &ws, StatusSection::Unstaged, 0, NONE);
    let s = state(cx);
    assert!(s.unstaged_selected && !s.staged_selected && s.has_wip);
    let ctx = s.palette_context();
    assert!(ctx.unstaged_file_selected && !ctx.in_stageable_diff);
}

fn press(cx: &mut TestAppContext, window: AnyWindowHandle, keys: &'static str) {
    for key in keys.split(' ') {
        cx.update_window(window, |_, w, cx| {
            w.dispatch_keystroke(gpui_kit::Keystroke::parse(key).unwrap(), cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

#[gpui_kit::test]
fn the_keyboard_walks_selects_stages_and_opens_the_diff(cx: &mut TestAppContext) {
    let repo = dirty();
    let (window, ws) = open_dirty(cx, &repo, 3, 0);
    run(cx, window, "palette.focus_status");
    press(cx, window, "down down");
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws
            .right
            .read(cx)
            .selected_paths(StatusSection::Unstaged)),
        vec![std::path::PathBuf::from("b.txt")],
        "Down walks to the second row"
    );
    press(cx, window, "shift-down");
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).selection_len()),
        2
    );
    press(cx, window, "escape");
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).selection_len()),
        0
    );
    press(cx, window, "ctrl-space");
    wait_until(cx, "the row staged by Ctrl+Space", |_| {
        staged_names(&repo) == ["n.txt"]
    });
    // Enter opens the cursor row's diff.
    run(cx, window, "palette.focus_status");
    press(cx, window, "enter");
    wait_until(cx, "the diff of the cursor row", |cx| {
        ws.read_with(cx, |ws, _| ws.diff_is_open())
    });
}

#[gpui_kit::test]
fn ctrl_enter_in_the_summary_commits_and_c_focuses_it(cx: &mut TestAppContext) {
    let repo = dirty();
    repo.git(&["add", "b.txt"]);
    let (window, ws) = open_dirty(cx, &repo, 2, 1);
    stage_and_message(cx, window, &ws, "Keyboard commit", "");
    run(cx, window, "palette.focus_status");
    press(cx, window, "c");
    let focused = cx
        .update_window(window, |_, w, cx| {
            ws.read(cx).right.read(cx).composer_focused(w, cx)
        })
        .unwrap();
    assert!(focused, "c puts the caret in the summary");
    press(cx, window, "ctrl-enter");
    wait_until(cx, "the commit", |_| {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "Keyboard commit"
    });
}

#[gpui_kit::test]
fn tree_mode_folds_and_unfolds_the_folders(cx: &mut TestAppContext) {
    let repo = base()
        .commit_file("src/x.txt", "x\n", "add x")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("src/x.txt"), "x changed\n").unwrap();
    std::fs::write(repo.root().join("top.txt"), "t\n").unwrap();
    let (window, ws) = open_dirty(cx, &repo, 2, 0);
    let shown = |cx: &mut TestAppContext| {
        ws.read_with(cx, |ws, cx| {
            ws.right.read(cx).row_paths(StatusSection::Unstaged)
        })
    };
    assert_eq!(shown(cx), vec!["src/x.txt", "top.txt"]);
    run(cx, window, "status.toggle_file_mode");
    assert!(ws.read_with(cx, |ws, cx| ws.right.read(cx).is_tree()));
    run(cx, window, "status.collapse_all_folders");
    assert_eq!(
        shown(cx),
        vec!["top.txt"],
        "the folder's file is folded away"
    );
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.right.read(cx).folded_count()),
        1
    );
    run(cx, window, "status.expand_all");
    assert_eq!(shown(cx), vec!["src/x.txt", "top.txt"]);
    // z folds everything, and unfolds when something is folded.
    run(cx, window, "status.collapse_all");
    assert_eq!(shown(cx), vec!["top.txt"]);
    run(cx, window, "status.collapse_all");
    assert_eq!(shown(cx), vec!["src/x.txt", "top.txt"]);
}
