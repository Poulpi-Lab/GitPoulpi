//! The terminal run mode against the fake interactive agent of the testkit:
//! a real pty, a real child process, no real agent and no network.
#![cfg(feature = "terminal-agent")]

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_ai::agent_terminal::TerminalAgentClient;
use gitpoulpi_ai::{AiClient, Error};
use gitpoulpi_config::{AgentRunMode, AiProfile, AiProviderKind};

fn profile(harness: &str, env: &[(&str, &str)]) -> AiProfile {
    let mut p = AiProfile::new("fake", "Fake", AiProviderKind::LocalAgent);
    p.command = gitpoulpi_testkit::terminal_agent::program()
        .to_string_lossy()
        .into_owned();
    p.harness = harness.into();
    p.run_mode = AgentRunMode::Terminal;
    p.timeout_secs = 40;
    for (k, v) in env {
        p.env.insert((*k).into(), (*v).into());
    }
    p
}

fn client(p: &AiProfile, dir: &Path) -> TerminalAgentClient {
    TerminalAgentClient::from_profile(p, None)
        .unwrap()
        .with_workdir(dir.join("work"))
}

fn run<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

#[test]
fn the_answer_between_the_markers_is_returned() {
    let dir = tempfile::tempdir().unwrap();
    let c = client(&profile("gemini", &[]), dir.path());
    let answer = run(c.complete("system", "user text")).unwrap();
    assert_eq!(answer, "feat: add the thing");
}

#[test]
fn a_multi_line_answer_keeps_its_blank_lines() {
    let dir = tempfile::tempdir().unwrap();
    let p = profile(
        "gemini",
        &[("FAKE_ANSWER", "fix: a subject\\n\\nA body line.")],
    );
    let answer = run(client(&p, dir.path()).complete("s", "u")).unwrap();
    assert_eq!(answer, "fix: a subject\n\nA body line.");
}

#[test]
fn an_answer_longer_than_the_screen_is_read_from_the_scrollback() {
    let dir = tempfile::tempdir().unwrap();
    let p = profile("codex", &[("FAKE_FLAVOR", "codex"), ("FAKE_LINES", "150")]);
    let answer = run(client(&p, dir.path()).complete("s", "u")).unwrap();
    let lines: Vec<&str> = answer.lines().collect();
    assert_eq!(lines.first(), Some(&"feat: add the thing"));
    assert_eq!(lines.last(), Some(&"filler line 149"));
    assert_eq!(lines.len(), 151);
}

#[test]
fn a_typed_prompt_works_when_bracketed_paste_is_off() {
    let dir = tempfile::tempdir().unwrap();
    let p = profile("gemini", &[("FAKE_NO_BRACKETED", "1")]);
    let answer = run(client(&p, dir.path()).complete("system", "line one\nline two")).unwrap();
    assert_eq!(answer, "feat: add the thing");
}

#[test]
fn a_first_run_dialog_fails_fast_and_names_the_agent_and_folder() {
    let dir = tempfile::tempdir().unwrap();
    let p = profile("gemini", &[("FAKE_TRUST", "1")]);
    let started = Instant::now();
    let err = run(client(&p, dir.path()).complete("s", "u")).unwrap_err();
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "{:?}",
        started.elapsed()
    );
    match err {
        Error::AgentSetup {
            agent,
            folder,
            detail,
        } => {
            assert_eq!(agent, "Gemini CLI");
            assert!(folder.ends_with("work"), "{folder}");
            assert_eq!(detail, "Do you trust the files in this folder?");
        }
        other => panic!("expected AgentSetup, got {other:?}"),
    }
}

#[test]
fn an_agent_that_never_answers_times_out_with_its_last_screen() {
    let dir = tempfile::tempdir().unwrap();
    let p = profile("gemini", &[("FAKE_HANG", "1")]);
    let c = client(&p, dir.path()).with_timeout(Duration::from_secs(6));
    let err = run(c.complete("s", "u")).unwrap_err();
    let Error::Agent(text) = err else {
        panic!("expected an agent error, got {err:?}")
    };
    assert!(text.contains("did not answer within 6 seconds"), "{text}");
    assert!(
        text.contains("esc to cancel"),
        "the last screen is attached: {text}"
    );
}

#[cfg(unix)]
fn alive(pid_file: &Path) -> bool {
    let pid = std::fs::read_to_string(pid_file).unwrap_or_default();
    !pid.trim().is_empty() && Path::new(&format!("/proc/{}", pid.trim())).exists()
}

#[cfg(unix)]
fn wait_gone(pid_file: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if !alive(pid_file) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[cfg(unix)]
#[test]
fn dropping_the_future_kills_the_agent() {
    let dir = tempfile::tempdir().unwrap();
    let pid_file = dir.path().join("pid");
    let p = profile(
        "gemini",
        &[
            ("FAKE_HANG", "1"),
            ("FAKE_PID_FILE", pid_file.to_str().unwrap()),
        ],
    );
    let c = client(&p, dir.path());
    run(async {
        // Canceled while the agent is busy: the select drops the future.
        tokio::select! {
            _ = c.complete("s", "u") => panic!("must not finish"),
            _ = tokio::time::sleep(Duration::from_secs(4)) => {}
        }
    });
    assert!(
        wait_gone(&pid_file),
        "the agent outlived its canceled request"
    );
}

#[cfg(unix)]
#[test]
fn an_agent_that_ignores_the_exit_command_is_killed() {
    let dir = tempfile::tempdir().unwrap();
    let pid_file = dir.path().join("pid");
    let p = profile(
        "gemini",
        &[
            ("FAKE_IGNORE_EXIT", "1"),
            ("FAKE_PID_FILE", pid_file.to_str().unwrap()),
        ],
    );
    let answer = run(client(&p, dir.path()).complete("s", "u")).unwrap();
    assert_eq!(answer, "feat: add the thing");
    assert!(wait_gone(&pid_file), "the agent was left running");
}

#[test]
fn three_runs_at_once_all_finish_with_two_slots() {
    let dir = tempfile::tempdir().unwrap();
    let p = profile("gemini", &[]);
    let answers = run(async {
        let a = client(&p, dir.path());
        let b = client(&p, dir.path());
        let c = client(&p, dir.path());
        tokio::join!(
            a.complete("s", "1"),
            b.complete("s", "2"),
            c.complete("s", "3")
        )
    });
    for answer in [answers.0, answers.1, answers.2] {
        assert_eq!(answer.unwrap(), "feat: add the thing");
    }
}

#[test]
fn claude_answers_come_from_the_session_transcript_and_the_file_is_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("claude-config");
    std::fs::create_dir_all(&config).unwrap();
    let args_file = dir.path().join("args");
    let p = profile(
        "claude",
        &[
            ("FAKE_FLAVOR", "claude"),
            ("FAKE_TRANSCRIPT", config.to_str().unwrap()),
            ("CLAUDE_CONFIG_DIR", config.to_str().unwrap()),
            ("FAKE_ARGS_FILE", args_file.to_str().unwrap()),
            ("FAKE_ANSWER", "docs: from the transcript"),
        ],
    );
    let answer = run(client(&p, dir.path()).complete("s", "u")).unwrap();
    // The fake keeps the answer OFF the screen: only the transcript has it.
    assert_eq!(answer, "docs: from the transcript");
    let args = std::fs::read_to_string(&args_file).unwrap();
    assert!(args.contains("--session-id "), "{args}");
    let left: Vec<_> = std::fs::read_dir(config.join("projects/fake"))
        .unwrap()
        .collect();
    assert!(left.is_empty(), "the transcript was not deleted: {left:?}");
}

#[test]
fn the_profiles_terminal_arguments_and_working_folder_reach_the_agent() {
    let dir = tempfile::tempdir().unwrap();
    let args_file = dir.path().join("args");
    let mut p = profile("gemini", &[("FAKE_ARGS_FILE", args_file.to_str().unwrap())]);
    p.terminal_args = vec![
        "--approval-mode".into(),
        "plan".into(),
        "-m".into(),
        "flash".into(),
    ];
    let c = client(&p, dir.path());
    run(c.complete("s", "u")).unwrap();
    let written = std::fs::read_to_string(&args_file).unwrap();
    let mut lines = written.lines();
    assert_eq!(lines.next(), Some("--approval-mode plan -m flash"));
    let cwd = lines.next().unwrap();
    assert!(cwd.ends_with("work"), "{cwd}");
}

#[test]
fn an_agent_type_without_a_terminal_mode_is_refused() {
    let mut p = profile("aider", &[]);
    p.harness = "aider".into();
    assert!(matches!(
        TerminalAgentClient::from_profile(&p, None),
        Err(Error::BadProfile(_))
    ));
}
