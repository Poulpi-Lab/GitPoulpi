// A fake interactive agent for the terminal run mode tests. Compiled by
// `gitpoulpi_testkit::terminal_agent` (std only, so it behaves the same on
// every platform and needs no real agent, account or network).
//
// It paints what a real agent paints: a ready marker, a busy marker while it
// "works", the answer with a bullet and a two-space gutter between the
// sentinel markers the prompt asks for. The pty echoes what is typed, so the
// echoed prompt (with the marker names mid-sentence) is on the screen too.
//
// Knobs (environment):
//   FAKE_FLAVOR=gemini|claude|codex  which marker strings to paint (default gemini)
//   FAKE_ANSWER=text                 the answer (default: a commit subject)
//   FAKE_LINES=n                     n extra answer lines (scrollback)
//   FAKE_TRUST=1                     show a folder-trust dialog, never get ready
//   FAKE_HANG=1                      after the prompt, stay busy forever
//   FAKE_IGNORE_EXIT=1               ignore the exit command
//   FAKE_NO_BRACKETED=1              do not turn bracketed paste on
//   FAKE_TRANSCRIPT=dir              claude flavor: write the answer to
//                                    <dir>/projects/x/<session-id>.jsonl and
//                                    keep it OFF the screen
//   FAKE_ARGS_FILE=path              write argv and the working folder there
//   FAKE_PID_FILE=path               write the process id there

use std::io::{BufRead, Write};

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(path) = env("FAKE_ARGS_FILE") {
        let cwd = std::env::current_dir().unwrap_or_default();
        let _ = std::fs::write(
            path,
            format!("{}\n{}\n", args.join(" "), cwd.display()),
        );
    }
    if let Some(path) = env("FAKE_PID_FILE") {
        let _ = std::fs::write(path, std::process::id().to_string());
    }
    let flavor = env("FAKE_FLAVOR").unwrap_or_else(|| "gemini".into());
    let (ready, busy) = match flavor.as_str() {
        "claude" => ("? for shortcuts", "esc to interrupt"),
        "codex" => ("Ask Codex to do anything", "Working (1s - esc to interrupt)"),
        _ => ("Type your message or @path/to/file", "(esc to cancel, 1s)"),
    };
    let session_id = args
        .iter()
        .position(|a| a == "--session-id")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let mut out = std::io::stdout();
    if env("FAKE_TRUST").is_some() {
        let _ = write!(out, "Do you trust the files in this folder?\r\n 1. Yes\r\n 2. No\r\n");
        let _ = out.flush();
        std::thread::sleep(std::time::Duration::from_secs(600));
        return;
    }
    let bracketed = env("FAKE_NO_BRACKETED").is_none();
    if bracketed {
        let _ = write!(out, "\x1b[?2004h");
    }
    let _ = write!(out, "Fake agent v1\r\n{ready}\r\n");
    let _ = out.flush();

    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut prompt = String::new();
    let mut pasted = false;
    loop {
        let mut raw = String::new();
        match input.read_line(&mut raw) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        let line = raw.trim_end_matches(['\r', '\n']).to_string();
        let starts = line.contains("\x1b[200~");
        let ends = line.contains("\x1b[201~");
        let text = line.replace("\x1b[200~", "").replace("\x1b[201~", "");
        if !pasted && !starts && matches!(text.trim(), "/quit" | "/exit") {
            if env("FAKE_IGNORE_EXIT").is_some() {
                continue;
            }
            return;
        }
        if pasted || starts {
            pasted = !ends;
            prompt.push_str(&text);
            prompt.push('\n');
            // The Enter that follows the paste ends the same terminal line.
            if pasted {
                continue;
            }
        } else if let Some(stripped) = text.strip_suffix('\u{1b}') {
            prompt.push_str(stripped);
            prompt.push('\n');
            continue;
        } else {
            prompt.push_str(&text);
        }
        let nonce = prompt
            .split("<<GP-")
            .filter(|rest| !rest.starts_with("END-"))
            .find_map(|rest| rest.split_once(">>").map(|(n, _)| n.to_string()))
            .unwrap_or_default();
        prompt.clear();

        let _ = write!(out, "{busy}");
        let _ = out.flush();
        if env("FAKE_HANG").is_some() {
            std::thread::sleep(std::time::Duration::from_secs(600));
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
        // The busy line is erased when the work is done, as a real agent does.
        let _ = write!(out, "\r\x1b[2K");

        let answer = env("FAKE_ANSWER").unwrap_or_else(|| "feat: add the thing".into());
        let extra: usize = env("FAKE_LINES").and_then(|n| n.parse().ok()).unwrap_or(0);
        let mut body = String::new();
        for line in answer.split("\\n") {
            body.push_str(line);
            body.push('\n');
        }
        for n in 0..extra {
            body.push_str(&format!("filler line {n}\n"));
        }
        let framed = format!("<<GP-{nonce}>>\n{body}<<GP-END-{nonce}>>");

        let on_screen = !(flavor == "claude" && env("FAKE_TRANSCRIPT").is_some());
        if let (Some(dir), Some(id)) = (env("FAKE_TRANSCRIPT"), &session_id) {
            let folder = std::path::Path::new(&dir).join("projects").join("fake");
            let _ = std::fs::create_dir_all(&folder);
            let entry = format!(
                "{{\"type\":\"assistant\",\"message\":{{\"role\":\"assistant\",\"content\":[{{\"type\":\"text\",\"text\":{}}}],\"stop_reason\":\"end_turn\"}}}}\n",
                json_string(&framed)
            );
            let user = "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"prompt\"}}\n";
            let _ = std::fs::write(folder.join(format!("{id}.jsonl")), format!("{user}{entry}"));
        }
        // Clear the busy line by moving on: the screen keeps scrolling.
        if on_screen {
            let mut first = true;
            for line in framed.lines() {
                if first {
                    let _ = write!(out, "\u{25cf} {line}\r\n");
                    first = false;
                } else {
                    let _ = write!(out, "  {line}\r\n");
                }
            }
        }
        let _ = write!(out, "{ready}\r\n");
        let _ = out.flush();
    }
}

fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
