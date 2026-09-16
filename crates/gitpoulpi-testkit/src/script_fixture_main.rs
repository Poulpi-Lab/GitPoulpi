// The program that stands in for Python (and an installed Lua) in the history
// rewrite tests: it is compiled by `gitpoulpi_testkit::script_fixture`, so no
// interpreter has to be installed on CI. It is run as `<program> [flags] FILE`
// like an interpreter, reads FILE, and speaks the external script contract
// (JSON Lines on stdin and stdout) in the mode FILE names on a `mode=...` line.
//
// It parses the request by hand (std only). The requests' keys are sorted, so
// `"author":{"email":...,"name":"..."` is where the author's name sits.

use std::io::{BufRead, Write};

fn after<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    text.find(marker).map(|at| &text[at + marker.len()..])
}

fn author_name(request: &str) -> Option<String> {
    let author = after(request, "\"author\":{")?;
    let name = after(author, "\"name\":\"")?;
    Some(name[..name.find('"')?].to_string())
}

fn counter(request: &str) -> i64 {
    after(request, "\"state\":{\"n\":")
        .and_then(|rest| {
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        })
        .unwrap_or(0)
}

fn setting<'a>(script: &'a str, key: &str) -> Option<&'a str> {
    script
        .lines()
        .find_map(|l| l.trim().strip_prefix(key))
        .map(str::trim)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("fixture-interpreter 3.12.0");
        return;
    }
    let script = args
        .last()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let mode = setting(&script, "mode=").unwrap_or("upper").to_string();
    if let Some(path) = setting(&script, "pidfile=") {
        let _ = std::fs::write(path, std::process::id().to_string());
    }
    if let Some(dir) = setting(&script, "cwdfile=") {
        let cwd = std::env::current_dir().unwrap_or_default();
        let _ = std::fs::write(dir, cwd.display().to_string());
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for (n, line) in stdin.lock().lines().enumerate() {
        let Ok(request) = line else { return };
        let answer = match mode.as_str() {
            // Uppercase the author's name.
            "upper" => match author_name(&request) {
                Some(name) => format!("{{\"author\":{{\"name\":\"{}\"}}}}", name.to_uppercase()),
                None => "{}".to_string(),
            },
            // The author's name is the number of commits seen so far; the
            // count travels in `state`.
            "count" => {
                let seen = counter(&request);
                format!(
                    "{{\"author\":{{\"name\":\"n{seen}\"}},\"state\":{{\"n\":{}}}}}",
                    seen + 1
                )
            }
            // Reports what the environment held.
            "env" => {
                let value = std::env::var("GITPOULPI_PLANTED_SECRET")
                    .map(|v| format!("set:{v}"))
                    .unwrap_or_else(|_| "unset".into());
                format!("{{\"message\":\"secret={value}\\n\"}}")
            }
            // Never answers.
            "hang" => loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
            },
            // Dies on the third request.
            "crash" => {
                if n == 2 {
                    eprintln!("fixture crash");
                    std::process::exit(7);
                }
                "{}".to_string()
            }
            "junk" => "this is not json".to_string(),
            "bad_field" => "{\"tree\":\"0000\"}".to_string(),
            "error" => "{\"error\":\"nope\"}".to_string(),
            _ => "{}".to_string(),
        };
        if writeln!(stdout, "{answer}").is_err() || stdout.flush().is_err() {
            return;
        }
    }
}
