//! `gitpoulpi completions <shell>` prints a completion script and starts no
//! daemon (the config dir stays empty).

use assert_cmd::Command;

fn completions(config: &std::path::Path, shell: &str) -> std::process::Output {
    Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .args(["completions", shell])
        .output()
        .unwrap()
}

#[test]
fn every_supported_shell_gets_a_script_that_knows_the_subcommands() {
    let config = tempfile::tempdir().unwrap();
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let output = completions(config.path(), shell);
        assert!(output.status.success(), "{shell}: {:?}", output);
        let script = String::from_utf8(output.stdout).unwrap();
        assert!(script.contains("gitpoulpi"), "{shell}: {script}");
        assert!(script.contains("session"), "{shell} lacks `session`");
    }
    // No daemon was started for any of them.
    assert!(!config.path().join("sessions").exists());
}

#[test]
fn an_unknown_shell_is_refused() {
    let config = tempfile::tempdir().unwrap();
    let output = completions(config.path(), "tcsh");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("tcsh"));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
