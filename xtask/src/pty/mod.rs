//! Drive a terminal program through a pty and read back the screen it
//! painted -- no window, no focus, no compositor.
//!
//! This is the default way the GitPoulpi TUI is verified; see
//! `docs/developer/visual-verification.md` for when it is *not* enough.
//! Driving a real desktop is global state (agents steal each other's focus,
//! synthetic pointer input drops events, none of it runs in CI); a pty has
//! none of those properties: the bytes go down one file descriptor to one
//! process, any number of instances coexist, and the result is
//! deterministic.
//!
//! **Cross-platform by construction.** `portable-pty` gives a real pty on
//! Unix and a ConPTY on Windows behind one API, so this module carries no
//! `cfg(windows)`/`cfg(unix)` (architecture principle 4). The one place the
//! platforms genuinely differ is documented at `Terminal::stop`.
//! **Coordinates are zero-based `(col, row)`**, matching `ratatui::Rect`
//! and every hit target in the TUI. The SGR wire protocol is one-based;
//! that conversion happens once, in `mouse`.
//!
//! ```no_run
//! use std::time::Duration;
//! use xtask::pty::Terminal;
//!
//! let mut term = Terminal::builder("gitpoulpi-tui")
//!     .args(["-s", "my-session"])
//!     .cwd("/path/to/repo")
//!     .size(200, 50)
//!     .spawn()?;
//! // Readiness is "it drew something", never "it set a terminal mode" --
//! // see `wait_for_first_output` and `mouse_capture_enabled` for why.
//! term.wait_for_first_output(Duration::from_secs(20))?;
//! let (col, row) = term.wait_for_text("WORKTREES", Duration::from_secs(10))?;
//! term.click(col, row)?;
//! // The predicate carries the whole post-condition -- see `wait_until`.
//! term.wait_for("the section to collapse", Duration::from_secs(5), |s| {
//!     s.line(row).contains('\u{25b8}')
//! })?;
//! # Ok::<(), xtask::pty::Error>(())
//! ```

mod input;
pub mod keys;
pub mod mouse;
pub mod screen;
mod wait;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gitpoulpi_pty::{answer_queries_with, Replies, QUERY_CARRY};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize};

pub use mouse::Button;
pub use screen::{Cell, Color, Screen};
pub use wait::{scaled, TIME_SCALE_ENV};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("pty: {0}")]
    Pty(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("bad key spec: {0}")]
    Keys(String),
    /// Carries the screen as it stood when the deadline passed, plus how
    /// many bytes ever arrived: the byte count tells "the child drew
    /// nothing" from "the child drew and this driver failed to parse it".
    #[error(
        "timed out after {waited:?} waiting for {what}; \
         the child is alive and has written {bytes} bytes. Screen was:\n{screen}"
    )]
    Timeout {
        what: String,
        waited: Duration,
        bytes: u64,
        screen: String,
    },
    /// The child is alive but has produced nothing at all - a different
    /// failure from "drew the wrong thing", and one that points at the
    /// program or its launch rather than at the assertion.
    #[error(
        "timed out after {waited:?} waiting for {what}; \
         the child is alive but has written nothing at all. \
         Either it has not reached its first draw, or it never will."
    )]
    NoOutput { what: String, waited: Duration },
    /// The child is gone. Reported the moment it happens rather than at
    /// the deadline: waiting 60 seconds for a process that exited in the
    /// first 50 ms is pure cost, and four of those was most of a
    /// six-minute Windows CI job.
    #[error(
        "the child exited ({status}) while waiting for {what}, \
         after writing {bytes} bytes. Its output was:\n{screen}"
    )]
    ChildExited {
        what: String,
        status: String,
        bytes: u64,
        screen: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

/// How often `wait_until` re-reads the screen. Small enough that a test
/// is not paying for the poll, large enough not to spin a core.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// How long [`Terminal::stop`] will wait for the reader thread before
/// giving up on it and detaching.
///
/// Generous next to what a healthy shutdown costs (microseconds), and
/// nothing next to the 30-minute CI cap this exists to stay clear of.
const READER_GRACE: Duration = Duration::from_secs(5);

/// `JoinHandle::join`, but only if it will return promptly.
///
/// `std` has no join-with-deadline, and the whole point here is to be
/// unhangable, so this polls `is_finished` and joins only once the thread
/// is known to be done. Returns whether it was joined.
fn join_before(handle: std::thread::JoinHandle<()>, grace: Duration) -> bool {
    let deadline = Instant::now() + grace;
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            // Dropping a `JoinHandle` detaches the thread. It is parked
            // in a read on a pty nobody holds any more; the process is
            // on its way out regardless.
            return false;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    let _ = handle.join();
    true
}

/// Set to a number of milliseconds to make the reader thread crawl:
/// 64-byte reads with that delay between them, which turns a repaint
/// into hundreds of separately-observable fragments.
pub const TEAR_ENV: &str = "GITPOULPI_PTY_TEAR_MS";

/// How badly to tear frames on purpose, from [`TEAR_ENV`].
///
/// A torn frame is the one failure this driver cannot prevent (see
/// `wait_until`'s contract) and is invisible on a fast machine. With
/// `GITPOULPI_PTY_TEAR_MS=10` it is reproducible on any machine, every run.
///
/// Unset (the normal case) it costs one env lookup per spawn.
fn tear_delay() -> Option<Duration> {
    std::env::var(TEAR_ENV)
        .ok()?
        .parse()
        .ok()
        .map(Duration::from_millis)
}

/// What `Terminal::builder` collects before the child is spawned.
pub struct Builder {
    program: OsString,
    args: Vec<OsString>,
    cwd: Option<PathBuf>,
    env: Vec<(OsString, OsString)>,
    cols: u16,
    rows: u16,
    /// Whether this terminal answers the kitty keyboard protocol's
    /// `CSI ? u` query (see [`Builder::answer_kitty_keyboard`]).
    kitty_keyboard: bool,
    /// The color this terminal reports for an OSC 11 background query,
    /// as the `rgb:rrrr/gggg/bbbb` body (see [`Builder::answer_background`]).
    background: Option<String>,
    /// The parameters of the device attributes reply
    /// (see [`Builder::answer_attributes`]).
    attributes: Option<String>,
    /// The XTVERSION reply text (see [`Builder::answer_version`]).
    version: Option<String>,
    /// See [`Builder::mute_attributes`].
    mute_attributes: bool,
}

impl Builder {
    /// Makes this terminal answer the primary device attributes query with
    /// `CSI ? <params> c`; `62;4;22` is a terminal that draws sixel. The
    /// default is `1;2`, a plain VT100.
    pub fn answer_attributes(mut self, params: &str) -> Self {
        self.attributes = Some(params.to_string());
        self
    }

    /// Makes this terminal never answer the device attributes query, as a
    /// terminal that hears nothing of the start-up probe: the child has to
    /// go on by its own deadline. Do not use on Windows, where the console
    /// host does not start the child until it is answered.
    pub fn mute_attributes(mut self) -> Self {
        self.mute_attributes = true;
        self
    }

    /// Makes this terminal answer XTVERSION (`CSI > 0 q`) with `name`, as
    /// `DCS > | name ST`. Off by default: the child must cope with silence.
    pub fn answer_version(mut self, name: &str) -> Self {
        self.version = Some(name.to_string());
        self
    }

    /// Makes this terminal speak the kitty keyboard protocol far enough to
    /// answer the `CSI ? u` flags query (with `CSI ? 0 u`), as kitty,
    /// WezTerm and foot do. Off by default: a plain xterm-like pty
    /// answers only the device-attributes query, and the child must cope
    /// with that silence.
    pub fn answer_kitty_keyboard(mut self) -> Self {
        self.kitty_keyboard = true;
        self
    }

    /// Makes this terminal answer the OSC 11 background-color query with
    /// `rgb` (an `rgb:rrrr/gggg/bbbb` body), as most modern terminals do.
    /// Off by default: the child must cope with silence.
    pub fn answer_background(mut self, rgb: &str) -> Self {
        self.background = Some(rgb.to_string());
        self
    }

    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn cwd(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cwd = Some(dir.into());
        self
    }

    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// The terminal size the child is told about. Fix it in every test:
    /// a layout assertion is only meaningful at a known width, and the
    /// inherited size of whatever terminal ran `cargo test` is not one.
    pub fn size(mut self, cols: u16, rows: u16) -> Self {
        self.cols = cols;
        self.rows = rows;
        self
    }

    pub fn spawn(self) -> Result<Terminal> {
        Terminal::spawn(self)
    }
}

/// The write half, shared with the reader thread and droppable on
/// demand. See the `writer` field on [`Terminal`].
type SharedWriter = Arc<Mutex<Option<Box<dyn Write + Send>>>>;

/// Shared between the caller and the reader thread. The parser is fed
/// from the thread and read from `screen()`, so it lives behind a lock;
/// `bytes` backs `bytes_received`; `last_output` is kept for the same
/// diagnostic purpose.
struct Inner {
    parser: vt100::Parser,
    /// Expands the `CSI n b` a ConPTY sends, which the parser ignores.
    rep: gitpoulpi_pty::RepExpander,
    bytes: u64,
    /// How many reads delivered those bytes: a proxy for the child's writes.
    chunks: u64,
    /// Every byte the child wrote, verbatim, for tests that assert on
    /// escape sequences the cell grid cannot show (image protocols).
    raw: Vec<u8>,
    /// Bytes that arrived before anything was drawn - see
    /// [`Terminal::bytes_drawn`]. On Unix this stays 0.
    preamble: u64,
    /// Latched the first time the grid holds anything, which is what
    /// freezes `preamble`.
    drew: bool,
    last_output: Instant,
    eof: bool,
}

impl Inner {
    /// Feeds one chunk to the parser and keeps the byte accounting straight.
    ///
    /// **A ConPTY writes 27 bytes of its own** (mode setting, a clear,
    /// cursor placement) before the child has run at all. Counting those as
    /// the child's would make `bytes == 0` unreachable on Windows (so
    /// `Error::NoOutput` could never fire) and let `wait_for_first_output`
    /// return on the preamble before the child drew.
    ///
    /// So "has the child produced output?" is answered by the *grid*, which
    /// the preamble does not touch: bytes arriving while the grid is empty
    /// are preamble, and the first chunk that puts something on screen
    /// latches `drew` and freezes the baseline.
    ///
    /// On a Unix pty nothing arrives before the child writes, so `preamble`
    /// stays 0 and `bytes_drawn == bytes`.
    fn ingest(&mut self, chunk: &[u8]) {
        let expanded = self.rep.feed(chunk);
        self.parser.process(&expanded);
        self.raw.extend_from_slice(chunk);
        self.bytes += chunk.len() as u64;
        self.chunks += 1;
        self.last_output = Instant::now();

        if !self.drew {
            if grid_is_empty(self.parser.screen()) {
                self.preamble = self.bytes;
            } else {
                // If this one chunk carried both the preamble and the
                // child's first draw, the preamble is charged to the
                // child. That over-reports rather than under-reports:
                // the failure mode is a byte count a little too high,
                // never a child wrongly called silent.
                self.drew = true;
            }
        }
    }
}

/// Whether anything has been painted - the test for "the child has
/// drawn", as opposed to "bytes have arrived".
///
/// An untouched `vt100` cell has empty contents, and a cell a clear
/// wrote holds a space, so both read as blank here. That is what makes
/// a ConPTY's `\e[2J` preamble count as "nothing drawn".
fn grid_is_empty(screen: &vt100::Screen) -> bool {
    screen.contents().trim().is_empty()
}

/// A child process attached to an emulated terminal.
pub struct Terminal {
    // Held so the pty stays open; the reader thread has its own clone of
    // the read half and the writer is taken out once, below.
    //
    // `Option` so shutdown can *drop* it at a chosen moment: releasing it
    // before joining the reader avoids a Windows deadlock (see `stop`).
    master: Option<Box<dyn MasterPty + Send>>,
    // Behind a `Mutex` only so the `&self` wait methods can call
    // `try_wait`. That is what lets a wait fail the instant the program
    // under test dies, instead of sitting out its whole timeout -- the
    // difference between a 5-second CI failure that names an exit status
    // and a 60-second one that says nothing.
    child: Mutex<Box<dyn Child + Send + Sync>>,
    // Shared with the reader thread, which needs to write back: a
    // terminal has to *answer* the queries a pty layer asks it (see
    // `answer_queries`). The `Option` is shutdown's: dropping the inner
    // box is what closes the handle, and on Windows the reader stays
    // blocked until it does -- so this cannot be a plain `Arc`, whose
    // clone in the reader thread would keep the handle alive.
    writer: SharedWriter,
    inner: Arc<Mutex<Inner>>,
    reader: Option<std::thread::JoinHandle<()>>,
    cols: u16,
    rows: u16,
    /// When input was last written; `settle` skips deliberately paced input.
    last_input: Instant,
}

impl Terminal {
    pub fn builder(program: impl Into<OsString>) -> Builder {
        Builder {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            // Every Copy a pty test drives would otherwise land on the
            // real system clipboard of whoever ran `cargo test`,
            // clobbering what they had copied (a screenshot, say). The
            // TUI keeps copies in process memory instead -- see
            // `gitpoulpi-tui`'s `clipboard.rs`. A test may override it.
            env: vec![(
                OsString::from("GITPOULPI_CLIPBOARD"),
                OsString::from("memory"),
            )],
            cols: 120,
            rows: 40,
            kitty_keyboard: false,
            background: None,
            attributes: None,
            version: None,
            mute_attributes: false,
        }
    }

    fn spawn(builder: Builder) -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        let size = PtySize {
            rows: builder.rows,
            cols: builder.cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system
            .openpty(size)
            .map_err(|e| Error::Pty(e.to_string()))?;

        let mut cmd = CommandBuilder::new(&builder.program);
        cmd.args(&builder.args);
        if let Some(dir) = &builder.cwd {
            cmd.cwd(dir);
        }
        // Set unconditionally rather than under `cfg(unix)`: ConPTY ignores
        // it, and a `cfg` would put a platform difference in the *API*.
        // crossterm needs a terminal that advertises color, and an
        // inherited `TERM=dumb` from a CI runner would downgrade every
        // style the tests assert on.
        cmd.env("TERM", "xterm-256color");
        // Anything that makes the child's output depend on the machine
        // running the test is a flake waiting to happen. `NO_COLOR` set
        // in the developer's own shell would strip exactly the styling
        // the color assertions are there to check.
        cmd.env("COLORTERM", "truecolor");
        // The TUI's language follows the OS locale (`ui.language = auto`):
        // on a French machine every screen assertion would turn French.
        // Pinned to English like `TERM`; a test about languages overrides it
        // with `.env("GITPOULPI_LANG", ..)` (applied below).
        cmd.env("GITPOULPI_LANG", "en");
        cmd.env_remove("NO_COLOR");
        // The terminal the developer runs the tests in must not leak in
        // either: the TUI paints another gear under kitty/Ghostty, and the
        // tests find the standard one. A test about a terminal sets its own
        // variables with `.env(..)` (applied below).
        for name in [
            "KITTY_WINDOW_ID",
            "GHOSTTY_RESOURCES_DIR",
            "TERM_PROGRAM",
            "LC_TERMINAL",
            "ITERM_SESSION_ID",
        ] {
            cmd.env_remove(name);
        }
        // Nor a multiplexer the developer runs them in: the TUI sends no
        // pointer shape (`OSC 22`) and no picture through one.
        for name in [
            "TMUX",
            "ZELLIJ",
            "STY",
            "HERDR_ENV",
            "HERDR_PANE_ID",
            "HERDR_SOCKET_PATH",
            "HERDR_TAB_ID",
            "HERDR_WORKSPACE_ID",
        ] {
            cmd.env_remove(name);
        }
        // Nor the developer's AI credentials: a test reaches a model only
        // through a fake agent or the mock host, never a real account.
        for name in [
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "OPENAI_API_KEY",
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY",
            "MISTRAL_API_KEY",
            "CLAUDE_CODE_OAUTH_TOKEN",
        ] {
            cmd.env_remove(name);
        }
        // On Windows a ConPTY holds the child alone in its console, which the
        // TUI reads as "started by a double-click" (launcher start: another
        // startup tab, a possible relaunch in a terminal). Say a shell shares
        // it, as it does for a user typing the command; a test about the
        // launcher passes `--launcher`.
        cmd.env("GITPOULPI_TEST_SHARED_CONSOLE", "1");
        // A PTY test never reaches a real provider: the TUI and the daemon it
        // starts refuse non-loopback hosts (`gitpoulpi_hosting::http::
        // external_host_refusal`) unless the live-test switch is set.
        cmd.env("GITPOULPI_BLOCK_EXTERNAL_HOSTS", "1");
        // Tests never touch the developer's real user state: when the child
        // has its own config dir, its home, app-data folders and global git
        // config (and with it the credential helper) live inside it.
        if let Some((_, dir)) = builder
            .env
            .iter()
            .rev()
            .find(|(k, _)| k == "GITPOULPI_CONFIG_DIR")
        {
            for (key, value) in isolated_user_env(std::path::Path::new(dir)) {
                cmd.env(key, value);
            }
        }
        for (key, value) in &builder.env {
            cmd.env(key, value);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| Error::Pty(e.to_string()))?;
        // The slave handle must go now. While this process still holds
        // one, the pty never reaches EOF when the child exits, and the
        // reader thread below would hang forever instead of noticing.
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| Error::Pty(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| Error::Pty(e.to_string()))?;

        let inner = Arc::new(Mutex::new(Inner {
            // No scrollback: the TUI lives on the alternate screen, and
            // keeping scrollback only invites assertions against rows
            // that are not on screen.
            parser: vt100::Parser::new(builder.rows, builder.cols, 0),
            rep: gitpoulpi_pty::RepExpander::default(),
            bytes: 0,
            chunks: 0,
            raw: Vec::new(),
            preamble: 0,
            drew: false,
            last_output: Instant::now(),
            eof: false,
        }));

        let writer: SharedWriter = Arc::new(Mutex::new(Some(writer)));

        let tear = tear_delay();
        let kitty = builder.kitty_keyboard;
        let background = builder.background.clone();
        let attributes = builder.attributes.clone();
        let version = builder.version.clone();
        let mute_attributes = builder.mute_attributes;
        let thread_inner = Arc::clone(&inner);
        let thread_writer = Arc::clone(&writer);
        let handle = std::thread::Builder::new()
            .name("pty-reader".to_string())
            .spawn(move || {
                // 64 KiB, not the more usual 8: a full 200x50 repaint runs
                // to tens of kilobytes, and every chunk boundary is an
                // instant at which `screen()` can observe half of one frame
                // and half of the next. A bigger buffer makes that rarer but
                // cannot close it (the pty chunks the writes itself); the
                // real guarantee is the contract on `wait_until`.
                let mut buf = vec![0u8; if tear.is_some() { 64 } else { 65536 }];
                // The last few bytes of the previous read, so a query
                // split across a chunk boundary is still recognized.
                let mut carry: Vec<u8> = Vec::new();
                loop {
                    match std::io::Read::read(&mut reader, &mut buf) {
                        Ok(0) | Err(_) => {
                            if let Ok(mut inner) = thread_inner.lock() {
                                inner.eof = true;
                            }
                            return;
                        }
                        Ok(n) => {
                            if let Some(delay) = tear {
                                std::thread::sleep(delay);
                            }
                            let cursor = match thread_inner.lock() {
                                Ok(mut inner) => {
                                    inner.ingest(&buf[..n]);
                                    inner.parser.screen().cursor_position()
                                }
                                Err(_) => (0, 0),
                            };

                            // A terminal answers what it is asked --
                            // and on some console hosts the child does
                            // not start until it does. See
                            // `answer_queries`.
                            let mut hay = std::mem::take(&mut carry);
                            let from = hay.len();
                            hay.extend_from_slice(&buf[..n]);
                            let reply = answer_queries_with(
                                &hay,
                                from,
                                cursor,
                                &Replies {
                                    kitty,
                                    background: background.as_deref(),
                                    attributes: attributes.as_deref(),
                                    version: version.as_deref(),
                                    mute_attributes,
                                },
                            );
                            if !reply.is_empty() {
                                if let Ok(mut guard) = thread_writer.lock() {
                                    if let Some(writer) = guard.as_mut() {
                                        let _ = writer.write_all(&reply);
                                        let _ = writer.flush();
                                    }
                                }
                            }
                            let keep = hay.len().saturating_sub(QUERY_CARRY);
                            carry = hay[keep..].to_vec();
                        }
                    }
                }
            })?;

        Ok(Self {
            master: Some(pair.master),
            child: Mutex::new(child),
            writer,
            last_input: Instant::now(),
            inner,
            reader: Some(handle),
            cols: builder.cols,
            rows: builder.rows,
        })
    }

    // --- output -----------------------------------------------------

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // A poisoned lock here means the reader thread panicked, which is
        // a bug in this module rather than in the program under test --
        // take the data anyway so the caller sees the real screen in the
        // failure message rather than a second panic.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// An owned snapshot of the screen as it stands right now (the waits
    /// poll this one).
    fn screen_now(&self) -> Screen {
        let inner = self.lock();
        snapshot(inner.parser.screen(), self.cols, self.rows)
    }

    /// An owned snapshot of the screen. On Windows it is taken once the grid
    /// has held still for a moment: ConPTY forwards a frame in pieces, and
    /// a test that reads the screen right after a wait would otherwise find
    /// half a menu (`wait.rs`, `after_frame_settled`). Elsewhere it is the
    /// screen as it stands.
    pub fn screen(&self) -> Screen {
        if cfg!(windows) {
            return self.settled_screen();
        }
        self.screen_now()
    }

    /// Convenience for the overwhelmingly common assertion shape.
    pub fn text(&self) -> String {
        self.screen().text()
    }

    /// True once the child has turned mouse reporting on, which for this
    /// TUI means `EnableMouseCapture` has run.
    ///
    /// **Not a portable readiness signal.** It observes *terminal state*,
    /// not screen content, and a pseudo-console may consume a mode-setting
    /// sequence into its own state instead of forwarding it (Windows'
    /// ConPTY re-renders what the child writes). Wait for something the
    /// program *drew* instead - see `wait_for_first_output` and `wait_for`.
    ///
    /// On a plain Unix pty it precisely says the program is past startup
    /// and listening for mouse reports, useful when driving a screen by hand.
    pub fn mouse_capture_enabled(&self) -> bool {
        self.lock().parser.screen().mouse_protocol_mode() != vt100::MouseProtocolMode::None
    }

    /// The terminal's cursor as the child left it: `(col, row, visible)`,
    /// zero-based. Where the text caret shows when `ui.text_cursor = terminal`.
    pub fn cursor(&self) -> (u16, u16, bool) {
        let inner = self.lock();
        let screen = inner.parser.screen();
        let (row, col) = screen.cursor_position();
        (col, row, !screen.hide_cursor())
    }

    /// True while the child has bracketed paste (`CSI ? 2004 h`) on.
    pub fn bracketed_paste_enabled(&self) -> bool {
        self.lock().parser.screen().bracketed_paste()
    }

    /// True once the child has switched to the alternate screen.
    pub fn alternate_screen(&self) -> bool {
        self.lock().parser.screen().alternate_screen()
    }

    /// Whether the child's end of the pty has closed - it exited.
    ///
    /// On Windows this alone is not enough: ConPTY keeps its output pipe
    /// open until the pseudo console itself is closed, so a child that
    /// has exited is never seen as end-of-file by the reader. The child's
    /// own exit status is asked for as well, which is the answer on every
    /// platform.
    pub fn exited(&self) -> bool {
        if self.lock().eof {
            return true;
        }
        self.child
            .lock()
            .map(|mut child| matches!(child.try_wait(), Ok(Some(_))))
            .unwrap_or(false)
    }

    /// Every byte the child wrote so far, verbatim (escape sequences
    /// included), for assertions the cell grid cannot express.
    pub fn raw_output(&self) -> Vec<u8> {
        self.lock().raw.clone()
    }

    /// Every byte that has come down the pty, whoever wrote it, including a
    /// ConPTY's own preamble: truthful about the wire.
    ///
    /// For "did the *child* draw?", use [`bytes_drawn`](Self::bytes_drawn):
    /// this one is never 0 on Windows.
    pub fn bytes_received(&self) -> u64 {
        self.lock().bytes
    }

    /// How many reads have delivered output so far: a proxy for how many
    /// times the child wrote to the terminal.
    pub fn chunks_received(&self) -> u64 {
        self.lock().chunks
    }

    /// The child's process id, while it is alive.
    pub fn process_id(&self) -> Option<u32> {
        self.child.lock().ok()?.process_id()
    }

    /// How many bytes the child has written since it started, with any
    /// preamble the pty layer wrote for itself discounted (see
    /// [`Inner::ingest`]).
    ///
    /// Zero after several seconds means the child never drew, not that the
    /// assertion is wrong. Every wait and failure message is phrased in
    /// these bytes, so they mean the same thing on every platform.
    pub fn bytes_drawn(&self) -> u64 {
        let inner = self.lock();
        inner.bytes.saturating_sub(inner.preamble)
    }

    // --- shutdown ---------------------------------------------------

    /// Kills the child and waits for the reader thread to finish.
    ///
    /// The one place the platforms genuinely differ, handled by
    /// `portable-pty` rather than a `cfg`: on Unix a signal to the process,
    /// on Windows `TerminateProcess` against the ConPTY's child.
    ///
    /// # Why the master is dropped before the join
    ///
    /// On Windows killing the child does not end the reader: a ConPTY keeps
    /// the write end of the pipe open for as long as *this* process holds
    /// the master (and the writer taken from it), so the reader stays
    /// parked in a `read()` that never returns. Join first and the three
    /// wait on each other forever: the join waits for the read, the read
    /// waits for the master to close, the master closes after the join.
    ///
    /// So: kill the child, release every handle to the pty, *then* join.
    /// On a Unix pty the order is immaterial (EOF arrives when the child
    /// dies), which is why this is ordering rather than a `cfg`.
    ///
    /// # And why the join is bounded
    ///
    /// A test that hangs is worse than one that fails. If the reader has
    /// not come home within [`READER_GRACE`] this gives up and leaves it
    /// detached (the process is exiting anyway).
    ///
    /// On Windows that is only a warning: the child is already killed and
    /// reaped, and under a parallel load the console host can take longer
    /// than the grace to close its end of the pipe, which failed a test whose
    /// every assertion had passed. On Unix EOF follows the child's death, so
    /// a reader still blocked there means something else holds the pty.
    pub fn stop(mut self) -> Result<()> {
        self.shutdown();
        let Some(handle) = self.reader.take() else {
            return Ok(());
        };
        if join_before(handle, READER_GRACE) {
            return Ok(());
        }
        let msg = format!(
            "the pty reader was still blocked {READER_GRACE:?} after the child was \
             killed and the pty closed; detaching it"
        );
        if cfg!(windows) {
            eprintln!("warning: {msg}");
            Ok(())
        } else {
            Err(Error::Pty(msg))
        }
    }

    /// Kills the child but keeps the `Terminal`, so a test can watch what
    /// the driver does when the program under test goes away - which is
    /// how `exited()` earns its keep as "the program died" rather than
    /// "the program is slow".
    pub fn kill_child(&mut self) {
        self.shutdown();
    }

    /// Kill the child and let go of the pty, in that order. Dropping the
    /// writer and the master is what unblocks the reader thread on Windows
    /// (see `stop`); done here too so a test that panics mid-assertion
    /// leaves no thread parked in a read.
    fn shutdown(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Ok(mut guard) = self.writer.lock() {
            drop(guard.take());
        }
        drop(self.master.take());
    }

    /// The child's exit status, if it has one yet. `None` means still
    /// running.
    fn child_status(&self) -> Option<String> {
        let mut child = self.child.lock().ok()?;
        match child.try_wait() {
            Ok(Some(status)) => Some(match status.signal() {
                Some(signal) => format!("killed by {signal}"),
                None => format!("exit code {}", status.exit_code()),
            }),
            // An error here means the handle is unusable, which is itself
            // "the child is not there any more" as far as a test cares.
            Ok(None) => None,
            Err(e) => Some(format!("status unavailable: {e}")),
        }
    }
}

impl Drop for Terminal {
    /// A test that panics mid-assertion must not leave a TUI process (and
    /// its in-process daemon) alive. Killing in `Drop` rather than only
    /// in `stop` is what makes `assert!` safe to use between steps.
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn snapshot(screen: &vt100::Screen, cols: u16, rows: u16) -> Screen {
    let mut cells = Vec::with_capacity(cols as usize * rows as usize);
    for row in 0..rows {
        for col in 0..cols {
            cells.push(match screen.cell(row, col) {
                Some(cell) => Cell {
                    text: cell.contents(),
                    fg: convert_color(cell.fgcolor()),
                    bg: convert_color(cell.bgcolor()),
                    bold: cell.bold(),
                    italic: cell.italic(),
                    underline: cell.underline(),
                    inverse: cell.inverse(),
                    wide_continuation: cell.is_wide_continuation(),
                },
                None => Cell {
                    text: String::new(),
                    fg: Color::Default,
                    bg: Color::Default,
                    bold: false,
                    italic: false,
                    underline: false,
                    inverse: false,
                    wide_continuation: false,
                },
            });
        }
    }
    Screen::from_cells(cols, rows, cells)
}

fn convert_color(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Default,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

/// The environment that cuts a child process off from the developer's real
/// user state, rooted in `dir` (a test's temp config dir): `HOME`,
/// `USERPROFILE`, `APPDATA` and `LOCALAPPDATA` become subdirectories, the
/// system git config is ignored and the global one is a file in `dir` whose
/// credential helper is `store --file <dir>/git-credentials`. So connecting
/// an integration in a test stores its token in a throwaway file, never in
/// the machine's keychain or Credential Manager, and concurrent tests cannot
/// overwrite each other's token. Creates the directories and the git config.
pub fn isolated_user_env(dir: &std::path::Path) -> Vec<(&'static str, OsString)> {
    let home = dir.join("home");
    let appdata = dir.join("appdata");
    let local = dir.join("localappdata");
    for d in [&home, &appdata, &local] {
        let _ = std::fs::create_dir_all(d);
    }
    let gitconfig = dir.join("global.gitconfig");
    if !gitconfig.exists() {
        let creds = dir
            .join("git-credentials")
            .to_string_lossy()
            .replace('\\', "/");
        let _ = std::fs::write(
            &gitconfig,
            format!(
                "[user]\n\tname = GitPoulpi Test\n\temail = test@gitpoulpi.invalid\n\
                 [credential]\n\thelper = store --file \"{creds}\"\n"
            ),
        );
    }
    let mut env = vec![
        ("GIT_CONFIG_GLOBAL", gitconfig.into_os_string()),
        ("GIT_CONFIG_NOSYSTEM", OsString::from("1")),
    ];
    // Not on Windows: redirected there, the Git for Windows shell (sh.exe,
    // ssh-keygen) failed to start under the parallel runner ("add_item
    // (\"\\??\\C:\\Program Files\\Git\", \"/\") failed, errno 1") in 8 PTY
    // tests. The git config above keeps the credential helper isolated.
    if cfg!(not(windows)) {
        env.extend([
            ("HOME", home.clone().into_os_string()),
            ("USERPROFILE", home.into_os_string()),
            ("APPDATA", appdata.into_os_string()),
            ("LOCALAPPDATA", local.into_os_string()),
        ]);
    }
    env
}
