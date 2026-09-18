//! External scripts in the workbench (slice S11, TUI): the language picker
//! with its greyed rows, the "not sandboxed" words, the trust question that
//! has no default, and which scripts the window vouches for. The daemon's
//! answers are put in by hand.

use super::render::text;
use super::script::{answer_check, script_text, script_view, set_script};
use super::*;
use crate::daemon_link::Reply;
use crate::rewrite_view::script::{AfterTrust, CheckView, TestState};
use crate::rewrite_view::tools::{tool_hint, ToolsState};
use gitpoulpi_core::domain::rewrite::script::ScriptCheck;
use gitpoulpi_core::domain::rewrite::{
    script_hash, OsFamily, RewriteTools, ScriptLanguage, ScriptSource, ScriptSwitches, ScriptTrust,
    Tool, ToolCheck, ToolStatus,
};

/// Row of a language in the dropdown (the order of `rules_form::LANGUAGE`).
const LUA: usize = 0;
const SHELL: usize = 1;
const PYTHON: usize = 2;
const LUA_EXTERNAL: usize = 3;

fn installed() -> ToolStatus {
    ToolStatus::Installed {
        version: String::new(),
    }
}

fn known(python: ToolStatus, switches: ScriptSwitches) -> ToolsState {
    let os = OsFamily::current();
    ToolsState::Known(RewriteTools {
        tools: vec![
            ToolCheck::new(Tool::Python, python, os),
            ToolCheck::new(Tool::Lua, installed(), os),
            ToolCheck::new(Tool::Sh, installed(), os),
            ToolCheck::new(Tool::Bash, installed(), os),
            ToolCheck::new(Tool::Pwsh, installed(), os),
        ],
        external_scripts: Some(switches),
        signing_format: Default::default(),
    })
}

fn switches(shell: bool, python: bool, lua: bool) -> ScriptSwitches {
    ScriptSwitches { shell, python, lua }
}

fn language(v: &RewriteView) -> ScriptLanguage {
    v.script_ref().expect("a script rule").language
}

fn pick_language(v: &mut RewriteView, row: usize) {
    v.open_field(0);
    v.popup_pick(row);
}

fn notice(v: &RewriteView) -> String {
    v.notice
        .as_ref()
        .map(|n| n.text.clone())
        .unwrap_or_default()
}

// ------------------------------------------------------------- languages

#[test]
fn a_language_that_is_off_in_the_settings_is_greyed_and_refused_with_its_reason() {
    let (mut v, _q, _t) = script_view();
    v.tools = known(installed(), switches(true, false, false));
    v.open_field(0);
    let rows = v.popup_rows();
    assert_eq!(rows.len(), 4);
    assert!(rows[LUA].disabled.is_none() && rows[SHELL].disabled.is_none());
    assert_eq!(rows[PYTHON].disabled, Some(t::SCRIPT_LANG_OFF));
    assert_eq!(rows[LUA_EXTERNAL].disabled, Some(t::SCRIPT_LANG_OFF));

    v.popup_pick(PYTHON);
    assert_eq!(language(&v), ScriptLanguage::Lua, "the pick was refused");
    let why = notice(&v);
    assert!(
        why.contains("Python")
            && why.contains("\"python\"")
            && why.contains("rewrite.external_scripts"),
        "{why}"
    );

    // Switched on, it can be picked.
    v.tools = known(installed(), switches(true, true, false));
    pick_language(&mut v, PYTHON);
    assert_eq!(language(&v), ScriptLanguage::Python);
}

#[test]
fn a_missing_interpreter_greys_its_language_with_the_install_hint() {
    let (mut v, _q, _t) = script_view();
    v.tools = known(ToolStatus::Missing, switches(true, true, true));
    v.open_field(0);
    assert_eq!(
        v.popup_rows()[PYTHON].disabled,
        Some(t::SCRIPT_LANG_MISSING)
    );
    assert!(v.popup_rows()[LUA_EXTERNAL].disabled.is_none());
    v.popup_pick(PYTHON);
    assert_eq!(language(&v), ScriptLanguage::Lua);
    let why = notice(&v);
    assert!(why.contains("python"), "{why}");
    assert!(
        why.contains(tool_hint(Tool::Python, OsFamily::current()).get()),
        "{why}"
    );
}

#[test]
fn nothing_is_greyed_before_the_tool_check_answers() {
    let (mut v, _q, _t) = script_view();
    v.tools = ToolsState::Unknown;
    v.open_field(0);
    assert!(v.popup_rows().iter().all(|r| r.disabled.is_none()));
    v.popup_pick(PYTHON);
    assert_eq!(language(&v), ScriptLanguage::Python);
}

#[test]
fn an_untouched_script_follows_the_language_and_a_written_one_stays() {
    let (mut v, _q, _t) = script_view();
    v.tools = ToolsState::Unknown;
    assert_eq!(script_text(&v), rules_form::STARTER_SCRIPT);
    pick_language(&mut v, PYTHON);
    assert_eq!(script_text(&v), rules_form::STARTER_PYTHON);
    pick_language(&mut v, LUA_EXTERNAL);
    assert_eq!(script_text(&v), rules_form::STARTER_LUA_EXTERNAL);
    set_script(&mut v, "my own script\n");
    pick_language(&mut v, SHELL);
    assert_eq!(language(&v), ScriptLanguage::Shell);
    assert_eq!(script_text(&v), "my own script\n");
}

#[test]
fn a_shell_script_header_uses_hash_lines_for_its_parameters() {
    let (mut v, _q, _t) = script_view();
    pick_language(&mut v, SHELL);
    set_script(
        &mut v,
        "#!/bin/sh\n#! name: Rename\n#! param who: text \"Name to replace\"\n:\n",
    );
    assert_eq!(v.param_fields().len(), 1);
    assert!(text(&v).contains("Name to replace"));
}

// ------------------------------------------------------------ the words

#[test]
fn an_external_script_says_it_is_not_sandboxed_and_untrusted_until_trusted() {
    let (mut v, _q, _t) = script_view();
    pick_language(&mut v, SHELL);
    let hash = script_hash(&script_text(&v));
    let check = ScriptCheck {
        hash: Some(hash.clone()),
        trust: ScriptTrust::Untrusted,
        ..Default::default()
    };
    let key = v.script_key().unwrap();
    v.script.check.sent(key, Reply::ready(Ok(check)));
    v.poll(None);
    assert_eq!(v.check_view(), CheckView::Untrusted);
    let s = text(&v);
    assert!(s.contains("Script (not sandboxed)"), "{s}");
    assert!(s.contains("Not trusted yet"), "{s}");

    let ok = ScriptCheck {
        hash: Some(hash),
        trust: ScriptTrust::Trusted,
        ..Default::default()
    };
    v.script.check.answer = None;
    v.script.check.sent(key, Reply::ready(Ok(ok)));
    v.poll(None);
    assert_eq!(v.check_view(), CheckView::ExternalOk);

    // The rule's own pane says it too.
    v.close_script_editor();
    assert!(
        text(&v).contains("Not sandboxed: it runs with your rights."),
        "{}",
        text(&v)
    );
}

// ------------------------------------------------------------------ trust

#[test]
fn typed_scripts_are_vouched_for_and_loaded_ones_are_not() {
    let (mut v, _q, _t) = script_view();
    pick_language(&mut v, SHELL);
    // The starter nobody typed, and a text that came from a file or a
    // recipe: not vouched.
    assert!(v.vouched_hashes().is_empty());
    set_script(&mut v, "echo from a shared recipe\n");
    assert!(v.vouched_hashes().is_empty());
    // Typed here: vouched, by content.
    v.on_key(key(KeyCode::Char('x')));
    let typed = script_text(&v);
    assert_eq!(v.vouched_hashes(), vec![script_hash(&typed)]);
    // Changing it by loading something else drops the vouching.
    set_script(&mut v, "echo loaded\n");
    assert!(v.vouched_hashes().is_empty());
    // A Lua script runs in the sandbox: nothing to vouch for.
    pick_language(&mut v, LUA);
    v.on_key(key(KeyCode::Char('y')));
    assert!(v.vouched_hashes().is_empty());
}

fn refuse_untrusted(v: &mut RewriteView, hash: &str) {
    v.run = Run::Previewing {
        reply: Reply::ready(Err(CoreError::RewriteRefused(
            RewriteRefusal::ScriptNotTrusted,
            hash.to_string(),
        ))),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
}

#[test]
fn an_untrusted_script_asks_and_there_is_no_default_answer() {
    let (mut v, _q, _t) = script_view();
    pick_language(&mut v, SHELL);
    let hash = script_hash(&script_text(&v));
    refuse_untrusted(&mut v, &hash);
    assert!(!v.busy());
    assert!(matches!(
        v.question,
        Some(Question::TrustScript {
            then: AfterTrust::Preview,
            cursor: None,
            ..
        })
    ));
    let shown = v.question_view().unwrap();
    assert_eq!(shown.options, vec![t::Q_TRUST_YES, t::Q_TRUST_NO]);
    assert_eq!(shown.cursor, usize::MAX, "nothing is highlighted");
    assert!(shown.prompt.contains(&hash[..12]), "{}", shown.prompt);

    // Enter alone answers nothing.
    v.on_key(key(KeyCode::Enter));
    assert!(v.question.is_some() && v.script.trusting.is_none());
    // Esc leaves it unanswered, and so does the second option.
    v.on_key(key(KeyCode::Esc));
    assert!(v.question.is_none() && v.script.trusting.is_none());
    refuse_untrusted(&mut v, &hash);
    v.on_key(key(KeyCode::Char('2')));
    assert!(v.question.is_none() && v.script.trusting.is_none());
    assert!(!v.busy(), "refusing to trust sends nothing");
}

#[test]
fn trusting_sends_the_trust_and_then_repeats_what_was_refused() {
    let (mut v, _q, _t) = script_view();
    pick_language(&mut v, SHELL);
    let hash = script_hash(&script_text(&v));
    refuse_untrusted(&mut v, &hash);
    // An arrow starts the highlight, Enter then answers it.
    v.on_key(key(KeyCode::Down));
    assert!(matches!(
        v.question,
        Some(Question::TrustScript {
            cursor: Some(0),
            ..
        })
    ));
    v.on_key(key(KeyCode::Enter));
    assert!(v.question.is_none());
    let (_, then) = v.script.trusting.as_ref().expect("the trust was sent");
    assert_eq!(*then, AfterTrust::Preview);

    // The daemon granted it: the preview that was refused runs again.
    v.script.trusting = Some((
        Reply::ready(Ok(serde_json::Value::Null)),
        AfterTrust::Preview,
    ));
    v.poll(None);
    assert!(v.script.trusting.is_none());
    assert!(v.busy(), "the preview is sent again");
}

#[test]
fn a_test_of_an_untrusted_script_asks_too_and_a_refused_trust_says_why() {
    let (mut v, _q, _t) = script_view();
    pick_language(&mut v, SHELL);
    answer_check(&mut v, &[]);
    let hash = script_hash(&script_text(&v));
    v.script.test = TestState::Running {
        reply: Reply::ready(Err(CoreError::RewriteRefused(
            RewriteRefusal::ScriptNotTrusted,
            hash.clone(),
        ))),
        since: Instant::now(),
        count: 2,
    };
    v.poll(None);
    assert!(matches!(
        v.question,
        Some(Question::TrustScript {
            then: AfterTrust::Test,
            ..
        })
    ));
    assert!(matches!(v.script.test, TestState::Idle));

    v.question = None;
    v.script.trusting = Some((
        Reply::ready(Err(CoreError::Io(std::io::Error::other("disk full")))),
        AfterTrust::Test,
    ));
    v.poll(None);
    assert!(notice(&v).contains("disk full"), "{}", notice(&v));
}

#[test]
fn the_refusals_of_the_settings_and_of_trust_have_words_in_the_ui_language() {
    use crate::daemon_link::error_text::rewrite_refusal;
    assert!(rewrite_refusal(RewriteRefusal::ScriptsDisabled)
        .get()
        .contains("rewrite.external_scripts"));
    assert!(!rewrite_refusal(RewriteRefusal::ScriptNotTrusted)
        .get()
        .is_empty());
    // Whatever the kind of script, a loaded recipe keeps its source.
    let (v, _q, _t) = script_view();
    assert!(matches!(
        v.script_ref().map(|s| &s.source),
        Some(ScriptSource::Inline { .. })
    ));
}
