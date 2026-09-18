//! The Size tab (slice S18, TUI): the analysis as a job, the three lists and
//! their order, ticking, and the rules "Remove from history" writes.

use super::render::{paint, rect_of, text};
use super::*;
use crate::daemon_link::Reply;
use crate::rewrite_view::size::{SizeSort, SizeView};
use gitpoulpi_core::domain::rewrite::{
    BlobRow, BlobState, DeletedRow, FolderRow, RewritePhase, Scope, SizeAnalysis,
};

const W: u16 = 120;
const H: u16 = 36;
const MIB: u64 = 1024 * 1024;

fn blob(n: u8, path: &str, size: u64, state: BlobState) -> BlobRow {
    BlobRow {
        path: path.into(),
        more_paths: 0,
        blob: Oid([n; 20]),
        size,
        disk_size: size / 2,
        first_commit: Oid([0xa1; 20]),
        last_commit: Oid([0xb2; 20]),
        state,
    }
}

fn analysis() -> SizeAnalysis {
    SizeAnalysis {
        blobs: vec![
            blob(1, "assets/video.bin", 260 * MIB, BlobState::InHead),
            blob(2, "old/dump.sql", 40 * MIB, BlobState::Deleted),
            blob(3, "data/[raw] *.bin", 12 * MIB, BlobState::Replaced),
            blob(4, "build/app.zip", 3 * MIB, BlobState::Deleted),
        ],
        folders: vec![
            FolderRow {
                path: "assets".into(),
                size: 300 * MIB,
                blobs: 5,
                in_head: true,
            },
            FolderRow {
                path: "old".into(),
                size: 41 * MIB,
                blobs: 2,
                in_head: false,
            },
        ],
        deleted: vec![
            DeletedRow {
                path: "old/dump.sql".into(),
                versions: 2,
                size: 41 * MIB,
                first_commit: Oid([0xa1; 20]),
                last_commit: Oid([0xb2; 20]),
            },
            DeletedRow {
                path: "build/app.zip".into(),
                versions: 1,
                size: 3 * MIB,
                first_commit: Oid([0xa1; 20]),
                last_commit: Oid([0xb2; 20]),
            },
        ],
        top: 200,
        commits: 412,
        blobs_total: 1_980,
        bytes_total: 900 * MIB,
        disk_total: 400 * MIB,
        bytes_in_head: 300 * MIB,
        bytes_history_only: 600 * MIB,
        deleted_paths: 2,
        ..SizeAnalysis::default()
    }
}

fn analysed() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    v.tab = PreviewTab::Size;
    v.start_analysis();
    v.size.running.as_mut().unwrap().reply = Reply::ready(Ok(analysis()));
    v.poll(None);
    (v, q, t)
}

fn paths_of(v: &RewriteView) -> Vec<String> {
    v.rows().iter().map(|r| r.path().to_string()).collect()
}

#[test]
fn the_tab_says_what_it_does_before_anything_was_analysed() {
    let (mut v, _q, _t) = view();
    v.tab = PreviewTab::Size;
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("Size"), "{s}");
    assert!(s.contains("Nothing analysed yet"), "{s}");
    assert!(s.contains("never file content"), "{s}");
    rect_of(&hits, RewriteHit::SizeRun);
    insta::assert_snapshot!("idle", s);
}

#[test]
fn the_analysis_is_a_job_labelled_by_what_it_is_with_a_cancel() {
    let (mut v, _q, _t) = view();
    v.tab = PreviewTab::Size;
    v.start_analysis();
    assert!(v.analysing() && v.busy());
    assert!(!v.can_preview(), "one job at a time");
    // The last report of an earlier job is not this one's.
    v.poll(Some((
        3,
        RewriteProgress {
            phase: RewritePhase::Updating,
            done: 9,
            total: 9,
        },
    )));
    assert_eq!(v.size.running.as_ref().unwrap().progress, None);
    v.poll(Some((
        8,
        RewriteProgress {
            phase: RewritePhase::Rewriting,
            done: 120,
            total: 500,
        },
    )));
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("Analysing: 120 / 500"), "{s}");
    assert!(!s.contains("Rewriting:"), "the label is the job's\n{s}");
    rect_of(&hits, RewriteHit::Cancel);
    // Esc stops it before it closes the workbench.
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Stay);
    assert_eq!(v.request_close(), RewriteKeyOutcome::Stay);
}

#[test]
fn a_canceled_analysis_is_a_line_and_a_failed_one_shows_its_reason() {
    let (mut v, _q, _t) = analysed();
    v.start_analysis();
    v.size.running.as_mut().unwrap().reply =
        Reply::ready(Err(gitpoulpi_core::Error::Canceled { job: 8 }));
    v.poll(None);
    assert!(!v.analysing());
    assert_eq!(v.size_len(), 4, "the old rows stay");
    assert!(v
        .notice
        .as_ref()
        .is_some_and(|n| n.text == "Analysis canceled." && !n.error));
    v.start_analysis();
    v.size.running.as_mut().unwrap().reply = Reply::ready(Err(gitpoulpi_core::Error::Backend(
        "repository is shallow".into(),
    )));
    v.poll(None);
    assert!(text(&v).contains("repository is shallow"));
}

#[test]
fn the_files_are_a_master_detail_list_with_totals_and_the_state_of_each() {
    let (v, _q, _t) = analysed();
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(
        s.contains("412 commits walked, 1980 unique file versions, 900 MiB (400 MiB on disk)"),
        "{s}"
    );
    assert!(
        s.contains("300 MiB are in HEAD; 600 MiB are kept only by the history."),
        "{s}"
    );
    assert!(s.contains("260 MiB"), "{s}");
    assert!(s.contains("assets/video.bin"), "{s}");
    assert!(s.contains("in HEAD") && s.contains("deleted") && s.contains("old version"));
    assert!(
        s.contains("First seen in a1a1a1a, last touched in b2b2b2b"),
        "{s}"
    );
    assert!(s.contains("260 MiB, 130 MiB stored on disk"), "{s}");
    for hit in [
        RewriteHit::SizeRow(0),
        RewriteHit::SizeRow(3),
        RewriteHit::SizeRemovePaths,
        RewriteHit::SizeRemoveOver,
        RewriteHit::SizeAll,
        RewriteHit::SizeNone,
        RewriteHit::SizeView(0),
        RewriteHit::SizeView(2),
        RewriteHit::SizeSort(0),
        RewriteHit::SizeSort(2),
    ] {
        rect_of(&hits, hit);
    }
    insta::assert_snapshot!("files", s);
}

/// The order of each list: size (biggest first), path, deleted first.
#[test]
fn the_lists_sort_by_size_by_path_and_deleted_first() {
    let (mut v, _q, _t) = analysed();
    assert_eq!(
        paths_of(&v),
        [
            "assets/video.bin",
            "old/dump.sql",
            "data/[raw] *.bin",
            "build/app.zip"
        ]
    );
    v.set_size_sort(SizeSort::Path);
    assert_eq!(
        paths_of(&v),
        [
            "assets/video.bin",
            "build/app.zip",
            "data/[raw] *.bin",
            "old/dump.sql"
        ]
    );
    v.set_size_sort(SizeSort::DeletedFirst);
    assert_eq!(
        paths_of(&v),
        [
            "old/dump.sql",
            "build/app.zip",
            "assets/video.bin",
            "data/[raw] *.bin"
        ],
        "deleted ones first, each group biggest first"
    );
    v.set_size_view(SizeView::Folders);
    assert_eq!(
        paths_of(&v),
        ["old", "assets"],
        "a deleted folder first too"
    );
    v.set_size_sort(SizeSort::Size);
    assert_eq!(paths_of(&v), ["assets", "old"]);
    v.set_size_view(SizeView::Deleted);
    assert_eq!(paths_of(&v), ["old/dump.sql", "build/app.zip"]);
}

#[test]
fn the_folders_and_the_deleted_files_have_their_own_details() {
    let (mut v, _q, _t) = analysed();
    v.set_size_view(SizeView::Folders);
    let s = text(&v);
    assert!(s.contains("assets/") && s.contains("old/"), "{s}");
    assert!(s.contains("300 MiB in 5 file versions"), "{s}");
    v.set_size_view(SizeView::Deleted);
    let s = text(&v);
    assert!(s.contains("old/dump.sql"), "{s}");
    assert!(s.contains("41.0 MiB in 2 versions"), "{s}");
    insta::assert_snapshot!("deleted", s);
}

#[test]
fn ticked_rows_say_what_they_would_free_and_when() {
    let (mut v, _q, _t) = analysed();
    v.on_press(RewriteHit::SizeRow(1), 0, 0);
    v.on_press(RewriteHit::SizeRow(3), 0, 0);
    assert_eq!(v.ticked_rows(), 2);
    let s = text(&v);
    assert!(s.contains("Remove from history (2)"), "{s}");
    assert!(
        s.contains("The 2 selected rows free up to about 43.0 MiB, and only after a purge"),
        "{s}"
    );
    // The size limit sits just under the smallest ticked file.
    assert_eq!(v.over_limit(), Some(3 * MIB - 1));
    assert!(s.contains("Remove every file over 3.0 MiB"), "{s}");
    assert!(s.contains("That is 4 listed files"), "{s}");
}

#[test]
fn remove_from_history_adds_one_path_rule_with_the_glob_characters_escaped() {
    let (mut v, _q, _t) = analysed();
    v.on_press(RewriteHit::SizeRow(2), 0, 0);
    v.on_press(RewriteHit::SizeRow(1), 0, 0);
    v.on_press(RewriteHit::SizeRemovePaths, 0, 0);
    let RuleKind::RemovePaths { globs } = &v.recipe.rules[0].kind else {
        panic!("{:?}", v.recipe.rules)
    };
    assert_eq!(globs, &["old/dump.sql", "data/[[]raw] [*].bin"]);
    // What the rule matches is exactly the rows' paths.
    let set = gitpoulpi_core::domain::rewrite::GlobSet::new(globs).unwrap();
    assert!(set.is_match("data/[raw] *.bin") && !set.is_match("data/raw x.bin"));
    assert!(v
        .notice
        .as_ref()
        .is_some_and(|n| n.text == "Rule 1 will remove 2 paths. Preview, then apply."));
    // Again: nothing new.
    v.on_press(RewriteHit::SizeRemovePaths, 0, 0);
    assert_eq!(v.recipe.rules.len(), 1);
    assert!(v
        .notice
        .as_ref()
        .is_some_and(|n| n.text == "These paths are already in rule 1."));
    // The rule is selected in the workbench, ready to preview.
    assert_eq!(v.rules.cursor, 0);
    assert!(v.can_preview());
}

#[test]
fn remove_every_file_over_writes_a_size_rule_just_under_the_smallest_ticked_file() {
    let (mut v, _q, _t) = analysed();
    // Nothing ticked: a reason, no rule.
    v.on_press(RewriteHit::SizeRemoveOver, 0, 0);
    assert!(v.recipe.rules.is_empty());
    assert!(v.notice.as_ref().is_some_and(|n| n.error));
    v.on_press(RewriteHit::SizeRow(0), 0, 0);
    v.on_press(RewriteHit::SizeRow(2), 0, 0);
    v.on_press(RewriteHit::SizeRemoveOver, 0, 0);
    assert_eq!(
        v.recipe.rules[0].kind,
        RuleKind::RemoveBlobsOver {
            bytes: 12 * MIB - 1
        }
    );
    // A second, lower limit lowers the same rule: one size limit is enough.
    v.on_press(RewriteHit::SizeRow(3), 0, 0);
    v.on_press(RewriteHit::SizeRemoveOver, 0, 0);
    assert_eq!(v.recipe.rules.len(), 1);
    assert_eq!(
        v.recipe.rules[0].kind,
        RuleKind::RemoveBlobsOver { bytes: 3 * MIB - 1 }
    );
    // Only the Files list sets a size limit.
    v.set_size_view(SizeView::Folders);
    v.on_press(RewriteHit::SizeRemoveOver, 0, 0);
    assert!(v.notice.as_ref().is_some_and(|n| n.error));
}

#[test]
fn keys_and_clicks_do_the_same() {
    let (mut by_key, _q1, _t1) = analysed();
    let (mut by_mouse, _q2, _t2) = analysed();
    by_key.focus = Focus::Preview;
    by_key.on_key(key(KeyCode::Down));
    by_key.on_key(key(KeyCode::Char(' ')));
    let (_, hits, _) = paint(&by_mouse, W, H, None);
    let at = rect_of(&hits, RewriteHit::SizeRow(1));
    by_mouse.on_press(RewriteHit::SizeRow(1), at.x, at.y);
    assert_eq!((by_key.ticked_rows(), by_mouse.ticked_rows()), (1, 1));
    by_key.on_key(key(KeyCode::Char('r')));
    by_mouse.on_press(RewriteHit::SizeRemovePaths, 0, 0);
    assert_eq!(by_key.recipe.rules, by_mouse.recipe.rules);
    assert_eq!(by_key.recipe.rules.len(), 1);

    // The size rule: `b` and the button.
    by_key.on_key(key(KeyCode::Char('b')));
    by_mouse.on_press(RewriteHit::SizeRemoveOver, 0, 0);
    assert_eq!(by_key.recipe.rules, by_mouse.recipe.rules);
    assert_eq!(by_key.recipe.rules.len(), 2);

    // The list and the order: `v`, `o` and the buttons.
    by_key.on_key(key(KeyCode::Char('v')));
    by_mouse.on_press(RewriteHit::SizeView(1), 0, 0);
    by_key.on_key(key(KeyCode::Char('o')));
    by_mouse.on_press(RewriteHit::SizeSort(1), 0, 0);
    assert_eq!(by_key.size.view, SizeView::Folders);
    assert_eq!(
        (by_key.size.view, by_key.size.sort),
        (by_mouse.size.view, by_mouse.size.sort)
    );

    // Check all and uncheck all: the chords and the buttons.
    let (mut a, _q3, _t3) = analysed();
    let (mut b, _q4, _t4) = analysed();
    a.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT));
    b.on_press(RewriteHit::SizeAll, 0, 0);
    assert_eq!((a.ticked_rows(), b.ticked_rows()), (4, 4));
    a.on_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::ALT));
    b.on_press(RewriteHit::SizeNone, 0, 0);
    assert_eq!((a.ticked_rows(), b.ticked_rows()), (0, 0));
}

#[test]
fn the_palette_action_opens_the_tab_and_runs_the_analysis() {
    let (mut v, _q, _t) = view();
    assert!(!v.analysing());
    v.on_key(key(KeyCode::F(7)));
    assert!(v.analysing());
    assert_eq!(v.tab, PreviewTab::Size);
    assert_eq!(v.focus, Focus::Preview);
}

#[test]
fn rows_are_stale_after_a_rewrite_or_a_scope_change() {
    let (mut v, _q, _t) = analysed();
    assert!(!text(&v).contains("analyse again"));
    v.recipe.scope = Scope::AllLocal;
    assert!(text(&v).contains("The scope changed since this analysis: analyse again."));
    v.recipe.scope = Scope::default();
    v.finish_apply(RewriteOutcome {
        run: RunId("run-1".into()),
        moved: Vec::new(),
        commits_changed: 1,
    });
    v.tab = PreviewTab::Size;
    assert!(v.size.stale);
}

/// What a click on a row reaches: the first target under the pointer, as the
/// event loop resolves it. The preview pane's own target is registered
/// before the rows, so a row that is not a "fine" target is never hit.
#[test]
fn a_click_on_a_row_reaches_the_row_not_the_pane_behind_it() {
    let (v, _q, _t) = analysed();
    let (_, hits, _) = paint(&v, W, H, None);
    for slot in 0..4 {
        let at = rect_of(&hits, RewriteHit::SizeRow(slot));
        let first = hits
            .iter()
            .find(|(rect, _)| rect.contains((at.x + 12, at.y).into()))
            .map(|(_, hit)| *hit);
        assert_eq!(first, Some(RewriteHit::SizeRow(slot)));
    }
}
