//! The workbench drawn: every screen state as a snapshot, plus the checks a
//! snapshot cannot make (hit targets, hover, order of the targets, the
//! mouse reaching the same methods as the keys).

use super::*;
use gitpoulpi_core::domain::rewrite::{
    CommitChange, FieldChange, RewriteField, RewritePhase, RuleCount,
};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

const W: u16 = 110;
const H: u16 = 34;

pub(super) fn paint(
    v: &RewriteView,
    w: u16,
    h: u16,
    hovered: Option<RewriteHit>,
) -> (String, Vec<(Rect, RewriteHit)>, ratatui::buffer::Buffer) {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|f| hits = v.render(f, Rect::new(0, 0, w, h), hovered))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    (crate::dialog_kit::buffer_text(&buf), hits, buf)
}

pub(super) fn text(v: &RewriteView) -> String {
    paint(v, W, H, None).0
}

pub(super) fn rect_of(hits: &[(Rect, RewriteHit)], hit: RewriteHit) -> Rect {
    hits.iter()
        .find(|(_, h)| *h == hit)
        .unwrap_or_else(|| panic!("no hit rect for {hit:?} in {hits:?}"))
        .0
}

pub(super) fn row(n: u8, field: Option<(&str, &str)>, pushed: bool) -> CommitChange {
    CommitChange {
        old: Oid([n; 20]),
        new: Some(Oid([n + 1; 20])),
        fields: field
            .map(|(old, new)| FieldChange {
                field: RewriteField::AuthorEmail,
                old: old.into(),
                new: new.into(),
            })
            .into_iter()
            .collect(),
        paths: Vec::new(),
        pushed,
    }
}

/// An identity rule, a shift rule, and a preview that came back.
pub(super) fn previewed() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    with_identity_rule(&mut v);
    v.add_rule(RuleTemplate::DateShift);
    v.edit_field(0, FieldInput::Text("+2h".into()));
    let mut p = plan(2, &["refs/heads/main"], true);
    p.counts.commits_walked = 9_800;
    p.counts.commits_changed = 412;
    p.counts.tags_moved = 4;
    p.per_rule = vec![RuleCount {
        rule: 0,
        commits: 412,
    }];
    p.warnings = vec![
        RewriteWarning::SignaturesAffected { count: 31 },
        RewriteWarning::PushedRefs { count: 1 },
    ];
    p.refs[0].included = true;
    v.rows.loaded = vec![
        row(1, Some(("old@example.com", "new@example.org")), true),
        row(3, Some(("old@example.com", "new@example.org")), false),
    ];
    v.rows.total = 412;
    v.plan = Some(p);
    v.plan_state = PlanState::Current;
    (v, q, t)
}

#[test]
fn the_empty_workbench_invites_to_add_a_rule() {
    let (v, _q, _t) = view();
    let s = text(&v);
    assert!(s.contains("Add a rule to start"), "{s}");
    assert!(s.contains("Rewrite history"), "{s}");
    assert!(!s.contains("Esc"), "no Esc hint is drawn\n{s}");
    insta::assert_snapshot!("empty", s);
}

#[test]
fn rules_and_their_fields_are_listed() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.add_rule(RuleTemplate::DateShift);
    v.edit_field(0, FieldInput::Text("+2h".into()));
    v.add_rule(RuleTemplate::MessageReplace);
    v.move_rule_cursor(-2);
    let s = text(&v);
    assert!(s.contains("old@example.com"), "{s}");
    assert!(s.contains("Shift dates"), "{s}");
    insta::assert_snapshot!("rules", s);
}

#[test]
fn a_running_preview_shows_a_progress_bar_and_cancel() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.preview();
    assert!(v.busy());
    v.poll(Some((
        7,
        RewriteProgress {
            phase: RewritePhase::Rewriting,
            done: 6_120,
            total: 9_800,
        },
    )));
    let s = text(&v);
    assert!(s.contains("6120 / 9800"), "{s}");
    assert!(s.contains('\u{2593}') && s.contains('\u{2591}'), "{s}");
    assert!(s.contains("Cancel"), "{s}");
    insta::assert_snapshot!("running", s);
}

#[test]
fn the_summary_counts_and_warns() {
    let (v, _q, _t) = previewed();
    let s = text(&v);
    assert!(s.contains("412 of 9800"), "{s}");
    assert!(s.contains("31 signed"), "{s}");
    assert!(s.contains("changed nothing"), "rule 2 changed nothing\n{s}");
    insta::assert_snapshot!("summary", s);
}

#[test]
fn the_commits_tab_is_a_table_of_what_changes() {
    let (mut v, _q, _t) = previewed();
    v.tab = PreviewTab::Commits;
    let s = text(&v);
    assert!(s.contains("author email: old@example.com"), "{s}");
    assert!(s.contains('\u{25CF}'), "the pushed dot\n{s}");
    insta::assert_snapshot!("commits", s);
}

#[test]
fn the_refs_tab_lists_moves_and_protected_branches() {
    let (mut v, _q, _t) = previewed();
    v.tab = PreviewTab::Refs;
    let s = text(&v);
    assert!(s.contains("Protected branches already pushed"), "{s}");
    assert!(s.contains("[ ] main"), "{s}");
    insta::assert_snapshot!("refs", s);
}

#[test]
fn the_impact_tab_explains_the_force_push() {
    let (mut v, _q, _t) = previewed();
    v.tab = PreviewTab::Impact;
    v.plan.as_mut().unwrap().refs[0].protected = false;
    let s = text(&v);
    assert!(s.contains("force push"), "{s}");
    insta::assert_snapshot!("impact", s);
}

#[test]
fn the_result_panel_offers_undo_and_close() {
    let (mut v, _q, _t) = previewed();
    let moved = v.plan.as_ref().unwrap().refs.clone();
    v.finish_apply(RewriteOutcome {
        run: RunId("run-1".into()),
        moved,
        commits_changed: 412,
    });
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("Rewrote 412"), "{s}");
    rect_of(&hits, RewriteHit::Undo);
    rect_of(&hits, RewriteHit::ResultClose);
    insta::assert_snapshot!("result", s);
}

#[test]
fn a_question_has_no_default_and_covers_the_workbench() {
    let (mut v, _q, _t) = previewed();
    v.question = Some(Question::Signatures {
        count: Some(31),
        cursor: 0,
    });
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("Remove the signatures"), "{s}");
    assert!(s.contains("Re-sign only"), "{s}");
    let answers: Vec<usize> = hits
        .iter()
        .filter_map(|(_, h)| match h {
            RewriteHit::Answer(i) => Some(*i),
            _ => None,
        })
        .collect();
    assert_eq!(answers, vec![0, 1, 2, 3]);
    // The options and the cross come first; the shield swallows the rest, so
    // nothing behind the question is reachable.
    let shield = hits
        .iter()
        .position(|(_, h)| *h == RewriteHit::QuestionShield)
        .unwrap();
    let first_other = hits
        .iter()
        .position(|(_, h)| !matches!(h, RewriteHit::Answer(_) | RewriteHit::QuestionCancel))
        .unwrap();
    assert_eq!(first_other, shield);
    insta::assert_snapshot!("question", s);
}

#[test]
fn a_rule_that_needs_a_missing_tool_is_greyed_with_its_reason() {
    use gitpoulpi_core::domain::rewrite::{OsFamily, RewriteTools, Tool, ToolCheck, ToolStatus};
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::LfsImport);
    let tools = RewriteTools {
        external_scripts: None,
        signing_format: Default::default(),
        tools: vec![ToolCheck::new(
            Tool::GitLfs,
            ToolStatus::Missing,
            OsFamily::Linux,
        )],
    };
    v.tools = ToolsState::Known(tools);
    v.add_rule(RuleTemplate::DateShift);
    assert!(!v.missing_tools(0).is_empty());
    // Cursor on the other rule: the greyed row says what it needs.
    let (s, buf) = {
        let (s, _, buf) = paint(&v, W, H, None);
        (s, buf)
    };
    assert!(s.contains("needs g"), "{s}");
    let line = s
        .lines()
        .position(|l| l.contains("Migrate files to LFS"))
        .unwrap();
    let text_line = s.lines().nth(line).unwrap();
    let col = text_line[..text_line.find("Migrate").unwrap()]
        .chars()
        .count() as u16;
    assert_eq!(buf[(col, line as u16)].fg, crate::th().ui.disabled);
    // Cursor on the greyed rule: the pane says why and offers a new check.
    v.move_rule_cursor(-1);
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("is needed and was not found"), "{s}");
    rect_of(&hits, RewriteHit::CheckTools);
}

#[test]
fn hovering_lights_the_button_and_the_cross() {
    let (v, _q, _t) = previewed();
    let (_, hits, plain) = paint(&v, W, H, None);
    for hit in [RewriteHit::Preview, RewriteHit::Apply, RewriteHit::Close] {
        let r = rect_of(&hits, hit);
        let (_, _, hot) = paint(&v, W, H, Some(hit));
        assert_ne!(
            plain[(r.x + r.width / 2, r.y)],
            hot[(r.x + r.width / 2, r.y)],
            "{hit:?} must change when hovered"
        );
    }
}

#[test]
fn every_target_has_a_glyph_under_it() {
    let (mut v, _q, _t) = previewed();
    v.tab = PreviewTab::Refs;
    for (_, hits, buf) in [paint(&v, W, H, None), paint(&v, 70, 24, None)] {
        for (r, hit) in &hits {
            if matches!(hit, RewriteHit::Pane(_)) {
                continue;
            }
            let blank = (r.x..r.right()).all(|x| {
                buf[(x, r.y)].symbol().trim().is_empty() && buf[(x, r.y)].bg == crate::th().ui.panel
            });
            assert!(!blank, "{hit:?} at {r:?} has nothing drawn under it");
        }
    }
}

#[test]
fn the_cross_closes_but_a_running_preview_is_stopped_first() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    let (_, hits, _) = paint(&v, W, H, None);
    let cross = rect_of(&hits, RewriteHit::Close);
    assert_eq!(
        v.on_press(RewriteHit::Close, cross.x, cross.y),
        RewriteKeyOutcome::Close
    );
    v.preview();
    assert!(v.busy());
    assert_eq!(v.on_press(RewriteHit::Close, 0, 0), RewriteKeyOutcome::Stay);
}

#[test]
fn a_click_outside_keeps_a_recipe_with_rules_and_closes_an_empty_one() {
    let (mut v, _q, _t) = view();
    paint(&v, W, H, None);
    assert_eq!(v.outside_press(), RewriteKeyOutcome::Close);
    with_identity_rule(&mut v);
    assert_eq!(v.outside_press(), RewriteKeyOutcome::Stay);
    assert!(v.paint.flash.get(), "the cross flashes");
}

#[test]
fn the_add_menu_picks_by_click_and_the_rule_gets_its_fields() {
    let (mut v, _q, _t) = view();
    paint(&v, W, H, None);
    v.on_press(RewriteHit::AddRule, 0, 0);
    assert!(matches!(v.popup, Some(Popup::AddRule { .. })));
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(
        s.contains("Replace an author") && s.contains("Authors"),
        "{s}"
    );
    // The dropdown is in front of everything, and its rows are targets.
    assert!(matches!(hits.first().unwrap().1, RewriteHit::PopupRow(_)));
    v.on_press(RewriteHit::PopupRow(0), 0, 0);
    assert!(v.popup.is_none());
    assert_eq!(v.recipe.rules.len(), 1);
    assert_eq!(v.focus, Focus::Detail);
}

#[test]
fn a_choice_field_opens_a_dropdown_instead_of_cycling() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::Identity);
    let at = v
        .detail
        .fields
        .iter()
        .position(|f| matches!(f.value, FieldValue::Choice { .. }))
        .expect("a choice field");
    let before = v.detail.fields[at].value.clone();
    v.on_press(RewriteHit::Field(at), 0, 0);
    assert!(matches!(v.popup, Some(Popup::Choice { .. })));
    assert_eq!(v.detail.fields[at].value, before, "a click does not cycle");
    let (s, _, _) = paint(&v, W, H, None);
    assert!(s.contains("Author and committer"), "{s}");
    v.on_press(RewriteHit::PopupRow(0), 0, 0);
    assert!(v.popup.is_none());
}

#[test]
fn the_scope_dropdown_lists_the_kinds() {
    let (mut v, _q, _t) = view();
    v.on_press(RewriteHit::ScopeCombo, 0, 0);
    let (s, _, _) = paint(&v, W, H, None);
    assert!(s.contains("All local branches"), "{s}");
    v.on_press(RewriteHit::PopupRow(1), 0, 0);
    assert_eq!(v.scope_kind(), ScopeKind::AllLocal);
}

#[test]
fn a_small_window_says_so_instead_of_drawing_a_broken_layout() {
    let (v, _q, _t) = view();
    let (s, hits, _) = paint(&v, 50, 12, None);
    assert!(s.contains("larger"), "{s}");
    assert!(hits.iter().any(|(_, h)| *h == RewriteHit::Close));
}

#[test]
fn the_commits_tab_asks_the_next_page_from_the_window_it_painted() {
    let (mut v, _q, _t) = previewed();
    v.tab = PreviewTab::Commits;
    v.rows.total = 1_000;
    paint(&v, W, H, None);
    v.poll(None);
    assert!(v.rows.pending.is_some(), "poll pages the rows in");
}
