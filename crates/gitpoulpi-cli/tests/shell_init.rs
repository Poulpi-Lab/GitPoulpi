//! `gitpoulpi shell-init <shell>` prints a wrapper script (no daemon, no rc
//! file touched), and the bash wrapper really leaves the shell in the folder
//! the TUI wrote to `--cwd-file`.

use assert_cmd::Command;

fn shell_init(args: &[&str]) -> std::process::Output {
    let config = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .arg("shell-init")
        .args(args)
        .output()
        .unwrap();
    // Printing a script starts no daemon.
    assert!(!config.path().join("sessions").exists());
    output
}

#[test]
fn every_shell_prints_a_wrapper_around_cwd_file() {
    for shell in ["bash", "zsh", "fish", "pwsh", "powershell"] {
        let output = shell_init(&[shell]);
        assert!(output.status.success(), "{shell}: {output:?}");
        let script = String::from_utf8(output.stdout).unwrap();
        assert!(script.contains("--cwd-file"), "{shell}: {script}");
        assert!(script.contains("gp"), "{shell}");
    }
}

#[test]
fn a_custom_name_replaces_gp_and_a_bad_one_is_refused() {
    let output = shell_init(&["bash", "--name", "gpl"]);
    let script = String::from_utf8(output.stdout).unwrap();
    assert!(script.contains("function gpl {") && !script.contains("function gp {"));
    let bad = shell_init(&["bash", "--name", "a;b"]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("function name"));
}

#[cfg(unix)]
#[test]
fn the_bash_wrapper_ends_the_shell_in_the_folder_the_tui_wrote() {
    use std::os::unix::fs::PermissionsExt;
    let work = tempfile::tempdir().unwrap();
    let target = work.path().join("the repo");
    std::fs::create_dir(&target).unwrap();
    let bin = work.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    // A fake `gitpoulpi`: finds --cwd-file and writes $FAKE_TARGET into it,
    // unless FAKE_QUIET is set (a plain quit writes nothing).
    let fake = bin.join("gitpoulpi");
    std::fs::write(
        &fake,
        "#!/bin/sh\nwhile [ $# -gt 0 ]; do\n  if [ \"$1\" = --cwd-file ]; then f=\"$2\"; shift; fi\n  shift\ndone\n[ -n \"$FAKE_QUIET\" ] || printf '%s' \"$FAKE_TARGET\" > \"$f\"\nexit 7\n",
    )
    .unwrap();
    std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();

    let script = String::from_utf8(shell_init(&["bash"]).stdout).unwrap();
    let rc = work.path().join("rc.sh");
    std::fs::write(&rc, &script).unwrap();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());

    let run = |quiet: bool| {
        let mut cmd = std::process::Command::new("bash");
        cmd.arg("-c")
            .arg(format!(
                ". '{}'; cd /; gp --session x; echo \"rc=$? pwd=$PWD\"",
                rc.display()
            ))
            .env("PATH", &path)
            .env("TMPDIR", work.path())
            .env("FAKE_TARGET", &target);
        if quiet {
            cmd.env("FAKE_QUIET", "1");
        }
        String::from_utf8(cmd.output().unwrap().stdout).unwrap()
    };
    // The exit status of gitpoulpi survives, and the shell moved.
    assert_eq!(run(false).trim(), format!("rc=7 pwd={}", target.display()));
    // Nothing written: the shell stays where it was.
    assert_eq!(run(true).trim(), "rc=7 pwd=/");
    // No temp file is left behind.
    let left: Vec<_> = std::fs::read_dir(work.path())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("gitpoulpi-cwd"))
        .collect();
    assert!(left.is_empty(), "{left:?}");
}

#[cfg(unix)]
#[test]
fn the_bash_wrapper_does_not_override_an_existing_command() {
    let script = String::from_utf8(shell_init(&["bash"]).stdout).unwrap();
    let out = std::process::Command::new("bash")
        .arg("-c")
        .arg(format!(
            "alias gp='echo mine'; shopt -s expand_aliases; {script}\ntype -t gp"
        ))
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "alias");
    assert!(String::from_utf8_lossy(&out.stderr).contains("already exists"));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
