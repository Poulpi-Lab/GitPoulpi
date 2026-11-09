//! `cargo run -p xtask -- advisories [--json|--markdown] [--no-paths]`: asks
//! OSV (RustSec and GitHub advisories) about every crates.io crate of
//! `Cargo.lock`, drops what `deny.toml`'s `[advisories] ignore` already
//! accepts (by id or alias) and exits non-zero when anything remains.
//!
//! CI is disabled and `cargo-deny` runs only at release time, so the scheduled
//! workflow `.github/workflows/advisories.yml` runs this daily and keeps one
//! issue in sync with the result.
//!
//! The logic is pure over JSON values and strings; only [`osv_post`] and
//! [`osv_get`] touch the network, and the tests feed fixture JSON instead.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;
use std::time::Duration;

use serde_json::{json, Value};

use crate::lock_age::{registry_entries, Entry};

const HELP: &str = "\
Usage: cargo run -p xtask -- advisories [--json | --markdown] [--no-paths]
                                        [--lock <file>] [--deny <file>]

Queries https://api.osv.dev for every crates.io crate of Cargo.lock, drops the
ids (or aliases) listed in deny.toml [advisories] ignore, prints what is left
and exits 1 when anything is. --json and --markdown change the output format
(markdown is an issue body, empty when clean); --no-paths skips the
`cargo tree -i` dependency path of each hit.";

/// OSV accepts at most 1000 queries per `querybatch` call.
const BATCH: usize = 1000;

/// One open advisory on one locked crate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub name: String,
    pub version: String,
    pub id: String,
    pub aliases: Vec<String>,
    pub summary: String,
    pub fixed: Vec<String>,
    pub path: String,
}

/// The `querybatch` request body for `entries`.
pub fn batch_body(entries: &[Entry]) -> Value {
    let queries: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({"package": {"name": e.name, "ecosystem": "crates.io"}, "version": e.version})
        })
        .collect();
    json!({ "queries": queries })
}

/// The advisory ids per query of a `querybatch` answer, in request order.
pub fn parse_batch(answer: &Value, queries: usize) -> Result<Vec<Vec<String>>, String> {
    let results = answer["results"]
        .as_array()
        .ok_or("OSV answer has no `results` list")?;
    if results.len() != queries {
        return Err(format!(
            "OSV answered {} result(s) for {queries} queries",
            results.len()
        ));
    }
    Ok(results
        .iter()
        .map(|r| {
            r["vulns"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v["id"].as_str().map(str::to_string))
                .collect()
        })
        .collect())
}

/// The ids `deny.toml` ignores: `{ id = "...", reason = "..." }` tables or
/// bare strings of `[advisories] ignore`.
pub fn deny_ignores(deny: &str) -> Result<BTreeSet<String>, String> {
    let doc: toml::Value = toml::from_str(deny).map_err(|e| format!("deny.toml: {e}"))?;
    let list = doc
        .get("advisories")
        .and_then(|a| a.get("ignore"))
        .and_then(|i| i.as_array());
    Ok(list
        .into_iter()
        .flatten()
        .filter_map(|item| match item {
            toml::Value::String(s) => Some(s.clone()),
            toml::Value::Table(t) => t.get("id").and_then(|v| v.as_str()).map(str::to_string),
            _ => None,
        })
        .collect())
}

/// Versions that fix `vuln` for the crate `name`, sorted and unique.
pub fn fixed_versions(vuln: &Value, name: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for affected in vuln["affected"].as_array().into_iter().flatten() {
        if affected["package"]["name"].as_str() != Some(name) {
            continue;
        }
        for range in affected["ranges"].as_array().into_iter().flatten() {
            for event in range["events"].as_array().into_iter().flatten() {
                if let Some(v) = event["fixed"].as_str() {
                    out.insert(v.to_string());
                }
            }
        }
    }
    out.into_iter().collect()
}

/// Turns one OSV record into a finding, or `None` when it is withdrawn or
/// ignored by id or alias.
pub fn finding(
    entry: &Entry,
    vuln: &Value,
    ignored: &BTreeSet<String>,
    path: String,
) -> Option<Finding> {
    if vuln["withdrawn"].as_str().is_some() {
        return None;
    }
    let id = vuln["id"].as_str()?.to_string();
    let aliases: Vec<String> = vuln["aliases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|a| a.as_str().map(str::to_string))
        .collect();
    if ignored.contains(&id) || aliases.iter().any(|a| ignored.contains(a)) {
        return None;
    }
    let summary = vuln["summary"]
        .as_str()
        .or_else(|| vuln["details"].as_str().and_then(|d| d.lines().next()))
        .unwrap_or("(no summary)")
        .trim()
        .to_string();
    Some(Finding {
        name: entry.name.clone(),
        version: entry.version.clone(),
        fixed: fixed_versions(vuln, &entry.name),
        id,
        aliases,
        summary,
        path,
    })
}

fn fixed_text(f: &Finding) -> String {
    if f.fixed.is_empty() {
        "no fix".to_string()
    } else {
        f.fixed.join(", ")
    }
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

/// Plain-text table.
pub fn render_table(findings: &[Finding]) -> String {
    let rows: Vec<[String; 5]> = findings
        .iter()
        .map(|f| {
            [
                f.name.clone(),
                f.version.clone(),
                f.id.clone(),
                f.summary.clone(),
                fixed_text(f),
            ]
        })
        .collect();
    let head = ["crate", "version", "id", "summary", "fixed in"];
    let mut widths = head.map(str::len);
    for row in &rows {
        for (w, c) in widths.iter_mut().zip(row) {
            *w = (*w).max(c.chars().count());
        }
    }
    let line = |cols: &[String; 5]| {
        let mut s = String::new();
        for (i, c) in cols.iter().enumerate() {
            s.push_str(&format!("{c:<w$}  ", w = widths[i]));
        }
        s.trim_end().to_string()
    };
    let mut out = vec![line(&head.map(String::from))];
    for (row, f) in rows.iter().zip(findings) {
        out.push(line(row));
        if !f.path.is_empty() {
            out.push(format!("    via {}", f.path));
        }
    }
    out.join("\n")
}

/// Issue body.
pub fn render_markdown(findings: &[Finding]) -> String {
    let mut out = String::from(
        "`cargo run -p xtask -- advisories` found advisories on locked dependencies that \
         `deny.toml` does not accept. Upgrade the crate, or add an `ignore` entry with a reason.\n\n\
         | Crate | Version | Advisory | Summary | Fixed in | Via |\n|---|---|---|---|---|---|\n",
    );
    for f in findings {
        out.push_str(&format!(
            "| `{}` | {} | [{id}](https://osv.dev/vulnerability/{id}) | {} | {} | {} |\n",
            f.name,
            f.version,
            cell(&f.summary),
            cell(&fixed_text(f)),
            cell(&f.path),
            id = f.id,
        ));
    }
    out
}

/// JSON list.
pub fn render_json(findings: &[Finding]) -> String {
    let list: Vec<Value> = findings
        .iter()
        .map(|f| {
            json!({"crate": f.name, "version": f.version, "id": f.id, "aliases": f.aliases,
                   "summary": f.summary, "fixed": f.fixed, "path": f.path})
        })
        .collect();
    serde_json::to_string_pretty(&list).unwrap_or_default()
}

/// First dependency chain of `cargo tree -i --prefix depth` output, from the
/// crate up to a workspace member: `lru 0.12.5 <- ratatui 0.29.0 <- ...`.
pub fn first_path(tree: &str) -> String {
    let mut chain: Vec<String> = Vec::new();
    for line in tree.lines() {
        let digits = line.chars().take_while(char::is_ascii_digit).count();
        let Ok(depth) = line[..digits].parse::<usize>() else {
            continue;
        };
        if depth != chain.len() {
            break;
        }
        let text = line[digits..].split(" (").next().unwrap_or("").trim();
        chain.push(text.replace(" v", " "));
    }
    chain.join(" <- ")
}

fn dependency_path(entry: &Entry) -> String {
    let spec = format!("{}@{}", entry.name, entry.version);
    Command::new("cargo")
        .args([
            "tree",
            "-i",
            &spec,
            "-e",
            "normal,build",
            "--prefix",
            "depth",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| first_path(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default()
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(60))
        .user_agent("gitpoulpi-xtask advisories (+https://github.com/Poulpi-Lab/gitpoulpi)")
        .build()
}

fn osv_post(body: &Value) -> Result<Value, String> {
    agent()
        .post("https://api.osv.dev/v1/querybatch")
        .send_json(body.clone())
        .map_err(|e| format!("OSV querybatch failed: {e}"))?
        .into_json()
        .map_err(|e| format!("unreadable OSV answer: {e}"))
}

fn osv_get(id: &str) -> Result<Value, String> {
    agent()
        .get(&format!("https://api.osv.dev/v1/vulns/{id}"))
        .call()
        .map_err(|e| format!("OSV lookup of {id} failed: {e}"))?
        .into_json()
        .map_err(|e| format!("unreadable OSV answer for {id}: {e}"))
}

/// The scan over injected lookups: `batch` answers a `querybatch` body,
/// `detail` an advisory id, `path` a dependency chain.
pub fn scan(
    lock: &str,
    deny: &str,
    batch: &mut dyn FnMut(&Value) -> Result<Value, String>,
    detail: &mut dyn FnMut(&str) -> Result<Value, String>,
    path: &mut dyn FnMut(&Entry) -> String,
) -> Result<Vec<Finding>, String> {
    let entries: Vec<Entry> = registry_entries(lock)?.into_iter().collect();
    let ignored = deny_ignores(deny)?;
    let mut hits: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
    for chunk in entries.chunks(BATCH) {
        let answer = batch(&batch_body(chunk))?;
        for (entry, ids) in chunk.iter().zip(parse_batch(&answer, chunk.len())?) {
            for id in ids {
                hits.entry(id).or_default().push(entry);
            }
        }
    }
    let mut out = Vec::new();
    for (id, crates) in hits {
        let vuln = detail(&id)?;
        for entry in crates {
            if let Some(f) = finding(entry, &vuln, &ignored, String::new()) {
                out.push(Finding {
                    path: path(entry),
                    ..f
                });
            }
        }
    }
    out.sort();
    Ok(out)
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let (mut format, mut paths) = ("table", true);
    let (mut lock, mut deny) = ("Cargo.lock".to_string(), "deny.toml".to_string());
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return Ok(0);
            }
            "--json" => format = "json",
            "--markdown" => format = "markdown",
            "--no-paths" => paths = false,
            "--lock" => lock = it.next().cloned().ok_or("--lock needs a file")?,
            "--deny" => deny = it.next().cloned().ok_or("--deny needs a file")?,
            other => return Err(format!("unknown argument '{other}'\n\n{HELP}")),
        }
    }
    let lock_text = std::fs::read_to_string(&lock).map_err(|e| format!("{lock}: {e}"))?;
    let deny_text = std::fs::read_to_string(&deny).map_err(|e| format!("{deny}: {e}"))?;
    let findings = scan(
        &lock_text,
        &deny_text,
        &mut osv_post,
        &mut |id| osv_get(id),
        &mut |e| {
            if paths {
                dependency_path(e)
            } else {
                String::new()
            }
        },
    )?;
    match format {
        "json" => println!("{}", render_json(&findings)),
        "markdown" if !findings.is_empty() => print!("{}", render_markdown(&findings)),
        "markdown" => {}
        _ if findings.is_empty() => println!("advisories: no open advisory"),
        _ => println!("{}", render_table(&findings)),
    }
    if !findings.is_empty() {
        eprintln!("advisories: {} open advisory finding(s)", findings.len());
        return Ok(1);
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REG: &str = "registry+https://github.com/rust-lang/crates.io-index";

    fn lock() -> String {
        format!(
            "version = 4\n\n[[package]]\nname = \"lru\"\nversion = \"0.12.5\"\nsource = \"{REG}\"\n\n\
             [[package]]\nname = \"serde\"\nversion = \"1.0.0\"\nsource = \"{REG}\"\n\n\
             [[package]]\nname = \"local\"\nversion = \"0.1.0\"\n"
        )
    }

    fn vuln(id: &str, alias: &str) -> Value {
        json!({"id": id, "summary": "boom | bang", "aliases": [alias],
               "affected": [{"package": {"name": "lru"},
                   "ranges": [{"events": [{"introduced": "0"}, {"fixed": "0.16.3"}]}]},
                   {"package": {"name": "other"},
                   "ranges": [{"events": [{"fixed": "9.9.9"}]}]}]})
    }

    fn run_scan(deny: &str) -> Vec<Finding> {
        let mut batch = |body: &Value| -> Result<Value, String> {
            let queries = body["queries"].as_array().unwrap();
            assert_eq!(queries[0]["package"]["ecosystem"], "crates.io");
            assert_eq!(queries.len(), 2, "path crates are not queried");
            Ok(json!({"results": [{"vulns": [{"id": "RUSTSEC-1"}, {"id": "RUSTSEC-2"}]}, {}]}))
        };
        let mut detail =
            |id: &str| -> Result<Value, String> { Ok(vuln(id, &format!("GHSA-{id}"))) };
        scan(&lock(), deny, &mut batch, &mut detail, &mut |_| {
            "lru 0.12.5 <- ratatui 0.29.0".into()
        })
        .unwrap()
    }

    #[test]
    fn findings_carry_summary_fix_and_path() {
        let found = run_scan("[advisories]\nignore = []\n");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "lru");
        assert_eq!(found[0].fixed, vec!["0.16.3"]);
        assert_eq!(found[0].path, "lru 0.12.5 <- ratatui 0.29.0");
    }

    #[test]
    fn ignores_match_by_id_and_by_alias_in_both_shapes() {
        let deny =
            "[advisories]\nignore = [{ id = \"RUSTSEC-1\", reason = \"x\" }, \"GHSA-RUSTSEC-2\"]\n";
        assert!(run_scan(deny).is_empty());
        let one = run_scan("[advisories]\nignore = [{ id = \"RUSTSEC-1\", reason = \"x\" }]\n");
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].id, "RUSTSEC-2");
    }

    #[test]
    fn withdrawn_advisories_are_dropped() {
        let mut v = vuln("RUSTSEC-3", "GHSA-3");
        v["withdrawn"] = json!("2026-01-01T00:00:00Z");
        let e = Entry {
            name: "lru".into(),
            version: "0.12.5".into(),
        };
        assert!(finding(&e, &v, &BTreeSet::new(), String::new()).is_none());
    }

    #[test]
    fn batch_answer_length_must_match() {
        assert!(parse_batch(&json!({"results": [{}]}), 2).is_err());
        assert!(parse_batch(&json!({}), 1).is_err());
        let ids = parse_batch(&json!({"results": [{"vulns": [{"id": "A"}]}, {}]}), 2).unwrap();
        assert_eq!(ids, vec![vec!["A".to_string()], vec![]]);
    }

    #[test]
    fn renderers_escape_and_list_every_field() {
        let found = run_scan("");
        let md = render_markdown(&found);
        assert!(md.contains("boom \\| bang"));
        assert!(md.contains("https://osv.dev/vulnerability/RUSTSEC-1"));
        let json: Value = serde_json::from_str(&render_json(&found)).unwrap();
        assert_eq!(json[0]["crate"], "lru");
        assert_eq!(json[0]["fixed"][0], "0.16.3");
        assert!(render_table(&found).contains("via lru 0.12.5 <- ratatui 0.29.0"));
    }

    #[test]
    fn first_path_follows_the_first_chain() {
        let tree =
            "0lru v0.12.5\n1ratatui v0.29.0 (/w)\n2gitpoulpi-tui v0.1.0 (/w)\n1other v1.0.0\n";
        assert_eq!(
            first_path(tree),
            "lru 0.12.5 <- ratatui 0.29.0 <- gitpoulpi-tui 0.1.0"
        );
    }
}
