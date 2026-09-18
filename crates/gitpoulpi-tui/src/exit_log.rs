//! Why the TUI ended, written down every time.
//!
//! A process that ends without saying why cannot be debugged after the
//! fact (a `gp` that closes leaves no `gitpoulpi-tui-panic.txt`, so it is not
//! a panic, and nothing in the Windows event log, so not a WER-visible
//! crash). So every end of the process appends one record to
//! `gitpoulpi-tui-exit.log`, beside the panic report:
//!
//! - a **user quit** says how (which key, which button) and what was on
//!   screen when it happened (the central view, the focused pane), plus the
//!   last few input events, because "I was only scrolling" and "`q` was
//!   pressed while a diff was open" are both answers;
//! - an **error** carries the whole `source()` chain and the same recent
//!   input, and the caller restores the terminal and keeps the message on
//!   screen rather than letting the window vanish with it;
//! - a **panic** points at the panic report, which has the backtrace.
//!
//! Appended, never rewritten, and capped at [`MAX_LOG_BYTES`]: past the cap
//! the oldest half is dropped, so the file holds the most recent exits.
//! Writing is best-effort: a log that cannot be written must never turn a
//! clean exit into a failing one.

use std::collections::VecDeque;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The log is trimmed back to its newest half once it passes this size.
pub(crate) const MAX_LOG_BYTES: u64 = 256 * 1024;

/// How many recent input events an exit record quotes.
pub(crate) const RECENT_INPUTS: usize = 8;

/// Where exit records go: the panic report's own directory, so both
/// "why did it stop" files are found in one place.
pub(crate) fn exit_log_path() -> PathBuf {
    let report = crate::event_loop::panic_report_path();
    report
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir)
        .join("gitpoulpi-tui-exit.log")
}

/// Why the process is ending.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExitReason {
    /// The user asked for it. `how` names the gesture (`"q key"`,
    /// `"close button"`, ...), `context` what was on screen.
    UserQuit { how: String, context: String },
    /// An error ended the event loop. `chain` is the error and each of
    /// its sources, outermost first.
    Error { chain: Vec<String> },
    /// The UI thread panicked; the backtrace is in the panic report.
    Panic { message: String },
}

impl ExitReason {
    /// Builds an [`ExitReason::Error`] from any error, walking its
    /// `source()` chain so a wrapped OS error is not reduced to its
    /// outermost message.
    pub(crate) fn from_error(err: &(dyn std::error::Error + 'static)) -> Self {
        let mut chain = vec![err.to_string()];
        let mut source = err.source();
        while let Some(e) = source {
            chain.push(e.to_string());
            source = e.source();
        }
        ExitReason::Error { chain }
    }

    fn is_user_requested(&self) -> bool {
        matches!(self, ExitReason::UserQuit { .. })
    }
}

/// The last few input events, oldest first, as short debug strings. Kept by
/// the event loop and quoted by every exit record: the last thing the user
/// did is usually the best clue to why the window went away.
#[derive(Debug, Default, Clone)]
pub(crate) struct RecentInput {
    events: VecDeque<String>,
}

impl RecentInput {
    pub(crate) fn push(&mut self, event: String) {
        if self.events.len() == RECENT_INPUTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub(crate) fn lines(&self) -> Vec<String> {
        self.events.iter().cloned().collect()
    }
}

/// The process-wide recent-input buffer. A global rather than an event
/// loop local because the record that needs it most - an error that
/// unwound out of `run_in_repo` with `?` - is written by `run`, after
/// every one of the loop's locals is gone.
static RECENT: std::sync::Mutex<RecentInput> = std::sync::Mutex::new(RecentInput {
    events: VecDeque::new(),
});

/// Notes one input event for the next exit record (motion is skipped,
/// see [`describe_event`]).
pub(crate) fn note_input(event: &crossterm::event::Event) {
    if let Some(line) = describe_event(event) {
        note_line(line);
    }
}

/// Notes a free-form line (an input error, say) for the next record.
pub(crate) fn note_line(line: String) {
    if let Ok(mut recent) = RECENT.lock() {
        recent.push(line);
    }
}

/// The buffer's current lines, oldest first.
pub(crate) fn recent_inputs() -> Vec<String> {
    RECENT.lock().map(|r| r.lines()).unwrap_or_default()
}

/// A one-line summary of an input event, short enough to keep eight of
/// in a log record: mouse motion is the bulk of all events and says
/// nothing, so it is not recorded at all (`None`).
pub(crate) fn describe_event(event: &crossterm::event::Event) -> Option<String> {
    use crossterm::event::{Event, KeyEventKind, MouseEventKind};
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            Some(format!("key {:?} {:?}", key.code, key.modifiers)) // i18n-english
        }
        Event::Key(_) => None,
        Event::Mouse(m) if matches!(m.kind, MouseEventKind::Moved) => None,
        Event::Mouse(m) => Some(format!("mouse {:?} at {},{}", m.kind, m.column, m.row)), // i18n-english
        Event::Resize(w, h) => Some(format!("resize {w}x{h}")), // i18n-english
        Event::FocusGained => Some("focus gained".into()),      // i18n-english
        Event::FocusLost => Some("focus lost".into()),          // i18n-english
        Event::Paste(text) => Some(format!("paste of {} bytes", text.len())), // i18n-english
    }
}

/// `2026-09-25T14:03:07Z` for a unix timestamp - UTC, because the log
/// is read next to other logs and a timezone-free stamp cannot be
/// misread.
pub(crate) fn utc_stamp(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86_400);
    let secs = unix_secs.rem_euclid(86_400);
    let (y, m, d) = crate::ui_kit::civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// The text of one record, ending in a blank line.
pub(crate) fn format_record(
    unix_secs: i64,
    reason: &ExitReason,
    session: &str,
    recent: &[String],
) -> String {
    let mut out = format!(
        "[{}] gitpoulpi-tui {} (pid {}, session {session}): ", // i18n-english
        utc_stamp(unix_secs),
        env!("CARGO_PKG_VERSION"),
        std::process::id()
    );
    match reason {
        ExitReason::UserQuit { how, context } => {
            out.push_str(&format!("user quit ({how}) while {context}\n")); // i18n-english
        }
        ExitReason::Error { chain } => {
            out.push_str("ERROR, the event loop ended with an error\n"); // i18n-english
            for (depth, message) in chain.iter().enumerate() {
                let label = if depth == 0 { "error" } else { "caused by" }; // i18n-english
                out.push_str(&format!("  {label}: {message}\n"));
            }
        }
        ExitReason::Panic { message } => {
            out.push_str(&format!(
                "PANIC: {message} (backtrace in {})\n", // i18n-english
                crate::event_loop::panic_report_path().display()
            ));
        }
    }
    if !reason.is_user_requested() || !recent.is_empty() {
        if recent.is_empty() {
            out.push_str("  recent input: (none)\n"); // i18n-english
        } else {
            out.push_str("  recent input, oldest first:\n"); // i18n-english
            for line in recent {
                out.push_str(&format!("    {line}\n"));
            }
        }
    }
    out.push('\n');
    out
}

/// Appends `record` to the log at `path`, trimming the file back to its
/// newest half first when it has grown past `max_bytes`. Errors are
/// returned for the tests; production callers ignore them.
pub(crate) fn append_record(path: &Path, record: &str, max_bytes: u64) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if size + record.len() as u64 > max_bytes {
        let old = std::fs::read(path).unwrap_or_default();
        let keep_from = old.len().saturating_sub((max_bytes / 2) as usize);
        // Restart at a record boundary so the file never opens mid-line.
        let tail = &old[keep_from..];
        let start = tail
            .windows(2)
            .position(|w| w == b"\n[")
            .map(|p| p + 1)
            .unwrap_or(tail.len());
        std::fs::write(path, &tail[start..])?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(record.as_bytes())
}

/// Writes one exit record for this process to [`exit_log_path`].
/// Best-effort: never fails, never panics.
pub(crate) fn record_exit(reason: &ExitReason, session: &str, recent: &[String]) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let record = format_record(now, reason, session, recent);
    let _ = append_record(&exit_log_path(), &record, MAX_LOG_BYTES);
}

/// Wraps an `io::Error` with what was being done when it happened, so
/// the exit record says *which* of the event loop's many `?` gave up --
/// an OS error message alone ("The handle is invalid.") does not.
pub(crate) fn io_context(what: &'static str) -> impl FnOnce(std::io::Error) -> std::io::Error {
    move |err| std::io::Error::new(err.kind(), ContextError { what, source: err })
}

#[derive(Debug)]
struct ContextError {
    what: &'static str,
    source: std::io::Error,
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} failed", self.what)
    }
}

impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Puts the terminal back the way a shell expects it. The event loop only
/// does this on its *normal* way out; an error that leaves it by `?` skips
/// that, leaving raw mode on the alternate screen with mouse reporting on,
/// so the error message would land on a screen about to be thrown away.
/// Every step is a harmless no-op when it was never entered.
pub(crate) fn restore_terminal() {
    let mut out = std::io::stdout();
    // First: the keyboard mode stack belongs to the alternate screen.
    crate::kitty_keyboard::disable(&mut out);
    crate::cursor::reset_pointer(&mut out);
    let _ = crossterm::execute!(
        out,
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    );
    let _ = crossterm::terminal::disable_raw_mode();
}

/// Says, on the restored terminal, why the TUI stopped and where the record
/// is, then, when a person is at the keyboard, waits for Enter before the
/// process ends: the window `gp` runs in closes with the process, and
/// without the wait the explanation would close with it.
///
/// `GITPOULPI_NO_EXIT_PAUSE` (any value) skips the wait, for scripts.
pub(crate) fn explain_error_and_pause(err: &std::io::Error) {
    use std::io::IsTerminal;
    let reason = ExitReason::from_error(err);
    let mut message = String::from("\ngitpoulpi: the TUI stopped because of an error:\n"); // i18n-english
    if let ExitReason::Error { chain } = &reason {
        for (depth, line) in chain.iter().enumerate() {
            let label = if depth == 0 { "error" } else { "caused by" }; // i18n-english
            message.push_str(&format!("  {label}: {line}\n"));
        }
    }
    message.push_str(&format!(
        "This was recorded in {}\n", // i18n-english
        exit_log_path().display()
    ));
    eprint!("{message}");
    if std::env::var_os("GITPOULPI_NO_EXIT_PAUSE").is_none() && std::io::stdin().is_terminal() {
        eprint!("Press Enter to close."); // i18n-english
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    }
}

/// How many input errors in a row the event loop tolerates before it
/// gives up. One failed console read (Windows re-queries the console
/// screen buffer on every mouse event to translate its coordinates) is
/// not a reason to throw the whole session away; a read that keeps
/// failing is, and is then reported with its cause.
pub(crate) const MAX_CONSECUTIVE_INPUT_ERRORS: u32 = 20;

/// Decides what one input read error means: `Ok(())` to skip this read
/// and carry on (the error is noted in `recent`), `Err` to end the loop
/// with it once too many have happened in a row.
pub(crate) fn tolerate_input_error(
    err: std::io::Error,
    consecutive: &mut u32,
) -> std::io::Result<()> {
    *consecutive += 1;
    note_line(format!("input error #{consecutive}: {err}")); // i18n-english
    if *consecutive >= MAX_CONSECUTIVE_INPUT_ERRORS {
        Err(err)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_utc_stamp_is_iso_8601() {
        // 2026-09-25 14:03:07 UTC.
        assert_eq!(utc_stamp(1_790_344_987), "2026-09-25T14:03:07Z");
        assert_eq!(utc_stamp(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn an_error_record_carries_the_whole_source_chain_and_the_recent_input() {
        let inner = std::io::Error::other("The handle is invalid.");
        let wrapped = io_context("reading terminal input")(inner);
        let reason = ExitReason::from_error(&wrapped);
        let record = format_record(0, &reason, "s", &["key Char('j') NONE".to_string()]);
        assert!(record.contains("ERROR"), "{record}");
        assert!(
            record.contains("error: reading terminal input failed"),
            "{record}"
        );
        assert!(
            record.contains("caused by: The handle is invalid."),
            "{record}"
        );
        assert!(record.contains("key Char('j') NONE"), "{record}");
        assert!(record.ends_with("\n\n"), "records are blank-line separated");
    }

    #[test]
    fn a_user_quit_names_the_gesture_and_the_screen() {
        let reason = ExitReason::UserQuit {
            how: "q key".into(),
            context: "Log central view".into(),
        };
        let record = format_record(0, &reason, "s", &[]);
        assert!(
            record.contains("user quit (q key) while Log central view"),
            "{record}"
        );
    }

    #[test]
    fn recent_input_keeps_only_the_newest_events() {
        let mut recent = RecentInput::default();
        for i in 0..(RECENT_INPUTS + 3) {
            recent.push(format!("e{i}"));
        }
        let lines = recent.lines();
        assert_eq!(lines.len(), RECENT_INPUTS);
        assert_eq!(lines[0], "e3");
        assert_eq!(lines.last().unwrap(), &format!("e{}", RECENT_INPUTS + 2));
    }

    #[test]
    fn mouse_motion_is_not_worth_a_line() {
        use crossterm::event::{Event, KeyModifiers, MouseEvent, MouseEventKind};
        let moved = Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(describe_event(&moved), None);
        let wheel = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 3,
            row: 4,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(
            describe_event(&wheel).as_deref(),
            Some("mouse ScrollDown at 3,4")
        );
    }

    #[test]
    fn the_log_is_appended_to_and_trimmed_at_a_record_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("exit.log");
        let record = |i: usize| format!("[{i:04}] {}\n\n", "x".repeat(40));
        for i in 0..40 {
            append_record(&path, &record(i), 1024).unwrap();
        }
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.len() as u64 <= 1024, "capped: {} bytes", text.len());
        assert!(text.starts_with('['), "starts on a record: {text:?}");
        assert!(text.contains("[0039]"), "the newest record is kept");
        assert!(!text.contains("[0000]"), "the oldest record is gone");
    }

    #[test]
    fn a_few_input_errors_are_tolerated_and_a_run_of_them_is_not() {
        let mut consecutive = 0;
        for _ in 1..MAX_CONSECUTIVE_INPUT_ERRORS {
            let err = std::io::Error::other("flaky");
            assert!(tolerate_input_error(err, &mut consecutive).is_ok());
        }
        let err = std::io::Error::other("flaky");
        assert!(tolerate_input_error(err, &mut consecutive).is_err());
        assert!(recent_inputs().iter().any(|l| l.contains("input error")));
    }
}
