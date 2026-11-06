//! `scripts/build-custom.sh` with arguments is non-interactive and hands them
//! to `xtask build-custom`; with `--dry-run` it only prints the command, so
//! no real build ever runs here.

use std::path::Path;
use std::process::Command;

fn bash_available() -> bool {
    Command::new("bash")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn dry_run(args: &[&str]) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let out = Command::new("bash")
        .arg(root.join("scripts/build-custom.sh"))
        .args(args)
        .arg("--dry-run")
        .current_dir(root)
        .output()
        .expect("run build-custom.sh");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn script_dry_run_prints_the_cargo_command() {
    if !bash_available() {
        eprintln!("bash missing; skipping");
        return;
    }
    let out = dry_run(&[
        "--langs",
        "fr",
        "--no-ai",
        "--integrations",
        "none",
        "--no-mcp",
    ]);
    let cmd = out
        .lines()
        .find(|l| l.starts_with("RUSTC_WRAPPER= cargo build"))
        .unwrap_or_else(|| panic!("no cargo line in:\n{out}"));
    assert!(cmd.contains("--profile custom --locked"));
    assert!(cmd
        .contains("-p gitpoulpi-cli -p gitpoulpi-tui -p gitpoulpi-updater --no-default-features"));
    assert!(!cmd.contains("gitpoulpi-mcp"));
    assert!(cmd.contains("gitpoulpi-cli/lang-fr,gitpoulpi-cli/lfs"));
    assert!(!cmd.contains("/ai") && !cmd.contains("github") && !cmd.contains("gitlab"));
    assert!(out.contains("languages: en fr"));
}

#[test]
fn script_rejects_an_unknown_language() {
    if !bash_available() {
        return;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let out = Command::new("bash")
        .arg(root.join("scripts/build-custom.sh"))
        .args(["--langs", "xx", "--dry-run"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown language 'xx'"));
}

#[test]
fn script_dry_run_shows_the_theme_selection() {
    if !bash_available() {
        return;
    }
    let out = dry_run(&["--themes", "nord,catppuccin"]);
    let cmd = out
        .lines()
        .find(|l| l.contains("RUSTC_WRAPPER= cargo build"))
        .unwrap_or_else(|| panic!("no cargo line in:\n{out}"));
    assert!(
        cmd.starts_with("GITPOULPI_THEMES=catppuccin,nord RUSTC_WRAPPER= cargo build"),
        "{cmd}"
    );
    assert!(
        out.contains("themes: dark light terminal catppuccin nord"),
        "{out}"
    );
    let none = dry_run(&["--themes", "none"]);
    assert!(none.contains("GITPOULPI_THEMES=none "), "{none}");
    let all = dry_run(&[]);
    assert!(!all.contains("GITPOULPI_THEMES"), "{all}");
    assert!(all.contains("themes: all"), "{all}");
}

#[test]
fn script_rejects_an_unknown_theme_family() {
    if !bash_available() {
        return;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let out = Command::new("bash")
        .arg(root.join("scripts/build-custom.sh"))
        .args(["--themes", "nordd", "--dry-run"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown theme family 'nordd'"));
}

#[test]
fn script_dry_run_shows_the_logo_selection() {
    if !bash_available() {
        return;
    }
    let out = dry_run(&[
        "--logo-colors",
        "violet,teal",
        "--logo-default",
        "teal",
        "--icon-color",
        "sun",
    ]);
    let cmd = out.lines().find(|l| l.contains("cargo build")).unwrap();
    for part in [
        "GITPOULPI_LOGO_COLORS=violet,teal",
        "GITPOULPI_LOGO_DEFAULT=teal",
        "GITPOULPI_ICON_COLOR=sun",
    ] {
        assert!(cmd.contains(part), "{cmd}");
    }
    assert!(
        out.contains("logo: violet teal (default teal, icon sun)"),
        "{out}"
    );
    let all = dry_run(&[]);
    assert!(!all.contains("GITPOULPI_LOGO"), "{all}");
}

#[test]
fn script_rejects_a_default_logo_color_that_is_not_compiled() {
    if !bash_available() {
        return;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let out = Command::new("bash")
        .arg(root.join("scripts/build-custom.sh"))
        .args([
            "--logo-colors",
            "violet",
            "--logo-default",
            "teal",
            "--dry-run",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not among the compiled"));
}
