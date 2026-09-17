//! A line-oriented script front end for `pty::Terminal`.
//!
//! The library is what the automated tests use. This is for poking at the
//! TUI by hand, the way an agent or a contributor does when they cannot
//! yet write an assertion about the screen. It needs neither `bash` nor
//! `tmux`, neither of which exists on Windows.
//!
//! One process runs the whole script, because a pty dies with the process
//! that owns it (there is no "send a key to the instance I started last
//! time"); a script is reproducible and a sequence of shell invocations is
//! not.

use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use crate::pty::Terminal;

pub const USAGE: &str = "\
Usage: cargo run -p xtask -- tui <script.txt | ->

Drives a terminal program through a pty and prints what it drew. Reads
the script from a file, or from stdin when given `-`.

Setup (before `spawn`):
  size <cols> <rows>       terminal size (default 200x50)
  cwd <path>               working directory for the child
  env <KEY> <VALUE>        environment variable for the child
  timeout <ms>             default wait timeout (default 15000)

  spawn <program> [args...]  start the child

Input:
  keys <spec>              key spec, e.g. `Down Down Enter`, `C-g`, `M-x`
  text <literal>           the rest of the line, sent verbatim
  click <col> <row>        left click        (coordinates are 0-based)
  rclick <col> <row>       right click
  dblclick <col> <row>     double click
  move <col> <row>         pointer motion, for hover affordances
  drag <c1> <r1> <c2> <r2> press, walk, release
  down <col> <row>         press and hold, for asserting mid-drag
  dragto <col> <row>       motion with the button held
  up <col> <row>           release
  scroll up|down <col> <row>

Waiting (never sleep - these measure the program, not the clock):
  ready                    wait until the child enables mouse capture
  wait <needle>            wait until <needle> is on screen
  wait-gone <needle>       wait until it is not
  stable <ms>              wait until the screen stops changing for <ms>

Output:
  screen                   print the screen as text
  styled                   print every non-default-styled run
  find <needle>            print `col row` of the first match
  cell <col> <row>         print that cell's contents and style
  stop                     kill the child (implicit at end of script)

Blank lines and lines starting with `#` are ignored.";

pub fn run(args: &[String]) -> Result<(), String> {
    let source = match args.first().map(String::as_str) {
        None | Some("-h") | Some("--help") => return Err(USAGE.to_string()),
        Some("-") => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|e| format!("reading stdin: {e}"))?;
            text
        }
        Some(path) => std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?,
    };
    execute(&source)
}

struct Setup {
    cols: u16,
    rows: u16,
    cwd: Option<PathBuf>,
    env: Vec<(String, String)>,
    timeout: Duration,
}

fn execute(source: &str) -> Result<(), String> {
    let mut setup = Setup {
        cols: 200,
        rows: 50,
        cwd: None,
        env: Vec::new(),
        timeout: Duration::from_millis(15_000),
    };
    let mut term: Option<Terminal> = None;

    for (number, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        step(line, &mut setup, &mut term)
            .map_err(|e| format!("line {}: {}: {e}", number + 1, line))?;
    }
    if let Some(term) = term {
        term.stop().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn step(line: &str, setup: &mut Setup, term: &mut Option<Terminal>) -> Result<(), String> {
    let (command, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let rest = rest.trim();
    let words: Vec<&str> = rest.split_whitespace().collect();

    // Everything before `spawn` configures; everything after needs a
    // running child, so the lookup is done once here.
    match command {
        "size" => {
            setup.cols = number(words.first())?;
            setup.rows = number(words.get(1))?;
            return Ok(());
        }
        "cwd" => {
            setup.cwd = Some(PathBuf::from(rest));
            return Ok(());
        }
        "env" => {
            let (key, value) = rest
                .split_once(char::is_whitespace)
                .ok_or("env needs a key and a value")?;
            setup.env.push((key.to_string(), value.trim().to_string()));
            return Ok(());
        }
        "timeout" => {
            setup.timeout = Duration::from_millis(u64::from(number::<u32>(words.first())?));
            return Ok(());
        }
        "spawn" => {
            let (program, args) = words.split_first().ok_or("spawn needs a program")?;
            let mut builder = Terminal::builder(*program)
                .args(args.iter().copied())
                .size(setup.cols, setup.rows);
            if let Some(dir) = &setup.cwd {
                builder = builder.cwd(dir);
            }
            for (key, value) in &setup.env {
                builder = builder.env(key, value);
            }
            *term = Some(builder.spawn().map_err(|e| e.to_string())?);
            return Ok(());
        }
        _ => {}
    }

    let term = term.as_mut().ok_or("no child yet - `spawn` first")?;
    let timeout = setup.timeout;
    match command {
        "keys" => term.send_keys(rest).map_err(|e| e.to_string())?,
        "text" => term.send_text(rest).map_err(|e| e.to_string())?,
        "click" => term
            .click(number(words.first())?, number(words.get(1))?)
            .map_err(|e| e.to_string())?,
        "rclick" => term
            .right_click(number(words.first())?, number(words.get(1))?)
            .map_err(|e| e.to_string())?,
        "dblclick" => term
            .double_click(number(words.first())?, number(words.get(1))?)
            .map_err(|e| e.to_string())?,
        "move" => term
            .move_to(number(words.first())?, number(words.get(1))?)
            .map_err(|e| e.to_string())?,
        "drag" => term
            .drag(
                (number(words.first())?, number(words.get(1))?),
                (number(words.get(2))?, number(words.get(3))?),
            )
            .map_err(|e| e.to_string())?,
        "down" => term
            .press(number(words.first())?, number(words.get(1))?)
            .map_err(|e| e.to_string())?,
        "dragto" => term
            .drag_to(number(words.first())?, number(words.get(1))?)
            .map_err(|e| e.to_string())?,
        "up" => term
            .release(number(words.first())?, number(words.get(1))?)
            .map_err(|e| e.to_string())?,
        "scroll" => {
            let down = match words.first() {
                Some(&"down") => true,
                Some(&"up") => false,
                _ => return Err("scroll needs `up` or `down`".to_string()),
            };
            term.scroll(down, number(words.get(1))?, number(words.get(2))?)
                .map_err(|e| e.to_string())?;
        }
        "ready" => term
            .wait_for_mouse_capture(timeout)
            .map_err(|e| e.to_string())?,
        "wait" => {
            let (col, row) = term
                .wait_for_text(rest, timeout)
                .map_err(|e| e.to_string())?;
            println!("{col} {row}");
        }
        "wait-gone" => term
            .wait_for_text_to_go(rest, timeout)
            .map_err(|e| e.to_string())?,
        "stable" => {
            let stable_for = Duration::from_millis(u64::from(number::<u32>(words.first())?));
            term.wait_for_stable(stable_for, timeout)
                .map_err(|e| e.to_string())?;
        }
        "screen" => println!("{}", term.text()),
        "styled" => println!("{}", styled_dump(term)),
        "find" => match term.screen().find(rest) {
            Some((col, row)) => println!("{col} {row}"),
            None => println!("not found"),
        },
        "cell" => {
            let (col, row) = (number(words.first())?, number(words.get(1))?);
            match term.screen().cell(col, row) {
                Some(cell) => println!(
                    "{col} {row} {:?} fg={:?} bg={:?} bold={} italic={} underline={} inverse={}",
                    cell.display(),
                    cell.fg,
                    cell.bg,
                    cell.bold,
                    cell.italic,
                    cell.underline,
                    cell.inverse
                ),
                None => println!("{col} {row} out of range"),
            }
        }
        "stop" => return Err("`stop` is implicit at end of script".to_string()),
        other => return Err(format!("unknown command {other:?}")),
    }
    Ok(())
}

/// One line per run of cells that share a non-default style, so the
/// output stays readable at 200 columns while still showing every color
/// on screen. This is the replacement for `tmux capture-pane -e`, whose
/// inline SGR codes were unreadable at exactly the moment you needed
/// them.
fn styled_dump(term: &Terminal) -> String {
    let screen = term.screen();
    let mut out = String::new();
    for row in 0..screen.rows() {
        let mut col = 0;
        while col < screen.cols() {
            let Some(cell) = screen.cell(col, row) else {
                break;
            };
            let style = (cell.fg, cell.bg, cell.bold, cell.italic, cell.underline);
            let default = style
                == (
                    crate::pty::Color::Default,
                    crate::pty::Color::Default,
                    false,
                    false,
                    false,
                );
            if default {
                col += 1;
                continue;
            }
            let start = col;
            let mut text = String::new();
            while col < screen.cols() {
                let Some(next) = screen.cell(col, row) else {
                    break;
                };
                if (next.fg, next.bg, next.bold, next.italic, next.underline) != style {
                    break;
                }
                text.push_str(next.display());
                col += 1;
            }
            out.push_str(&format!(
                "{row:>3},{start:>3}-{:<3} fg={:?} bg={:?}{}{}{} {text:?}\n",
                col - 1,
                style.0,
                style.1,
                if style.2 { " bold" } else { "" },
                if style.3 { " italic" } else { "" },
                if style.4 { " underline" } else { "" },
            ));
        }
    }
    out
}

fn number<T: std::str::FromStr>(word: Option<&&str>) -> Result<T, String> {
    word.ok_or_else(|| "missing a numeric argument".to_string())?
        .parse()
        .map_err(|_| format!("{:?} is not a number", word.unwrap()))
}
