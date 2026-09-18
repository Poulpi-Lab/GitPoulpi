//! The system clipboard, as one long-lived handle the whole TUI shares.
//!
//! # Why a handle and not a free function
//!
//! On X11 and Wayland the clipboard is not a buffer anybody owns: the
//! process that copied last is expected to *stay alive* and answer requests
//! for the data. Drop the handle, or exit, and the contents vanish (the
//! "selection ownership" problem arboard's README describes).
//!
//! So `run_in_repo` owns one handle for the whole session and every Copy
//! entry borrows it, as arboard documents. `SetExtLinux::wait()` is
//! deliberately *not* used: it blocks the calling thread until another
//! application pastes, and this runs on the input loop. A TUI that freezes
//! until you paste is worse than a clipboard that empties on exit.
//!
//! On Windows and macOS none of this applies: the OS copies the bytes into
//! a system-owned buffer and the value outlives the process.
//!
//! # Failure is a status-line message, never a panic
//!
//! Headless CI, a bare tty with no X11/Wayland display, or a Wayland
//! compositor without the data-control protocol make
//! `arboard::Clipboard::new()` fail. Every entry point returns a sentence
//! for `last_message` and nothing here can panic or block, which is why
//! `copy` takes `&mut self` and hands back a `String` rather than a
//! `Result` the caller could forget to look at.
//!
//! # Tests never touch the real clipboard
//!
//! A test that copies would overwrite whatever the developer running `cargo
//! test` had copied. Unit tests (`cfg(test)`) and any process started with
//! `GITPOULPI_CLIPBOARD=memory` (every pty test, set by `xtask::pty`) keep
//! copies in a private in-memory buffer, so copy/paste still round-trips and
//! "copied ..." is still what the status line says.
//! A pty test reads the copied text back whole through `GITPOULPI_CLIPBOARD_DUMP`
//! (a file path that receives every in-memory copy).

use gitpoulpi_i18n::msg;

/// Set to `memory` to keep copies in-process instead of on the system
/// clipboard.
const BACKEND_ENV: &str = "GITPOULPI_CLIPBOARD";

/// With `GITPOULPI_CLIPBOARD=memory`, a path here also receives every copied
/// text (the last copy wins), so a test can assert on it whole.
const DUMP_ENV: &str = "GITPOULPI_CLIPBOARD_DUMP";

enum Backend {
    System(arboard::Clipboard),
    Memory(String),
}

fn open_backend() -> Result<Backend, String> {
    if cfg!(test) || std::env::var(BACKEND_ENV).is_ok_and(|v| v == "memory") {
        return Ok(Backend::Memory(String::new()));
    }
    arboard::Clipboard::new()
        .map(Backend::System)
        .map_err(|e| e.to_string())
}

/// Lazily-opened, then kept for the process's life. `None` means "not
/// tried yet"; a stored `Err` means the last attempt failed and why.
pub(crate) struct Clipboard {
    inner: Option<Result<Backend, String>>,
}

impl Clipboard {
    pub(crate) fn new() -> Self {
        // Deliberately not opened here: on a headless machine opening one
        // is a failure, and a failure at startup would have to be either
        // swallowed or shown to a user who has not asked for a clipboard
        // yet. The first Copy is the first moment the answer matters.
        Self { inner: None }
    }

    /// Puts `value` on the system clipboard and returns the sentence to show
    /// in the status line, whether it worked or not.
    ///
    /// `what` names the thing in the user's own words ("short SHA", "author
    /// email") so the confirmation says what was copied without echoing a
    /// commit message into a one-line status bar.
    pub(crate) fn copy(&mut self, what: &str, value: &str) -> String {
        let handle = self.inner.get_or_insert_with(open_backend);
        let set = match handle {
            Ok(Backend::System(clipboard)) => clipboard
                .set_text(value.to_string())
                .map_err(|e| e.to_string()),
            Ok(Backend::Memory(buffer)) => {
                *buffer = value.to_string();
                // Test hook: the PTY tests cannot read an in-memory buffer
                // of another process, so each copy is also written here.
                if let Ok(path) = std::env::var(DUMP_ENV) {
                    let _ = std::fs::write(path, value);
                }
                Ok(())
            }
            Err(why) => return msg::input::no_clipboard(why),
        };
        match set {
            Ok(()) => msg::input::copied(what, preview(value)),
            Err(e) => {
                // Do not cache this as a dead handle: a transient
                // failure (another app holding the Windows clipboard
                // open) must not disable Copy for the rest of the
                // session.
                msg::input::copy_failed(what, e)
            }
        }
    }

    /// The clipboard's text, for the editor's `Ctrl+V` - most terminals turn a paste into a
    /// bracketed-paste event
    /// on their own, but a `Ctrl+V` that reaches the app as a key is
    /// answered from here. `Err` is the status-line sentence.
    pub(crate) fn paste(&mut self) -> Result<String, String> {
        let handle = self.inner.get_or_insert_with(open_backend);
        match handle {
            Ok(Backend::System(clipboard)) => {
                clipboard.get_text().map_err(msg::input::nothing_to_paste)
            }
            Ok(Backend::Memory(buffer)) => Ok(buffer.clone()),
            Err(why) => Err(msg::input::no_clipboard(why)),
        }
    }
}

thread_local! {
        /// The handle the dialogs' text fields copy through. The dialogs are
        /// owned structs with no path back to the event loop's handle, and a
        /// handle made per copy would be dropped, taking the copied text with it
        /// on X11/Wayland, the moment the copy returned. One per thread (the
        /// input loop's) lives for the session.
    static SHARED: std::cell::RefCell<Clipboard> = std::cell::RefCell::new(Clipboard::new());
}

/// [`Clipboard::copy`] through the thread's long-lived shared handle.
pub(crate) fn copy_shared(what: &str, value: &str) -> String {
    SHARED.with(|clipboard| clipboard.borrow_mut().copy(what, value))
}

/// A one-line echo of what was copied, so the confirmation is verifiable
/// without pasting. Newlines become spaces and anything long is cut --
/// a commit message's body must not push the status line off the screen.
fn preview(value: &str) -> String {
    const MAX: usize = 40;
    let flat: String = value
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let flat = flat.trim();
    if flat.chars().count() <= MAX {
        return flat.to_string();
    }
    let cut: String = flat.chars().take(MAX - 1).collect();
    format!("{}\u{2026}", cut.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_value_is_echoed_whole() {
        assert_eq!(preview("abc1234"), "abc1234");
    }

    #[test]
    fn a_multiline_message_is_flattened_and_cut_to_one_line() {
        let msg = "feat: a summary line\n\nand a body that goes on for a while indeed";
        let p = preview(msg);
        assert!(!p.contains('\n'), "newline survived into {p:?}");
        assert!(p.chars().count() <= 40, "too long: {p:?}");
        assert!(p.ends_with('\u{2026}'), "not marked as truncated: {p:?}");
    }

    #[test]
    fn exactly_forty_characters_is_not_truncated() {
        let v = "x".repeat(40);
        assert_eq!(preview(&v), v);
        assert_eq!(preview(&"x".repeat(41)).chars().count(), 40);
    }

    /// Under `cfg(test)` a copy goes to the in-memory buffer, never to the
    /// system clipboard of whoever runs `cargo test`.
    #[test]
    fn tests_copy_into_memory_and_paste_it_back() {
        let mut c = Clipboard::new();
        assert_eq!(c.copy("short SHA", "abc1234"), "copied short SHA (abc1234)");
        assert!(matches!(c.inner, Some(Ok(Backend::Memory(_)))));
        assert_eq!(c.paste().as_deref(), Ok("abc1234"));
    }
}
