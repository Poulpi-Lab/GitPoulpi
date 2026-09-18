//! The About window's third-party crate list: every external crate a
//! workspace crate reachable from `gitpoulpi-tui` depends on directly
//! (`[dependencies]` and target-specific ones, not dev or build ones), with
//! the version `Cargo.lock` resolved, one `name version url license` per line
//! in `$OUT_DIR/third_party_crates.txt`. The license is the SPDX expression in
//! the crate's own `Cargo.toml`, read from the local cargo registry (no
//! network), `?` when it cannot be found. The url is the manifest's
//! `repository` when it is a web address, else the crate's crates.io page
//! (git sources use their own address, path sources `-`: no link).

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::path::Path;

pub fn run() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo");
    let root = Path::new(&manifest).join("../..");
    let lock_path = root.join("Cargo.lock");
    println!("cargo:rerun-if-changed={}", lock_path.display());
    let out =
        Path::new(&std::env::var("OUT_DIR").expect("set by cargo")).join("third_party_crates.txt");
    let list = std::fs::read_to_string(&lock_path)
        .ok()
        .and_then(|lock| collect(&root, &lock))
        .unwrap_or_default();
    std::fs::write(out, list).expect("OUT_DIR is writable");
}

struct Package {
    version: String,
    source: Option<String>,
    local: bool,
    deps: Vec<String>,
}

fn collect(root: &Path, lock: &str) -> Option<String> {
    let lock: toml::Value = lock.parse().ok()?;
    let mut packages: HashMap<String, Vec<Package>> = HashMap::new();
    for p in lock.get("package")?.as_array()? {
        let name = p.get("name")?.as_str()?.to_string();
        packages.entry(name).or_default().push(Package {
            version: p.get("version")?.as_str()?.to_string(),
            source: p.get("source").and_then(|v| v.as_str()).map(str::to_string),
            local: p.get("source").is_none(),
            deps: (p.get("dependencies").and_then(|d| d.as_array()))
                .map(|d| {
                    d.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
        });
    }
    let local = |name: &str| packages.get(name).and_then(|v| v.iter().find(|p| p.local));
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from(["gitpoulpi-tui".to_string()]);
    let mut out = BTreeSet::new();
    while let Some(member) = queue.pop_front() {
        if !seen.insert(member.clone()) {
            continue;
        }
        let Some(pkg) = local(&member) else { continue };
        let normal = normal_deps(&root.join("crates").join(&member).join("Cargo.toml"));
        for entry in &pkg.deps {
            let mut parts = entry.split(' ');
            let name = parts.next().unwrap_or_default();
            let version = parts.next();
            if !normal.contains(name) {
                continue;
            }
            if local(name).is_some() {
                queue.push_back(name.to_string());
                continue;
            }
            let version = version.map(str::to_string).or_else(|| {
                let all = packages.get(name)?;
                (all.len() == 1).then(|| all[0].version.clone())
            });
            if let Some(version) = version {
                let source = packages
                    .get(name)
                    .and_then(|v| v.iter().find(|p| p.version == version))
                    .and_then(|p| p.source.as_deref());
                let manifest = manifest_of(name, &version);
                let license = manifest
                    .as_deref()
                    .and_then(parse_license)
                    .unwrap_or_else(|| "?".to_string());
                let repository = manifest.as_deref().and_then(parse_repository);
                let url = link_for(repository.as_deref(), source, name, &version);
                out.insert(format!("{name} {version} {url} {license}"));
            }
        }
    }
    Some(out.into_iter().collect::<Vec<_>>().join("\n"))
}

/// The package names `manifest` depends on outside dev and build
/// dependencies (a `package = "..."` rename gives the real name).
fn normal_deps(manifest: &Path) -> BTreeSet<String> {
    println!("cargo:rerun-if-changed={}", manifest.display());
    let Some(doc) = std::fs::read_to_string(manifest)
        .ok()
        .and_then(|t| t.parse::<toml::Value>().ok())
    else {
        return BTreeSet::new();
    };
    let mut tables = vec![doc.get("dependencies")];
    if let Some(targets) = doc.get("target").and_then(|t| t.as_table()) {
        tables.extend(targets.values().map(|t| t.get("dependencies")));
    }
    let mut names = BTreeSet::new();
    for table in tables.into_iter().flatten().filter_map(|t| t.as_table()) {
        for (key, spec) in table {
            let real = spec.get("package").and_then(|p| p.as_str()).unwrap_or(key);
            names.insert(real.to_string());
        }
    }
    names
}

/// The directories holding unpacked registry sources
/// (`$CARGO_HOME/registry/src/*`), `CARGO_HOME` defaulting to `~/.cargo`
/// (`%USERPROFILE%\.cargo` on Windows).
fn registry_sources() -> Vec<std::path::PathBuf> {
    let home = std::env::var_os("CARGO_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(|h| std::path::PathBuf::from(h).join(".cargo"))
        });
    let Some(src) = home.map(|h| h.join("registry").join("src")) else {
        return Vec::new();
    };
    std::fs::read_dir(src)
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default()
}

/// The text of `name` `version`'s `Cargo.toml` in the local registry.
fn manifest_of(name: &str, version: &str) -> Option<String> {
    registry_sources()
        .iter()
        .map(|dir| dir.join(format!("{name}-{version}")).join("Cargo.toml"))
        .find_map(|manifest| {
            println!("cargo:rerun-if-changed={}", manifest.display());
            std::fs::read_to_string(manifest).ok()
        })
}

/// The `package.repository` of a manifest when it is a web address.
fn parse_repository(manifest: &str) -> Option<String> {
    let doc: toml::Value = manifest.parse().ok()?;
    let text = doc.get("package")?.get("repository")?.as_str()?.trim();
    (text.starts_with("https://") || text.starts_with("http://")).then(|| text.to_string())
}

/// The address a crate's row links to: its repository, else its crates.io
/// page; a git source is its own address, anything else (a path) has none (`-`).
fn link_for(repository: Option<&str>, source: Option<&str>, name: &str, version: &str) -> String {
    if let Some(repository) = repository.filter(|r| !r.contains(char::is_whitespace)) {
        return repository.to_string();
    }
    match source {
        Some(s) if s.starts_with("git+") => {
            let url = s.trim_start_matches("git+");
            let end = url.find(['?', '#']).unwrap_or(url.len());
            url[..end].to_string()
        }
        Some(_) => format!("https://crates.io/crates/{name}/{version}"),
        None => "-".to_string(),
    }
}

/// The `package.license` of a manifest, whitespace collapsed to single
/// spaces (a few crates break the expression over lines).
fn parse_license(manifest: &str) -> Option<String> {
    let doc: toml::Value = manifest.parse().ok()?;
    let text = doc.get("package")?.get("license")?.as_str()?;
    let one_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!one_line.is_empty()).then_some(one_line)
}

#[cfg(test)]
mod tests {
    use super::{link_for, parse_license, parse_repository};

    #[test]
    fn a_crate_links_to_its_repository_else_crates_io() {
        let reg = Some("registry+https://github.com/rust-lang/crates.io-index");
        assert_eq!(
            link_for(Some("https://github.com/a/b"), reg, "b", "1.0.0"),
            "https://github.com/a/b"
        );
        assert_eq!(
            link_for(None, reg, "b", "1.0.0"),
            "https://crates.io/crates/b/1.0.0"
        );
        assert_eq!(
            link_for(
                None,
                Some("git+https://example.test/x?rev=1#abc"),
                "x",
                "1.0.0"
            ),
            "https://example.test/x"
        );
        assert_eq!(link_for(None, None, "x", "1.0.0"), "-");
        let manifest = "[package]\nrepository = \"https://github.com/a/b\"\n";
        assert_eq!(
            parse_repository(manifest).as_deref(),
            Some("https://github.com/a/b")
        );
        assert_eq!(
            parse_repository("[package]\nrepository = \"git@x:y\"\n"),
            None
        );
    }

    #[test]
    fn reads_the_spdx_expression_of_a_manifest() {
        let manifest = "[package]\nname = \"x\"\nlicense = \"MIT OR\\n  Apache-2.0\"\n";
        assert_eq!(
            parse_license(manifest).as_deref(),
            Some("MIT OR Apache-2.0")
        );
        assert_eq!(parse_license("[package]\nname = \"x\"\n"), None);
        assert_eq!(parse_license("not toml ["), None);
    }
}
