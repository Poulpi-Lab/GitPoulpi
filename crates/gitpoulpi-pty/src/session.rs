use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{Child, CommandBuilder, MasterPty, PtySize};

use crate::{answer_queries, RepExpander, QUERY_CARRY};

/// What to run and how big its terminal is.
#[derive(Debug, Clone)]
pub struct SpawnOptions {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub cwd: Option<PathBuf>,
    pub env: Vec<(OsString, OsString)>,
    pub env_remove: Vec<OsString>,
    pub cols: u16,
    pub rows: u16,
    /// Lines of history kept above the screen.
    pub scrollback: usize,
}

impl SpawnOptions {
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: Vec::new(),
            env_remove: Vec::new(),
            cols: 120,
            rows: 40,
            scrollback: 0,
        }
    }
}

struct Inner {
    parser: vt100::Parser,
    rep: RepExpander,
    bytes: u64,
    last_output: Instant,
    eof: bool,
}

type SharedWriter = Arc<Mutex<Option<Box<dyn Write + Send>>>>;

fn io_err(e: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::other(e.to_string())
}

/// A child process attached to an emulated terminal.
pub struct PtySession {
    master: Option<Box<dyn MasterPty + Send>>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    writer: SharedWriter,
    inner: Arc<Mutex<Inner>>,
    reader: Option<std::thread::JoinHandle<()>>,
    rows: u16,
}

impl PtySession {
    pub fn spawn(options: SpawnOptions) -> std::io::Result<Self> {
        let pair = portable_pty::native_pty_system()
            .openpty(PtySize {
                rows: options.rows,
                cols: options.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(io_err)?;
        let mut cmd = CommandBuilder::new(&options.program);
        cmd.args(&options.args);
        if let Some(dir) = &options.cwd {
            cmd.cwd(dir);
        }
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        for name in &options.env_remove {
            cmd.env_remove(name);
        }
        for (key, value) in &options.env {
            cmd.env(key, value);
        }
        let child = pair.slave.spawn_command(cmd).map_err(io_err)?;
        // The slave must go now, or the pty never reaches EOF.
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().map_err(io_err)?;
        let writer = pair.master.take_writer().map_err(io_err)?;
        let inner = Arc::new(Mutex::new(Inner {
            parser: vt100::Parser::new(options.rows, options.cols, options.scrollback),
            rep: RepExpander::default(),
            bytes: 0,
            last_output: Instant::now(),
            eof: false,
        }));
        let writer: SharedWriter = Arc::new(Mutex::new(Some(writer)));
        let (thread_inner, thread_writer) = (Arc::clone(&inner), Arc::clone(&writer));
        let handle = std::thread::Builder::new()
            .name("pty-reader".into())
            .spawn(move || {
                let mut buf = vec![0u8; 65536];
                let mut carry: Vec<u8> = Vec::new();
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => {
                            if let Ok(mut inner) = thread_inner.lock() {
                                inner.eof = true;
                            }
                            return;
                        }
                        Ok(n) => {
                            let cursor = match thread_inner.lock() {
                                Ok(mut inner) => {
                                    let expanded = inner.rep.feed(&buf[..n]);
                                    inner.parser.process(&expanded);
                                    inner.bytes += n as u64;
                                    inner.last_output = Instant::now();
                                    inner.parser.screen().cursor_position()
                                }
                                Err(_) => (0, 0),
                            };
                            let mut hay = std::mem::take(&mut carry);
                            let from = hay.len();
                            hay.extend_from_slice(&buf[..n]);
                            let reply = answer_queries(&hay, from, cursor, false, None);
                            if !reply.is_empty() {
                                if let Ok(mut guard) = thread_writer.lock() {
                                    if let Some(w) = guard.as_mut() {
                                        let _ = w.write_all(&reply);
                                        let _ = w.flush();
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
            inner,
            reader: Some(handle),
            rows: options.rows,
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Sends bytes to the child as if typed.
    pub fn write(&self, bytes: &[u8]) -> std::io::Result<()> {
        let mut guard = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        match guard.as_mut() {
            Some(w) => {
                w.write_all(bytes)?;
                w.flush()
            }
            None => Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe)),
        }
    }

    /// The visible screen, rows joined by newlines, trailing blanks trimmed.
    pub fn screen_text(&self) -> String {
        self.lock().parser.screen().contents()
    }

    /// The history above the screen followed by the screen, one entry per
    /// logical line (rows the terminal soft-wrapped are joined back),
    /// trailing blank lines dropped. Needs `scrollback > 0`.
    pub fn full_lines(&self) -> Vec<String> {
        let mut inner = self.lock();
        inner.parser.set_scrollback(usize::MAX);
        let (rows, cols) = inner.parser.screen().size();
        // `vt100` 0.15 can look back at most one screenful at a time (a
        // larger offset overflows in its grid), so the reach of this is the
        // screen's height: callers wanting a long answer use a tall screen.
        let history = inner.parser.screen().scrollback().min(usize::from(rows));
        // (text, wrapped into the next row)
        let mut physical: Vec<(String, bool)> = Vec::new();
        // At offset `o` the top row is line `history - o` of the whole
        // text, so the top row of each offset walks the history in order.
        for offset in (1..=history).rev() {
            inner.parser.set_scrollback(offset);
            let screen = inner.parser.screen();
            if let Some(row) = screen.rows(0, cols).next() {
                physical.push((row, screen.row_wrapped(0)));
            }
        }
        inner.parser.set_scrollback(0);
        let screen = inner.parser.screen();
        for (n, row) in screen.rows(0, cols).enumerate() {
            physical.push((row, n < usize::from(rows) && screen.row_wrapped(n as u16)));
        }
        let mut lines: Vec<String> = Vec::new();
        let mut joining = false;
        for (text, wrapped) in physical {
            if joining {
                if let Some(last) = lines.last_mut() {
                    last.push_str(&text);
                }
            } else {
                lines.push(text);
            }
            joining = wrapped;
        }
        for line in &mut lines {
            *line = line.trim_end().to_string();
        }
        while lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }
        lines
    }

    /// The last `n` lines of the visible screen, up to the last one that
    /// holds text.
    pub fn last_screen_lines(&self, n: usize) -> Vec<String> {
        let inner = self.lock();
        let screen = inner.parser.screen();
        let (_, cols) = screen.size();
        let mut rows: Vec<String> = screen
            .rows(0, cols)
            .map(|r| r.trim_end().to_string())
            .collect();
        while rows.last().is_some_and(|l| l.is_empty()) {
            rows.pop();
        }
        let skip = rows.len().saturating_sub(n);
        rows.split_off(skip)
    }

    /// Whether the child turned bracketed paste on (`CSI ? 2004 h`).
    pub fn bracketed_paste(&self) -> bool {
        self.lock().parser.screen().bracketed_paste()
    }

    /// How long since the child last wrote anything.
    pub fn idle_for(&self) -> Duration {
        self.lock().last_output.elapsed()
    }

    pub fn bytes_received(&self) -> u64 {
        self.lock().bytes
    }

    /// The child's exit status, once it has exited.
    pub fn exited(&self) -> Option<String> {
        if let Ok(mut child) = self.child.lock() {
            if let Ok(Some(status)) = child.try_wait() {
                return Some(format!("{status}"));
            }
        }
        self.lock().eof.then(|| "closed".to_string())
    }

    /// Rows of the terminal.
    pub fn rows(&self) -> u16 {
        self.rows
    }

    /// Ends the session: kills the child, then closes the pty (which hangs
    /// up on whatever it started), without waiting for the reader beyond a
    /// short grace.
    ///
    /// Order matters on Windows: the ConPTY output pipe stays open until
    /// the pseudo console is closed, so the master is dropped *before* the
    /// reader is joined, and a reader that still has not returned is
    /// detached rather than waited for.
    pub fn stop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
        if let Ok(mut w) = self.writer.lock() {
            *w = None;
        }
        self.master = None;
        if let Some(handle) = self.reader.take() {
            let deadline = Instant::now() + Duration::from_secs(2);
            while !handle.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(5));
            }
            if handle.is_finished() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        self.stop();
    }
}
