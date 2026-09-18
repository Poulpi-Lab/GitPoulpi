//! ✦ Create Rules with AI (TUI): the button, the dialog that shows what is
//! sent, the answer that fills the workbench, and the guarantee that nothing
//! is previewed or applied on the way. The window's side (the worker, the
//! model) is put in by hand: here the dialog's wishes are read and answered.

use super::render::{paint, rect_of};
use super::*;
use crate::rewrite_view::{
    AiSystem, AiWish, ProposalProblem, ProposalSettings, RulesCtx, RulesProposal, SampleRow,
};
use crate::th;
use gitpoulpi_core::domain::rewrite::{CommitterPolicy, Rule, RuleKind, Scope};

const W: u16 = 160;
const H: u16 = 50;

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn screen(v: &RewriteView) -> String {
    paint(v, W, H, None).0
}

fn remove(path: &str) -> Rule {
    Rule::new(RuleKind::RemovePaths {
        globs: vec![path.into()],
    })
}

fn proposal(rules: Vec<Rule>) -> RulesProposal {
    RulesProposal {
        rules,
        ..RulesProposal::default()
    }
}

/// A workbench with AI on, authors to offer, and the dialog open and
/// described by the window.
fn open_dialog() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    v.scope_ctx.sample = vec![SampleRow {
        author_name: "Ada Lovelace".into(),
        author_email: "ada@secret.example".into(),
        timestamp: 1_700_000_000,
        subject: "fix: the parser".into(),
    }];
    v.set_ai(true, false);
    v.open_rules_ai();
    assert_eq!(v.take_ai_wish(), Some(AiWish::DescribeRules));
    v.set_rules_system(Ok(AiSystem {
        prompt: "You prepare a recipe.\nVOCABULARY-LINE".into(),
        via: "Fake agent".into(),
    }));
    (v, q, t)
}

/// Nothing was previewed, applied or asked of the daemon.
fn assert_nothing_ran(v: &RewriteView) {
    assert!(matches!(v.run, Run::Idle), "no job was started");
    assert!(v.plan.is_none(), "no preview");
    assert!(!v.can_apply(), "Apply stays unavailable");
    assert!(!v.busy());
}

#[test]
fn with_ai_off_there_is_no_button_and_the_key_says_why() {
    let (mut v, _q, _t) = view();
    let s = screen(&v);
    assert!(!s.contains("Create Rules with AI"), "{s}");
    v.on_key(alt('i'));
    assert!(!v.rules_ai_open());
    assert_eq!(v.take_ai_wish(), None, "nothing is asked of the window");
    assert!(
        v.notice
            .as_ref()
            .is_some_and(|n| n.text.contains("turned off")),
        "it says why: {:?}",
        v.notice
    );
}

#[test]
fn the_button_carries_the_sparkle_and_opens_the_dialog_that_only_describes() {
    let (mut v, _q, _t) = view();
    v.set_ai(true, false);
    let (text, hits, buf) = paint(&v, W, H, None);
    assert!(text.contains("\u{2726} Create Rules with AI..."), "{text}");
    let r = rect_of(&hits, RewriteHit::RulesAi);
    let spark = (r.x..r.right())
        .find(|x| buf[(*x, r.y)].symbol() == crate::theme::AI_SPARK)
        .expect("the sparkle is on the button");
    assert_eq!(buf[(spark, r.y)].fg, th().status.ai);
    assert_eq!(v.take_ai_wish(), None, "showing it asks nothing");
    v.on_press(RewriteHit::RulesAi, r.x + 2, r.y);
    assert!(v.rules_ai_open());
    assert_eq!(v.take_ai_wish(), Some(AiWish::DescribeRules));
    assert_nothing_ran(&v);
}

#[test]
fn the_keyboard_twin_opens_the_dialog() {
    let (mut v, _q, _t) = view();
    v.set_ai(true, false);
    v.on_key(alt('i'));
    assert!(v.rules_ai_open());
}

#[test]
fn a_running_request_refuses_the_dialog() {
    let (mut v, _q, _t) = view();
    v.set_ai(true, true);
    v.open_rules_ai();
    assert!(!v.rules_ai_open());
    assert_eq!(v.take_ai_wish(), None);
}

#[test]
#[cfg(feature = "ai")]
fn the_dialog_shows_exactly_what_is_sent_and_the_context_is_off() {
    let (mut v, _q, _t) = open_dialog();
    chars(&mut v, "remove secrets.txt");
    let s = screen(&v);
    assert!(s.contains("What is sent"), "{s}");
    assert!(s.contains("VOCABULARY-LINE"), "the system prompt\n{s}");
    assert!(s.contains("What the history rewrite must do:"), "{s}");
    assert!(s.contains("remove secrets.txt"), "{s}");
    assert!(s.contains("Sent to: Fake agent"), "{s}");
    assert!(s.contains("[ ] Send the scope"), "off by default\n{s}");
    for private in ["Ada", "secret.example", "the current branch", "dev"] {
        assert!(!s.contains(private), "{private:?} is shown as sent\n{s}");
    }
    let hits = paint(&v, W, H, None).1;
    let r = rect_of(&hits, RewriteHit::RaContext);
    v.on_press(RewriteHit::RaContext, r.x, r.y);
    let s = screen(&v);
    assert!(s.contains("[x] Send the scope"), "{s}");
    assert!(s.contains("Ada Lovelace <ada@secret.example>"), "{s}");
    assert!(s.contains("current branch: main"), "{s}");
}

#[test]
fn enter_is_a_new_line_and_ctrl_s_sends() {
    let (mut v, _q, _t) = open_dialog();
    // Nothing typed: refused, nothing asked.
    v.on_key(ctrl('s'));
    assert_eq!(v.take_ai_wish(), None);
    chars(&mut v, "line one");
    v.on_key(key(KeyCode::Enter));
    chars(&mut v, "line two");
    assert_eq!(v.take_ai_wish(), None, "Enter does not send");
    v.on_key(ctrl('s'));
    assert_eq!(
        v.take_ai_wish(),
        Some(AiWish::SendRules {
            description: "line one\nline two".into(),
            context: RulesCtx::default(),
        }),
        "the context stays empty until the box is ticked"
    );
}

#[test]
fn ctrl_enter_sends_too_and_the_ticked_context_goes_along() {
    let (mut v, _q, _t) = open_dialog();
    chars(&mut v, "go");
    v.on_key(key(KeyCode::Tab));
    v.on_key(key(KeyCode::Char(' ')));
    v.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL));
    let Some(AiWish::SendRules { context, .. }) = v.take_ai_wish() else {
        panic!("sent");
    };
    assert_eq!(context.current_branch.as_deref(), Some("main"));
    assert_eq!(context.refs, ["main", "dev", "v1"]);
    assert_eq!(
        context.authors,
        [("Ada Lovelace".to_string(), "ada@secret.example".to_string())]
    );
}

#[test]
fn escape_cancels_the_dialog_and_a_running_request() {
    let (mut v, _q, _t) = open_dialog();
    v.on_key(key(KeyCode::Esc));
    assert!(!v.rules_ai_open());
    assert_eq!(v.take_ai_wish(), None, "nothing was running");

    let (mut v, _q, _t) = open_dialog();
    chars(&mut v, "go");
    v.on_key(ctrl('s'));
    assert!(matches!(v.take_ai_wish(), Some(AiWish::SendRules { .. })));
    v.on_key(key(KeyCode::Esc));
    assert_eq!(v.take_ai_wish(), Some(AiWish::Cancel));
    assert!(v.rules_ai_open(), "the dialog waits for the cancel");
    v.rules_ai_stopped("canceled".into());
    assert!(s_contains(&v, "canceled"));
}

fn s_contains(v: &RewriteView, needle: &str) -> bool {
    screen(v).contains(needle)
}

#[test]
fn the_cross_closes_the_dialog() {
    let (mut v, _q, _t) = open_dialog();
    let hits = paint(&v, W, H, None).1;
    let r = rect_of(&hits, RewriteHit::RaClose);
    v.on_press(RewriteHit::RaClose, r.x, r.y);
    assert!(!v.rules_ai_open());
}

#[test]
fn the_answer_fills_the_workbench_and_nothing_runs() {
    let (mut v, _q, _t) = open_dialog();
    // A preview on screen goes out of date, like after any edit.
    v.plan = Some(plan(0, &[], false));
    v.plan_state = PlanState::Current;
    v.put_ai_rules(RulesProposal {
        rules: vec![remove("secrets.txt"), remove("*.key")],
        scope: Some(Scope::AllLocal),
        settings: ProposalSettings {
            include_tags: Some(false),
            committer: Some(CommitterPolicy::MeNow),
            ..ProposalSettings::default()
        },
        ..RulesProposal::default()
    });
    assert_eq!(v.recipe.rules.len(), 2);
    assert_eq!(v.recipe.scope, Scope::AllLocal);
    assert!(!v.recipe.scope_options.include_tags);
    assert_eq!(v.recipe.options.committer, CommitterPolicy::MeNow);
    assert_eq!(v.plan_state, PlanState::OutOfDate);
    assert!(matches!(v.run, Run::Idle), "no job was started");
    assert!(!v.can_apply(), "Apply needs a fresh Preview");
    assert!(!v.rules_ai_open(), "nothing left out: the dialog closes");
    assert!(
        v.notice
            .as_ref()
            .is_some_and(|n| n.text.contains("Rules prepared by the AI: 2")),
        "{:?}",
        v.notice
    );
    let s = screen(&v);
    assert!(s.contains("secrets.txt") || s.contains("Remove"), "{s}");
}

#[test]
fn rules_are_added_after_the_current_ones_unless_replace_is_ticked() {
    let (mut v, _q, _t) = view();
    v.set_ai(true, false);
    v.recipe.rules.push(remove("old.txt"));
    v.open_rules_ai();
    v.take_ai_wish();
    v.put_ai_rules(proposal(vec![remove("new.txt")]));
    assert_eq!(v.recipe.rules.len(), 2, "added after");
    assert_eq!(v.rules.cursor, 1, "the first new rule is selected");

    let (mut v, _q, _t) = view();
    v.set_ai(true, false);
    v.recipe.rules.push(remove("old.txt"));
    v.open_rules_ai();
    v.take_ai_wish();
    // The replace box exists only because rules exist: Tab, Tab, Space.
    v.on_key(key(KeyCode::Tab));
    v.on_key(key(KeyCode::Tab));
    v.on_key(key(KeyCode::Char(' ')));
    v.put_ai_rules(proposal(vec![remove("new.txt")]));
    assert_eq!(v.recipe.rules.len(), 1, "replaced");
    assert!(matches!(
        &v.recipe.rules[0].kind,
        RuleKind::RemovePaths { globs } if globs == &["new.txt".to_string()]
    ));
}

#[test]
fn entries_that_failed_are_listed_with_why_and_the_valid_rules_are_kept() {
    let (mut v, _q, _t) = open_dialog();
    v.put_ai_rules(RulesProposal {
        rules: vec![remove("a.txt")],
        problems: vec![ProposalProblem {
            position: 2,
            what: "Zero shift".into(),
            reason: "a shift of zero changes nothing".into(),
        }],
        notes: vec!["I cannot limit it to March.".into()],
        ..RulesProposal::default()
    });
    assert_eq!(v.recipe.rules.len(), 1, "the valid one is in");
    assert!(v.rules_ai_open(), "the list stays to be read");
    let s = screen(&v);
    assert!(s.contains("Left out"), "{s}");
    assert!(
        s.contains("2. Zero shift: a shift of zero changes nothing"),
        "{s}"
    );
    assert!(s.contains("I cannot limit it to March."), "{s}");
    assert!(s.contains("Nothing is previewed or applied"), "{s}");
    assert_nothing_ran(&v);
    v.on_key(key(KeyCode::Enter));
    assert!(!v.rules_ai_open());
}

#[test]
fn an_answer_with_no_usable_rule_says_so_and_changes_no_rule() {
    let (mut v, _q, _t) = open_dialog();
    v.put_ai_rules(RulesProposal {
        problems: vec![ProposalProblem {
            position: 1,
            what: "x".into(),
            reason: "no".into(),
        }],
        ..RulesProposal::default()
    });
    assert!(v.recipe.rules.is_empty());
    assert!(s_contains(&v, "The AI proposed no usable rule."));
}

#[test]
fn a_scope_naming_unknown_refs_keeps_the_known_ones_and_reports_the_rest() {
    let (mut v, _q, _t) = open_dialog();
    v.put_ai_rules(RulesProposal {
        rules: vec![remove("a.txt")],
        scope: Some(Scope::Refs {
            refs: vec!["dev".into(), "ghost".into()],
        }),
        ..RulesProposal::default()
    });
    assert_eq!(
        v.recipe.scope,
        Scope::Refs {
            refs: vec!["dev".into()]
        }
    );
    assert!(s_contains(&v, "no branch or tag named ghost"));
}

#[test]
fn an_answer_arriving_after_the_dialog_closed_still_fills_without_running() {
    let (mut v, _q, _t) = open_dialog();
    v.on_key(key(KeyCode::Esc));
    v.put_ai_rules(proposal(vec![remove("a.txt")]));
    assert_eq!(v.recipe.rules.len(), 1);
    assert_nothing_ran(&v);
}
