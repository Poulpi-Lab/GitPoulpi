//! The SSH & credentials page: its place in Settings, its rows, what Save
//! sends, the confirmation and the results panel. Nothing here talks to a
//! daemon: a `RawQueue` stands in for the link, and the answers are put in
//! by hand. The end-to-end paths are `pty_repo_auth_key` and
//! `pty_repo_auth_command_confirm`.

use super::*;
use crate::daemon_link::RawQueue;
use crate::preferences::tests::{draw_screen, state_at};
use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use gitpoulpi_core::domain::{RemoteIdentity, RemoteTransport, SshAuth};
use gitpoulpi_core::failure::ErrorDetail;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn page(dir: &std::path::Path) -> (PreferencesState, RawQueue) {
    let mut st = state_at(dir, "RepoAuth");
    let queue = RawQueue::default();
    st.set_repo_auth_requester(queue.requester());
    st.focus = PrefFocus::Fields;
    (st, queue)
}

fn loaded(st: &mut PreferencesState) {
    st.ra_take_view(
        RepoAuthView {
            remotes: vec![RemoteIdentity {
                remote: "origin".into(),
                url: "https://github.com/acme/app.git".into(),
                transport: RemoteTransport::Https,
                identity: "system git".into(),
            }],
            ..RepoAuthView::default()
        },
        RepoAuthProfile::default(),
    );
}

fn labels(st: &PreferencesState) -> Vec<String> {
    st.fields().iter().map(|f| f.label.clone()).collect()
}

fn select(st: &mut PreferencesState, label: &str) {
    st.field = st
        .fields()
        .iter()
        .position(|f| f.label == label)
        .unwrap_or_else(|| panic!("no `{label}` in {:?}", labels(st)));
}

fn type_into(st: &mut PreferencesState, label: &str, text: &str) {
    select(st, label);
    st.activate();
    for c in text.chars() {
        st.on_key(key(KeyCode::Char(c)));
    }
    st.on_key(key(KeyCode::Enter));
}

/// Picks `option` of the choice row `label` through its dropdown.
fn pick(st: &mut PreferencesState, label: &str, option: &str) {
    select(st, label);
    let Control::Choice { options, index } = st.fields()[st.field].control.clone() else {
        panic!("`{label}` is not a choice");
    };
    let target = options.iter().position(|o| o == option).unwrap();
    st.activate();
    st.on_key(key(KeyCode::Home));
    for _ in 0..target {
        st.on_key(key(KeyCode::Down));
    }
    st.on_key(key(KeyCode::Enter));
    let _ = index;
}

fn press(st: &mut PreferencesState, label: &str) {
    let fields = st.fields();
    let (at, i) = fields
        .iter()
        .enumerate()
        .find_map(|(f, field)| match &field.control {
            Control::Buttons(b) => b.iter().position(|b| b.label == label).map(|i| (f, i)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no button `{label}`"));
    st.field = at;
    st.ai.chip = i;
    st.activate();
}

fn button_enabled(st: &PreferencesState, label: &str) -> bool {
    st.fields()
        .iter()
        .find_map(|f| match &f.control {
            Control::Buttons(b) => b.iter().find(|b| b.label == label).map(|b| b.enabled),
            _ => None,
        })
        .unwrap()
}

#[test]
fn the_page_is_in_the_repository_group_after_git_and_encoding_with_a_stable_id() {
    let section = PrefSection::RepoAuth;
    assert!(section.is_repo_scoped());
    assert_eq!(section.id(), "ssh-credentials");
    assert_eq!(section.title(), "SSH & credentials");
    let page = gitpoulpi_ipc::ui::SettingsPage::parse("ssh-credentials").unwrap();
    assert_eq!(PrefSection::from_page(page), section);
    assert!(section.is_built());
    let at = |s| PrefSection::ALL.iter().position(|x| *x == s).unwrap();
    assert_eq!(at(section), at(PrefSection::GitAndEncoding) + 1);
    assert!(gitpoulpi_ipc::ui::SettingsPage::IDS.contains(&"ssh-credentials"));
}

#[test]
fn the_default_page_has_the_intro_both_choices_the_remotes_and_four_buttons() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    loaded(&mut st);
    let l = labels(&st);
    assert!(l[0].starts_with("Used by fetch, pull, push, submodules and LFS"));
    for want in ["HTTPS", "SSH", "Remotes", "origin", "Git username"] {
        assert!(l.iter().any(|x| x == want), "{want} in {l:?}");
    }
    let origin = st.fields().into_iter().find(|f| f.label == "origin").unwrap();
    assert_eq!(origin.value_text(), "https  system git");
    for b in ["Test connection", "Save", "Revert", "Use defaults"] {
        assert!(st.fields().iter().any(|f| matches!(&f.control,
            Control::Buttons(bs) if bs.iter().any(|x| x.label == b))), "{b}");
    }
    assert!(!button_enabled(&st, "Save") && !button_enabled(&st, "Revert"));
    assert!(button_enabled(&st, "Test connection"));
}

#[test]
fn the_helper_and_token_command_choices_show_the_boxed_warning_and_the_argv_preview() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    loaded(&mut st);
    pick(&mut st, "Sign in with", "Token command");
    let fields = st.fields();
    let command = fields.iter().find(|f| f.label == "Command").unwrap();
    assert!(command.danger);
    assert!(command
        .warning
        .as_deref()
        .unwrap()
        .starts_with("This command runs on your computer each time this repository"));
    type_into(&mut st, "Command", "pass show \"my token\"");
    let preview = st
        .fields()
        .into_iter()
        .find(|f| f.label == "Runs as")
        .unwrap();
    assert_eq!(preview.value_text(), "[pass] [show] [my token]");
    for want in ["Hosts", "Cache (minutes)", "Git username"] {
        assert!(labels(&st).iter().any(|x| x == want), "{want}");
    }
    pick(&mut st, "Sign in with", "Credential helper");
    assert!(labels(&st).iter().any(|x| x == "Helper"));
    assert!(st.fields().iter().any(|f| matches!(&f.control,
        Control::Buttons(b) if b[0].label == "Use this repository's current helper")));
}

#[test]
fn the_system_choice_lists_the_repositorys_own_helpers_with_their_origin_file() {
    use gitpoulpi_core::domain::AuthOverride;
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    st.ra_take_view(
        RepoAuthView {
            overrides: vec![
                AuthOverride {
                    key: "credential.helper".into(),
                    value: "!sh evil.sh".into(),
                    scope: "local".into(),
                    origin: "file:.git/config".into(),
                    overridden: false,
                },
                AuthOverride {
                    key: "credential.helper".into(),
                    value: "manager".into(),
                    scope: "global".into(),
                    origin: "file:/home/u/.gitconfig".into(),
                    overridden: false,
                },
            ],
            ..RepoAuthView::default()
        },
        RepoAuthProfile::default(),
    );
    let rows: Vec<_> = st
        .fields()
        .into_iter()
        .filter(|f| f.label == "Set by this repository")
        .map(|f| f.value_text())
        .collect();
    assert_eq!(rows, ["!sh evil.sh  (file:.git/config)"]);
    assert!(labels(&st)
        .iter()
        .any(|x| x == "Ignore credential helpers set by this repository"));
}

#[test]
fn editing_enables_save_and_revert_and_revert_restores_the_saved_choices() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    loaded(&mut st);
    type_into(&mut st, "Git username", "alice");
    assert!(button_enabled(&st, "Save") && button_enabled(&st, "Revert"));
    assert!(st.ra_dirty(), "closing Settings would lose the edit");
    // Test connection refuses to run against unsaved choices.
    press(&mut st, "Test connection");
    assert_eq!(
        st.repo_auth.status.as_deref(),
        Some("Save first: the test uses the saved choices.")
    );
    press(&mut st, "Revert");
    assert!(!st.ra_dirty());
    let user = st.fields().into_iter().find(|f| f.label == "Git username");
    assert_eq!(user.unwrap().value_text(), "(not set)");
}

#[test]
fn save_sends_the_whole_profile_and_a_plain_source_needs_no_confirmation() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = page(dir.path());
    loaded(&mut st);
    queue.queued();
    st.repo_auth.draft.https = HttpsKind::System;
    st.repo_auth.draft.ssh = SshKind::KeyFile;
    st.repo_auth.draft.key = "/home/u/.ssh/work".into();
    st.repo_auth.draft.strict = true;
    press(&mut st, "Save");
    assert!(st.repo_auth.confirm.is_none(), "a key needs no confirmation");
    let sent = queue.queued();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, OperationKind::RepoAuthSet);
    let args: RepoAuthSetArgs = serde_json::from_value(sent[0].1.clone()).unwrap();
    assert_eq!(
        args.profile.ssh,
        Some(SshAuth::KeyFile {
            key: "/home/u/.ssh/work".into(),
            known_hosts: None,
            strict_host_key: true,
        })
    );
    assert!(args.profile.https.is_none());
    assert!(!args.confirm);
}

#[test]
fn the_answer_of_a_save_refreshes_the_view_and_a_refusal_is_shown_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = page(dir.path());
    loaded(&mut st);
    st.repo_auth.draft.username = "alice".into();
    st.repo_auth.set = Some(Reply::ready(Err(gitpoulpi_core::Error::RepoAuthUnavailable {
        reason: "the credential helper and token command sources are not available yet".into(),
    })));
    st.poll_repo_auth();
    let status = st.repo_auth.status.clone().unwrap();
    assert!(status.starts_with("Not saved: "), "{status}");
    assert!(status.contains("not available yet"), "{status}");
    assert!(st.ra_dirty(), "the edit stays after a refusal");

    st.repo_auth.set = Some(Reply::ready(Ok(serde_json::Value::Null)));
    st.poll_repo_auth();
    assert_eq!(
        st.repo_auth.status.as_deref(),
        Some("Saved. This repository now signs in as chosen.")
    );
    let sent = queue.queued();
    assert!(
        sent.iter().any(|(op, _)| *op == OperationKind::RepoAuthGet),
        "a fresh read follows the save: {sent:?}"
    );
}

fn command_page(dir: &std::path::Path) -> (PreferencesState, RawQueue) {
    let (mut st, queue) = page(dir);
    loaded(&mut st);
    queue.queued();
    pick(&mut st, "Sign in with", "Token command");
    type_into(&mut st, "Command", "pass show work");
    type_into(&mut st, "Hosts", "github.com");
    (st, queue)
}

#[test]
fn saving_a_command_opens_the_confirmation_with_no_default_answer() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = command_page(dir.path());
    press(&mut st, "Save");
    assert!(st.ra_overlay_open());
    assert!(queue.queued().is_empty(), "nothing is sent before the answer");
    let screen = draw_screen(&mut st);
    for want in [
        "Allow this command?",
        "Token command",
        "$ pass show work",
        "SHA-256: ",
        "1. Save and allow",
        "2. Cancel",
    ] {
        assert!(screen.contains(want), "{want}\n{screen}");
    }
    let hash = gitpoulpi_core::domain::command_sha256(&["pass", "show", "work"]);
    assert!(screen.contains(&hash), "the hash of the argv is shown");
    assert_eq!(st.repo_auth.confirm.as_ref().unwrap().cursor(), None);
    // Enter with nothing chosen answers nothing.
    st.on_key(key(KeyCode::Enter));
    assert!(st.ra_overlay_open());
    assert!(queue.queued().is_empty());
}

#[test]
fn escape_the_cross_and_cancel_do_not_save() {
    let dir = tempfile::tempdir().unwrap();
    for how in ["esc", "two", "cross"] {
        let (mut st, queue) = command_page(dir.path());
        press(&mut st, "Save");
        draw_screen(&mut st);
        match how {
            "esc" => {
                st.on_key(key(KeyCode::Esc));
            }
            "two" => {
                st.on_key(key(KeyCode::Char('2')));
            }
            _ => {
                let cross = st.repo_auth.confirm.as_ref().unwrap().close_rect().unwrap();
                st.on_mouse(MouseEventKind::Down(MouseButton::Left), cross.x, cross.y);
            }
        }
        assert!(!st.ra_overlay_open(), "{how}: the question closed");
        assert!(queue.queued().is_empty(), "{how}: nothing was saved");
        assert!(st.ra_dirty(), "{how}: the edit is still there");
    }
}

#[test]
fn one_saves_with_the_confirmation_and_so_does_arrow_then_enter() {
    let dir = tempfile::tempdir().unwrap();
    for digit in [true, false] {
        let (mut st, queue) = command_page(dir.path());
        press(&mut st, "Save");
        draw_screen(&mut st);
        if digit {
            st.on_key(key(KeyCode::Char('1')));
        } else {
            st.on_key(key(KeyCode::Down));
            assert_eq!(st.repo_auth.confirm.as_ref().unwrap().cursor(), Some(0));
            st.on_key(key(KeyCode::Enter));
        }
        let sent = queue.queued();
        assert_eq!(sent.len(), 1, "{sent:?}");
        let args: RepoAuthSetArgs = serde_json::from_value(sent[0].1.clone()).unwrap();
        assert!(args.confirm);
        match args.profile.https {
            Some(HttpsAuth::TokenCommand { argv, hosts, .. }) => {
                assert_eq!(argv, ["pass", "show", "work"]);
                assert_eq!(hosts, ["github.com"]);
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn importing_the_repositorys_helper_fills_the_field_and_still_asks() {
    use gitpoulpi_core::domain::AuthOverride;
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = page(dir.path());
    st.ra_take_view(
        RepoAuthView {
            overrides: vec![AuthOverride {
                key: "credential.helper".into(),
                value: "!sh repo.sh".into(),
                scope: "local".into(),
                origin: "file:.git/config".into(),
                overridden: false,
            }],
            ..RepoAuthView::default()
        },
        RepoAuthProfile::default(),
    );
    pick(&mut st, "Sign in with", "Credential helper");
    press(&mut st, "Use this repository's current helper");
    let helper = st.fields().into_iter().find(|f| f.label == "Helper").unwrap();
    assert_eq!(helper.value_text(), "!sh repo.sh");
    queue.queued();
    press(&mut st, "Save");
    assert!(st.ra_overlay_open(), "an imported helper is confirmed too");
    assert!(queue.queued().is_empty());
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Credential helper") && screen.contains("$ !sh repo.sh"));
}

#[test]
fn a_long_command_is_wrapped_never_cut_and_a_control_character_is_spelled_out() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    loaded(&mut st);
    pick(&mut st, "Sign in with", "Credential helper");
    st.repo_auth.draft.helper = format!("!{}\u{1b}[2J", "Q".repeat(2000));
    press(&mut st, "Save");
    let mut seen = 0;
    for _ in 0..60 {
        let screen = draw_screen(&mut st);
        assert!(!screen.contains('\u{1b}'));
        seen += screen.matches('Q').count();
        st.on_key(key(KeyCode::PageDown));
    }
    assert!(seen >= 2000, "every Q was on screen at some point: {seen}");
    st.on_key(key(KeyCode::Home));
    draw_screen(&mut st);
    let confirm = st.repo_auth.confirm.as_ref().unwrap();
    assert!(confirm.pane().track().is_some(), "a long command has a scrollbar");
    assert_eq!(st.repo_auth.confirm.as_ref().unwrap().pane().top(), 0);
}

#[test]
fn use_defaults_asks_first_and_only_yes_clears() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = page(dir.path());
    loaded(&mut st);
    queue.queued();
    press(&mut st, "Use defaults");
    assert!(st.confirm_action.is_some());
    assert!(queue.queued().is_empty());
    st.on_key(key(KeyCode::Esc));
    assert!(st.confirm_action.is_none() && queue.queued().is_empty());
    // The palette twin asks as well.
    st.repo_auth_defaults_command();
    assert!(st.confirm_action.is_some());
    st.on_key(key(KeyCode::Right));
    st.on_key(key(KeyCode::Enter));
    let sent = queue.queued();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, OperationKind::RepoAuthClear);
}

fn report(remote: &str, push: bool, ok: bool) -> RemoteAccessReport {
    RemoteAccessReport {
        remote: remote.into(),
        url: "ssh://git@stub.invalid/x".into(),
        transport: RemoteTransport::Ssh,
        push,
        ok,
        ref_count: if ok { 2 } else { 0 },
        duration_ms: 42,
        ssh_accepted_key: ok.then(|| "/keys/work_ed25519".to_string()),
        ssh_authenticated_to: None,
        accepted_identity: None,
        error: (!ok).then(|| {
            ErrorDetail::from_git(
                &["ls-remote"],
                Some(128),
                "git@stub.invalid: Permission denied (publickey).",
            )
        }),
    }
}

#[test]
fn the_results_name_the_identity_the_timing_and_read_a_failure_kindly() {
    let ids = [RemoteIdentity {
        remote: "origin".into(),
        url: "ssh://git@stub.invalid/x".into(),
        transport: RemoteTransport::Ssh,
        identity: "ssh key work_ed25519".into(),
    }];
    let lines = results::lines(
        &ids,
        &[report("origin", false, true)],
        &[report("origin", true, false)],
    );
    let text: Vec<&str> = lines.iter().map(|(_, l)| l.as_str()).collect();
    let joined = text.join("\n");
    assert!(joined.contains("origin  (ssh, ssh key work_ed25519)"), "{joined}");
    assert!(
        joined.contains("Fetch: ok  2 branches, 42 ms, accepted key /keys/work_ed25519"),
        "{joined}"
    );
    assert!(joined.contains("Push: failed  42 ms"), "{joined}");
    assert!(
        joined.contains("refused the credentials"),
        "a failure gets its friendly reading: {joined}"
    );
}

#[test]
fn a_test_runs_fetch_then_push_off_the_render_thread_and_opens_the_results() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = page(dir.path());
    loaded(&mut st);
    queue.queued();
    press(&mut st, "Test connection");
    st.poll_repo_auth();
    let sent = queue.queued();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, OperationKind::RepoAuthTest);
    let args: RepoAuthTestArgs = serde_json::from_value(sent[0].1.clone()).unwrap();
    assert!(!args.push, "the fetch check goes first");
    assert!(st.ra_animating(), "frames keep coming while it runs");
    // The fetch answered: the push check is asked.
    if let Some(run) = st.repo_auth.test.as_mut() {
        run.stage = Stage::Fetch(Reply::ready(Ok(vec![report("origin", false, true)])));
    }
    st.poll_repo_auth();
    let sent = queue.queued();
    let args: RepoAuthTestArgs = serde_json::from_value(sent[0].1.clone()).unwrap();
    assert!(args.push);
    if let Some(run) = st.repo_auth.test.as_mut() {
        run.stage = Stage::Push(Reply::ready(Ok(vec![report("origin", true, true)])));
    }
    st.poll_repo_auth();
    assert!(!st.ra_animating());
    assert!(st.ra_overlay_open());
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Connection test"), "{screen}");
    assert!(screen.contains("accepted key /keys/work_ed25519"), "{screen}");
    // Esc closes it.
    st.on_key(key(KeyCode::Esc));
    assert!(!st.ra_overlay_open());
}

#[test]
fn the_spinner_shows_only_after_150_ms() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    loaded(&mut st);
    press(&mut st, "Test connection");
    st.poll_repo_auth();
    let running = |st: &PreferencesState| labels(st).into_iter().any(|l| l.contains("Testing the connection"));
    assert!(!running(&st), "not at once");
    if let Some(run) = st.repo_auth.test.as_mut() {
        run.started = Instant::now() - Duration::from_millis(400);
    }
    assert!(running(&st), "after the threshold");
}

#[test]
fn the_results_pane_scrolls_by_keys_wheel_and_a_draggable_scrollbar() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    loaded(&mut st);
    let many: Vec<RemoteAccessReport> = (0..40)
        .map(|i| report(&format!("remote-{i}"), false, true))
        .collect();
    st.repo_auth.results = Some(ResultsPane::new(results::lines(&[], &many, &[])));
    draw_screen(&mut st);
    let pane = |st: &PreferencesState| st.repo_auth.results.as_ref().unwrap().pane() as *const _;
    let _ = pane;
    let top = |st: &PreferencesState| st.repo_auth.results.as_ref().unwrap().pane().top();
    let track = st.repo_auth.results.as_ref().unwrap().pane().track().expect("a bar");
    assert_eq!(top(&st), 0);
    st.on_key(key(KeyCode::PageDown));
    assert!(top(&st) > 0);
    st.on_key(key(KeyCode::Home));
    assert_eq!(top(&st), 0);
    st.on_key(key(KeyCode::Down));
    assert_eq!(top(&st), 1);
    st.on_key(key(KeyCode::End));
    let end = top(&st);
    assert!(end > 1);
    st.on_key(key(KeyCode::Home));
    // Drag the thumb to the end of the track.
    let (col, row) = (track.rect.x, track.rect.y + track.thumb_start);
    st.on_mouse(MouseEventKind::Down(MouseButton::Left), col, row);
    st.on_mouse(
        MouseEventKind::Drag(MouseButton::Left),
        col,
        track.rect.y + track.track_len,
    );
    st.on_mouse(MouseEventKind::Up(MouseButton::Left), col, row);
    assert_eq!(top(&st), end, "dragged to the end");
    // The cross closes it.
    draw_screen(&mut st);
    st.on_key(key(KeyCode::Esc));
    assert!(!st.ra_overlay_open());
}

#[test]
fn closing_settings_with_unsaved_choices_asks_first() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    loaded(&mut st);
    assert!(!st.request_close(), "nothing to lose closes quietly");
    st.repo_auth.draft.username = "alice".into();
    assert!(st.request_close(), "the question is open");
    assert!(st.close_ask.is_some());
}

#[test]
fn the_page_draws_at_narrow_and_wide_widths() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let tmp = tempfile::tempdir().unwrap();
    // A fixed repository name: the sidebar shows it.
    let dir = tmp.path().join("app");
    std::fs::create_dir_all(&dir).unwrap();
    for (w, h) in [(60u16, 40u16), (140, 45)] {
        let (mut st, _q) = page(&dir);
        loaded(&mut st);
        st.repo_auth.draft.ssh = SshKind::KeyFile;
        st.repo_auth.draft.key = "/home/u/.ssh/work_ed25519".into();
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| st.render(f, f.area())).unwrap();
        let b = terminal.backend().buffer();
        let text = (0..h)
            .map(|y| {
                (0..w)
                    .map(|x| b[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(format!("ssh_credentials_{w}"), text);
    }
}
