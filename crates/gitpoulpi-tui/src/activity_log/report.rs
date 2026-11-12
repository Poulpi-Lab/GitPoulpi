//! The blocks of text the Copy buttons put on the clipboard: made to be
//! pasted into a chat with someone (or an AI) who has to explain a failure.
//! Always English, always redacted, never localised.

use super::feed::Entry;

// i18n-scan: off

/// Facts about this run: version, system, terminal, backend.
pub(crate) fn environment() -> Vec<String> {
    let mut out = vec![format!(
        "GitPoulpi {}{}",
        env!("CARGO_PKG_VERSION"),
        gitpoulpi_ipc::build_git_sha()
            .map(|sha| format!(" ({sha})"))
            .unwrap_or_default()
    )];
    out.push(format!(
        "os: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    let var = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    let mut terminal = Vec::new();
    for name in [
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
        "LC_TERMINAL",
        "TERM",
        "COLORTERM",
        "WT_SESSION",
    ] {
        if let Some(v) = var(name) {
            terminal.push(format!("{name}={v}"));
        }
    }
    let size = crossterm::terminal::size()
        .map(|(w, h)| format!("{w}x{h}"))
        .unwrap_or_else(|_| "unknown size".to_string());
    out.push(format!("terminal: {} ({size})", terminal.join(" ")));
    let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
    out.extend(super::capabilities::lines(
        &crate::terminal_image::detect::ambient_env,
        &|path| std::fs::read_to_string(path).ok(),
        &super::capabilities::current_facts(settings.as_ref().map(|s| &s.config().ui)),
    ));
    let backend = settings
        .as_ref()
        .map(|s| format!("{:?}", s.config().general.default_backend).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());
    out.push(format!("backend: {backend}"));
    out.push(format!("git: {}", git_version()));
    out
}

/// `git --version`, asked once on a thread of its own: the first report may
/// not have it yet, the next one does.
fn git_version() -> String {
    static VERSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    static ASKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if let Some(v) = VERSION.get() {
        return v.clone();
    }
    if !ASKED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        let _ = std::thread::Builder::new()
            .name("git-version".into())
            .spawn(|| {
                let text = std::process::Command::new(gitpoulpi_cli_backend::git_program())
                    .arg("--version")
                    .output()
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| "not found".to_string());
                let _ = VERSION.set(text);
            });
    }
    "(still being asked)".to_string()
}

/// Starts the `git --version` probe early (opening the Activity Log, the
/// first failure), so a report made a moment later has it.
pub(crate) fn warm_up() {
    let _ = git_version();
}

/// One entry as a report: header, the operation's facts, the raw message.
pub(crate) fn entry_block(entry: &Entry) -> String {
    let mut lines = vec!["GitPoulpi error report".to_string()];
    lines.extend(environment());
    lines.push(format!("time: {}", gitpoulpi_log::timestamp(entry.ts_ms)));
    lines.push(format!("title: {}", entry.title));
    lines.extend(context_lines(entry));
    if entry.context.is_none() {
        if let Some(action) = &entry.action_text {
            lines.push(format!("action: {action}"));
        }
    }
    if let Some(ms) = entry.duration_ms {
        lines.push(format!("duration: {ms} ms"));
    }
    if let Some(summary) = &entry.summary {
        lines.push(format!("reading: {summary}"));
    }
    lines.push(format!("message: {}", entry.message));
    if let Some(detail) = &entry.detail {
        lines.extend(detail.lines());
    }
    redacted(lines.join("\n"))
}

/// What produced the entry: "operation:", "arguments:", "repository:",
/// "requested by:", "operation id:". An entry with no context (a plain status
/// message) still names its repository.
pub(crate) fn context_lines(entry: &Entry) -> Vec<String> {
    let Some(context) = &entry.context else {
        return entry
            .repo
            .as_deref()
            .map(|repo| vec![format!("repository: {}", repo.display())])
            .unwrap_or_default();
    };
    let mut lines = context.report_lines();
    if context.repo.is_none() {
        if let Some(repo) = &entry.repo {
            lines.push(format!("repository: {}", repo.display()));
        }
    }
    lines
}

/// The "Copy diagnostics" block: the environment, the latest failure and the
/// tail of the log at the level shown.
pub(crate) fn diagnostics(
    latest: Option<&Entry>,
    log: &[gitpoulpi_log::Record],
    level: gitpoulpi_log::Level,
) -> String {
    let mut lines = vec!["GitPoulpi diagnostics".to_string()];
    lines.extend(environment());
    lines.push(format!("log level shown: {}", level.as_str()));
    if let Some(entry) = latest {
        lines.push(String::new());
        lines.push("latest problem:".to_string());
        lines.push(format!("  time: {}", gitpoulpi_log::timestamp(entry.ts_ms)));
        lines.push(format!("  title: {}", entry.title));
        lines.extend(context_lines(entry).into_iter().map(|l| format!("  {l}")));
        if entry.context.is_none() {
            if let Some(action) = &entry.action_text {
                lines.push(format!("  action: {action}"));
            }
        }
        lines.push(format!("  message: {}", entry.message));
        if let Some(detail) = &entry.detail {
            lines.extend(detail.lines().into_iter().map(|l| format!("  {l}")));
        }
    }
    lines.push(String::new());
    lines.push(format!("last {} log lines:", log.len()));
    let mut text = lines.join("\n");
    text.push('\n');
    text.push_str(&gitpoulpi_log::to_text(log));
    redacted(text)
}

fn redacted(text: String) -> String {
    gitpoulpi_core::redact(&text)
}

// i18n-scan: on

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_log::feed::Severity;
    use gitpoulpi_core::ErrorDetail;

    #[test]
    fn an_entry_block_names_the_command_and_stderr_and_hides_secrets() {
        let mut entry = Entry::new(
            Severity::Error,
            "Push failed".into(),
            "git push failed: fatal: Authentication failed for 'https://me:hunter2@host/r.git/'"
                .into(),
        );
        entry.detail = Some(ErrorDetail::from_git(
            &["push", "--porcelain", "origin"],
            Some(128),
            "fatal: Authentication failed for 'https://me:hunter2@host/r.git/'\n",
        ));
        let block = entry_block(&entry);
        assert!(block.starts_with("GitPoulpi error report"), "{block}");
        assert!(block.contains("command: git push --porcelain origin"));
        assert!(block.contains("exit code: 128"));
        assert!(block.contains("Authentication failed"));
        assert!(block.contains("title: Push failed"));
        assert!(!block.contains("hunter2"), "{block}");
    }

    #[test]
    fn a_report_names_the_action_that_failed_with_its_arguments() {
        let mut entry = Entry::new(
            Severity::Error,
            "The operation failed".into(),
            "backend error: no such remote: cm6".into(),
        );
        entry.action_text = Some(crate::activity_log::friendly::action_text(
            &crate::daemon_link::Action::RemoteRemove("cm6".into()),
        ));
        let block = entry_block(&entry);
        assert!(block.contains("title: The operation failed"), "{block}");
        assert!(block.contains("action: RemoteRemove(\"cm6\")"), "{block}");
    }

    #[test]
    fn an_entry_block_names_the_operation_and_the_repository() {
        let mut entry = Entry::new(
            Severity::Error,
            "Remove remote failed".into(),
            "backend error: no such remote: cm6".into(),
        );
        let mut context = gitpoulpi_ipc::op_context::context_of(
            gitpoulpi_core::domain::OperationKind::RemoteRemove,
            &serde_json::json!({ "name": "cm6" }),
            Some("/data/Projects/demo"),
            12,
        );
        context.frontend = Some("TUI".into());
        entry.context = Some(context);
        let block = entry_block(&entry);
        assert!(
            block.contains("operation: Remove remote (RemoteRemove)"),
            "{block}"
        );
        assert!(block.contains("arguments: remote=cm6"), "{block}");
        assert!(block.contains("repository: /data/Projects/demo"), "{block}");
        assert!(block.contains("requested by: TUI"), "{block}");
        assert!(block.contains("operation id: 12"), "{block}");
        // An entry with no context still says where it happened.
        let mut plain = Entry::new(Severity::Info, "Saved".into(), "ok".into());
        plain.repo = Some("/data/Projects/demo".into());
        assert!(entry_block(&plain).contains("repository: /data/Projects/demo"));
    }

    #[test]
    fn diagnostics_carry_the_environment_the_latest_problem_and_the_log() {
        let entry = Entry::new(Severity::Error, "Pull failed".into(), "boom".into());
        let record = gitpoulpi_log::Record {
            ts_ms: gitpoulpi_log::now_ms(),
            level: gitpoulpi_log::Level::Warn,
            process: "tui".into(),
            pid: 1,
            target: "daemon".into(),
            message: "token=abcdef123456 leaked?".into(),
            repo: None,
        };
        let text = diagnostics(Some(&entry), &[record], gitpoulpi_log::Level::Debug);
        assert!(text.starts_with("GitPoulpi diagnostics"));
        assert!(text.contains(&format!("GitPoulpi {}", env!("CARGO_PKG_VERSION"))));
        assert!(text.contains("os: "));
        assert!(text.contains("backend: "));
        assert!(text.contains("latest problem:"));
        assert!(text.contains("title: Pull failed"));
        assert!(text.contains("last 1 log lines:"));
        assert!(text.contains("log level shown: DEBUG"));
        assert!(text.contains("WARN"));
        assert!(!text.contains("abcdef123456"), "{text}");
    }
}
