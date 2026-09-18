//! The LFS rules' forms, the preview's LFS numbers and the result panel's
//! "Push LFS objects" (slice S21, TUI). The daemon's answers are put in by
//! hand.

use super::render::{paint, rect_of, text};
use super::*;
use gitpoulpi_core::domain::rewrite::{LfsExportRule, LfsImportRule, PathChange, RewriteNote};

const W: u16 = 120;

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

#[test]
fn the_import_form_has_the_patterns_and_an_optional_minimum_size() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::LfsImport);
    set(&mut v, FieldId::Patterns, "*.psd, assets/**");
    let s = text(&v);
    for want in [
        "Files to move to LFS",
        "Only files of at least",
        "empty: every matching file",
        "Needs git-lfs",
    ] {
        assert!(s.contains(want), "{want}\n{s}");
    }
    set(&mut v, FieldId::MinSize, "10 MiB");
    assert_eq!(
        v.recipe.rules[0].kind,
        RuleKind::LfsImport(LfsImportRule {
            patterns: vec!["*.psd".into(), "assets/**".into()],
            min_size: Some(10 << 20),
        })
    );
    // Empty means every matching file.
    set(&mut v, FieldId::MinSize, "");
    assert!(matches!(
        v.recipe.rules[0].kind,
        RuleKind::LfsImport(LfsImportRule { min_size: None, .. })
    ));
    // A glob the engine would refuse is refused beside the field.
    set(&mut v, FieldId::Patterns, "../x");
    assert!(v.detail.fields[field_at(&v, FieldId::Patterns)]
        .error
        .is_some());
}

#[test]
fn the_export_form_has_only_the_patterns() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::LfsExport);
    set(&mut v, FieldId::Patterns, "*.zip");
    assert_eq!(
        v.recipe.rules[0].kind,
        RuleKind::LfsExport(LfsExportRule {
            patterns: vec!["*.zip".into()],
        })
    );
    let s = text(&v);
    assert!(s.contains("Files to bring back"), "{s}");
    assert!(!s.contains("Only files of at least"), "{s}");
}

#[test]
fn the_summary_and_the_rows_say_what_moves_to_lfs() {
    let (mut v, _q, _t) = super::render::previewed();
    let mut plan = v.plan.clone().expect("a plan");
    plan.counts.lfs_blobs = 3;
    plan.counts.lfs_bytes = 12 << 20;
    plan.counts.lfs_objects = 3;
    plan.counts.lfs_restored = 2;
    plan.counts.lfs_run = true;
    v.rows.loaded[0].paths = vec![PathChange::LfsImported {
        path: "art/big.psd".into(),
    }];
    v.rows.loaded[1].paths = vec![PathChange::LfsExported {
        path: "old.zip".into(),
    }];
    v.plan = Some(plan);
    let s = text(&v);
    assert!(
        s.contains("3 files (12 MiB) move to LFS. LFS objects to write: 3."),
        "{s}"
    );
    assert!(
        s.contains("2 LFS pointers become the real files again."),
        "{s}"
    );
    v.tab = PreviewTab::Commits;
    let s = text(&v);
    assert!(s.contains("moves art/big.psd to LFS"), "{s}");
    assert!(s.contains("brings old.zip back from LFS"), "{s}");
}

#[test]
fn after_an_lfs_run_the_result_panel_offers_to_push_the_objects_and_the_note_mentions_lfs() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    let mut plan = super::render::previewed().0.plan.clone().expect("a plan");
    plan.counts.lfs_run = true;
    v.plan = Some(plan);
    v.finish_apply(RewriteOutcome {
        run: RunId("run-1".into()),
        moved: vec![RefChange {
            name: "refs/heads/main".into(),
            old: Oid([1; 20]),
            new: Some(Oid([2; 20])),
            pushed: PushState::OnRemote {
                remote: "origin".into(),
                remote_oid: Oid([1; 20]),
                rewritten_on_remote: 1,
            },
            protected: false,
            included: true,
            checked_out_in: None,
        }],
        commits_changed: 1,
    });
    let (s, hits, _) = paint(&v, W, 40, None);
    assert!(s.contains("Send LFS objects"), "{s}");
    let at = rect_of(&hits, RewriteHit::LfsPush);
    v.on_press(RewriteHit::LfsPush, at.x, at.y);
    assert!(
        v.push.lfs_wait.is_some(),
        "the existing LFS push was requested"
    );

    let note = crate::rewrite_view::push::note_text_with(
        &RewriteNote {
            run: RunId("run-1".into()),
            refs: Vec::new(),
            text: String::new(),
        },
        true,
    );
    // Nothing was pushed in this note, so the LFS paragraph is for a note
    // that has refs.
    assert!(!note.contains("git lfs"), "{note}");
    let with_refs = crate::rewrite_view::push::note_text_with(
        &RewriteNote {
            run: RunId("run-1".into()),
            refs: vec![gitpoulpi_core::domain::rewrite::NoteRef {
                name: "refs/heads/main".into(),
                short: "main".into(),
                kind: gitpoulpi_core::domain::rewrite::NoteRefKind::Branch,
                remote: "origin".into(),
                old: Oid([1; 20]),
                new: Oid([2; 20]),
            }],
            text: String::new(),
        },
        true,
    );
    assert!(with_refs.contains("git lfs install"), "{with_refs}");
    assert!(with_refs.contains("git lfs pull"), "{with_refs}");
}

#[test]
fn a_run_that_did_not_touch_lfs_offers_no_lfs_push() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.finish_apply(RewriteOutcome {
        run: RunId("run-1".into()),
        moved: Vec::new(),
        commits_changed: 1,
    });
    assert!(!v.push.lfs);
    assert!(!text(&v).contains("Send LFS objects"));
}

#[test]
fn missing_lfs_objects_are_listed_and_can_be_fetched_from_the_refusal() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::LfsExport);
    set(&mut v, FieldId::Patterns, "*.zip");
    let oid = "ab".repeat(32);
    v.on_error(&gitpoulpi_core::Error::RewriteRefused(
        RewriteRefusal::LfsObjectsMissing,
        format!("{oid} dist/app.zip"),
    ));
    let (s, hits, _) = paint(&v, W, 40, None);
    for want in [
        "LFS objects the rules need are not in the local LFS store",
        "LFS objects not in the local store:",
        "dist/app.zip",
        "Download LFS objects",
        "Download them from the remote, then preview again.",
    ] {
        assert!(s.contains(want), "{want}\n{s}");
    }
    let at = rect_of(&hits, RewriteHit::LfsFetch);
    v.on_press(RewriteHit::LfsFetch, at.x, at.y);
    assert!(
        v.push.lfs_fetch_wait.is_some(),
        "the existing LFS fetch was requested"
    );
}
