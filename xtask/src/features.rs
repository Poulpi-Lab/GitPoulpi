//! `xtask features [--quick] [--dry-run]`: the feature-matrix runner of plan
//! section 11. It builds on the same choice -> feature mapping as
//! `build-custom`, so the matrix tests exactly what the script can produce.
//! Needs the S1+ feature graph in the crates' Cargo.toml to pass.

use crate::custom_build::{available_languages, Choices};
use std::path::Path;
use std::process::Command;

const USAGE: &str = "\
Usage: cargo run -p xtask -- features [--quick] [--dry-run]

  --quick     clippy runs, each twice (bins only, as a custom build compiles
              them, then --all-targets): everything off, history rewrite with scripts off, `lang-fr` + github,
              `lang-fr` + jira (a provider outside GitHub/GitLab), and
              `lang-fr` + linear / youtrack / redmine / gerrit alone
  --dry-run   print the cargo commands without running them
  (default)   each feature turned off once, no integrations, then
              `cargo test --no-default-features` on the library crates and
              the tui lib tests with `all-langs`";

type Toggle = (&'static str, fn(&mut Choices));

const BINS: [&str; 3] = ["gitpoulpi-cli", "gitpoulpi-tui", "gitpoulpi-mcp"];

/// One cargo invocation (arguments after `cargo`). `all_targets` adds tests,
/// examples and benches; without it the run compiles only what
/// `xtask build-custom` compiles (the libraries and the three binaries).
///
/// The two are not interchangeable. `xtask` is a dev-dependency of
/// `gitpoulpi-cli` and `gitpoulpi-tui` (its pty driver) and itself depends on
/// `gitpoulpi-daemon`, `-cli-backend` and `-git2-backend` with their default
/// features, so with `--all-targets` cargo's feature unification turns those
/// defaults (`lfs`, `worktree`, ...) back on and a broken feature-off build
/// of the backends stays green. The bins-only run has no dev-dependencies
/// in its graph, so it sees the features exactly as a custom build does.
pub fn clippy(choices: &Choices, all_targets: bool) -> Vec<String> {
    let mut a: Vec<String> = ["clippy", "--locked"].map(String::from).to_vec();
    for p in BINS {
        a.push("-p".into());
        a.push(p.into());
    }
    if all_targets {
        a.push("--all-targets".into());
    }
    a.push("--no-default-features".into());
    let f = choices.features();
    if !f.is_empty() {
        a.push("--features".into());
        a.push(f.join(","));
    }
    a.extend(["--", "-D", "warnings"].map(String::from));
    a
}

/// History rewriting on, its embedded Lua scripts off: a Script rule is a
/// typed refusal and nothing links Lua. `Choices` ties the two together, so
/// the feature is dropped from the generated list.
fn scripts_off() -> Vec<Run> {
    let mut c = Choices::none();
    c.history_rewrite = true;
    let strip = |mut args: Vec<String>| {
        for a in args.iter_mut() {
            if a.contains("/history-rewrite") {
                *a = a
                    .split(',')
                    .filter(|f| !f.ends_with("/rewrite-scripts"))
                    .collect::<Vec<_>>()
                    .join(",");
            }
        }
        args
    };
    both("history-rewrite, scripts off", &c, |c, all| {
        strip(clippy(c, all))
    })
}

/// The bins-only run (the one that matches a custom build) then the
/// all-targets run, both labeled after `label`.
fn both(label: &str, c: &Choices, args: impl Fn(&Choices, bool) -> Vec<String>) -> Vec<Run> {
    vec![
        (format!("{label} (bins only)"), args(c, false), c.env()),
        (label.to_string(), args(c, true), c.env()),
    ]
}

fn clippy_pair(label: &str, c: &Choices) -> Vec<Run> {
    both(label, c, clippy)
}

/// The command list, labeled.
/// One labeled cargo run: arguments after `cargo` and extra environment.
pub type Run = (String, Vec<String>, Vec<(&'static str, String)>);

pub fn plan(all_langs: &[String], quick: bool) -> Vec<Run> {
    let mut out = Vec::new();
    // The matrix always builds the three binary crates, mcp included: with
    // the feature off, `gitpoulpi-mcp` still has to compile on its own.
    if quick {
        out.extend(clippy_pair("minimal", &Choices::none()));
        let mut c = Choices::none();
        c.langs = vec!["fr".into()];
        c.integrations = vec!["github".into()];
        out.extend(clippy_pair("lang-fr + github", &c));
        // A provider outside the GitHub/GitLab family on its own: the
        // `hosting` guards once assumed one of those two was always on.
        c.integrations = vec!["jira".into()];
        out.extend(clippy_pair("lang-fr + jira", &c));
        for only in ["linear", "youtrack", "redmine", "gerrit"] {
            c.integrations = vec![only.into()];
            out.extend(clippy_pair(&format!("lang-fr + {only}"), &c));
        }
        out.extend(scripts_off());
        return out;
    }
    let full = Choices::all(all_langs);
    let toggles: [Toggle; 29] = [
        ("github", |c| c.integrations.retain(|i| i != "github")),
        ("gitlab", |c| c.integrations.retain(|i| i != "gitlab")),
        ("azure-devops", |c| {
            c.integrations.retain(|i| i != "azure-devops")
        }),
        ("bitbucket", |c| c.integrations.retain(|i| i != "bitbucket")),
        ("jira", |c| c.integrations.retain(|i| i != "jira")),
        ("trello", |c| c.integrations.retain(|i| i != "trello")),
        ("gitea", |c| c.integrations.retain(|i| i != "gitea")),
        ("linear", |c| c.integrations.retain(|i| i != "linear")),
        ("youtrack", |c| c.integrations.retain(|i| i != "youtrack")),
        ("redmine", |c| c.integrations.retain(|i| i != "redmine")),
        ("gerrit", |c| c.integrations.retain(|i| i != "gerrit")),
        ("local-issues", |c| {
            c.integrations.retain(|i| i != "local-issues")
        }),
        ("no integrations", |c| c.integrations.clear()),
        ("ai", |c| c.ai = false),
        ("mcp", |c| c.mcp = false),
        ("lfs", |c| c.lfs = false),
        ("worktree", |c| c.worktree = false),
        ("submodule", |c| c.submodules = false),
        ("avatar-fetch", |c| c.avatar_fetch = false),
        ("image-view", |c| c.image_view = false),
        ("radar", |c| c.radar = false),
        ("mermaid", |c| c.mermaid = false),
        ("board", |c| c.board = false),
        ("review", |c| c.review = false),
        ("conversation", |c| c.conversation = false),
        ("history-rewrite", |c| c.history_rewrite = false),
        ("global-search", |c| c.global_search = false),
        ("workspaces", |c| c.workspaces = false),
        ("themes", |c| c.themes = Some(vec![])),
    ];
    for (name, off) in toggles {
        let mut c = full.clone();
        off(&mut c);
        out.extend(clippy_pair(&format!("full minus {name}"), &c));
    }
    out.extend(scripts_off());
    let lib_crates = [
        "gitpoulpi-i18n",
        "gitpoulpi-commands",
        "gitpoulpi-mcp",
        "gitpoulpi-cli-backend",
        "gitpoulpi-git2-backend",
        "gitpoulpi-hosting",
    ];
    // `--lib --tests`: no doctests (the i18n one shows French output).
    let mut t = ["test", "--no-default-features", "--lib", "--tests"]
        .map(String::from)
        .to_vec();
    for p in lib_crates {
        t.push("-p".into());
        t.push(p.into());
    }
    out.push(("tests, minimal libs".into(), t, vec![]));
    // The theme tests (contrast, ordering, golden showcase) run over
    // whatever is compiled: with no optional theme and with one family.
    for themes in ["none", "nord"] {
        out.push((
            format!("theme tests, themes {themes}"),
            ["test", "-p", "gitpoulpi-config", "--lib", "theme"]
                .map(String::from)
                .to_vec(),
            vec![("GITPOULPI_THEMES", themes.to_string())],
        ));
    }
    // The UI tests assume the shipped languages (French goldens, month
    // names), so they run with `all-langs` and everything else off.
    out.push((
        "tui lib tests, all-langs only".into(),
        [
            "test",
            "-p",
            "gitpoulpi-tui",
            "--no-default-features",
            "--features",
            "all-langs",
            "--lib",
        ]
        .map(String::from)
        .to_vec(),
        vec![],
    ));
    out.push((
        "tui build_features tests".into(),
        [
            "test",
            "-p",
            "gitpoulpi-tui",
            "--no-default-features",
            "--lib",
            "build_features",
        ]
        .map(String::from)
        .to_vec(),
        vec![],
    ));
    out
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let mut quick = false;
    let mut dry = false;
    for a in args {
        match a.as_str() {
            "--quick" => quick = true,
            "--dry-run" => dry = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(0);
            }
            other => return Err(format!("unknown option '{other}'.\n\n{USAGE}")),
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let langs: Vec<String> = available_languages(root)?
        .into_iter()
        .map(|(c, _)| c)
        .collect();
    let mut failed = Vec::new();
    for (label, args, env) in plan(&langs, quick) {
        let env_text: String = env.iter().map(|(k, v)| format!("{k}={v} ")).collect();
        println!("== {label}\n{env_text}cargo {}", args.join(" "));
        if dry {
            continue;
        }
        let ok = Command::new("cargo")
            .args(&args)
            .envs(env)
            .current_dir(root)
            .status()
            .map_err(|e| format!("running cargo: {e}"))?
            .success();
        if !ok {
            failed.push(label);
        }
    }
    if failed.is_empty() {
        Ok(0)
    } else {
        eprintln!("\nFailed: {}", failed.join("; "));
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_is_eight_feature_sets_each_checked_twice() {
        let p = plan(&["fr".to_string()], true);
        assert_eq!(p.len(), 16);
        // Pairs: the bins-only run, then the --all-targets run.
        for pair in p.chunks(2) {
            assert!(pair[0].0.ends_with(" (bins only)"), "{}", pair[0].0);
            assert!(!pair[0].1.contains(&"--all-targets".to_string()));
            assert!(pair[1].1.contains(&"--all-targets".to_string()));
            // Same features and env either way, only the targets differ.
            let strip = |a: &[String]| -> Vec<String> {
                a.iter()
                    .filter(|x| *x != "--all-targets")
                    .cloned()
                    .collect()
            };
            assert_eq!(strip(&pair[0].1), strip(&pair[1].1));
            assert_eq!(pair[0].2, pair[1].2);
        }
        assert!(p[0].1.contains(&"--no-default-features".to_string()));
        assert!(!p[0].1.contains(&"--features".to_string()));
        assert!(p[2].1.join(" ").contains("gitpoulpi-tui/lang-fr"));
        let jira = p[4].1.join(" ");
        assert!(jira.contains("/jira") && !jira.contains("/github") && !jira.contains("/gitlab"));
        for (run, only) in [
            (6, "linear"),
            (8, "youtrack"),
            (10, "redmine"),
            (12, "gerrit"),
        ] {
            let args = p[run].1.join(" ");
            assert!(args.contains(&format!("/{only}")), "{args}");
            assert!(
                !args.contains("/jira") && !args.contains("/github"),
                "{args}"
            );
        }
    }

    #[test]
    fn quick_has_a_run_with_scripts_off() {
        let p = plan(&["fr".to_string()], true);
        for run in &p[p.len() - 2..] {
            let args = run.1.join(" ");
            assert!(args.contains("/history-rewrite"), "{args}");
            assert!(!args.contains("rewrite-scripts"), "{args}");
        }
        let full = plan(&["fr".to_string()], false);
        for l in [
            "history-rewrite, scripts off",
            "history-rewrite, scripts off (bins only)",
        ] {
            assert!(full.iter().any(|(label, _, _)| label == l), "{l}");
        }
    }

    #[test]
    fn full_matrix_turns_each_feature_off_once() {
        let p = plan(&["fr".to_string()], false);
        let minus_ai = &p.iter().find(|(l, _, _)| l == "full minus ai").unwrap().1;
        assert!(!minus_ai.join(" ").contains("/ai"));
        assert!(minus_ai.join(" ").contains("/lfs"));
        // 28 toggles + the scripts-off run, each in two forms, then 5 test runs.
        assert_eq!(p.len(), (28 + 1) * 2 + 5);
        let bins = p
            .iter()
            .find(|(l, _, _)| l == "full minus lfs (bins only)")
            .unwrap();
        assert!(!bins.1.contains(&"--all-targets".to_string()));
        assert!(!bins.1.join(" ").contains("/lfs"));
        let themes = p.iter().find(|(l, _, _)| l == "full minus themes").unwrap();
        assert_eq!(themes.2, vec![("GITPOULPI_THEMES", "none".to_string())]);
    }
}
