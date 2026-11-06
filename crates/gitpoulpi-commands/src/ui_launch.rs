//! Cross-session viewer discovery and the opt-in "launch a window" step of
//! `gitpoulpi ui show` (the mcp-show-ui design §3.4).
//!
//! Discovery reads every `sessions/*.port`, asks each live daemon for its
//! viewers over one short loopback connection (`UiQuery(Viewers)`, read-only)
//! and never starts a daemon. Launching expands the configured template
//! (`gitpoulpi_config::launch`) into an argv and spawns it detached, with no
//! shell unless the template starts one.

use std::path::Path;
use std::time::Duration;

use gitpoulpi_config::launch::{self, LaunchVars};
use gitpoulpi_config::{ConfigPaths, Overrides, Settings};
use gitpoulpi_core::domain::RepoId;

/// How long `--launch` waits for the new window to attach.
pub(crate) const DEFAULT_WAIT: Duration = Duration::from_secs(10);

pub(crate) use gitpoulpi_daemon::join::SessionViewers;

/// The viewers of every live session except `skip` (the caller's own, which
/// it asks over its own connection). Dead port files are skipped silently.
pub(crate) async fn scan_sessions(skip: Option<&str>) -> Vec<SessionViewers> {
    scan_sessions_within(skip, Duration::from_millis(1500)).await
}

/// [`scan_sessions`] with a shorter patience per daemon (the launcher).
pub(crate) async fn scan_sessions_within(
    skip: Option<&str>,
    patience: Duration,
) -> Vec<SessionViewers> {
    gitpoulpi_daemon::join::scan_sessions_within(skip, patience).await
}

/// Which window to route a view intent to.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Pick {
    /// Exactly one window qualifies: `(session, port)`.
    One(String, u16),
    /// Several do; the user must choose (`-s <session>`).
    Many(Vec<String>),
    None,
}

/// The windows (that accept agent control) with `repo` in a tab, or any
/// window when the intent names no repository.
pub(crate) fn pick(found: &[SessionViewers], repo: Option<&RepoId>) -> Pick {
    let mut hits: Vec<(&str, u16)> = Vec::new();
    for s in found {
        for v in &s.viewers {
            let has_repo = repo.map_or(true, |r| {
                v.tabs.contains(r) || v.active_repo.as_ref() == Some(r)
            });
            if v.agent_control && has_repo {
                hits.push((s.session.as_str(), s.port));
            }
        }
    }
    hits.sort_unstable();
    hits.dedup();
    match hits.as_slice() {
        [] => Pick::None,
        [(session, port)] => Pick::One((*session).to_string(), *port),
        many => Pick::Many(many.iter().map(|(s, _)| (*s).to_string()).collect()),
    }
}

/// The `[ui]` launch template from the resolved config, `None` when launching
/// is off. An invalid setting is an error naming the key.
pub(crate) fn configured_template() -> Result<Option<String>, String> {
    let paths = ConfigPaths::discover();
    let settings = Settings::resolve(paths.as_ref(), &Overrides::default())
        .map_err(|e| format!("could not read the config: {e}"))?;
    let ui = &settings.config().ui;
    launch::resolve_template(&ui.launch, &ui.launch_command)
}

/// The `gitpoulpi` executable: this one, or (from the MCP server) the one
/// installed beside it, or whatever `PATH` finds.
pub(crate) fn gitpoulpi_exe() -> String {
    if let Ok(exe) = std::env::current_exe() {
        let stem = exe.file_stem().map(|s| s.to_string_lossy().to_string());
        if stem.as_deref() == Some("gitpoulpi") {
            return exe.to_string_lossy().to_string();
        }
        let sibling = exe.with_file_name(format!("gitpoulpi{}", std::env::consts::EXE_SUFFIX));
        if sibling.exists() {
            return sibling.to_string_lossy().to_string();
        }
    }
    "gitpoulpi".to_string()
}

/// Spawns the launcher for `template`, detached, with `repo` as its working
/// directory. Fails when the program cannot start or exits non-zero at once.
pub(crate) fn spawn(template: &str, repo: &Path, session: &str) -> Result<Vec<String>, String> {
    let exe = gitpoulpi_exe();
    let repo_text = repo.to_string_lossy().to_string();
    let argv = launch::expand(
        template,
        &LaunchVars {
            repo: &repo_text,
            session,
            gitpoulpi: &exe,
        },
    )?;
    let mut command = std::process::Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .current_dir(repo)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("could not run the launch command `{}`: {e}", argv[0]))?;
    // A launcher that fails (tmux with no server, a missing terminal) dies at
    // once; a terminal emulator stays up. Give it a moment to tell us.
    std::thread::sleep(Duration::from_millis(300));
    if let Ok(Some(status)) = child.try_wait() {
        if !status.success() {
            return Err(format!(
                "the launch command `{}` exited with {status}",
                argv[0]
            ));
        }
    }
    Ok(argv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_ipc::ui::{ViewerId, ViewerInfo};

    fn viewer(tabs: &[&str], control: bool) -> ViewerInfo {
        ViewerInfo {
            id: ViewerId("v".into()),
            pid: 1,
            active_repo: tabs.first().map(|t| RepoId((*t).into())),
            tabs: tabs.iter().map(|t| RepoId((*t).into())).collect(),
            focused: None,
            agent_control: control,
            last_focus_at: None,
        }
    }

    fn sv(session: &str, port: u16, viewers: Vec<ViewerInfo>) -> SessionViewers {
        SessionViewers {
            token: None,
            session: session.into(),
            port,
            viewers,
        }
    }

    #[test]
    fn pick_needs_exactly_one_session_with_the_repo() {
        let a = RepoId("/a".into());
        let found = vec![
            sv("s1", 1, vec![viewer(&["/b"], true)]),
            sv("s2", 2, vec![viewer(&["/a", "/b"], true)]),
        ];
        assert_eq!(pick(&found, Some(&a)), Pick::One("s2".into(), 2));
        let two = vec![
            sv("s1", 1, vec![viewer(&["/a"], true)]),
            sv("s2", 2, vec![viewer(&["/a"], true)]),
        ];
        assert_eq!(
            pick(&two, Some(&a)),
            Pick::Many(vec!["s1".into(), "s2".into()])
        );
        assert_eq!(pick(&[], Some(&a)), Pick::None);
    }

    #[test]
    fn pick_skips_windows_that_refuse_agent_control_and_handles_repoless_intents() {
        let a = RepoId("/a".into());
        let found = vec![sv("s1", 1, vec![viewer(&["/a"], false)])];
        assert_eq!(pick(&found, Some(&a)), Pick::None);
        let found = vec![sv("s1", 1, vec![viewer(&["/z"], true)])];
        assert_eq!(pick(&found, None), Pick::One("s1".into(), 1));
        // Two windows of one session count once.
        let found = vec![sv(
            "s1",
            1,
            vec![viewer(&["/a"], true), viewer(&["/a"], true)],
        )];
        assert_eq!(pick(&found, Some(&a)), Pick::One("s1".into(), 1));
    }
}
