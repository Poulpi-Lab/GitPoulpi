//! The failures file of `xtask test`: what failed in the last run, so
//! `xtask test --failed` can rerun only that.
//!
//! `target/xtask-test-failures.tsv` has one row per failing test binary:
//! `package<TAB>target<TAB>kind<TAB>test name<TAB>test name...`. A binary that
//! failed without a parseable `test x ... FAILED` line (a crash, a timeout, a
//! build error) has no test names and is rerun whole. The doc-test job is the
//! row `-<TAB>doc<TAB>doc`. A green run leaves an empty file, so "last run was
//! green" differs from "no run yet" (no file).

use std::path::{Path, PathBuf};

/// A cargo target: enough to build it again (`-p package --test name`...).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TargetId {
    pub package: String,
    pub name: String,
    /// cargo's first target kind: `test`, `lib`, `bin`, or `doc`.
    pub kind: String,
}

impl TargetId {
    pub fn doc() -> Self {
        TargetId {
            package: "-".into(),
            name: "doc".into(),
            kind: "doc".into(),
        }
    }

    pub fn is_doc(&self) -> bool {
        self.kind == "doc"
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub target: TargetId,
    /// Failing test names; empty = rerun the whole binary.
    pub tests: Vec<String>,
}

pub fn path() -> PathBuf {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target"));
    target.join("xtask-test-failures.tsv")
}

/// Names of the tests libtest reported as `test <name> ... FAILED`, in order,
/// without duplicates.
pub fn failed_test_names(output: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in output.lines() {
        let Some(rest) = line.strip_prefix("test ") else {
            continue;
        };
        let Some(name) = rest.strip_suffix(" ... FAILED") else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty() && !names.iter().any(|n| n == name) {
            names.push(name.to_string());
        }
    }
    names
}

/// The package name of a cargo `package_id` (both the old
/// `name 0.1.0 (path+file://...)` and the new `path+file://...#name@0.1.0`
/// spellings); `dir_name` when the id only carries a version.
pub fn package_name(package_id: &str, dir_name: &str) -> String {
    if let Some((name, _)) = package_id.split_once(' ') {
        return name.to_string();
    }
    match package_id.rsplit_once('#') {
        Some((_, frag)) => match frag.split_once('@') {
            Some((name, _)) => name.to_string(),
            None => dir_name.to_string(),
        },
        None => dir_name.to_string(),
    }
}

pub fn format(failures: &[Failure]) -> String {
    let mut rows: Vec<&Failure> = failures.iter().collect();
    rows.sort_by(|a, b| a.target.cmp(&b.target));
    let mut s = String::new();
    for f in rows {
        s.push_str(&format!(
            "{}\t{}\t{}",
            f.target.package, f.target.name, f.target.kind
        ));
        for t in &f.tests {
            s.push('\t');
            s.push_str(t);
        }
        s.push('\n');
    }
    s
}

pub fn parse(text: &str) -> Vec<Failure> {
    let mut out = Vec::new();
    for line in text.lines() {
        let mut p = line.split('\t');
        let (Some(package), Some(name), Some(kind)) = (p.next(), p.next(), p.next()) else {
            continue;
        };
        out.push(Failure {
            target: TargetId {
                package: package.into(),
                name: name.into(),
                kind: kind.into(),
            },
            tests: p.filter(|t| !t.is_empty()).map(String::from).collect(),
        });
    }
    out
}

/// Writes through a temporary file and a rename, so a reader never sees half
/// a file.
pub fn write(path: &Path, failures: &[Failure]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }
    let tmp = path.with_extension("tsv.tmp");
    std::fs::write(&tmp, format(failures))?;
    std::fs::rename(&tmp, path)
}

/// `None` when there is no file (no run yet).
pub fn read(path: &Path) -> Option<Vec<Failure>> {
    std::fs::read_to_string(path).ok().map(|t| parse(&t))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(p: &str, n: &str, k: &str) -> TargetId {
        TargetId {
            package: p.into(),
            name: n.into(),
            kind: k.into(),
        }
    }

    #[test]
    fn parses_failed_names_with_module_paths() {
        let out = "running 3 tests\n\
test a::b::works ... ok\n\
test a::b::broken ... FAILED\n\
test top ... FAILED\n\
test a::b::broken ... FAILED\n\
\nfailures:\n    a::b::broken\n    top\n\ntest result: FAILED. 1 passed; 2 failed\n";
        assert_eq!(failed_test_names(out), ["a::b::broken", "top"]);
    }

    #[test]
    fn a_panicking_binary_has_no_names() {
        let out = "running 1 test\nthread 'main' panicked at x.rs:1\nSIGABRT\n";
        assert!(failed_test_names(out).is_empty());
        let green = "test a ... ok\ntest b ... ignored\ntest result: ok. 1 passed\n";
        assert!(failed_test_names(green).is_empty());
    }

    #[test]
    fn doc_test_names_keep_their_spaces() {
        let out = "test src/lib.rs - foo::bar (line 12) ... FAILED\n";
        assert_eq!(failed_test_names(out), ["src/lib.rs - foo::bar (line 12)"]);
    }

    #[test]
    fn package_names_from_both_id_spellings() {
        assert_eq!(package_name("foo 0.1.0 (path+file:///w/foo)", "d"), "foo");
        assert_eq!(
            package_name("path+file:///w/crates/foo#foo@0.1.0", "d"),
            "foo"
        );
        assert_eq!(
            package_name("path+file:///w/crates/foo#0.1.0", "foo"),
            "foo"
        );
    }

    #[test]
    fn file_round_trips_and_green_is_empty() {
        let f = vec![
            Failure {
                target: id("b", "pty_x", "test"),
                tests: vec!["m::t1".into(), "t2".into()],
            },
            Failure {
                target: id("a", "a", "lib"),
                tests: vec![],
            },
            Failure {
                target: TargetId::doc(),
                tests: vec!["src/lib.rs - f (line 1)".into()],
            },
        ];
        let text = format(&f);
        let back = parse(&text);
        assert_eq!(back.len(), 3);
        assert_eq!(back[0].target, id("-", "doc", "doc"));
        assert_eq!(back[1].tests, Vec::<String>::new());
        assert_eq!(back[2].tests, ["m::t1", "t2"]);
        assert_eq!(format(&[]), "");
        assert!(parse("").is_empty());
    }

    #[test]
    fn write_is_atomic_and_distinguishes_missing_from_green() {
        let dir = std::env::temp_dir().join(format!("xtask-failures-{}", std::process::id()));
        let p = dir.join("sub").join("f.tsv");
        assert!(read(&p).is_none());
        write(&p, &[]).unwrap();
        assert_eq!(read(&p), Some(vec![]));
        let f = vec![Failure {
            target: id("a", "t", "test"),
            tests: vec!["x".into()],
        }];
        write(&p, &f).unwrap();
        assert_eq!(read(&p), Some(f));
        assert!(!p.with_extension("tsv.tmp").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
