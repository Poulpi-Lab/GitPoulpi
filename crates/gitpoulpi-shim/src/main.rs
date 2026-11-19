//! The stub that ships under the old program names `gitpoulpi-tui` and
//! `gitpoulpi-mcp`. It finds the `gitpoulpi` executable next to its own
//! (symlink-resolved) path and starts it as `gitpoulpi --gitpoulpi-as <own
//! stem> <args...>`. Std only, so it stays tiny.
//!
//! Unix: `exec`, no extra process. Windows: a child with the inherited
//! standard handles, waited for, its exit code returned; Ctrl+C is ignored
//! here because the child receives it too and decides what to do.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The hidden flag `gitpoulpi` reads (see `gitpoulpi-cli`'s `persona.rs`).
const FLAG: &str = "--gitpoulpi-as";

#[cfg(windows)]
const TARGET: &str = "gitpoulpi.exe";
#[cfg(not(windows))]
const TARGET: &str = "gitpoulpi";

/// The persona name: the file stem of `exe`, without a `.exe` suffix.
fn persona_of(exe: &Path) -> Option<String> {
    let name = exe.file_name()?.to_str()?;
    let stem = match name.len().checked_sub(4) {
        Some(at) if name.is_char_boundary(at) && name[at..].eq_ignore_ascii_case(".exe") => {
            &name[..at]
        }
        _ => name,
    };
    (!stem.is_empty()).then(|| stem.to_string())
}

/// `gitpoulpi` next to `exe`.
fn target_of(exe: &Path) -> Option<PathBuf> {
    Some(exe.parent()?.join(TARGET))
}

/// The arguments for the real program: the flag, the persona, then `rest`.
fn child_args(persona: &str, rest: impl Iterator<Item = OsString>) -> Vec<OsString> {
    let mut args = vec![OsString::from(FLAG), OsString::from(persona)];
    args.extend(rest);
    args
}

fn fail(message: &str) -> ! {
    eprintln!("gitpoulpi: {message}");
    std::process::exit(127);
}

fn main() {
    let own = std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .unwrap_or_else(|e| fail(&format!("cannot locate this program: {e}")));
    let persona = persona_of(&own).unwrap_or_else(|| {
        fail(&format!(
            "cannot tell the program name from {}",
            own.display()
        ))
    });
    let target = target_of(&own).unwrap_or_else(|| fail("cannot find the install folder"));
    if !target.is_file() {
        fail(&format!(
            "{} was not found next to {} (an incomplete install?)",
            TARGET,
            own.display()
        ));
    }
    let args = child_args(&persona, std::env::args_os().skip(1));
    run(&target, &args);
}

#[cfg(unix)]
fn run(target: &Path, args: &[OsString]) -> ! {
    use std::os::unix::process::CommandExt;
    let err = std::process::Command::new(target).args(args).exec();
    fail(&format!("cannot start {}: {err}", target.display()));
}

#[cfg(windows)]
fn run(target: &Path, args: &[OsString]) -> ! {
    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleCtrlHandler(handler: *const core::ffi::c_void, add: i32) -> i32;
    }
    // SAFETY: a null handler with `add` true makes this process ignore Ctrl+C.
    unsafe {
        SetConsoleCtrlHandler(std::ptr::null(), 1);
    }
    match std::process::Command::new(target).args(args).status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => fail(&format!("cannot start {}: {e}", target.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persona_drops_the_exe_suffix() {
        for (path, want) in [
            ("/x/gitpoulpi-tui", Some("gitpoulpi-tui")),
            ("x/gitpoulpi-mcp.exe", Some("gitpoulpi-mcp")),
            ("gitpoulpi-mcp.EXE", Some("gitpoulpi-mcp")),
            (".exe", None),
        ] {
            assert_eq!(persona_of(Path::new(path)).as_deref(), want, "{path}");
        }
    }

    #[test]
    fn target_is_next_to_the_stub() {
        let got = target_of(Path::new("/opt/gp/gitpoulpi-tui")).unwrap();
        assert_eq!(got, Path::new("/opt/gp").join(TARGET));
    }

    #[test]
    fn args_start_with_the_flag_and_persona() {
        let got = child_args("gitpoulpi-mcp", ["--http".into()].into_iter());
        assert_eq!(got, ["--gitpoulpi-as", "gitpoulpi-mcp", "--http"]);
    }
}
