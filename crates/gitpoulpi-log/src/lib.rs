//! The application log.
//!
//! A `tracing` sink with four jobs:
//!
//! - **a level**: events more verbose than the chosen level are not recorded
//!   at all ([`set_level`] changes it live);
//! - **an in-memory tail** (the last [`RING_CAPACITY`] records) for a view
//!   that must not touch the disk;
//! - **daily files** under one directory, `gitpoulpi-YYYY-MM-DD.log`, written
//!   by a background thread (the render thread only ever pushes into a
//!   channel), rotated at a size cap and pruned by age ([`prune`]);
//! - **redaction**: every message goes through [`gitpoulpi_core::redact`]
//!   before it is stored, so a token that ends up in a log call is not in the
//!   file, the tail or a pasted report.
//!
//! The TUI, a standalone daemon and (embedded) the CLI each append to the
//! same day's file, tagged `[tui:1234]`, so the Activity Log's application
//! tab shows the daemon's lines as well as the window's without any IPC.
//!
//! One line per record: `2026-10-08T14:03:22.123+02:00 WARN  [tui:4242]
//! target: message`. A message with newlines continues on lines that start
//! with a tab.

use std::collections::VecDeque;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};

use chrono::{DateTime, Local, NaiveDate, Utc};

/// How many records the in-memory tail keeps.
pub const RING_CAPACITY: usize = 2000;

/// The most of one file [`read_records`] looks at (its tail).
pub const READ_TAIL_BYTES: u64 = 2 * 1024 * 1024;

const FILE_PREFIX: &str = "gitpoulpi-";
const FILE_SUFFIX: &str = ".log";
const ROTATED_SUFFIX: &str = ".log.old";

/// A record's severity. `Error` is the most severe and the least verbose:
/// a level admits every level up to and including itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl Level {
    pub const ALL: [Level; 5] = [
        Level::Error,
        Level::Warn,
        Level::Info,
        Level::Debug,
        Level::Trace,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        }
    }

    /// `error`, `WARN`, ... (any case); `None` for anything else.
    pub fn parse(text: &str) -> Option<Level> {
        Level::ALL
            .into_iter()
            .find(|l| l.as_str().eq_ignore_ascii_case(text.trim()))
    }

    fn from_u8(n: u8) -> Level {
        Level::ALL
            .get(usize::from(n))
            .copied()
            .unwrap_or(Level::Trace)
    }

    fn from_tracing(level: &tracing::Level) -> Level {
        if *level == tracing::Level::ERROR {
            Level::Error
        } else if *level == tracing::Level::WARN {
            Level::Warn
        } else if *level == tracing::Level::INFO {
            Level::Info
        } else if *level == tracing::Level::DEBUG {
            Level::Debug
        } else {
            Level::Trace
        }
    }
}

/// One log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Unix milliseconds.
    pub ts_ms: i64,
    pub level: Level,
    /// Which process wrote it: `tui`, `daemon`, `cli`.
    pub process: String,
    pub pid: u32,
    pub target: String,
    pub message: String,
    /// The repository the line is about (its root path), when the event
    /// carried a `repo` field. `None` is an app-wide line (startup, config,
    /// the daemon itself): the Activity Log scopes by this.
    pub repo: Option<String>,
}

/// Escapes what would break the `[process:pid|repo]` tag: `%`, `]` and
/// line breaks.
fn escape_repo(repo: &str) -> String {
    let mut out = String::with_capacity(repo.len());
    for c in repo.chars() {
        match c {
            '%' => out.push_str("%25"),
            ']' => out.push_str("%5D"),
            '\n' => out.push_str("%0A"),
            '\r' => out.push_str("%0D"),
            c => out.push(c),
        }
    }
    out
}

fn unescape_repo(text: &str) -> String {
    text.replace("%5D", "]")
        .replace("%0A", "\n")
        .replace("%0D", "\r")
        .replace("%25", "%")
}

impl Record {
    /// The record as its one text line (plus tab-led continuation lines).
    pub fn to_line(&self) -> String {
        let when = DateTime::<Utc>::from_timestamp_millis(self.ts_ms)
            .unwrap_or_default()
            .with_timezone(&Local)
            .format("%Y-%m-%dT%H:%M:%S%.3f%:z");
        let message = self.message.replace('\n', "\n\t");
        let repo = self
            .repo
            .as_deref()
            .map(|r| format!("|{}", escape_repo(r)))
            .unwrap_or_default();
        format!(
            "{when} {:<5} [{}:{}{repo}] {}: {message}",
            self.level.as_str(),
            self.process,
            self.pid,
            self.target
        )
    }

    /// The time of day the record was written, local, `HH:MM:SS`.
    pub fn clock(&self) -> String {
        clock(self.ts_ms)
    }
}

/// Unix milliseconds now.
pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

/// `HH:MM:SS` local time of a unix-millisecond stamp.
pub fn clock(ts_ms: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(ts_ms)
        .unwrap_or_default()
        .with_timezone(&Local)
        .format("%H:%M:%S")
        .to_string()
}

/// `2026-10-08 14:03:22 +02:00`, for a report a person reads.
pub fn timestamp(ts_ms: i64) -> String {
    DateTime::<Utc>::from_timestamp_millis(ts_ms)
        .unwrap_or_default()
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S %:z")
        .to_string()
}

/// Parses text written by [`Record::to_line`]; lines that are not a record
/// (a torn write, a foreign file) are skipped, tab-led lines continue the
/// previous record.
pub fn parse_records(text: &str) -> Vec<Record> {
    let mut out: Vec<Record> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix('\t') {
            if let Some(last) = out.last_mut() {
                last.message.push('\n');
                last.message.push_str(rest);
            }
        } else if let Some(record) = parse_line(line) {
            out.push(record);
        }
    }
    out
}

fn parse_line(line: &str) -> Option<Record> {
    let (when, rest) = line.split_once(' ')?;
    let ts_ms = DateTime::parse_from_rfc3339(when).ok()?.timestamp_millis();
    let (level, rest) = rest.trim_start().split_once(' ')?;
    let level = Level::parse(level)?;
    let rest = rest.trim_start().strip_prefix('[')?;
    let (tag, rest) = rest.split_once("] ")?;
    let (process, pid) = tag.split_once(':')?;
    let (pid, repo) = match pid.split_once('|') {
        Some((pid, repo)) => (pid, Some(unescape_repo(repo))),
        None => (pid, None),
    };
    let (target, message) = rest.split_once(": ").unwrap_or(("", rest));
    Some(Record {
        ts_ms,
        level,
        process: process.to_string(),
        pid: pid.parse().ok()?,
        target: target.to_string(),
        message: message.to_string(),
        repo,
    })
}

/// The records as text, one line each, oldest first.
pub fn to_text(records: &[Record]) -> String {
    let mut out = String::new();
    for r in records {
        out.push_str(&r.to_line());
        out.push('\n');
    }
    out
}

/// What `init` needs.
#[derive(Debug, Clone)]
pub struct Options {
    /// Where the day files go; `None` keeps the log in memory only.
    pub dir: Option<PathBuf>,
    pub level: Level,
    /// Days of files kept, today included (at least 1).
    pub retention_days: u32,
    pub max_file_bytes: u64,
    /// `tui`, `daemon`, `cli`.
    pub process: &'static str,
}

enum Msg {
    Line(String),
    Flush(mpsc::SyncSender<()>),
    /// Empties the log files (see [`clear_dir`]); answers with the count.
    Clear(mpsc::SyncSender<usize>),
}

/// The log: a level, a tail and (optionally) a file writer.
pub struct Logger {
    level: AtomicU8,
    max_file_bytes: Arc<AtomicU64>,
    process: &'static str,
    dir: Option<PathBuf>,
    tx: Option<Mutex<mpsc::Sender<Msg>>>,
    ring: Mutex<VecDeque<Record>>,
}

impl Logger {
    /// Starts the writer thread when there is a directory. Pruning old files
    /// runs on a thread of its own: opening the app never waits for the disk.
    pub fn new(options: Options) -> Arc<Logger> {
        let max = Arc::new(AtomicU64::new(options.max_file_bytes.max(1024)));
        let tx = options.dir.clone().map(|dir| {
            let retention = options.retention_days.max(1);
            let prune_dir = dir.clone();
            let _ = std::thread::Builder::new()
                .name("log-prune".into())
                .spawn(move || {
                    prune(&prune_dir, Local::now().date_naive(), retention);
                });
            let (tx, rx) = mpsc::channel();
            let size = max.clone();
            let _ = std::thread::Builder::new()
                .name("log-writer".into())
                .spawn(move || writer(&dir, &rx, &size));
            Mutex::new(tx)
        });
        Arc::new(Logger {
            level: AtomicU8::new(options.level as u8),
            max_file_bytes: max,
            process: options.process,
            dir: options.dir,
            tx,
            ring: Mutex::new(VecDeque::new()),
        })
    }

    /// Changes the size at which a day's file is rotated.
    pub fn set_max_file_bytes(&self, bytes: u64) {
        self.max_file_bytes
            .store(bytes.max(1024), Ordering::Relaxed);
    }

    pub fn level(&self) -> Level {
        Level::from_u8(self.level.load(Ordering::Relaxed))
    }

    pub fn set_level(&self, level: Level) {
        self.level.store(level as u8, Ordering::Relaxed);
    }

    /// Whether a record at `level` would be kept.
    pub fn enabled(&self, level: Level) -> bool {
        level <= self.level()
    }

    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// Records `message` (redacted) at `level`, if the level admits it.
    pub fn emit(&self, level: Level, target: &str, message: &str) {
        self.emit_for(level, target, message, None);
    }

    /// [`Logger::emit`] for a line about the repository rooted at `repo`.
    pub fn emit_for(&self, level: Level, target: &str, message: &str, repo: Option<&str>) {
        if !self.enabled(level) {
            return;
        }
        let record = Record {
            ts_ms: Utc::now().timestamp_millis(),
            level,
            process: self.process.to_string(),
            pid: std::process::id(),
            target: target.to_string(),
            message: gitpoulpi_core::redact(message),
            repo: repo.filter(|r| !r.is_empty()).map(str::to_string),
        };
        if let Some(tx) = &self.tx {
            let line = record.to_line();
            if let Ok(tx) = tx.lock() {
                let _ = tx.send(Msg::Line(line));
            }
        }
        let mut ring = self.ring.lock().unwrap_or_else(|e| e.into_inner());
        if ring.len() >= RING_CAPACITY {
            ring.pop_front();
        }
        ring.push_back(record);
    }

    /// The in-memory tail, oldest first.
    pub fn recent(&self) -> Vec<Record> {
        self.ring
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .cloned()
            .collect()
    }

    /// Clears the log: the in-memory tail and every file in the directory
    /// (see [`clear_dir`]). The writer thread runs it between two lines, so
    /// nothing emitted before is written after, and it keeps writing
    /// normally afterwards. Returns how many files it emptied or removed.
    pub fn clear(&self) -> usize {
        self.ring.lock().unwrap_or_else(|e| e.into_inner()).clear();
        let Some(tx) = &self.tx else {
            return 0;
        };
        let (done_tx, done_rx) = mpsc::sync_channel(0);
        let sent = tx
            .lock()
            .map(|tx| tx.send(Msg::Clear(done_tx)).is_ok())
            .unwrap_or(false);
        if sent {
            done_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Waits until everything emitted so far is on disk (tests, and the
    /// moment a report is built).
    pub fn flush(&self) {
        if let Some(tx) = &self.tx {
            let (done_tx, done_rx) = mpsc::sync_channel(0);
            let sent = tx
                .lock()
                .map(|tx| tx.send(Msg::Flush(done_tx)).is_ok())
                .unwrap_or(false);
            if sent {
                let _ = done_rx.recv_timeout(std::time::Duration::from_secs(2));
            }
        }
    }
}

/// Appends lines to today's file, rotating at the size cap. Runs until every
/// sender is gone.
fn writer(dir: &Path, rx: &mpsc::Receiver<Msg>, max: &AtomicU64) {
    let _ = std::fs::create_dir_all(dir);
    let mut open: Option<(PathBuf, std::fs::File, u64)> = None;
    while let Ok(first) = rx.recv() {
        let mut batch = vec![first];
        while let Ok(more) = rx.try_recv() {
            batch.push(more);
        }
        let mut waiters = Vec::new();
        for msg in batch {
            match msg {
                Msg::Line(line) => append(dir, &mut open, &line, max.load(Ordering::Relaxed)),
                Msg::Flush(done) => waiters.push(done),
                Msg::Clear(done) => {
                    // Let go of the file first: the next line reopens it.
                    open = None;
                    let _ = done.send(clear_dir(dir, Local::now().date_naive()));
                }
            }
        }
        if let Some((_, file, _)) = open.as_mut() {
            let _ = file.flush();
        }
        for done in waiters {
            let _ = done.send(());
        }
    }
}

fn append(dir: &Path, open: &mut Option<(PathBuf, std::fs::File, u64)>, line: &str, max: u64) {
    let path = dir.join(file_name(Local::now().date_naive()));
    if open.as_ref().is_some_and(|(p, _, _)| *p != path) {
        *open = None;
    }
    if open.as_ref().is_some_and(|(_, _, size)| *size >= max) {
        // Over the cap: the full file becomes the day's one `.old` backup.
        *open = None;
        let _ = std::fs::rename(&path, rotated_name(&path));
    }
    if open.is_none() {
        let _ = std::fs::create_dir_all(dir);
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let size = file.metadata().map(|m| m.len()).unwrap_or(0);
            if size >= max {
                drop(file);
                let _ = std::fs::rename(&path, rotated_name(&path));
                if let Ok(file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                {
                    *open = Some((path.clone(), file, 0));
                }
            } else {
                *open = Some((path.clone(), file, size));
            }
        }
    }
    if let Some((_, file, size)) = open.as_mut() {
        let text = format!("{line}\n");
        if file.write_all(text.as_bytes()).is_ok() {
            *size += text.len() as u64;
        }
    }
}

fn rotated_name(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".old");
    PathBuf::from(name)
}

/// `gitpoulpi-2026-10-08.log`.
pub fn file_name(date: NaiveDate) -> String {
    format!("{FILE_PREFIX}{}{FILE_SUFFIX}", date.format("%Y-%m-%d"))
}

/// The file name of the day `days` days before today (tests seed old files
/// with it).
pub fn file_name_days_ago(days: i64) -> String {
    file_name(Local::now().date_naive() - chrono::Duration::days(days))
}

/// The day a log file name stands for (`.log` or its `.log.old` backup).
pub fn file_date(name: &str) -> Option<NaiveDate> {
    let stem = name
        .strip_prefix(FILE_PREFIX)?
        .strip_suffix(FILE_SUFFIX)
        .or_else(|| {
            name.strip_prefix(FILE_PREFIX)
                .and_then(|n| n.strip_suffix(ROTATED_SUFFIX))
        })?;
    NaiveDate::parse_from_str(stem, "%Y-%m-%d").ok()
}

/// Deletes the log files older than `keep_days` days counting `today` as
/// the first (1 keeps only today). Returns how many it removed. Other files
/// in the directory are never touched.
pub fn prune(dir: &Path, today: NaiveDate, keep_days: u32) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let keep = i64::from(keep_days.max(1));
    let mut removed = 0;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(date) = file_date(&name) else {
            continue;
        };
        if (today - date).num_days() >= keep && std::fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    removed
}

/// Empties the log: today's `.log` is truncated in place (a process that has
/// it open for appending, the daemon, keeps writing to the same inode, and
/// Windows refuses to delete an open file), every other log file (older
/// days, `.old` backups) is deleted, or truncated when it cannot be. Other
/// files in the directory are never touched. Returns how many files it
/// emptied or removed.
pub fn clear_dir(dir: &Path, today: NaiveDate) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let live = file_name(today);
    let mut cleared = 0;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_date(&name).is_none() {
            continue;
        }
        let path = entry.path();
        let removed = name != live && std::fs::remove_file(&path).is_ok();
        let emptied = removed
            || std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)
                .is_ok();
        if emptied {
            cleared += 1;
        }
    }
    cleared
}

/// Every record in the directory's log files, oldest first (by time), at
/// most `limit` of the newest. Reads only the tail of each file.
pub fn read_records(dir: &Path, limit: usize) -> Vec<Record> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(NaiveDate, bool, PathBuf)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            file_date(&name).map(|d| (d, !name.ends_with(ROTATED_SUFFIX), e.path()))
        })
        .collect();
    // Oldest day first, a day's backup before its live file.
    files.sort();
    let mut records = Vec::new();
    for (_, _, path) in files {
        records.extend(parse_records(&read_tail(&path)));
    }
    records.sort_by_key(|r| r.ts_ms);
    let skip = records.len().saturating_sub(limit);
    records.split_off(skip)
}

fn read_tail(path: &Path) -> String {
    let Ok(mut file) = std::fs::File::open(path) else {
        return String::new();
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let start = len.saturating_sub(READ_TAIL_BYTES);
    if start > 0 && file.seek(SeekFrom::Start(start)).is_err() {
        return String::new();
    }
    let mut bytes = Vec::new();
    let _ = file.read_to_end(&mut bytes);
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if start > 0 {
        // The cut landed mid-line: drop the torn first line.
        return text
            .split_once('\n')
            .map(|(_, rest)| rest.to_string())
            .unwrap_or_default();
    }
    text
}

// ---- the tracing subscriber and the process-wide logger ----

static GLOBAL: OnceLock<Arc<Logger>> = OnceLock::new();

/// Starts the process-wide log and installs it as the `tracing` subscriber.
/// The first call wins; later ones return the logger already running.
pub fn init(options: Options) -> Arc<Logger> {
    if let Some(existing) = GLOBAL.get() {
        return existing.clone();
    }
    let logger = Logger::new(options);
    let installed = GLOBAL.get_or_init(|| logger.clone()).clone();
    if Arc::ptr_eq(&installed, &logger) {
        // A test harness may already own the global subscriber: keep it.
        let _ = tracing::subscriber::set_global_default(Subscriber(installed.clone()));
    }
    installed
}

/// [`init`] with the user's `[log]` settings and the OS (or
/// `GITPOULPI_CONFIG_DIR`) log directory; the frontends' one call at
/// start-up. `process` is `tui`, `daemon` or `cli`.
pub fn init_from_settings(process: &'static str) -> Arc<Logger> {
    let paths = gitpoulpi_config::ConfigPaths::discover();
    let config = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().log.clone())
        .unwrap_or_default();
    init(Options {
        dir: paths.map(|p| p.log_dir()),
        level: level_of(config.level),
        retention_days: config.retention_days_clamped(),
        max_file_bytes: config.max_file_bytes(),
        process,
    })
}

/// The config's level as the log's.
pub fn level_of(level: gitpoulpi_config::LogLevel) -> Level {
    match level {
        gitpoulpi_config::LogLevel::Error => Level::Error,
        gitpoulpi_config::LogLevel::Warn => Level::Warn,
        gitpoulpi_config::LogLevel::Info => Level::Info,
        gitpoulpi_config::LogLevel::Debug => Level::Debug,
        gitpoulpi_config::LogLevel::Trace => Level::Trace,
    }
}

/// The process-wide logger, once [`init`] has run.
pub fn global() -> Option<Arc<Logger>> {
    GLOBAL.get().cloned()
}

/// Changes the recorded level of the process-wide log, if there is one.
pub fn set_level(level: Level) {
    if let Some(g) = GLOBAL.get() {
        g.set_level(level);
    }
}

/// The `tracing` subscriber feeding a [`Logger`].
pub struct Subscriber(pub Arc<Logger>);

#[derive(Default)]
struct Fields {
    message: String,
    /// The `repo` field: the repository the event is about.
    repo: Option<String>,
    rest: Vec<String>,
}

impl tracing::field::Visit for Fields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else if field.name() == "repo" {
            self.repo = Some(format!("{value:?}"));
        } else {
            self.rest.push(format!("{}={value:?}", field.name()));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else if field.name() == "repo" {
            self.repo = Some(value.to_string());
        } else {
            self.rest.push(format!("{}={value}", field.name()));
        }
    }
}

impl tracing::Subscriber for Subscriber {
    fn register_callsite(
        &self,
        _: &'static tracing::Metadata<'static>,
    ) -> tracing::subscriber::Interest {
        // The level changes while the app runs: ask `enabled` every time.
        tracing::subscriber::Interest::sometimes()
    }

    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(tracing::level_filters::LevelFilter::TRACE)
    }

    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        self.0.enabled(Level::from_tracing(metadata.level()))
    }

    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        tracing::span::Id::from_u64(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let mut fields = Fields::default();
        event.record(&mut fields);
        let mut message = fields.message;
        for extra in fields.rest {
            message.push(' ');
            message.push_str(&extra);
        }
        let meta = event.metadata();
        self.0.emit_for(
            Level::from_tracing(meta.level()),
            meta.target(),
            &message,
            fields.repo.as_deref(),
        );
    }

    fn enter(&self, _: &tracing::span::Id) {}

    fn exit(&self, _: &tracing::span::Id) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logger(dir: &Path, level: Level) -> Arc<Logger> {
        Logger::new(Options {
            dir: Some(dir.to_path_buf()),
            level,
            retention_days: 1,
            max_file_bytes: 1024 * 1024,
            process: "test",
        })
    }

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn a_record_round_trips_through_its_line_with_a_multiline_message() {
        let r = Record {
            ts_ms: 1_760_000_000_123,
            level: Level::Warn,
            process: "tui".into(),
            pid: 4242,
            target: "daemon".into(),
            message: "Push failed\nstderr: nope".into(),
            repo: None,
        };
        let text = r.to_line();
        assert_eq!(text.lines().count(), 2);
        assert_eq!(parse_records(&format!("{text}\n")), vec![r]);
        // Garbage lines are skipped, not fatal.
        assert!(parse_records("not a record\n\tstray continuation").is_empty());
    }

    #[test]
    fn events_below_the_level_are_not_recorded_and_the_level_is_live() {
        let dir = tempfile::tempdir().unwrap();
        let log = logger(dir.path(), Level::Warn);
        log.emit(Level::Error, "t", "an error");
        log.emit(Level::Warn, "t", "a warning");
        log.emit(Level::Info, "t", "info is below warn");
        assert_eq!(log.recent().len(), 2);
        log.set_level(Level::Debug);
        log.emit(Level::Info, "t", "now info counts");
        log.emit(Level::Trace, "t", "trace still does not");
        let all: Vec<_> = log.recent().into_iter().map(|r| r.message).collect();
        assert_eq!(all, ["an error", "a warning", "now info counts"]);
    }

    #[test]
    fn secrets_never_reach_the_ring_or_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let log = logger(dir.path(), Level::Info);
        log.emit(
            Level::Info,
            "t",
            "cloning https://me:hunter2@host/r.git with Authorization: Bearer abcdefghijkl and ghp_0123456789abcdefghij",
        );
        log.flush();
        let on_disk = read_records(dir.path(), 10);
        assert_eq!(on_disk.len(), 1);
        for text in [
            log.recent()[0].message.clone(),
            on_disk[0].message.clone(),
            std::fs::read_to_string(dir.path().join(file_name(Local::now().date_naive()))).unwrap(),
        ] {
            for secret in ["hunter2", "abcdefghijkl", "ghp_0123456789"] {
                assert!(!text.contains(secret), "{secret} leaked: {text}");
            }
        }
    }

    #[test]
    fn tracing_events_land_in_the_log_with_their_fields() {
        let dir = tempfile::tempdir().unwrap();
        let log = logger(dir.path(), Level::Info);
        tracing::subscriber::with_default(Subscriber(log.clone()), || {
            tracing::warn!(target: "daemon", "Push failed: {}", "boom");
            tracing::info!(code = 7, "with a field");
            tracing::debug!("filtered out");
        });
        let seen = log.recent();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].level, Level::Warn);
        assert_eq!(seen[0].target, "daemon");
        assert_eq!(seen[0].message, "Push failed: boom");
        assert_eq!(seen[1].message, "with a field code=7");
    }

    #[test]
    fn the_repo_field_becomes_the_records_repo_and_survives_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let log = logger(dir.path(), Level::Info);
        tracing::subscriber::with_default(Subscriber(log.clone()), || {
            tracing::warn!(target: "daemon", repo = "/work/a [x]/100%", "Push failed");
            tracing::info!(target: "daemon", "daemon started");
        });
        log.flush();
        let seen = log.recent();
        assert_eq!(seen[0].repo.as_deref(), Some("/work/a [x]/100%"));
        assert_eq!(
            seen[0].message, "Push failed",
            "the field is not in the text"
        );
        assert_eq!(seen[1].repo, None);
        let on_disk = read_records(dir.path(), 10);
        assert_eq!(on_disk.len(), 2);
        assert_eq!(on_disk[0].repo.as_deref(), Some("/work/a [x]/100%"));
        assert_eq!(on_disk[0].message, "Push failed");
        assert_eq!(on_disk[1].repo, None);
    }

    #[test]
    fn pruning_keeps_only_the_retention_window_and_only_log_files() {
        let dir = tempfile::tempdir().unwrap();
        for d in [
            "2026-10-01",
            "2026-10-05",
            "2026-10-06",
            "2026-10-07",
            "2026-10-08",
        ] {
            std::fs::write(dir.path().join(file_name(day(d))), "x").unwrap();
        }
        std::fs::write(dir.path().join("gitpoulpi-2026-10-01.log.old"), "x").unwrap();
        std::fs::write(dir.path().join("notes.txt"), "keep me").unwrap();
        std::fs::write(dir.path().join("gitpoulpi-latest.log"), "keep me").unwrap();

        // The day: only today survives.
        assert_eq!(prune(dir.path(), day("2026-10-08"), 1), 5);
        let left = |d: &Path| {
            let mut v: Vec<_> = std::fs::read_dir(d)
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            v.sort();
            v
        };
        assert_eq!(
            left(dir.path()),
            [
                "gitpoulpi-2026-10-08.log",
                "gitpoulpi-latest.log",
                "notes.txt"
            ]
        );

        // The week: seven days counting today.
        for d in ["2026-10-01", "2026-10-02", "2026-10-03", "2026-10-07"] {
            std::fs::write(dir.path().join(file_name(day(d))), "x").unwrap();
        }
        assert_eq!(prune(dir.path(), day("2026-10-08"), 7), 1);
        assert!(!dir.path().join(file_name(day("2026-10-01"))).exists());
        assert!(dir.path().join(file_name(day("2026-10-02"))).exists());
    }

    #[test]
    fn a_year_of_retention_keeps_old_days_and_drops_older_ones() {
        let dir = tempfile::tempdir().unwrap();
        let today = day("2026-10-08");
        for d in ["2026-10-08", "2026-06-01", "2025-10-10", "2025-10-08"] {
            std::fs::write(dir.path().join(file_name(day(d))), "x").unwrap();
        }
        // 365 days counting today: 2025-10-10 is the oldest kept day.
        assert_eq!(prune(dir.path(), today, 365), 1);
        assert!(!dir.path().join(file_name(day("2025-10-08"))).exists());
        assert!(dir.path().join(file_name(day("2025-10-10"))).exists());
        // Three months drops the year-old file and keeps June.
        assert_eq!(prune(dir.path(), today, 180), 1);
        assert!(dir.path().join(file_name(day("2026-06-01"))).exists());
    }

    #[test]
    fn a_file_over_the_cap_rotates_to_one_backup_and_both_are_read() {
        let dir = tempfile::tempdir().unwrap();
        let log = Logger::new(Options {
            dir: Some(dir.path().to_path_buf()),
            level: Level::Info,
            retention_days: 1,
            max_file_bytes: 1024,
            process: "test",
        });
        for i in 0..40 {
            log.emit(
                Level::Info,
                "t",
                &format!("line number {i:03} {}", "x".repeat(60)),
            );
            log.flush();
        }
        let today = Local::now().date_naive();
        let live = dir.path().join(file_name(today));
        assert!(live.exists());
        assert!(
            std::fs::metadata(&live).unwrap().len() <= 1024 + 200,
            "the live file stays near the cap"
        );
        assert!(rotated_name(&live).exists(), "a backup was made");
        let read = read_records(dir.path(), 1000);
        assert!(read.len() > 5 && read.last().unwrap().message.contains("039"));
        assert!(read.windows(2).all(|w| w[0].ts_ms <= w[1].ts_ms));
    }

    #[test]
    fn file_names_carry_their_date() {
        assert_eq!(file_name(day("2026-10-08")), "gitpoulpi-2026-10-08.log");
        assert_eq!(
            file_date("gitpoulpi-2026-10-08.log"),
            Some(day("2026-10-08"))
        );
        assert_eq!(
            file_date("gitpoulpi-2026-10-08.log.old"),
            Some(day("2026-10-08"))
        );
        assert_eq!(file_date("gitpoulpi-x.log"), None);
        assert_eq!(file_date("other-2026-10-08.log"), None);
    }
    #[test]
    fn clear_empties_every_day_keeps_the_live_file_and_the_writer_keeps_writing() {
        let dir = tempfile::tempdir().unwrap();
        let log = Logger::new(Options {
            dir: Some(dir.path().to_path_buf()),
            level: Level::Info,
            retention_days: 30,
            max_file_bytes: 1024 * 1024,
            process: "test",
        });
        let old = dir.path().join(file_name_days_ago(3));
        let backup = rotated_name(&dir.path().join(file_name_days_ago(0)));
        let record = Record {
            ts_ms: 1_760_000_000_000,
            level: Level::Info,
            process: "x".into(),
            pid: 1,
            target: "t".into(),
            message: "old line".into(),
            repo: None,
        };
        std::fs::write(&old, format!("{}\n", record.to_line())).unwrap();
        std::fs::write(&backup, format!("{}\n", record.to_line())).unwrap();
        let other = dir.path().join("notes.txt");
        std::fs::write(&other, "keep me").unwrap();
        log.emit(Level::Info, "t", "before the clear");
        log.flush();
        assert_eq!(read_records(dir.path(), 10).len(), 3);

        assert_eq!(log.clear(), 3);
        assert!(read_records(dir.path(), 10).is_empty());
        assert!(log.recent().is_empty());
        assert!(!old.exists() && !backup.exists(), "old days are deleted");
        let live = dir.path().join(file_name(Local::now().date_naive()));
        assert_eq!(
            std::fs::metadata(&live).unwrap().len(),
            0,
            "today is truncated"
        );
        assert_eq!(std::fs::read_to_string(&other).unwrap(), "keep me");

        log.emit(Level::Info, "t", "after the clear");
        log.flush();
        let after = read_records(dir.path(), 10);
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].message, "after the clear");
    }

    #[test]
    fn clear_without_a_directory_only_empties_the_memory_tail() {
        let log = Logger::new(Options {
            dir: None,
            level: Level::Info,
            retention_days: 1,
            max_file_bytes: 1024 * 1024,
            process: "test",
        });
        log.emit(Level::Info, "t", "hello");
        assert_eq!(log.clear(), 0);
        assert!(log.recent().is_empty());
    }
}
