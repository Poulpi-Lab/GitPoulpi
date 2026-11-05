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
    v.on_key(alt('w'));
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

// ----------------------------------------------- the "What is sent" pane

/// A system prompt of `n` numbered lines, so the open pane must scroll.
fn long_prompt(v: &mut RewriteView, n: usize) {
    let prompt = (0..n)
        .map(|i| format!("PROMPT-LINE-{i:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    v.set_rules_system(Ok(AiSystem {
        prompt,
        via: "Fake agent".into(),
    }));
}

fn sized(v: &RewriteView, w: u16, h: u16) -> (String, Vec<(Rect, RewriteHit)>, Buffer) {
    paint(v, w, h, None)
}

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

#[test]
#[cfg(feature = "ai")]
fn what_is_sent_is_collapsed_by_default_and_the_dialog_is_shorter_for_it() {
    let (mut v, _q, _t) = open_dialog();
    long_prompt(&mut v, 60);
    let (text, hits, _) = sized(&v, W, H);
    assert!(text.contains("\u{25B8} What is sent"), "{text}");
    assert!(text.contains("Alt+W: show or hide"), "{text}");
    assert!(!text.contains("PROMPT-LINE-000"), "{text}");
    assert!(!hits.iter().any(|(_, h)| *h == RewriteHit::RaSent));
    assert!(!hits.iter().any(|(_, h)| *h == RewriteHit::RaSentBar));
    let collapsed = rect_of(&hits, RewriteHit::RaPanel).height;
    v.on_key(alt('w'));
    let (text, hits, _) = sized(&v, W, H);
    assert!(text.contains("\u{25BE} What is sent"), "{text}");
    assert!(text.contains("PROMPT-LINE-000"), "{text}");
    let open = rect_of(&hits, RewriteHit::RaPanel).height;
    assert!(collapsed < open, "collapsed {collapsed} < open {open}");
    assert!(open <= H - 4, "the dialog stays on the screen");
    v.on_key(alt('w'));
    let (text, _, _) = sized(&v, W, H);
    assert!(!text.contains("PROMPT-LINE-000"), "{text}");
}

/// The pane's geometry at one screen size: its own bar on its right edge,
/// full height, clear of the text, tinted apart from the fields.
#[cfg(feature = "ai")]
fn check_pane(w: u16, h: u16) {
    let (mut v, _q, _t) = open_dialog();
    long_prompt(&mut v, 80);
    v.on_key(alt('w'));
    let (text, hits, buf) = sized(&v, w, h);
    let pane = rect_of(&hits, RewriteHit::RaSent);
    let bar = rect_of(&hits, RewriteHit::RaSentBar);
    let panel = rect_of(&hits, RewriteHit::RaPanel);
    assert!(pane.height >= 4, "{w}x{h}\n{text}");
    assert_eq!(bar.x, pane.right() - 1, "{w}x{h}: the pane's right edge");
    assert_eq!((bar.y, bar.height), (pane.y, pane.height), "{w}x{h}");
    assert!(bar.x < panel.right() - 1, "{w}x{h}: inside the border");
    assert_eq!(buf[(bar.x, bar.y)].symbol(), "\u{2191}", "{w}x{h}");
    assert_eq!(
        buf[(bar.x, bar.bottom() - 1)].symbol(),
        "\u{2193}",
        "{w}x{h}"
    );
    let thumb = (bar.y..bar.bottom())
        .filter(|y| buf[(bar.x, *y)].bg == th().scrollbar.thumb)
        .count();
    assert!(thumb >= 1, "{w}x{h}: a thumb is drawn\n{text}");
    // Text never reaches the bar column nor the cell before it.
    for y in pane.y..pane.bottom() {
        assert_eq!(
            buf[(bar.x - 1, y)].symbol(),
            " ",
            "{w}x{h}: text under the bar"
        );
    }
    // The tint: not the dialog's, not a field's.
    let tint = buf[(pane.x + 1, pane.y)].bg;
    assert_ne!(tint, th().ui.panel, "{w}x{h}");
    assert_ne!(tint, crate::input_bg(false, false), "{w}x{h}");
    assert_ne!(tint, crate::input_bg(true, false), "{w}x{h}");
    for y in pane.y..pane.bottom() {
        assert_eq!(buf[(pane.x, y)].bg, tint, "{w}x{h}: the whole pane");
    }
}

#[test]
#[cfg(feature = "ai")]
fn the_open_pane_has_its_own_scrollbar_at_a_small_and_a_large_size() {
    check_pane(60, 26);
    check_pane(90, 30);
    check_pane(200, 70);
}

#[test]
fn the_tint_differs_from_the_dialog_and_the_fields_in_every_theme() {
    for spec in gitpoulpi_config::theme::presets() {
        let theme = crate::theme::Theme::from(&spec);
        let _guard = crate::theme::override_theme(&theme);
        let tint = super::super::rules_ai::sent_bg_for_test();
        assert_ne!(tint, theme.ui.panel, "{}", spec.slug);
        assert_ne!(tint, theme.ui.text, "{}", spec.slug);
        // The themes that follow the terminal's own palette have only ANSI
        // tones to pick from: there the pane shares the field's tone.
        if matches!(theme.ui.panel, ratatui::style::Color::Rgb(..)) {
            assert_ne!(tint, crate::input_bg(false, false), "{}", spec.slug);
            assert_ne!(tint, crate::input_bg(true, false), "{}", spec.slug);
        }
    }
}

#[test]
#[cfg(feature = "ai")]
fn the_keys_toggle_the_pane_and_scroll_it() {
    let (mut v, _q, _t) = open_dialog();
    long_prompt(&mut v, 80);
    // Tab reaches the header; Enter opens it.
    for _ in 0..2 {
        v.on_key(key(KeyCode::Tab));
    }
    assert!(!screen(&v).contains("PROMPT-LINE-000"));
    v.on_key(key(KeyCode::Enter));
    let s = screen(&v);
    assert!(s.contains("PROMPT-LINE-000"), "{s}");
    for _ in 0..3 {
        v.on_key(key(KeyCode::Down));
    }
    let s = screen(&v);
    assert!(
        !s.contains("PROMPT-LINE-000") && s.contains("PROMPT-LINE-002"),
        "{s}"
    );
    v.on_key(key(KeyCode::PageDown));
    let s = screen(&v);
    assert!(!s.contains("PROMPT-LINE-002"), "{s}");
    v.on_key(key(KeyCode::End));
    assert!(screen(&v).contains("PROMPT-LINE-079"));
    v.on_key(key(KeyCode::Home));
    assert!(screen(&v).contains("PROMPT-LINE-000"));
    // Enter hides it again.
    v.on_key(key(KeyCode::Enter));
    assert!(!screen(&v).contains("PROMPT-LINE-000"));
}

#[test]
#[cfg(feature = "ai")]
fn the_mouse_toggles_wheels_and_drags_the_pane_scrollbar() {
    use crossterm::event::{MouseButton, MouseEventKind};
    let (mut v, _q, _t) = open_dialog();
    long_prompt(&mut v, 80);
    let (_, hits, _) = sized(&v, W, H);
    let head = rect_of(&hits, RewriteHit::RaSentHead);
    v.on_press(RewriteHit::RaSentHead, head.x + 3, head.y);
    let (s, _, _) = sized(&v, W, H);
    assert!(s.contains("PROMPT-LINE-000"), "{s}");
    v.on_rules_ai_wheel(RewriteHit::RaSent, 3);
    let s = screen(&v);
    assert!(
        !s.contains("PROMPT-LINE-000") && s.contains("PROMPT-LINE-003"),
        "{s}"
    );
    // Drag the thumb to the bottom of the track.
    let (_, hits, buf) = sized(&v, W, H);
    let bar = rect_of(&hits, RewriteHit::RaSentBar);
    let thumb_y = (bar.y..bar.bottom())
        .find(|y| buf[(bar.x, *y)].bg == th().scrollbar.thumb)
        .expect("a thumb");
    v.on_press(RewriteHit::RaSentBar, bar.x, thumb_y);
    let mouse = |kind, row| crossterm::event::MouseEvent {
        kind,
        column: bar.x,
        row,
        modifiers: KeyModifiers::NONE,
    };
    let below = bar.bottom() + 3;
    v.on_rules_ai_bar_mouse(mouse(MouseEventKind::Drag(MouseButton::Left), below));
    v.on_rules_ai_bar_mouse(mouse(MouseEventKind::Up(MouseButton::Left), below));
    let s = screen(&v);
    assert!(s.contains("PROMPT-LINE-079"), "{s}");
    // A click on the track above the thumb jumps back up.
    v.on_press(RewriteHit::RaSentBar, bar.x, bar.y + 1);
    v.on_rules_ai_bar_mouse(mouse(MouseEventKind::Up(MouseButton::Left), bar.y + 1));
    assert!(!screen(&v).contains("PROMPT-LINE-079"));
}

#[test]
fn the_description_gets_a_scrollbar_when_the_text_outgrows_the_field() {
    let (mut v, _q, _t) = open_dialog();
    let (_, hits, _) = sized(&v, W, H);
    assert!(!hits.iter().any(|(_, h)| *h == RewriteHit::RaTextBar));
    for i in 0..9 {
        chars(&mut v, &format!("line {i}"));
        v.on_key(key(KeyCode::Enter));
    }
    let (_, hits, buf) = sized(&v, W, H);
    let field = rect_of(&hits, RewriteHit::RaText);
    let bar = rect_of(&hits, RewriteHit::RaTextBar);
    assert_eq!(bar.x, field.right() - 1);
    assert_eq!((bar.y, bar.height), (field.y, field.height));
    assert_eq!(buf[(bar.x, bar.y)].symbol(), "\u{2191}");
    v.on_rules_ai_wheel(RewriteHit::RaText, -3);
}
