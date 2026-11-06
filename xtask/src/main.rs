//! cargo-xtask: repeatable dev/release tasks that don't warrant an
//! external task runner. Run with `cargo run -p xtask -- <task>`.

const USAGE: &str = "\
Usage: cargo run -p xtask -- <task>

Tasks:
  tui <script|->    drive a terminal program through a pty and print what
                    it drew - the default way to verify the TUI. See
                    docs/developer/visual-verification.md.
  desktop-lock      run a command holding the machine-wide desktop lock,
                    for the rare verification that needs real pixels.
  perf [options]    perf regression guard: time and git-spawn budget per
                    operation (--help for options).
  dist [options]    build the release binaries with the `dist` profile and
                    pack target/dist/gitpoulpi-<version>-<target>.<zip|tar.gz>
                    plus a SHA-256 file (--help for options).
  package-marker --channel <c> --version <v> --target <t> --out <file>
                    write the .gitpoulpi-dist marker a package installs, with
                    its install channel (--help).
  package-manifests [--dir <dir>]
                    write latest.json (version, notes, sha256 of every asset)
                    from the .sha256 files of a release (--help).
  build-custom [options]
                    size-optimised build with only the chosen languages,
                    integrations and features (--help). Interactive front
                    ends: scripts/build-custom.sh / .ps1.
  features [--quick]
                    feature-matrix check of the custom-build graph.
  test [--tier fast|pty|full] [--no-fail-fast] [--failed] [--dry-run]
                    the test suite in tiers: fast = everything but the pty_*
                    end-to-end targets, pty = only those, full = both.
  licenses [--check]
                    regenerate THIRD-PARTY-LICENSES.txt from Cargo.lock
                    (offline); --check only verifies it is current.
  merge-fixture     build a repository stopped on a conflicting merge, to open
                    the merge conflict editor on it (--help for options).
  mock-host [--port N] [--bare-root DIR]
                    fake GitHub + GitLab hosting service on 127.0.0.1 for the
                    Clone dialog's tests and doc screenshots (--help).
  emit <hex>        write exactly these bytes and stay alive - the
                    predictable child the pty driver's own tests drive.";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (task, rest) = match args.split_first() {
        Some((task, rest)) => (task.as_str(), rest),
        None => {
            eprintln!("xtask: no task given.\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    let outcome = match task {
        "tui" => xtask::script::run(rest).map(|()| 0),
        "desktop-lock" => xtask::desktop_lock::run(rest),
        "perf" => xtask::perf::run(rest),
        "dist" => xtask::dist::run(rest),
        "package-marker" => xtask::dist::package_marker(rest),
        "package-manifests" => xtask::dist::package_manifests(rest),
        "build-custom" => xtask::custom_build::run(rest),
        "features" => xtask::features::run(rest),
        "licenses" => xtask::licenses::run(rest),
        "test" => xtask::test_tiers::run(rest),
        "merge-fixture" => xtask::merge_fixture::run(rest),
        "mock-host" => xtask::mock_host::run(rest),
        "emit" => xtask::emit::run(rest),
        "-h" | "--help" | "help" => {
            println!("{USAGE}");
            return;
        }
        other => {
            eprintln!("xtask: unknown task '{other}'.\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    match outcome {
        Ok(code) => std::process::exit(code),
        Err(message) => {
            eprintln!("xtask {task}: {message}");
            std::process::exit(1);
        }
    }
}
