//! `cargo run -p xtask -- perf --tui`: what the real `gitpoulpi-tui`
//! release binary costs while it sits there and while a key is held.
//!
//! It drives the binary through the same pty the PTY tests use, on the
//! `perf` fixture, with an isolated config directory, and measures:
//!
//! - **idle**: CPU of the process (and of its `gitpoulpi-ui` thread, on
//!   Linux), terminal bytes and writes per second over a quiet window;
//! - **held key**: a held Down at a typematic rate: wall time until the
//!   screen has caught up, CPU and bytes;
//! - **latency**: key to first output byte and key to a changed screen,
//!   p50 / p90 / p95.
//!
//! CPU comes from `/proc/<pid>/task/*/stat` on Linux and `GetProcessTimes`
//! on Windows. The budgets (docs/developer/perf-budgets.md) always print as
//! ratios; they fail the run (exit 1) only under `--enforce`, which is what
//! a pre-merge check should pass.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::perf::{build_fixture, fixture_ready, today, workspace_root};
use crate::pty::Terminal;

pub const USAGE: &str = "\
Usage: cargo run -p xtask -- perf --tui [options]

Measures the release TUI binary on the perf fixture (needs one built:
`cargo build --release -p gitpoulpi-tui`, or pass --build).

  --build            build the release binary first
  --bin <path>       measure this binary (default target/release/gitpoulpi-tui)
  --fixture <dir>    fixture location (default target/perf/fixture)
  --files <n>        fixture size, default 2000
  --commits <n>      fixture history, default 5000
  --idle <secs>      idle window, default 10
  --held <n>         held-key repeats, default 300
  --hz <n>           held-key rate, default 100
  --samples <n>      latency samples, default 40
  --merge-only       only the merge conflict editor scenarios
  --startup-only     only the startup rows (spawn -> repository shown)
  --no-merge         skip the merge conflict editor scenarios
                     (the startup rows, clean and with a leftover port file, always run)
  --enforce          exit 1 when a budget is exceeded (default: report only;
                     budgets in docs/developer/perf-budgets.md)";

struct Opts {
    bin: Option<PathBuf>,
    build: bool,
    fixture: PathBuf,
    files: usize,
    commits: usize,
    idle: u64,
    held: usize,
    hz: u64,
    samples: usize,
    enforce: bool,
    merge: bool,
    merge_only: bool,
    startup_only: bool,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        bin: None,
        build: false,
        fixture: workspace_root().join("target/perf/fixture"),
        files: 2000,
        commits: 5000,
        idle: 10,
        held: 300,
        hz: 100,
        samples: 40,
        enforce: false,
        merge: true,
        merge_only: false,
        startup_only: false,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| it.next().cloned().ok_or(format!("{name} needs a value"));
        let num = |s: String, name: &str| s.parse::<u64>().map_err(|_| format!("bad {name}"));
        match a.as_str() {
            "--build" => o.build = true,
            "--bin" => o.bin = Some(PathBuf::from(val("--bin")?)),
            "--fixture" => o.fixture = PathBuf::from(val("--fixture")?),
            "--files" => o.files = num(val("--files")?, "--files")? as usize,
            "--commits" => o.commits = num(val("--commits")?, "--commits")? as usize,
            "--idle" => o.idle = num(val("--idle")?, "--idle")?,
            "--held" => o.held = num(val("--held")?, "--held")? as usize,
            "--hz" => o.hz = num(val("--hz")?, "--hz")?.max(1),
            "--samples" => o.samples = num(val("--samples")?, "--samples")? as usize,
            "--enforce" => o.enforce = true,
            "--merge-only" => o.merge_only = true,
            "--no-merge" => o.merge = false,
            "--startup-only" => o.startup_only = true,
            other => return Err(format!("unknown option '{other}'\n\n{USAGE}")),
        }
    }
    Ok(o)
}

// ---- CPU time ------------------------------------------------------------

/// CPU time a process has used, in total and per thread name.
#[derive(Clone, Default)]
struct Cpu {
    total: Duration,
    threads: BTreeMap<String, Duration>,
}

#[cfg(target_os = "linux")]
fn cpu_snapshot(pid: u32) -> Option<Cpu> {
    // USER_HZ is 100 on every Linux the /proc interface is stable on.
    const TICK: Duration = Duration::from_millis(10);
    fn ticks(stat: &str) -> Option<(String, u64)> {
        // `pid (comm) state ...`: comm may hold spaces and parentheses, so
        // split on the last `)`. utime and stime are fields 14 and 15.
        let open = stat.find('(')?;
        let close = stat.rfind(')')?;
        let rest: Vec<&str> = stat[close + 1..].split_whitespace().collect();
        let utime: u64 = rest.get(11)?.parse().ok()?;
        let stime: u64 = rest.get(12)?.parse().ok()?;
        Some((stat[open + 1..close].to_string(), utime + stime))
    }
    let (_, total) = ticks(&std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?)?;
    let mut threads = BTreeMap::new();
    for entry in std::fs::read_dir(format!("/proc/{pid}/task"))
        .ok()?
        .flatten()
    {
        if let Some((name, t)) = std::fs::read_to_string(entry.path().join("stat"))
            .ok()
            .and_then(|s| ticks(&s))
        {
            *threads.entry(name).or_default() += TICK * t as u32;
        }
    }
    Some(Cpu {
        total: TICK * total as u32,
        threads,
    })
}

#[cfg(windows)]
fn cpu_snapshot(pid: u32) -> Option<Cpu> {
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut core::ffi::c_void;
        fn CloseHandle(handle: *mut core::ffi::c_void) -> i32;
        fn GetProcessTimes(
            process: *mut core::ffi::c_void,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
    }
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    let to_100ns = |t: FileTime| (u64::from(t.high) << 32) | u64::from(t.low);
    // SAFETY: plain Win32 calls on a handle opened and closed here.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let (mut c, mut e, mut k, mut u) = Default::default();
        let ok = GetProcessTimes(handle, &mut c, &mut e, &mut k, &mut u);
        CloseHandle(handle);
        if ok == 0 {
            return None;
        }
        Some(Cpu {
            total: Duration::from_nanos((to_100ns(k) + to_100ns(u)) * 100),
            threads: BTreeMap::new(),
        })
    }
}

#[cfg(not(any(target_os = "linux", windows)))]
fn cpu_snapshot(_pid: u32) -> Option<Cpu> {
    None
}

/// CPU percentage of one core used between two snapshots.
fn percent(delta: Duration, wall: Duration) -> f64 {
    100.0 * delta.as_secs_f64() / wall.as_secs_f64().max(1e-9)
}

const UI_THREAD: &str = "gitpoulpi-ui";

pub(crate) struct Usage {
    pub(crate) wall: Duration,
    pub(crate) process_pct: Option<f64>,
    pub(crate) ui_pct: Option<f64>,
    pub(crate) bytes: u64,
    pub(crate) writes: u64,
}

/// A measured stretch: CPU, bytes and terminal writes.
pub(crate) struct Window {
    pid: u32,
    started: Instant,
    cpu: Option<Cpu>,
    bytes: u64,
    writes: u64,
}

impl Window {
    pub(crate) fn open(term: &Terminal, pid: u32) -> Self {
        Window {
            pid,
            started: Instant::now(),
            cpu: cpu_snapshot(pid),
            bytes: term.bytes_received(),
            writes: term.chunks_received(),
        }
    }

    pub(crate) fn close(self, term: &Terminal) -> Usage {
        let wall = self.started.elapsed();
        let now = cpu_snapshot(self.pid);
        let (mut process_pct, mut ui_pct) = (None, None);
        if let (Some(a), Some(b)) = (&self.cpu, &now) {
            process_pct = Some(percent(b.total.saturating_sub(a.total), wall));
            if let (Some(x), Some(y)) = (a.threads.get(UI_THREAD), b.threads.get(UI_THREAD)) {
                ui_pct = Some(percent(y.saturating_sub(*x), wall));
            }
        }
        Usage {
            wall,
            process_pct,
            ui_pct,
            bytes: term.bytes_received() - self.bytes,
            writes: term.chunks_received() - self.writes,
        }
    }
}

// ---- measurements ---------------------------------------------------------

pub(crate) fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted[((sorted.len() - 1) as f64 * q).round() as usize]
}

struct Latency {
    first_byte: Vec<f64>,
    screen: Vec<f64>,
    misses: usize,
}

fn measure_latency(term: &mut Terminal, samples: usize) -> Result<Latency, String> {
    let mut out = Latency {
        first_byte: Vec::new(),
        screen: Vec::new(),
        misses: 0,
    };
    for k in 0..samples {
        std::thread::sleep(Duration::from_millis(150));
        let bytes = term.bytes_received();
        let before = term.text();
        let t0 = Instant::now();
        term.send_keys(if k % 2 == 0 { "Down" } else { "Up" })
            .map_err(|e| e.to_string())?;
        let (mut first, mut changed) = (None, None);
        while t0.elapsed() < Duration::from_secs(2) && (first.is_none() || changed.is_none()) {
            if first.is_none() && term.bytes_received() > bytes {
                first = Some(t0.elapsed().as_secs_f64() * 1e3);
            }
            if changed.is_none() && term.text() != before {
                changed = Some(t0.elapsed().as_secs_f64() * 1e3);
            }
            std::thread::sleep(Duration::from_micros(300));
        }
        match (first, changed) {
            (Some(f), Some(c)) => {
                out.first_byte.push(f);
                out.screen.push(c);
            }
            (Some(f), None) => {
                out.first_byte.push(f);
                out.misses += 1;
            }
            _ => out.misses += 1,
        }
    }
    out.first_byte.sort_by(|a, b| a.total_cmp(b));
    out.screen.sort_by(|a, b| a.total_cmp(b));
    Ok(out)
}

// ---- startup --------------------------------------------------------------

/// Spawn to "first refresh applied" (the repository is on screen), seconds.
const STARTUP_BUDGET_S: f64 = 1.0;

pub(crate) struct Startup {
    /// A start with nothing left over from a previous run.
    pub(crate) clean: f64,
    /// A start with a leftover `.port` file naming a dead port and no
    /// recorded process (what an older build left behind on every exit): the
    /// start must not wait on a connect to it. On Windows a refused loopback
    /// connect takes about 2 s.
    pub(crate) leftover: f64,
}

/// The unix ms of the first `tui-open first refresh applied` line in `log`.
pub(crate) fn first_refresh_ms(log: &str) -> Option<u128> {
    log.lines().find_map(|line| {
        let mut words = line.splitn(3, ' ');
        let at = words.next()?.parse().ok()?;
        (words.next()? == "tui-open" && words.next()?.starts_with("first refresh applied"))
            .then_some(at)
    })
}

/// A loopback port nothing listens on.
fn dead_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
    listener
        .local_addr()
        .map(|a| a.port())
        .map_err(|e| e.to_string())
}

/// One start in a fresh config dir and temp dir (the timing log lives in the
/// temp dir), through to the first applied refresh.
fn startup_once(
    bin: &std::path::Path,
    fixture: &std::path::Path,
    out_dir: &std::path::Path,
    leftover: bool,
) -> Result<f64, String> {
    let tag = format!(
        "{}-{}",
        std::process::id(),
        if leftover { "left" } else { "clean" }
    );
    let config = out_dir.join(format!("startup-cfg-{tag}"));
    let tmp = out_dir.join(format!("startup-tmp-{tag}"));
    let session = format!("perf-startup-{tag}");
    for dir in [&config, &tmp] {
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    if leftover {
        let sessions = config.join("sessions");
        std::fs::create_dir_all(&sessions).map_err(|e| e.to_string())?;
        std::fs::write(
            sessions.join(format!("{session}.port")),
            dead_port()?.to_string(),
        )
        .map_err(|e| e.to_string())?;
    }
    let started = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let term = Terminal::builder(bin)
        .args(["-s", &session])
        .cwd(fixture)
        .env("GITPOULPI_CONFIG_DIR", &config)
        .env("GITPOULPI_TERMINAL_IMAGE", "off")
        .env("GITPOULPI_LANG", "en")
        .env("GITPOULPI_NO_EXIT_PAUSE", "1")
        .env("GITPOULPI_TIMING", "1")
        .env("TMPDIR", &tmp)
        .env("TMP", &tmp)
        .env("TEMP", &tmp)
        .size(200, 60)
        .spawn()
        .map_err(|e| e.to_string())?;
    let log = tmp.join("gitpoulpi-timing.log");
    let deadline = Instant::now() + Duration::from_secs(120);
    let applied = loop {
        if let Some(at) = std::fs::read_to_string(&log)
            .ok()
            .as_deref()
            .and_then(first_refresh_ms)
        {
            break at;
        }
        if Instant::now() > deadline {
            return Err("the first refresh was never applied".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    term.stop().ok();
    let _ = std::fs::remove_dir_all(&config);
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(applied.saturating_sub(started) as f64 / 1000.0)
}

/// Measures the startup rows, prints them and adds their budget checks.
fn record_startup(
    bin: &std::path::Path,
    fixture: &std::path::Path,
    out_dir: &std::path::Path,
    checks: &mut Vec<Check>,
) -> Result<Startup, String> {
    let startup = measure_startup(bin, fixture, out_dir)?;
    for (what, secs) in [
        ("start -> repository shown (clean)", startup.clean),
        ("start -> repository shown (leftover)", startup.leftover),
    ] {
        println!("startup: {what}: {secs:.2} s");
        checks.push(Check {
            what: what.into(),
            value: secs,
            max: STARTUP_BUDGET_S,
            unit: "s",
        });
    }
    Ok(startup)
}

/// The startup rows: the faster of two starts each, so one cold file cache
/// is not blamed on the code.
pub(crate) fn measure_startup(
    bin: &std::path::Path,
    fixture: &std::path::Path,
    out_dir: &std::path::Path,
) -> Result<Startup, String> {
    let best = |leftover: bool| -> Result<f64, String> {
        let a = startup_once(bin, fixture, out_dir, leftover)?;
        let b = startup_once(bin, fixture, out_dir, leftover)?;
        Ok(a.min(b))
    };
    Ok(Startup {
        clean: best(false)?,
        leftover: best(true)?,
    })
}

// ---- budgets --------------------------------------------------------------

pub(crate) struct Check {
    pub(crate) what: String,
    pub(crate) value: f64,
    pub(crate) max: f64,
    pub(crate) unit: &'static str,
}

impl Check {
    pub(crate) fn over(&self) -> bool {
        self.value > self.max
    }
}

pub fn run(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(0);
    }
    let o = parse(args)?;
    let root = workspace_root();
    if o.build {
        let status = Command::new("cargo")
            .args(["build", "--release", "-p", "gitpoulpi-tui"])
            .current_dir(&root)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("cargo build --release failed".into());
        }
    }
    let bin = o.bin.clone().unwrap_or_else(|| {
        root.join("target/release")
            .join(format!("gitpoulpi-tui{}", std::env::consts::EXE_SUFFIX))
    });
    if !bin.exists() {
        return Err(format!(
            "{} not found: run `cargo build --release -p gitpoulpi-tui` or pass --build",
            bin.display()
        ));
    }
    if o.merge_only {
        let out_dir = root.join("target/perf");
        std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        let mut checks = Vec::new();
        let mut json = serde_json::Map::new();
        crate::perf_merge::run(&bin, &out_dir, &mut checks, &mut json)?;
        return Ok(print_checks(&checks, o.enforce));
    }
    if !fixture_ready(&o.fixture, o.files, o.commits) {
        eprintln!(
            "building {}x{} fixture in {} ...",
            o.files,
            o.commits,
            o.fixture.display()
        );
        build_fixture(&o.fixture, o.files, o.commits)?;
    }

    if o.startup_only {
        let out_dir = root.join("target/perf");
        std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        let mut checks = Vec::new();
        record_startup(&bin, &o.fixture, &out_dir, &mut checks)?;
        return Ok(print_checks(&checks, o.enforce));
    }
    let out_dir = root.join("target/perf");
    let config = out_dir.join(format!("tui-cfg-{}", std::process::id()));
    std::fs::create_dir_all(&config).map_err(|e| e.to_string())?;
    let result = measure(&o, &bin, &config, &out_dir);
    let _ = std::fs::remove_dir_all(&config);
    result
}

fn measure(
    o: &Opts,
    bin: &std::path::Path,
    config: &std::path::Path,
    out_dir: &std::path::Path,
) -> Result<i32, String> {
    let mut term = Terminal::builder(bin)
        .args(["-s", &format!("perf-tui-{}", std::process::id())])
        .cwd(&o.fixture)
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_TERMINAL_IMAGE", "off")
        .env("GITPOULPI_LANG", "en")
        .env("GITPOULPI_NO_EXIT_PAUSE", "1")
        .size(200, 60)
        .spawn()
        .map_err(|e| e.to_string())?;
    let pid = term.process_id().ok_or("no pid for the TUI")?;
    term.wait_for_text("Message", Duration::from_secs(120))
        .map_err(|e| format!("the TUI never showed the Log: {e}"))?;
    if cpu_snapshot(pid).is_none() {
        eprintln!("note: no CPU accounting on this OS; CPU figures are n/a");
    }
    println!("fixture {}x{}, {}\n", o.files, o.commits, bin.display());

    // Idle: let the first draws and the daemon's first reads finish, then
    // look at a quiet window. Settled = no byte for longer than a status
    // message's life (3 s minimum, then the next refresh clears it), so the
    // one repaint that ends it is not counted as idle output.
    let settle_deadline = Instant::now() + Duration::from_secs(60);
    let mut last = term.bytes_received();
    let mut quiet_since = Instant::now();
    while quiet_since.elapsed() < Duration::from_secs(6) {
        if Instant::now() > settle_deadline {
            return Err("the TUI never went quiet: it keeps writing while idle".into());
        }
        std::thread::sleep(Duration::from_millis(50));
        let now = term.bytes_received();
        if now != last {
            last = now;
            quiet_since = Instant::now();
        }
    }
    let window = Window::open(&term, pid);
    std::thread::sleep(Duration::from_secs(o.idle));
    let idle = window.close(&term);

    // Held key: Down at `hz`, then wait for the screen to catch up.
    let window = Window::open(&term, pid);
    let period = Duration::from_micros(1_000_000 / o.hz);
    let start = Instant::now();
    for i in 0..o.held {
        term.send_keys("Down").map_err(|e| e.to_string())?;
        let due = start + period * (i as u32 + 1);
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
    }
    let sent = start.elapsed();
    // The idle redraw never goes quiet on the wire, so "caught up" is a
    // screen that has stopped changing.
    term.wait_for_stable(Duration::from_millis(250), Duration::from_secs(60))
        .map_err(|e| e.to_string())?;
    let mut held = window.close(&term);
    held.wall = held.wall.saturating_sub(Duration::from_millis(250));

    let latency = measure_latency(&mut term, o.samples)?;
    term.stop().ok();

    let idle_secs = idle.wall.as_secs_f64();
    let na = |v: Option<f64>| v.map_or("n/a".to_string(), |v| format!("{v:.1} %"));
    println!("idle ({idle_secs:.0} s, Log open)");
    println!(
        "  CPU: process {}, gitpoulpi-ui thread {}",
        na(idle.process_pct),
        na(idle.ui_pct)
    );
    println!(
        "  terminal: {:.0} bytes/s, {:.1} writes/s ({} bytes, {} writes)",
        idle.bytes as f64 / idle_secs,
        idle.writes as f64 / idle_secs,
        idle.bytes,
        idle.writes
    );
    println!(
        "held Down x{} at {} Hz (sent in {:.2} s)",
        o.held,
        o.hz,
        sent.as_secs_f64()
    );
    println!(
        "  wall {:.2} s, CPU: process {}, gitpoulpi-ui thread {}, {} bytes ({:.0} KB/s)",
        held.wall.as_secs_f64(),
        na(held.process_pct),
        na(held.ui_pct),
        held.bytes,
        held.bytes as f64 / 1024.0 / held.wall.as_secs_f64()
    );
    println!(
        "key latency ({} samples, {} misses)",
        o.samples, latency.misses
    );
    for (name, v) in [
        ("key -> first byte", &latency.first_byte),
        ("key -> changed screen", &latency.screen),
    ] {
        println!(
            "  {name:<22} p50 {:.1} ms  p90 {:.1} ms  p95 {:.1} ms",
            quantile(v, 0.5),
            quantile(v, 0.9),
            quantile(v, 0.95)
        );
    }

    // Budgets: docs/developer/perf-budgets.md.
    let mut checks = vec![
        // An idle TUI writes nothing (an empty frame diff is dropped by
        // `QuietBackend`).
        Check {
            what: "idle terminal bytes/s".into(),
            value: idle.bytes as f64 / idle_secs,
            max: 0.0,
            unit: "B/s",
        },
        Check {
            what: "idle terminal writes/s".into(),
            value: idle.writes as f64 / idle_secs,
            max: 0.0,
            unit: "w/s",
        },
        Check {
            what: "held key wall time".into(),
            value: held.wall.as_secs_f64(),
            max: 3.3,
            unit: "s",
        },
        Check {
            what: "key -> first byte p90".into(),
            value: quantile(&latency.first_byte, 0.9),
            max: 6.0,
            unit: "ms",
        },
    ];
    if let Some(v) = idle.ui_pct.or(idle.process_pct) {
        checks.push(Check {
            what: "idle CPU (ui thread, else process)".into(),
            value: v,
            max: 2.0,
            unit: "%",
        });
    }
    if let Some(v) = held.ui_pct.or(held.process_pct) {
        checks.push(Check {
            what: "held key CPU (ui thread, else process)".into(),
            value: v,
            max: 45.0,
            unit: "%",
        });
    }
    let startup = record_startup(bin, &o.fixture, out_dir, &mut checks)?;
    let mut merge_json = serde_json::Map::new();
    if o.merge {
        crate::perf_merge::run(bin, out_dir, &mut checks, &mut merge_json)?;
    }
    print_checks(&checks, o.enforce);

    let report = json!({
        "date": today(),
        "idle": {
            "secs": idle_secs, "process_cpu_pct": idle.process_pct,
            "ui_thread_cpu_pct": idle.ui_pct, "bytes": idle.bytes, "writes": idle.writes,
        },
        "held": {
            "keys": o.held, "hz": o.hz, "wall_s": held.wall.as_secs_f64(),
            "cpu_pct": held.process_pct, "ui_thread_cpu_pct": held.ui_pct, "bytes": held.bytes,
        },
        "latency_ms": {
            "first_byte": {"p50": quantile(&latency.first_byte, 0.5), "p90": quantile(&latency.first_byte, 0.9), "p95": quantile(&latency.first_byte, 0.95)},
            "screen": {"p50": quantile(&latency.screen, 0.5), "p90": quantile(&latency.screen, 0.9), "p95": quantile(&latency.screen, 0.95)},
            "misses": latency.misses,
        },
        "startup_s": {"clean": startup.clean, "leftover_port_file": startup.leftover},
        "merge_editor": merge_json,
        "over_budget": checks.iter().filter(|c| c.over()).map(|c| c.what.clone()).collect::<Vec<_>>(),
    });
    let file = out_dir.join(format!("tui-{}.json", today()));
    std::fs::write(&file, serde_json::to_string_pretty(&report).unwrap())
        .map_err(|e| e.to_string())?;
    println!("\nwrote {}", file.display());

    Ok(i32::from(o.enforce && checks.iter().any(Check::over)))
}

fn print_checks(checks: &[Check], enforce: bool) -> i32 {
    println!(
        "\nbudgets ({})",
        if enforce {
            "enforced"
        } else {
            "report-only; --enforce fails on OVER"
        }
    );
    for c in checks {
        println!(
            "  {:<36} {:>8.1} {:<6} budget <= {:<5} ratio {:>5.2}  {}",
            c.what,
            c.value,
            c.unit,
            c.max,
            if c.max > 0.0 {
                c.value / c.max
            } else {
                c.value
            },
            if c.over() { "OVER" } else { "ok" }
        );
    }

    i32::from(enforce && checks.iter().any(Check::over))
}

#[cfg(test)]
mod startup_tests {
    use super::first_refresh_ms;

    #[test]
    fn the_first_applied_refresh_is_found_in_the_timing_log() {
        let log = "1000 tui-open first frame drawn (+12.0ms)\n\
                   1500 tui-stall tick 50.0ms\n\
                   2250 tui-open first refresh applied (+812.3ms)\n\
                   9000 tui-open first refresh applied (+1.0ms)\n";
        assert_eq!(first_refresh_ms(log), Some(2250));
        assert_eq!(first_refresh_ms("1000 tui-open first frame drawn\n"), None);
    }
}
