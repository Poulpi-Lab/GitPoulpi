//! `cargo run -p xtask -- licenses`: writes `THIRD-PARTY-LICENSES.txt`, the
//! notices that ship with every binary (archives, installers, the About
//! window's Licenses view, `gitpoulpi licenses`).
//!
//! Offline by construction: `cargo metadata --offline` for the crate graph,
//! the license files read from the crates' sources in the local cargo
//! registry. No network and no extra global tool (cargo-about, cargo-deny).
//!
//! Layout of the output: a hand-written header and credits
//! (`packaging/notices/*.txt`), the native code and data bundled inside
//! dependencies (libgit2's `COPYING` is read from the `libgit2-sys` source, so
//! it always matches the version that is linked), one summary line per crate,
//! then every distinct license text once with the crates that carry it.
//!
//! The file starts with a fingerprint of `Cargo.lock`; a test fails when the
//! lock file changes without a regeneration (`licenses --check` does the same
//! from the command line).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use sha2::{Digest, Sha256};

/// The committed output, at the repository root.
pub const OUTPUT: &str = "THIRD-PARTY-LICENSES.txt";
/// The prefix of the fingerprint line.
pub const FINGERPRINT_PREFIX: &str = "Cargo.lock sha256: ";
/// Widest line of the output: the About view shows it unwrapped.
const WIDTH: usize = 92;

const HELP: &str = "\
Usage: cargo run -p xtask -- licenses [--check]

Regenerates THIRD-PARTY-LICENSES.txt from Cargo.lock and the license files in
the local cargo registry (offline). With --check, only verifies that the
committed file matches the current Cargo.lock.";

pub fn run(args: &[String]) -> Result<i32, String> {
    let root = repo_root();
    match args {
        [] => {
            let text = generate(&root)?;
            std::fs::write(root.join(OUTPUT), text).map_err(|e| e.to_string())?;
            println!("wrote {OUTPUT}");
            Ok(0)
        }
        [flag] if flag == "--check" => {
            if is_fresh(&root)? {
                println!("{OUTPUT} is up to date");
                Ok(0)
            } else {
                eprintln!("{OUTPUT} is stale: run `cargo run -p xtask -- licenses`");
                Ok(1)
            }
        }
        [flag] if flag == "-h" || flag == "--help" => {
            println!("{HELP}");
            Ok(0)
        }
        _ => Err(format!("unexpected arguments\n\n{HELP}")),
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// SHA-256 of `Cargo.lock` with line endings normalized (a Windows checkout
/// with CRLF must give the same value).
pub fn lock_fingerprint(root: &Path) -> Result<String, String> {
    let lock = std::fs::read_to_string(root.join("Cargo.lock"))
        .map_err(|e| format!("cannot read Cargo.lock: {e}"))?;
    let normalized = lock.replace("\r\n", "\n");
    Ok(Sha256::digest(normalized.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// The fingerprint a committed notices file records.
pub fn recorded_fingerprint(notices: &str) -> Option<&str> {
    notices
        .lines()
        .find_map(|l| l.strip_prefix(FINGERPRINT_PREFIX))
        .map(str::trim)
}

/// Whether the committed file was generated from the current `Cargo.lock`.
pub fn is_fresh(root: &Path) -> Result<bool, String> {
    let notices = std::fs::read_to_string(root.join(OUTPUT))
        .map_err(|e| format!("cannot read {OUTPUT}: {e}"))?;
    Ok(recorded_fingerprint(&notices) == Some(lock_fingerprint(root)?.as_str()))
}

struct Krate {
    name: String,
    version: String,
    license: String,
    dir: PathBuf,
}

fn metadata(root: &Path) -> Result<Value, String> {
    let out = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["metadata", "--format-version", "1", "--offline"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cannot run cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())
}

/// Every external crate a shipped binary can contain: the normal-dependency
/// closure of the workspace members (xtask excluded) over all targets.
fn shipped_crates(meta: &Value) -> Result<Vec<Krate>, String> {
    let packages = meta["packages"].as_array().ok_or("no packages")?;
    let by_id: BTreeMap<&str, &Value> = packages
        .iter()
        .filter_map(|p| Some((p["id"].as_str()?, p)))
        .collect();
    let members: BTreeSet<&str> = meta["workspace_members"]
        .as_array()
        .ok_or("no workspace_members")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let nodes: BTreeMap<&str, &Value> = meta["resolve"]["nodes"]
        .as_array()
        .ok_or("no resolve graph")?
        .iter()
        .filter_map(|n| Some((n["id"].as_str()?, n)))
        .collect();
    let mut queue: Vec<&str> = members
        .iter()
        .copied()
        .filter(|id| by_id.get(id).is_some_and(|p| p["name"] != "xtask"))
        .collect();
    let mut seen: BTreeSet<&str> = queue.iter().copied().collect();
    while let Some(id) = queue.pop() {
        let Some(node) = nodes.get(id) else { continue };
        for dep in node["deps"].as_array().into_iter().flatten() {
            let normal = dep["dep_kinds"]
                .as_array()
                .is_some_and(|k| k.iter().any(|k| k["kind"].is_null()));
            let Some(dep_id) = dep["pkg"].as_str() else {
                continue;
            };
            if normal && seen.insert(dep_id) {
                queue.push(dep_id);
            }
        }
    }
    let mut out = Vec::new();
    for id in seen.difference(&members) {
        let p = by_id
            .get(id)
            .ok_or_else(|| format!("unknown package {id}"))?;
        let manifest = PathBuf::from(p["manifest_path"].as_str().unwrap_or_default());
        out.push(Krate {
            name: p["name"].as_str().unwrap_or_default().to_string(),
            version: p["version"].as_str().unwrap_or_default().to_string(),
            license: p["license"].as_str().unwrap_or("UNKNOWN").to_string(),
            dir: manifest.parent().map(Path::to_path_buf).unwrap_or_default(),
        });
    }
    out.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));
    Ok(out)
}

/// The license files at the top of a crate's source: `LICENSE*`, `LICENCE*`,
/// `COPYING*`, `UNLICENSE*`, `NOTICE*`, `COPYRIGHT*`.
fn license_files(dir: &Path) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let lower = name.to_lowercase();
        let wanted = [
            "license",
            "licence",
            "copying",
            "unlicense",
            "notice",
            "copyright",
        ]
        .iter()
        .any(|p| lower.starts_with(p));
        if wanted && entry.path().is_file() {
            if let Ok(text) = std::fs::read_to_string(entry.path()) {
                found.push((name, text));
            }
        }
    }
    found.sort();
    found
}

/// A text as the dedupe key: line endings and trailing blanks do not make a
/// license different.
fn normalize(text: &str) -> String {
    let text = text
        .replace("\r\n", "\n")
        .replace('\t', "    ")
        .replace('\u{2014}', "-");
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    lines.join("\n").trim().to_string()
}

/// Breaks lines longer than `WIDTH` at spaces (the license text is the same
/// words; the About view paints it without wrapping).
fn wrap_long(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if line.chars().count() <= WIDTH {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let indent: String = line.chars().take_while(|c| *c == ' ').collect();
        let mut cur = String::new();
        for word in line.split_whitespace() {
            let room = cur.chars().count() + 1 + word.chars().count();
            if !cur.is_empty() && room > WIDTH {
                out.push_str(&cur);
                out.push('\n');
                cur = indent.clone();
            }
            if !cur.trim().is_empty() {
                cur.push(' ');
            } else if cur.is_empty() {
                cur.push_str(&indent);
            }
            cur.push_str(word);
        }
        out.push_str(&cur);
        out.push('\n');
    }
    out
}

fn manual(root: &Path, file: &str) -> Result<String, String> {
    std::fs::read_to_string(root.join("packaging/notices").join(file))
        .map(|t| t.replace("\r\n", "\n"))
        .map_err(|e| format!("cannot read packaging/notices/{file}: {e}"))
}

pub fn generate(root: &Path) -> Result<String, String> {
    let meta = metadata(root)?;
    let crates = shipped_crates(&meta)?;
    let mut out = String::new();
    out.push_str(&manual(root, "header.txt")?);
    let _ = writeln!(out, "\n{FINGERPRINT_PREFIX}{}\n", lock_fingerprint(root)?);

    section(&mut out, "1. CREDITS: COLOR THEMES AND PALETTES");
    out.push_str(&manual(root, "credits.txt")?);

    section(&mut out, "2. CODE AND DATA BUNDLED INSIDE DEPENDENCIES");
    out.push_str(&manual(root, "bundled.txt")?);
    if let Some(libgit2) = crates.iter().find(|c| c.name == "libgit2-sys") {
        let copying = std::fs::read_to_string(libgit2.dir.join("libgit2/COPYING"))
            .map_err(|e| format!("cannot read libgit2's COPYING: {e}"))?;
        let _ = writeln!(
            out,
            "\nlibgit2 {} (bundled in libgit2-sys {}): COPYING, verbatim\n{}\n",
            libgit2.version.split('+').nth(1).unwrap_or("?"),
            libgit2.version,
            "-".repeat(WIDTH)
        );
        out.push_str(&wrap_long(&normalize(&copying)));
        out.push('\n');
    } else {
        return Err("libgit2-sys is not in the dependency graph".into());
    }
    if let Some(opt) = crates.iter().find(|c| c.license.contains("MPL-2.0")) {
        out.push('\n');
        out.push_str(
            &manual(root, "mpl.txt")?.replace("{crate}", &format!("{} {}", opt.name, opt.version)),
        );
    }

    section(&mut out, "3. RUST CRATES");
    let _ = writeln!(
        out,
        "{} crates, one per line: name version (license).\n",
        crates.len()
    );
    for c in &crates {
        let _ = writeln!(out, "{} {} ({})", c.name, c.version, c.license);
    }

    section(&mut out, "4. LICENSE TEXTS");
    out.push_str(
        "Each distinct text appears once, under the crates that carry it. A crate with\n\
         several license files appears under each; where its license says \"OR\", you may\n\
         use it under any one of them.\n",
    );
    let mut groups: BTreeMap<String, (Vec<String>, BTreeSet<String>)> = BTreeMap::new();
    let mut by_license: BTreeMap<String, String> = BTreeMap::new();
    let mut missing = Vec::new();
    for c in &crates {
        let files = license_files(&c.dir);
        if files.is_empty() {
            missing.push(c);
            continue;
        }
        by_license
            .entry(c.license.clone())
            .or_insert_with(|| normalize(&files[0].1));
        for (_, text) in files {
            let entry = groups.entry(normalize(&text)).or_default();
            entry.0.push(format!("{} {}", c.name, c.version));
            entry.1.insert(c.license.clone());
        }
    }
    for c in missing {
        let alternatives = ["MIT OR Apache-2.0", "Apache-2.0", "MIT"];
        let Some(text) = by_license
            .get(&c.license)
            .or_else(|| {
                alternatives
                    .iter()
                    .filter(|a| c.license.contains(*a))
                    .find_map(|a| by_license.get(*a))
            })
            .cloned()
        else {
            return Err(format!(
                "{} {} ({}) ships no license file and no other crate with the same license gives a text",
                c.name, c.version, c.license
            ));
        };
        let entry = groups.entry(text).or_default();
        entry.0.push(format!(
            "{} {} (its package has no license file: the same license's standard text)",
            c.name, c.version
        ));
        entry.1.insert(c.license.clone());
    }
    let mut ordered: Vec<_> = groups.into_iter().collect();
    ordered.sort_by(|a, b| a.1 .0[0].cmp(&b.1 .0[0]));
    for (text, (names, _)) in ordered {
        let _ = writeln!(out, "\n{}", "=".repeat(WIDTH));
        out.push_str(&wrap_long(&format!("Used by: {}", names.join(", "))));
        let _ = writeln!(out, "{}", "-".repeat(WIDTH));
        out.push_str(&wrap_long(&text));
    }
    Ok(out)
}

fn section(out: &mut String, title: &str) {
    let _ = writeln!(
        out,
        "\n\n{}\n{title}\n{}\n",
        "#".repeat(WIDTH),
        "#".repeat(WIDTH)
    );
}
