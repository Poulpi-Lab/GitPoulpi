//! The content rules' forms, the preview's content numbers, the blob diff
//! drill-in and the refusals that list what is in the way (slice S9, TUI).

use super::render::{paint, previewed, rect_of, row, text};
use super::*;
/// The focused pane's frame is `ui.border_focused`, not `ui.accent`.
#[test]
fn a_focused_rewrite_pane_is_framed_in_border_focused() {
    let mut theme = crate::theme::Theme::dark();
    theme.ui.accent = ratatui::style::Color::Rgb(1, 2, 3);
    theme.ui.border_focused = ratatui::style::Color::Rgb(231, 223, 0);
    let _guard = crate::theme::override_theme(&theme);
    let area = ratatui::layout::Rect::new(0, 0, 20, 5);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    super::super::render::pane(&mut buf, area, "Plan", true);
    assert_eq!(buf[(0, 0)].fg, theme.ui.border_focused);
}

use crate::daemon_link::Reply;
use gitpoulpi_core::domain::rewrite::{
    BlobChange, BlobChangeKind, BlobHunk, BlobLine, BlobLineKind, PathChange, PruneEmpty,
    REDACTED_LINE,
};
use ratatui::layout::Rect;

const W: u16 = 110;
const H: u16 = 34;

fn field_at(v: &RewriteView, id: FieldId) -> usize {
    v.detail
        .fields
        .iter()
        .position(|f| f.id == id)
        .unwrap_or_else(|| panic!("no {id:?} in {:?}", v.detail.fields))
}

fn set(v: &mut RewriteView, id: FieldId, value: &str) {
    let at = field_at(v, id);
    v.edit_field(at, FieldInput::Text(value.into()));
}

// ------------------------------------------------------------- the forms

#[test]
fn replace_text_shows_why_a_pattern_does_not_compile_beside_the_field() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::ContentReplace);
    let regex = field_at(&v, FieldId::FindRegex);
    v.edit_field(regex, FieldInput::Toggle(true));
    set(&mut v, FieldId::Find, "(unclosed");
    let find = &v.detail.fields[field_at(&v, FieldId::Find)];
    assert_eq!(find.error.as_deref(), Some("unclosed group"));
    // A literal pattern with the same text is fine.
    v.edit_field(regex, FieldInput::Toggle(false));
    assert_eq!(v.detail.fields[field_at(&v, FieldId::Find)].error, None);
    v.edit_field(regex, FieldInput::Toggle(true));
    v.detail.cursor = field_at(&v, FieldId::Find);
    let s = text(&v);
    assert!(s.contains("unclosed group"), "{s}");
    insta::assert_snapshot!("form_replace_text_error", s);
}

#[test]
fn replace_text_lists_its_options_and_the_default_size_limit() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::ContentReplace);
    set(&mut v, FieldId::Find, "foo");
    set(&mut v, FieldId::With, "bar");
    let s = text(&v);
    for label in ["Files", "Look for", "Find", "Match case", "Replace with"] {
        assert!(s.contains(label), "{label}\n{s}");
    }
    assert!(s.contains("default: 50 MiB"), "{s}");
    insta::assert_snapshot!("form_replace_text", s);
    // The limit takes units and refuses nonsense.
    set(&mut v, FieldId::MaxBlob, "2 MiB");
    assert!(matches!(
        &v.recipe.rules[0].kind,
        RuleKind::ContentReplace(c) if c.max_blob_bytes == Some(2 << 20)
    ));
    set(&mut v, FieldId::MaxBlob, "lots");
    assert!(v.detail.fields[field_at(&v, FieldId::MaxBlob)]
        .error
        .is_some());
}

#[test]
fn secrets_are_masked_in_the_field_the_pane_and_the_rows_and_never_scanned() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::ContentReplace);
    let mode = field_at(&v, FieldId::ContentMode);
    v.edit_field(mode, FieldInput::Choice(1));
    assert!(
        !v.detail.fields.iter().any(|f| f.id == FieldId::ScanSecrets),
        "scanning is not offered before S14"
    );
    assert!(matches!(
        &v.recipe.rules[0].kind,
        RuleKind::ContentReplace(c) if matches!(c.find, ContentFind::Secrets { scan: false, .. })
    ));
    // Typing: the screen shows bullets.
    v.focus = Focus::Detail;
    let at = field_at(&v, FieldId::SecretValues);
    v.detail.cursor = at;
    v.activate_field(at);
    chars(&mut v, "hunter2");
    let typing = text(&v);
    assert!(!typing.contains("hunter2"), "{typing}");
    assert!(typing.contains('\u{2022}'), "{typing}");
    v.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    let s = text(&v);
    assert!(!s.contains("hunter2"), "{s}");
    assert!(s.contains("Secret values"), "{s}");
    assert!(!s.contains("Also scan"), "{s}");
    assert!(v.recipe.holds_secrets());
    assert!(!format!("{:?}", v.recipe).contains("hunter2"));
    assert!(!serde_json::to_string(&v.recipe.saved())
        .unwrap()
        .contains("hunter2"));
    insta::assert_snapshot!("form_secrets", s);
}

#[test]
fn a_secret_rule_without_values_asks_for_them_without_mentioning_a_scanner() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::ContentReplace);
    let mode = field_at(&v, FieldId::ContentMode);
    v.edit_field(mode, FieldInput::Choice(1));
    match v.validation() {
        Validation::Invalid { text, .. } => {
            assert!(text.contains("secret value"), "{text}");
            assert!(!text.contains("scanner"), "{text}");
        }
        other => panic!("{other:?}"),
    }
    // A rule the Secrets tab made scans: it goes to the daemon as it is, and
    // an id-only rule needs no value of its own.
    if let RuleKind::ContentReplace(c) = &mut v.recipe.rules[0].kind {
        c.find = ContentFind::Secrets {
            literals: Vec::new(),
            scan: true,
        };
    }
    assert!(v.validation().is_ok());
    assert!(matches!(
        &v.request_recipe().rules[0].kind,
        RuleKind::ContentReplace(c) if matches!(c.find, ContentFind::Secrets { scan: true, .. })
    ));
}

#[test]
fn remove_paths_explains_depth_and_checks_each_pattern() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::RemovePaths);
    set(&mut v, FieldId::Globs, "secrets, *.log");
    let s = text(&v);
    assert!(s.contains("matches at any depth"), "{s}");
    assert!(v.detail.fields[field_at(&v, FieldId::Globs)]
        .error
        .is_none());
    insta::assert_snapshot!("form_remove_paths", s);
    set(&mut v, FieldId::Globs, "docs, ../x");
    let why = v.detail.fields[field_at(&v, FieldId::Globs)]
        .error
        .clone()
        .expect("a bad pattern is refused beside the field");
    assert!(why.contains(".."), "{why}");
    assert!(text(&v).contains(&why));
}

#[test]
fn drop_blobs_over_takes_units_and_at_least_one_byte() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::RemoveBlobsOver);
    set(&mut v, FieldId::Size, "100 MiB");
    assert!(matches!(
        v.recipe.rules[0].kind,
        RuleKind::RemoveBlobsOver { bytes } if bytes == 100 << 20
    ));
    let s = text(&v);
    assert!(s.contains("100 MiB"), "{s}");
    insta::assert_snapshot!("form_remove_blobs", s);
    set(&mut v, FieldId::Size, "0");
    let why = v.detail.fields[field_at(&v, FieldId::Size)]
        .error
        .clone()
        .expect("zero is refused");
    assert!(why.contains("at least 1"), "{why}");
    assert!(matches!(v.validation(), Validation::Invalid { .. }));
    set(&mut v, FieldId::Size, "2 GiB");
    assert!(matches!(
        v.recipe.rules[0].kind,
        RuleKind::RemoveBlobsOver { bytes } if bytes == 2 << 30
    ));
}

#[test]
fn the_empty_commit_option_is_a_run_option_of_the_content_kinds_only() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::RemovePaths);
    set(&mut v, FieldId::Globs, "secrets");
    let at = field_at(&v, FieldId::PruneEmpty);
    assert_eq!(v.recipe.options.prune_empty, PruneEmpty::Auto);
    v.edit_field(at, FieldInput::Choice(1));
    assert_eq!(v.recipe.options.prune_empty, PruneEmpty::Never);
    // The rule itself did not change, the recipe's run option did; the
    // preview on screen no longer matches.
    assert!(text(&v).contains("Keep them"));
    // The Add menu offers the three content kinds.
    let kinds: Vec<_> = RuleTemplate::ALL
        .into_iter()
        .filter(|k| {
            matches!(
                k,
                RuleTemplate::ContentReplace
                    | RuleTemplate::RemovePaths
                    | RuleTemplate::RemoveBlobsOver
            )
        })
        .collect();
    assert_eq!(kinds.len(), 3);
    v.add_rule(RuleTemplate::Identity);
    assert!(!v.detail.fields.iter().any(|f| f.id == FieldId::PruneEmpty));
}

// -------------------------------------------------------------- preview

#[test]
fn the_summary_counts_blobs_and_a_commit_the_rewrite_removes_stands_out() {
    let (mut v, _q, _t) = previewed();
    {
        let p = v.plan.as_mut().unwrap();
        p.counts.commits_dropped = 2;
        p.counts.blobs_changed = 14;
        p.counts.blobs_skipped_binary = 3;
        p.counts.blobs_skipped_large = 1;
    }
    let s = text(&v);
    assert!(s.contains("2 commits are removed"), "{s}");
    assert!(s.contains("14 file versions are rewritten"), "{s}");
    assert!(s.contains("3 binary files were left"), "{s}");
    assert!(s.contains("1 file over the size limit"), "{s}");
    insta::assert_snapshot!("summary_content", s);

    v.tab = PreviewTab::Commits;
    let mut dropped = row(5, None, false);
    dropped.new = None;
    dropped.paths = vec![PathChange::Removed {
        path: "secrets/key.pem".into(),
    }];
    v.rows.loaded = vec![dropped];
    v.rows.total = 1;
    let (s, _, buf) = paint(&v, W, H, None);
    assert!(s.contains("removed, now empty"), "{s}");
    assert!(s.contains("removes secrets/key.pem"), "{s}");
    // The row is in the warning colour, not the plain text colour.
    let y = s.lines().position(|l| l.contains("now empty")).unwrap() as u16;
    let x = s.lines().nth(y as usize).unwrap().find("removed").unwrap() as u16;
    assert_eq!(buf[(x, y)].fg, crate::th().status.warning);
    insta::assert_snapshot!("commits_dropped", s);
}

// --------------------------------------------------------------- refusals

pub(super) fn refused(v: &mut RewriteView, refusal: RewriteRefusal, detail: &str) {
    v.run = Run::Previewing {
        reply: Reply::ready(Err(CoreError::RewriteRefused(refusal, detail.into()))),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
}

#[test]
fn a_dirty_path_refusal_lists_the_files_and_says_what_to_do() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::RemovePaths);
    set(&mut v, FieldId::Globs, "src");
    refused(
        &mut v,
        RewriteRefusal::DirtyPathsTouched,
        "src/a.rs\nsrc/b.rs",
    );
    let s = text(&v);
    assert!(s.contains("uncommitted edits"), "{s}");
    assert!(s.contains("src/a.rs") && s.contains("src/b.rs"), "{s}");
    assert!(s.contains("discard"), "{s}");
    insta::assert_snapshot!("refusal_dirty", s);
    // A change to the rules, or another tab, clears it.
    v.step_tab(1);
    assert!(v.refusal.is_none());
}

#[test]
fn the_other_two_refusals_have_their_own_words() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::RemovePaths);
    set(&mut v, FieldId::Globs, "src");
    refused(
        &mut v,
        RewriteRefusal::CheckedOutElsewhere,
        "refs/heads/dev is checked out in /tmp/wt and its files would change",
    );
    let s = text(&v);
    assert!(s.contains("checked out in another worktree"), "{s}");
    assert!(s.contains("/tmp/wt"), "{s}");
    refused(&mut v, RewriteRefusal::PartialClone, "");
    let s = text(&v);
    assert!(s.contains("partial clone"), "{s}");
    assert!(s.contains("full clone"), "{s}");
}

// ------------------------------------------------------------- the drill

fn change(path: &str, redacted: bool, lines: Vec<(BlobLineKind, &str)>) -> BlobChange {
    BlobChange {
        run: RunId("run-1".into()),
        commit: Oid([1; 20]),
        path: path.into(),
        kind: BlobChangeKind::Modified,
        binary: false,
        redacted,
        old_size: 40,
        new_size: 32,
        hunks: vec![BlobHunk {
            old_start: 1,
            new_start: 1,
            lines: lines
                .into_iter()
                .map(|(kind, text)| BlobLine {
                    kind,
                    text: text.into(),
                })
                .collect(),
        }],
    }
}

/// A previewed workbench on the Commits tab whose first row has two files.
fn with_files() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = previewed();
    v.tab = PreviewTab::Commits;
    v.focus = Focus::Preview;
    v.rows.loaded[0].paths = vec![
        PathChange::Modified {
            path: "config/app.env".into(),
        },
        PathChange::Removed {
            path: "secrets/key.pem".into(),
        },
    ];
    v.rows.loaded[1].paths = Vec::new();
    (v, q, t)
}

fn deliver(v: &mut RewriteView, answer: BlobChange) {
    let cursor = v.drill.as_ref().unwrap().cursor;
    v.drill.as_mut().unwrap().pending = Some((cursor, Reply::ready(Ok(answer))));
    v.poll(None);
}

#[test]
fn enter_on_a_commit_row_opens_its_files_and_asks_for_the_first_diff() {
    let (mut v, _q, _t) = with_files();
    v.on_key(key(KeyCode::Enter));
    let d = v.drill.as_ref().expect("the drill-in opened");
    assert!(d.pending.is_some(), "the first file's diff was asked");
    assert!(matches!(d.diff, DrillDiff::Loading));
    let s = text(&v);
    assert!(s.contains("config/app.env"), "{s}");
    assert!(s.contains("secrets/key.pem"), "{s}");
    assert!(!s.contains("Esc"), "no Esc hint\n{s}");
    deliver(
        &mut v,
        change(
            "config/app.env",
            false,
            vec![
                (BlobLineKind::Context, "A=1"),
                (BlobLineKind::Removed, "TOKEN=abc"),
                (BlobLineKind::Added, "TOKEN=***REMOVED***"),
            ],
        ),
    );
    let s = text(&v);
    assert!(s.contains("- TOKEN=abc"), "{s}");
    assert!(s.contains("+ TOKEN=***REMOVED***"), "{s}");
    assert!(s.contains("@@ -1 +1 @@"), "{s}");
    insta::assert_snapshot!("drill_diff", s);
}

#[test]
fn a_redacted_diff_says_why_the_removed_lines_are_missing() {
    let (mut v, _q, _t) = with_files();
    v.open_drill(0);
    deliver(
        &mut v,
        change(
            "config/app.env",
            true,
            vec![
                (BlobLineKind::Removed, REDACTED_LINE),
                (BlobLineKind::Added, "TOKEN=***REMOVED***"),
            ],
        ),
    );
    let s = text(&v);
    assert!(
        s.contains("removed lines hidden: they held a secret"),
        "{s}"
    );
    assert!(!s.contains(REDACTED_LINE), "{s}");
    insta::assert_snapshot!("drill_redacted", s);
}

#[test]
fn a_commit_without_files_says_so_instead_of_opening() {
    let (mut v, _q, _t) = with_files();
    v.on_key(key(KeyCode::Down));
    v.on_key(key(KeyCode::Enter));
    assert!(v.drill.is_none());
    assert!(v.notice.as_ref().unwrap().text.contains("changes no file"));
}

#[test]
fn the_keyboard_walks_the_files_and_the_diff_and_closes() {
    let (mut v, _q, _t) = with_files();
    v.open_drill(0);
    deliver(&mut v, change("config/app.env", false, vec![]));
    // Down picks the second file and asks for its diff.
    v.on_key(key(KeyCode::Down));
    let d = v.drill.as_ref().unwrap();
    assert_eq!(d.cursor, 1);
    assert!(matches!(d.diff, DrillDiff::Loading));
    // An answer for the file left behind is not shown for this one.
    v.drill.as_mut().unwrap().pending =
        Some((0, Reply::ready(Ok(change("config/app.env", false, vec![])))));
    v.poll(None);
    assert!(matches!(v.drill.as_ref().unwrap().diff, DrillDiff::Loading));
    // Tab moves the keys to the diff; Esc closes the drill-in, not the view.
    v.on_key(key(KeyCode::Tab));
    assert!(v.drill.as_ref().unwrap().in_diff);
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Stay);
    assert!(v.drill.is_none());
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Close);
}

#[test]
fn the_drill_owns_the_pointer_and_mirrors_the_keys() {
    let (mut v, _q, _t) = with_files();
    v.open_drill(0);
    deliver(&mut v, change("config/app.env", false, vec![]));
    let (_, hits, _) = paint(&v, W, H, None);
    // The overlay's targets come first, the shield closes the list of them,
    // so nothing behind the overlay is reachable.
    let shield = hits
        .iter()
        .position(|(_, h)| *h == RewriteHit::DrillShield)
        .expect("a shield");
    assert!(hits[..shield].iter().all(|(_, h)| matches!(
        h,
        RewriteHit::DrillClose | RewriteHit::DrillPath(_) | RewriteHit::DrillDiff
    )));
    let preview_button = rect_of(&hits, RewriteHit::Preview);
    let first_at = |x: u16, y: u16| {
        hits.iter()
            .find(|(r, _)| r.contains((x, y).into()))
            .map(|(_, h)| *h)
    };
    assert_eq!(
        first_at(preview_button.x, preview_button.y),
        Some(RewriteHit::DrillShield),
        "a click behind the overlay closes it, it does not press Preview"
    );
    // A file row is the keyboard's Down.
    let second = rect_of(&hits, RewriteHit::DrillPath(1));
    assert_eq!(
        first_at(second.x + 1, second.y),
        Some(RewriteHit::DrillPath(1))
    );
    v.on_press(RewriteHit::DrillPath(1), second.x, second.y);
    assert_eq!(v.drill.as_ref().unwrap().cursor, 1);
    v.on_press(RewriteHit::DrillDiff, 0, 0);
    assert!(v.drill.as_ref().unwrap().in_diff);
    // The cross and the shield both close it.
    let cross = rect_of(&hits, RewriteHit::DrillClose);
    assert!(cross.width > 0);
    v.on_press(RewriteHit::DrillClose, cross.x, cross.y);
    assert!(v.drill.is_none());
    v.open_drill(0);
    v.on_press(RewriteHit::DrillShield, 0, 0);
    assert!(v.drill.is_none());
    // A press outside the frame closes it too.
    v.open_drill(0);
    assert_eq!(v.outside_press(), RewriteKeyOutcome::Stay);
    assert!(v.drill.is_none());
}

#[test]
fn two_presses_on_a_row_open_its_files_one_does_not() {
    let (mut v, _q, _t) = with_files();
    v.on_press(RewriteHit::Row(0), 0, 0);
    assert!(v.drill.is_none());
    v.on_press(RewriteHit::Row(0), 0, 0);
    assert!(v.drill.is_some());
    v.close_drill();
    // Another row between the two presses is not a double click.
    v.on_press(RewriteHit::Row(0), 0, 0);
    v.on_press(RewriteHit::Row(1), 0, 0);
    v.on_press(RewriteHit::Row(0), 0, 0);
    assert!(v.drill.is_none());
}

#[test]
fn the_diff_scrolls_with_the_wheel_and_the_keys() {
    let (mut v, _q, _t) = with_files();
    v.open_drill(0);
    let lines: Vec<(BlobLineKind, &str)> = (0..80).map(|_| (BlobLineKind::Added, "line")).collect();
    deliver(&mut v, change("config/app.env", false, lines));
    let _ = paint(&v, W, H, None);
    v.drill_scroll(5);
    assert_eq!(v.drill.as_ref().unwrap().scroll, 5);
    v.on_key(key(KeyCode::PageDown));
    assert!(v.drill.as_ref().unwrap().scroll > 5);
    v.on_key(key(KeyCode::Home));
    assert_eq!(v.drill.as_ref().unwrap().scroll, 0);
    let _: Rect = rect_of(&paint(&v, W, H, None).1, RewriteHit::DrillDiff);
}
