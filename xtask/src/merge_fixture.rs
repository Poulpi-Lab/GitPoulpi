//! `cargo run -p xtask -- merge-fixture [--kind text|multi] [--dir DIR]`:
//! builds a repository stopped in the middle of a conflicting merge, so the
//! merge conflict editor can be opened, driven and screenshotted by hand
//! (see `docs/developer/visual-verification.md`).
//!
//! Plain `git` only, isolated from the user's own configuration, and never
//! anywhere but `DIR` (default: a fresh directory under the system temp
//! dir). Prints the repository path and the commands that open it:
//!
//! ```text
//! cargo run -p xtask -- merge-fixture --dir /tmp/mf
//! GITPOULPI_CONFIG_DIR=/tmp/mf-config target/debug/gitpoulpi-tui -s mf   # from /tmp/mf
//! ```
//!
//! then press Enter on a `!` row in the Status pane (right).

use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "\
Usage: cargo run -p xtask -- merge-fixture [--kind text|multi|big] [--dir DIR]
                                            [--lines N] [--conflicts K]

  --kind text    one Rust file, two conflicts and a change only the incoming
                 side made (default)
  --kind multi   three files, the first with several conflicts
  --kind big     one Rust file of --lines lines (default 10000) with
                 --conflicts conflicts (default 50), for the perf pass
  --dir DIR      where to build it (must not exist or be empty); default: a
                 new directory under the system temp dir";

pub(crate) fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .current_dir(dir)
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.com",
        ])
        .args(["-c", "commit.gpgsign=false", "-c", "core.autocrlf=false"])
        .args(args)
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    Ok(format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ))
}

pub(crate) fn write(dir: &Path, rel: &str, text: &str) -> Result<(), String> {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, text).map_err(|e| e.to_string())
}

const BASE_RS: &str = "\
use std::fmt;

/// A user of the system.
pub struct User {
    pub name: String,
    pub age: u32,
}

impl User {
    pub fn new(name: &str, age: u32) -> User {
        User { name: name.to_string(), age }
    }

    pub fn greeting(&self) -> String {
        format!(\"Hello, {}\", self.name)
    }

    pub fn is_adult(&self) -> bool {
        self.age >= 18
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, \"{} ({})\", self.name, self.age)
    }
}

pub fn oldest(users: &[User]) -> Option<&User> {
    users.iter().max_by_key(|u| u.age)
}
";

fn ours_rs() -> String {
    BASE_RS
        .replace(
            "format!(\"Hello, {}\", self.name)",
            "format!(\"Welcome back, {}\", self.name)",
        )
        .replace("self.age >= 18", "self.age >= 21")
}

fn theirs_rs() -> String {
    BASE_RS
        .replace(
            "format!(\"Hello, {}\", self.name)",
            "format!(\"Hi {}!\", self.name)",
        )
        .replace("self.age >= 18", "self.age >= 16")
        .replace(
            "users.iter().max_by_key(|u| u.age)",
            "users.iter().max_by_key(|u| u.age).filter(|u| u.is_adult())",
        )
}

fn numbered(n: usize, changes: &[(usize, &str)]) -> String {
    (1..=n)
        .map(|i| {
            changes
                .iter()
                .find(|(l, _)| *l == i)
                .map_or_else(|| format!("line {i}"), |(_, t)| (*t).to_string())
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn num(v: Option<&String>, name: &str) -> Result<usize, String> {
    v.ok_or(format!("{name} needs a value"))?
        .parse()
        .map_err(|_| format!("bad {name}"))
}

/// The three versions of `src/big.rs`: `lines` lines of Rust-looking code,
/// `conflicts` three-line blocks both sides rewrite, plus changes only one
/// side makes (so the Full file mode has something to color).
pub fn big_versions(lines: usize, conflicts: usize) -> [String; 3] {
    let base_line = |i: usize| {
        if i.is_multiple_of(25) {
            format!("pub fn step_{i}(input: &[u32], offset: u32) -> Option<u32> {{")
        } else if i % 25 == 24 {
            "}".to_string()
        } else {
            format!("    let value_{i} = compute(input[{i} % 7], {i}) + offset; // step {i}")
        }
    };
    let mut base: Vec<String> = (0..lines).map(base_line).collect();
    let mut ours = base.clone();
    let mut theirs = base.clone();
    for c in 0..conflicts {
        let p = (c * 2 + 1) * lines / (conflicts * 2).max(1);
        for k in 0..3 {
            if p + k < lines {
                ours[p + k] = format!("    let ours_{c}_{k} = compute(input[{k}], {c}); // ours");
                theirs[p + k] =
                    format!("    let theirs_{c}_{k} = compute(input[{k}], {c}) * 2; // theirs");
            }
        }
    }
    for i in (53..lines).step_by(97) {
        if ours[i] == base[i] && theirs[i] == base[i] {
            theirs[i] =
                format!("    let value_{i} = compute(input[{i} % 7], {i}) - offset; // incoming");
        }
    }
    for i in (71..lines).step_by(131) {
        if ours[i] == base[i] && theirs[i] == base[i] {
            ours[i] =
                format!("    let value_{i} = compute(input[{i} % 7], {i}) * offset; // current");
        }
    }
    base.push(String::new());
    ours.push(String::new());
    theirs.push(String::new());
    [base.join("\n"), ours.join("\n"), theirs.join("\n")]
}

fn build_big_commits(dir: &Path, lines: usize, conflicts: usize) -> Result<(), String> {
    let [base, ours, theirs] = big_versions(lines, conflicts);
    write(dir, "src/big.rs", &base)?;
    git(dir, &["add", "."])?;
    git(dir, &["commit", "-q", "-m", "base"])?;
    git(dir, &["checkout", "-q", "-b", "feature"])?;
    write(dir, "src/big.rs", &theirs)?;
    git(dir, &["commit", "-q", "-am", "feature"])?;
    git(dir, &["checkout", "-q", "main"])?;
    write(dir, "src/big.rs", &ours)?;
    git(dir, &["commit", "-q", "-am", "main"])?;
    Ok(())
}

/// Builds the big fixture in `dir` (created) and leaves the merge
/// stopped on its conflicts. Used by `perf --tui`.
pub fn build_big(dir: &Path, lines: usize, conflicts: usize) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    git(dir, &["init", "-q", "-b", "main"])?;
    build_big_commits(dir, lines, conflicts)?;
    git(dir, &["merge", "feature"])?;
    Ok(())
}

/// Runs the task; returns the process exit code.
pub fn run(args: &[String]) -> Result<i32, String> {
    let mut kind = "text".to_string();
    let mut dir: Option<PathBuf> = None;
    let (mut lines, mut conflicts) = (10_000usize, 50usize);
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--kind" => kind = it.next().ok_or("--kind needs a value")?.clone(),
            "--lines" => lines = num(it.next(), "--lines")?,
            "--conflicts" => conflicts = num(it.next(), "--conflicts")?,
            "--dir" => dir = Some(PathBuf::from(it.next().ok_or("--dir needs a value")?)),
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(0);
            }
            other => return Err(format!("unknown option '{other}'\n\n{USAGE}")),
        }
    }
    let dir = dir.unwrap_or_else(|| {
        std::env::temp_dir().join(format!("gitpoulpi-merge-fixture-{}", std::process::id()))
    });
    if dir.exists()
        && std::fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .next()
            .is_some()
    {
        return Err(format!("{} is not empty", dir.display()));
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    git(&dir, &["init", "-q", "-b", "main"])?;
    match kind.as_str() {
        "text" => {
            write(&dir, "src/user.rs", BASE_RS)?;
            git(&dir, &["add", "."])?;
            git(&dir, &["commit", "-q", "-m", "base"])?;
            git(&dir, &["checkout", "-q", "-b", "feature"])?;
            write(&dir, "src/user.rs", &theirs_rs())?;
            git(
                &dir,
                &["commit", "-q", "-am", "feature: friendlier greeting"],
            )?;
            git(&dir, &["checkout", "-q", "main"])?;
            write(&dir, "src/user.rs", &ours_rs())?;
            git(&dir, &["commit", "-q", "-am", "main: welcome back"])?;
        }
        "big" => build_big_commits(&dir, lines, conflicts)?,
        "multi" => {
            write(&dir, "a.txt", &numbered(30, &[]))?;
            write(&dir, "b.txt", &numbered(10, &[]))?;
            write(&dir, "docs/c.txt", &numbered(10, &[]))?;
            git(&dir, &["add", "."])?;
            git(&dir, &["commit", "-q", "-m", "base"])?;
            git(&dir, &["checkout", "-q", "-b", "feature"])?;
            write(
                &dir,
                "a.txt",
                &numbered(
                    30,
                    &[
                        (3, "A3 theirs"),
                        (4, "A4 theirs"),
                        (15, "A15 theirs"),
                        (28, "A28 theirs"),
                    ],
                ),
            )?;
            write(&dir, "b.txt", &numbered(10, &[(5, "B5 theirs")]))?;
            write(&dir, "docs/c.txt", &numbered(10, &[(2, "C2 theirs")]))?;
            git(&dir, &["commit", "-q", "-am", "feature"])?;
            git(&dir, &["checkout", "-q", "main"])?;
            write(
                &dir,
                "a.txt",
                &numbered(
                    30,
                    &[
                        (3, "A3 ours"),
                        (4, "A4 ours"),
                        (15, "A15 ours"),
                        (28, "A28 ours"),
                    ],
                ),
            )?;
            write(&dir, "b.txt", &numbered(10, &[(5, "B5 ours")]))?;
            write(&dir, "docs/c.txt", &numbered(10, &[(2, "C2 ours")]))?;
            git(&dir, &["commit", "-q", "-am", "main"])?;
        }
        other => return Err(format!("unknown kind '{other}'\n\n{USAGE}")),
    }
    // The merge stops on its conflicts: that is the point.
    git(&dir, &["merge", "feature"])?;
    let status = git(&dir, &["status", "--short"])?;
    println!("repository: {}", dir.display());
    println!("{status}");
    println!(
        "open it (from that directory, with a throwaway config):\n  \
         GITPOULPI_CONFIG_DIR={cfg} <repo>/target/debug/gitpoulpi-tui -s merge-fixture\n\
         then Enter on a `!` row of the Status pane (or the banner's Resolve conflicts).",
        cfg = dir.with_extension("config").display()
    );
    Ok(0)
}
