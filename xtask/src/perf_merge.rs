//! `perf --tui`'s merge conflict editor scenarios (slice SP): the real
//! release TUI opened on a big conflicted file (10k lines / 50 conflicts,
//! 100k lines / 3 conflicts), driven through a pty:
//!
//! - **open**: Enter on the file in the Status pane to the first byte, to the
//!   editor on screen, and to a quiet terminal (highlighting caught up);
//! - **idle**: bytes and CPU over a quiet window once settled;
//! - **held keys**: Down and PageDown in A/B, Down, PageDown and typing in
//!   the Output, `L` (scroll right): UI-thread CPU per key, wall time to
//!   catch up, bytes;
//! - **one-shot operations**: the Full file toggle, pick and unpick: key to
//!   first byte and to the last byte of the burst.
//!
//! Budgets: docs/developer/perf-budgets.md.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::merge_fixture;
use crate::perf::workspace_root;
use crate::perf_tui::{quantile, Check, Window};
use crate::pty::Terminal;

/// `(label, lines, conflicts)`.
const SCENARIOS: [(&str, usize, usize); 2] = [("10k/50", 10_000, 50), ("100k/3", 100_000, 3)];

fn fixture(lines: usize, conflicts: usize) -> Result<PathBuf, String> {
    let dir = workspace_root()
        .join("target/perf")
        .join(format!("merge-{lines}-{conflicts}"));
    if dir.join(".git/MERGE_HEAD").exists() && dir.join("src/big.rs").exists() {
        return Ok(dir);
    }
    let _ = std::fs::remove_dir_all(&dir);
    eprintln!(
        "building the {lines}-line merge fixture in {} ...",
        dir.display()
    );
    merge_fixture::build_big(&dir, lines, conflicts)?;
    Ok(dir)
}

fn lines_hint(label: &str) -> usize {
    SCENARIOS
        .iter()
        .find(|(l, ..)| *l == label)
        .map_or(0, |(_, lines, _)| *lines)
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

/// Sends `spec` (a key spec, or text when `text`), then waits for the burst
/// of output it causes to end (80 ms of silence). Returns `(first byte,
/// last byte)` in ms; `None` when nothing was drawn.
fn one_shot(term: &mut Terminal, send: &dyn Fn(&mut Terminal)) -> Option<(f64, f64)> {
    let start_bytes = term.bytes_received();
    let t0 = Instant::now();
    send(term);
    let (mut first, mut last_bytes, mut last_at) = (None, start_bytes, t0);
    loop {
        std::thread::sleep(Duration::from_micros(300));
        let now = term.bytes_received();
        if now != last_bytes {
            last_bytes = now;
            last_at = Instant::now();
            first.get_or_insert(ms(t0.elapsed()));
        }
        let quiet = last_at.elapsed() > Duration::from_millis(80);
        if (first.is_some() && quiet) || t0.elapsed() > Duration::from_secs(20) {
            break;
        }
    }
    first.map(|f| (f, ms(last_at.duration_since(t0))))
}

struct Held {
    keys: usize,
    wall: Duration,
    ui_ms_per_key: Option<f64>,
    process_ms_per_key: Option<f64>,
    kb: f64,
}

/// `keys` presses of `send(i)` at `hz`, then until the screen is quiet.
fn hold(
    term: &mut Terminal,
    pid: u32,
    keys: usize,
    hz: u64,
    send: &dyn Fn(&mut Terminal, usize),
) -> Held {
    let window = Window::open(term, pid);
    let period = Duration::from_micros(1_000_000 / hz);
    let start = Instant::now();
    for i in 0..keys {
        send(term, i);
        let due = start + period * (i as u32 + 1);
        if let Some(wait) = due.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
        }
    }
    // Caught up = no byte for 150 ms.
    let (mut last, mut since) = (term.bytes_received(), Instant::now());
    while since.elapsed() < Duration::from_millis(150) && start.elapsed() < Duration::from_secs(120)
    {
        std::thread::sleep(Duration::from_millis(5));
        let now = term.bytes_received();
        if now != last {
            last = now;
            since = Instant::now();
        }
    }
    let wall = start.elapsed().saturating_sub(Duration::from_millis(150));
    let u = window.close(term);
    let per_key =
        |pct: Option<f64>| pct.map(|p| p / 100.0 * u.wall.as_secs_f64() * 1e3 / keys as f64);
    Held {
        keys,
        wall,
        ui_ms_per_key: per_key(u.ui_pct),
        process_ms_per_key: per_key(u.process_pct),
        kb: u.bytes as f64 / 1024.0,
    }
}

fn settle(term: &Terminal, quiet: Duration, timeout: Duration) -> Result<Duration, String> {
    let t0 = Instant::now();
    let (mut last, mut since) = (term.bytes_received(), Instant::now());
    while since.elapsed() < quiet {
        if t0.elapsed() > timeout {
            return Err("the TUI never went quiet".into());
        }
        std::thread::sleep(Duration::from_millis(20));
        let now = term.bytes_received();
        if now != last {
            last = now;
            since = Instant::now();
        }
    }
    Ok(t0.elapsed().saturating_sub(quiet))
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    quantile(v, 0.5)
}

pub(crate) fn run(
    bin: &Path,
    out_dir: &Path,
    checks: &mut Vec<Check>,
    json: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    println!("\nmerge conflict editor (200x60, release)");
    let dump = std::env::var_os("PERF_DUMP").is_some();
    for (label, lines, conflicts) in SCENARIOS {
        let repo = fixture(lines, conflicts)?;
        let config = out_dir.join(format!("tui-merge-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&config).map_err(|e| e.to_string())?;
        let result = scenario(bin, &repo, &config, label, conflicts, dump, checks, json);
        let _ = std::fs::remove_dir_all(&config);
        result?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn scenario(
    bin: &Path,
    repo: &Path,
    config: &Path,
    label: &str,
    _conflicts: usize,
    dump: bool,
    checks: &mut Vec<Check>,
    json: &mut serde_json::Map<String, serde_json::Value>,
) -> Result<(), String> {
    let mut term = Terminal::builder(bin)
        .args([
            "-s",
            &format!(
                "perf-merge-{}-{}",
                label.replace('/', "-"),
                std::process::id()
            ),
        ])
        .cwd(repo)
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_TERMINAL_IMAGE", "off")
        .env("GITPOULPI_LANG", "en")
        .env("GITPOULPI_NO_EXIT_PAUSE", "1")
        .size(200, 60)
        .spawn()
        .map_err(|e| e.to_string())?;
    let pid = term.process_id().ok_or("no pid for the TUI")?;
    let step = Duration::from_secs(120);
    let (c, r) = term
        .wait_for_text("big.rs", step)
        .map_err(|e| format!("the Status pane never showed big.rs: {e}"))?;
    term.click(c, r).map_err(|e| e.to_string())?;
    std::thread::sleep(Duration::from_millis(300));
    settle(&term, Duration::from_secs(1), Duration::from_secs(60))?;

    // Open.
    let header = "Conflict 1 / ";
    let bytes0 = term.bytes_received();
    let t0 = Instant::now();
    term.send_keys("Enter").map_err(|e| e.to_string())?;
    let (mut first, mut shown) = (None, None);
    while t0.elapsed() < Duration::from_secs(120) && shown.is_none() {
        if first.is_none() && term.bytes_received() > bytes0 {
            first = Some(ms(t0.elapsed()));
        }
        if term.text().contains(header) {
            shown = Some(ms(t0.elapsed()));
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    let shown =
        shown.ok_or_else(|| format!("{label}: the editor never opened\n{}", term.text()))?;
    let first = first.unwrap_or(shown);
    let settled = settle(&term, Duration::from_millis(600), Duration::from_secs(120))?;
    let open_total = shown + ms(settled);
    if dump {
        println!("{}", term.text());
    }

    // Idle once settled.
    settle(&term, Duration::from_secs(4), Duration::from_secs(120))?;
    let w = Window::open(&term, pid);
    std::thread::sleep(Duration::from_secs(4));
    let idle = w.close(&term);

    // A / B.
    let a_down = hold(&mut term, pid, 150, 100, &|t, _| {
        let _ = t.send_keys("Down");
    });
    let a_pg = hold(&mut term, pid, 120, 100, &|t, _| {
        let _ = t.send_keys("PgDn");
    });
    let _ = term.send_keys("Home");
    settle(&term, Duration::from_millis(300), Duration::from_secs(60))?;
    let hscroll = hold(&mut term, pid, 80, 100, &|t, _| {
        let _ = t.send_text("L");
    });
    let _ = term.send_text(&"H".repeat(80));
    settle(&term, Duration::from_millis(300), Duration::from_secs(60))?;

    // Full file toggle, pick, unpick (conflict 1 is current).
    let (mut full, mut pick) = (Vec::new(), Vec::new());
    for _ in 0..3 {
        if let Some((_, last)) = one_shot(&mut term, &|t| {
            let _ = t.send_keys("f");
        }) {
            full.push(last);
        }
    }
    let _ = term.send_keys("f"); // an odd count of toggles: back off
    settle(&term, Duration::from_millis(300), Duration::from_secs(60))?;
    for k in ["a", "x", "b", "x", "a", "x"] {
        if let Some((_, last)) = one_shot(&mut term, &|t| {
            let _ = t.send_keys(k);
        }) {
            pick.push(last);
        }
    }

    // Output.
    let _ = term.send_keys("F6");
    let _ = term.send_keys("F6");
    settle(&term, Duration::from_millis(300), Duration::from_secs(60))?;
    let o_down = hold(&mut term, pid, 150, 100, &|t, _| {
        let _ = t.send_keys("Down");
    });
    let o_pg = hold(&mut term, pid, 120, 100, &|t, _| {
        let _ = t.send_keys("PgDn");
    });
    let o_type = hold(&mut term, pid, 120, 50, &|t, i| {
        let _ = t.send_text(if i % 20 == 19 { "\r" } else { "x" });
    });
    let typed_one = one_shot(&mut term, &|t| {
        let _ = t.send_text("y");
    });
    if dump {
        println!("{}", term.text());
    }
    // Idle after all that.
    settle(&term, Duration::from_secs(4), Duration::from_secs(120))?;
    let w = Window::open(&term, pid);
    std::thread::sleep(Duration::from_secs(4));
    let idle_after = w.close(&term);
    term.stop().ok();

    // The first toggle is the one that computes the side diffs; the later
    // ones are cached, so the first is the number that matters.
    let full_ms = full.first().copied().unwrap_or(f64::NAN);
    let pick_ms = median(&mut pick);
    println!("  {label}");
    println!(
        "    open: first byte {first:.0} ms, editor on screen {shown:.0} ms, quiet (highlight done) {open_total:.0} ms"
    );
    println!(
        "    idle after settle: {} bytes / 4 s, ui CPU {:.1} %; after the run: {} bytes",
        idle.bytes,
        idle.ui_pct.or(idle.process_pct).unwrap_or(f64::NAN),
        idle_after.bytes
    );
    let line = |name: &str, h: &Held| {
        println!(
            "    {name:<22} x{:<4} wall {:>5.2} s  ui {:>5.2} ms/key (process {:>5.2})  {:>7.0} KB",
            h.keys,
            h.wall.as_secs_f64(),
            h.ui_ms_per_key.unwrap_or(f64::NAN),
            h.process_ms_per_key.unwrap_or(f64::NAN),
            h.kb
        );
    };
    line("A/B Down", &a_down);
    line("A/B PageDown", &a_pg);
    line("A/B scroll right", &hscroll);
    line("Output Down", &o_down);
    line("Output PageDown", &o_pg);
    line("Output typing (50 Hz)", &o_type);
    println!(
        "    one-shot to last byte: Full file {full_ms:.0} ms, pick/unpick {pick_ms:.0} ms, one typed char {:.0} ms",
        typed_one.map_or(f64::NAN, |t| t.1)
    );

    let mut add = |what: String, value: f64, max: f64, unit: &'static str| {
        checks.push(Check {
            what,
            value,
            max,
            unit,
        });
    };
    // The editor appears when the daemon has read the three sides and the
    // view is built (off the UI thread); until then the status line says
    // "Opening" and the spinner turns (first byte). The daemon read and
    // the IPC dominate on the big file.
    let open_max = if lines_hint(label) >= 100_000 {
        2000.0
    } else {
        600.0
    };
    add(
        format!("merge {label}: open to editor"),
        shown,
        open_max,
        "ms",
    );
    add(
        format!("merge {label}: open to first byte"),
        first,
        150.0,
        "ms",
    );
    add(
        format!("merge {label}: idle bytes"),
        (idle.bytes + idle_after.bytes) as f64,
        0.0,
        "B",
    );
    for (name, h) in [
        ("A/B Down", &a_down),
        ("A/B PageDown", &a_pg),
        ("A/B scroll right", &hscroll),
        ("Output Down", &o_down),
        ("Output PageDown", &o_pg),
        ("Output typing", &o_type),
    ] {
        // The Log's budget: 45 % of a core at 100 Hz is 4.5 ms a key.
        if let Some(v) = h.ui_ms_per_key.or(h.process_ms_per_key) {
            let per_key_budget = if name == "Output typing" { 9.0 } else { 4.5 };
            add(
                format!("merge {label}: {name} ui ms/key"),
                v,
                per_key_budget,
                "ms",
            );
        }
    }
    add(
        format!("merge {label}: Full file toggle (first)"),
        full_ms,
        250.0,
        "ms",
    );
    add(format!("merge {label}: pick/unpick"), pick_ms, 250.0, "ms");
    add(
        format!("merge {label}: one typed char"),
        typed_one.map_or(f64::NAN, |t| t.1),
        150.0,
        "ms",
    );
    json.insert(
        label.to_string(),
        serde_json::json!({
            "open_first_byte_ms": first, "open_editor_ms": shown, "open_quiet_ms": open_total,
            "idle_bytes": idle.bytes, "idle_after_bytes": idle_after.bytes,
            "full_file_ms": full_ms, "pick_ms": pick_ms,
            "a_down_ms_key": a_down.ui_ms_per_key, "a_pgdn_ms_key": a_pg.ui_ms_per_key,
            "hscroll_ms_key": hscroll.ui_ms_per_key,
            "out_down_ms_key": o_down.ui_ms_per_key, "out_pgdn_ms_key": o_pg.ui_ms_per_key,
            "out_type_ms_key": o_type.ui_ms_per_key,
            "typed_char_ms": typed_one.map(|t| t.1),
        }),
    );
    Ok(())
}
