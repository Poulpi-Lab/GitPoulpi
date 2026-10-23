use super::profiles::local_agent_state;
use super::*;
use gitpoulpi_config::{AgentRunMode, AgentWorkdir};

fn labels(state: &PreferencesState) -> Vec<String> {
    state.fields().into_iter().map(|f| f.label).collect()
}

#[test]
fn the_run_mode_switches_between_two_independent_argument_sets() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = local_agent_state(dir.path());
    assert!(labels(&state).contains(&"Run mode".to_string()));
    // Headless to start: the old file loads as before.
    assert_eq!(state.config.ai.profiles[0].args, ["-p"]);
    assert!(!file(dir.path()).contains("run_mode"));

    state.ai_commit_combo(AiTarget::ProfileRunMode, "terminal");
    let p = &state.config.ai.profiles[0];
    assert_eq!(p.run_mode, AgentRunMode::Terminal);
    assert_eq!(p.args, ["-p"], "headless arguments are kept");
    assert!(
        p.terminal_args.is_empty(),
        "Claude has no terminal defaults"
    );
    assert!(file(dir.path()).contains("run_mode = \"terminal\""));

    // The arguments row, the options and the model now edit the terminal set.
    state.ai_commit_text(AiTarget::ProfileArgs, "--effort low");
    state.ai_commit_combo(AiTarget::ProfileAgentModel, "haiku");
    let p = &state.config.ai.profiles[0];
    assert_eq!(p.terminal_args, ["--effort", "low", "--model", "haiku"]);
    assert_eq!(p.args, ["-p"]);
    let args_row = state
        .fields()
        .into_iter()
        .find(|f| f.label == "Arguments")
        .unwrap();
    assert!(
        matches!(&args_row.control, Control::Text(t) if t == "--effort low --model haiku"),
        "{:?}",
        args_row.control
    );
    // Only options valid without -p are offered (no --output-format).
    let options = state
        .fields()
        .into_iter()
        .find(|f| f.label == "Argument options")
        .unwrap();
    let Control::Combo { options, .. } = options.control else {
        panic!("a combo");
    };
    assert!(options.iter().all(|(l, _)| !l.contains("--output-format")));
    assert!(options.iter().any(|(l, _)| l.contains("--tools")));

    // Back to headless: its own arguments were never touched.
    state.ai_commit_combo(AiTarget::ProfileRunMode, "headless");
    let p = &state.config.ai.profiles[0];
    assert_eq!(p.args, ["-p"]);
    assert_eq!(p.terminal_args, ["--effort", "low", "--model", "haiku"]);
    // The file keeps the terminal set for the next switch.
    assert!(file(dir.path()).contains("terminal_args"));
}

#[test]
fn option_toggles_and_model_follow_the_agent_type_not_the_command_name() {
    // The command is an alias: the type, not the name, decides (the toggles
    // used to look the harness up by command and silently did nothing).
    let dir = tempfile::tempdir().unwrap();
    let mut state = local_agent_state(dir.path());
    state.ai_commit_text(AiTarget::ProfileCommand, "/opt/bin/work-claude");
    assert_eq!(state.config.ai.profiles[0].harness, "claude");
    state.ai_commit_combo(AiTarget::ProfileOptions, "0");
    assert_eq!(state.config.ai.profiles[0].args, ["-p", "--tools", ""]);
    state.ai_commit_combo(AiTarget::ProfileAgentModel, "sonnet");
    assert_eq!(
        state.config.ai.profiles[0].args,
        ["-p", "--tools", "", "--model", "sonnet"]
    );
}

#[test]
fn working_folder_and_timeout_rows_write_the_profile() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = local_agent_state(dir.path());
    assert!(labels(&state).contains(&"Working folder".to_string()));
    assert!(labels(&state).contains(&"Timeout (seconds)".to_string()));
    state.ai_commit_combo(AiTarget::ProfileWorkdir, "repository");
    state.ai_commit_text(AiTarget::ProfileTimeout, "90");
    let p = &state.config.ai.profiles[0];
    assert_eq!(p.workdir, AgentWorkdir::Repository);
    assert_eq!(p.timeout_secs, 90);
    let raw = file(dir.path());
    assert!(raw.contains("workdir = \"repository\""), "{raw}");
    assert!(raw.contains("timeout_secs = 90"), "{raw}");
    // Not a number: refused, with a message, nothing changed.
    state.ai_commit_text(AiTarget::ProfileTimeout, "soon");
    assert_eq!(state.config.ai.profiles[0].timeout_secs, 90);
    assert!(state.message.is_some());
    // Empty means the mode's default again, and leaves the file.
    state.ai_commit_text(AiTarget::ProfileTimeout, "");
    assert_eq!(state.config.ai.profiles[0].timeout_secs, 0);
    assert!(!file(dir.path()).contains("timeout_secs"));
}

#[test]
fn an_agent_type_with_no_terminal_mode_has_no_run_mode_row_and_goes_back_to_headless() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = local_agent_state(dir.path());
    state.ai_commit_combo(AiTarget::ProfileRunMode, "terminal");
    state.ai_commit_combo(AiTarget::ProfileHarness, "aider");
    let p = &state.config.ai.profiles[0];
    assert_eq!(p.run_mode, AgentRunMode::Headless);
    assert!(!labels(&state).contains(&"Run mode".to_string()));
    assert!(!labels(&state).contains(&"Open the agent...".to_string()));
    // And a terminal request on such a type is refused.
    state.ai_commit_combo(AiTarget::ProfileRunMode, "terminal");
    assert_eq!(state.config.ai.profiles[0].run_mode, AgentRunMode::Headless);
}

#[test]
fn picking_codex_in_terminal_mode_starts_from_its_inline_screen_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = local_agent_state(dir.path());
    state.ai_commit_combo(AiTarget::ProfileHarness, "codex");
    state.ai_commit_combo(AiTarget::ProfileRunMode, "terminal");
    let p = &state.config.ai.profiles[0];
    assert_eq!(p.args, ["exec", "-"]);
    assert_eq!(p.terminal_args, ["--no-alt-screen"]);
    assert!(labels(&state).contains(&"Open the agent...".to_string()));
}
