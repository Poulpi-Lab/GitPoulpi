//! Running a configured hook: a shell command, in the repository, with its
//! output captured whole (the caller shows it; nothing runs silently).

use std::path::PathBuf;

/// The most output a report keeps: the tail, where a failure usually is.
const MAX_OUTPUT: usize = 32 * 1024;

pub(super) struct HookResult {
    pub exit_code: i32,
    pub output: String,
}

fn shell() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    }
}

/// Runs `command` through the shell with the repository as working
/// directory. `Err` only when the shell could not be started.
pub(super) async fn run(
    root: PathBuf,
    command: String,
    action: &'static str,
    branch: Option<String>,
) -> Result<HookResult, String> {
    let joined = tokio::task::spawn_blocking(move || {
        let (program, flag) = shell();
        let mut cmd = std::process::Command::new(program);
        cmd.arg(flag)
            .arg(&command)
            .current_dir(&root)
            .env("GITPOULPI_FLOW_ACTION", action)
            .stdin(std::process::Stdio::null());
        if let Some(branch) = &branch {
            cmd.env("GITPOULPI_FLOW_BRANCH", branch);
        }
        cmd.output()
    })
    .await
    .map_err(|e| format!("the hook could not run: {e}"))?;
    let output = joined.map_err(|e| format!("could not start the shell: {e}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&stderr);
    }
    if text.len() > MAX_OUTPUT {
        let mut cut = text.len() - MAX_OUTPUT;
        while !text.is_char_boundary(cut) {
            cut += 1;
        }
        text = format!("... (output truncated)\n{}", &text[cut..]);
    }
    Ok(HookResult {
        exit_code: output.status.code().unwrap_or(-1),
        output: text,
    })
}
