//! What crates.io needs of the workspace, checked offline from `cargo metadata`
//! and the files: which crates are published, that the versions between them
//! are consistent, and that the copies of files a package embeds (a package
//! cannot read outside its own directory) match their originals.
//!
//! The authoritative check is `cargo publish --workspace --dry-run` (see
//! `docs/developer/releasing.md`); this keeps the common mistakes from reaching it.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Never published: the dev tasks, the test kit, the elevated install helper.
const UNPUBLISHED: &[&str] = &["xtask", "gitpoulpi-testkit", "gitpoulpi-updater"];

fn packages() -> Vec<Value> {
    let out = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(root())
        .output()
        .expect("cargo metadata runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    v["packages"].as_array().unwrap().clone()
}

fn is_published(p: &Value) -> bool {
    // `publish = false` is `[]` in the metadata, absent is `null`.
    p["publish"].as_array().is_none_or(|l| !l.is_empty())
}

fn name(p: &Value) -> &str {
    p["name"].as_str().unwrap()
}

#[test]
fn exactly_the_internal_crates_are_unpublished() {
    for p in packages() {
        assert_eq!(
            !is_published(&p),
            UNPUBLISHED.contains(&name(&p)),
            "{}: `publish = false` must be set on exactly {UNPUBLISHED:?}",
            name(&p)
        );
    }
}

#[test]
fn a_published_crate_has_the_metadata_crates_io_asks_for() {
    for p in packages().iter().filter(|p| is_published(p)) {
        for field in [
            "description",
            "license",
            "repository",
            "homepage",
            "readme",
            "rust_version",
        ] {
            assert!(
                p[field].as_str().is_some_and(|s| !s.is_empty()),
                "{}: `{field}` is missing",
                name(p)
            );
        }
        for field in ["keywords", "categories"] {
            assert!(
                p[field].as_array().is_some_and(|l| !l.is_empty()),
                "{}: `{field}` is missing",
                name(p)
            );
        }
    }
}

#[test]
fn published_crates_depend_on_published_crates_at_the_workspace_version() {
    let all = packages();
    let published: Vec<&str> = all.iter().filter(|p| is_published(p)).map(name).collect();
    for p in all.iter().filter(|p| is_published(p)) {
        let version = p["version"].as_str().unwrap();
        for d in p["dependencies"].as_array().unwrap() {
            let dep = d["name"].as_str().unwrap();
            if d["path"].is_null() {
                continue;
            }
            let req = d["req"].as_str().unwrap();
            if d["kind"].as_str() == Some("dev") && req == "*" {
                // A path dev-dependency without a version is dropped from
                // the published manifest.
                continue;
            }
            assert!(
                published.contains(&dep),
                "{}: depends on the unpublished {dep} (a dev-dependency on it must have no version)",
                name(p)
            );
            assert!(
                req.contains(version),
                "{}: its dependency {dep} asks `{req}`, the workspace version is {version}: update `[workspace.dependencies]`",
                name(p)
            );
        }
    }
}

#[test]
fn no_build_script_reads_outside_its_crate() {
    for p in packages().iter().filter(|p| is_published(p)) {
        let dir = Path::new(p["manifest_path"].as_str().unwrap())
            .parent()
            .unwrap()
            .to_path_buf();
        let build = dir.join("build.rs");
        if let Ok(text) = std::fs::read_to_string(&build) {
            assert!(
                !text.contains("\"../../") && !text.contains("\"../.."),
                "{}: build.rs reaches outside the crate",
                name(p)
            );
        }
    }
}

/// The three files a package embeds from the repository root. The original is
/// the one people edit; the copy is what `cargo package` carries.
#[test]
fn the_embedded_copies_match_their_originals() {
    for (original, copy) in [
        ("CHANGELOG.md", "crates/gitpoulpi-tui/CHANGELOG.md"),
        (
            "THIRD-PARTY-LICENSES.txt",
            "crates/gitpoulpi-core/THIRD-PARTY-LICENSES.txt",
        ),
        (
            "packaging/minisign.pub",
            "crates/gitpoulpi-update/minisign.pub",
        ),
    ] {
        let a = std::fs::read(root().join(original)).unwrap();
        let b = std::fs::read(root().join(copy)).unwrap();
        assert!(
            a == b,
            "{copy} differs from {original}: copy it again (`cp {original} {copy}`; `cargo run -p xtask -- licenses` writes both license files)"
        );
    }
}

/// `scripts/publish-crates.sh` lists every published crate, each after the crates it
/// depends on (a version-less path dev-dependency does not count: it is dropped).
#[test]
fn the_publish_script_lists_the_crates_in_dependency_order() {
    let script = std::fs::read_to_string(root().join("scripts/publish-crates.sh")).unwrap();
    let list = script
        .split("CRATES=(")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .expect("a CRATES=( ... ) list");
    let order: Vec<&str> = list.split_whitespace().collect();

    let all = packages();
    let mut published: Vec<&str> = all.iter().filter(|p| is_published(p)).map(name).collect();
    let mut listed = order.clone();
    published.sort_unstable();
    listed.sort_unstable();
    assert_eq!(
        listed, published,
        "the script's list and the published crates differ"
    );

    for p in all.iter().filter(|p| is_published(p)) {
        let at = order.iter().position(|n| *n == name(p)).unwrap();
        for d in p["dependencies"].as_array().unwrap() {
            let dep = d["name"].as_str().unwrap();
            if d["path"].is_null() || (d["kind"].as_str() == Some("dev") && d["req"] == "*") {
                continue;
            }
            let dep_at = order.iter().position(|n| *n == dep).unwrap();
            assert!(dep_at < at, "{dep} must be published before {}", name(p));
        }
    }
}
