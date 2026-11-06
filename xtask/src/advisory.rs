//! `cargo run -p xtask -- advisory add|check`: edits and validates
//! `security/advisories.json`, the source of truth of the security notices
//! published next to each release (`security.json`, see
//! `.github/workflows/security-advisory.yml`).
//!
//! An entry: `{id: "GPSA-YYYY-NNN", affected: ">=1.2.0, <1.4.3", fixed,
//! severity: low|medium|high|critical, url, summary}`. `check` requires valid
//! semver for `affected` and `fixed`, unique ids of the right shape and an
//! https url.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::{json, Value};

pub const DEFAULT_FILE: &str = "security/advisories.json";
const SEVERITIES: [&str; 4] = ["low", "medium", "high", "critical"];

const HELP: &str = "\
Usage: cargo run -p xtask -- advisory check [--file <path>]
       cargo run -p xtask -- advisory add --affected <range> --severity <s> --url <https url>
                                          --summary <text> [--fixed <version>] [--id <id>] [--file <path>]

check validates security/advisories.json (semver ranges, unique ids
GPSA-YYYY-NNN, https urls). add appends an entry; without --id the next free
GPSA-<year>-NNN is used. severity: low, medium, high or critical.";

/// `GPSA-YYYY-NNN`: four digits, then at least three.
pub fn valid_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("GPSA-") else {
        return false;
    };
    let Some((year, number)) = rest.split_once('-') else {
        return false;
    };
    year.len() == 4
        && year.bytes().all(|b| b.is_ascii_digit())
        && number.len() >= 3
        && number.bytes().all(|b| b.is_ascii_digit())
}

/// Every problem of a `advisories.json` text, empty when it is valid.
pub fn problems(text: &str) -> Vec<String> {
    let value: Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => return vec![format!("not valid JSON: {e}")],
    };
    let Some(list) = value.as_array() else {
        return vec!["the file must be a JSON list".to_string()];
    };
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, item) in list.iter().enumerate() {
        let field = |key: &str| item.get(key).and_then(Value::as_str);
        let label = field("id").map_or_else(|| format!("entry {i}"), str::to_string);
        match field("id") {
            Some(id) if valid_id(id) => {
                if !seen.insert(id.to_string()) {
                    out.push(format!("{label}: duplicate id"));
                }
            }
            _ => out.push(format!("{label}: id must look like GPSA-YYYY-NNN")),
        }
        match field("affected") {
            Some(range) if semver::VersionReq::parse(range).is_ok() => {}
            _ => out.push(format!(
                "{label}: affected must be a semver range such as \">=1.2.0, <1.4.3\""
            )),
        }
        match item.get("fixed") {
            None | Some(Value::Null) => {}
            Some(Value::String(v)) if semver::Version::parse(v).is_ok() => {}
            _ => out.push(format!("{label}: fixed must be a semver version")),
        }
        match field("severity") {
            Some(s) if SEVERITIES.contains(&s) => {}
            _ => out.push(format!(
                "{label}: severity must be low, medium, high or critical"
            )),
        }
        match field("url") {
            Some(u)
                if u.starts_with("https://")
                    && u.len() > "https://".len()
                    && !u.contains(char::is_whitespace) => {}
            _ => out.push(format!("{label}: url must be an https url")),
        }
        if !field("summary").is_some_and(|s| !s.trim().is_empty()) {
            out.push(format!("{label}: summary must not be empty"));
        }
    }
    out
}

/// The next free id of `year` in an already valid list.
pub fn next_id(list: &[Value], year: i64) -> String {
    let prefix = format!("GPSA-{year}-");
    let highest = list
        .iter()
        .filter_map(|v| v.get("id").and_then(Value::as_str))
        .filter_map(|id| id.strip_prefix(&prefix))
        .filter_map(|n| n.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("{prefix}{:03}", highest + 1)
}

/// Appends an entry and returns the new file text, or the problems.
pub fn add_entry(text: &str, entry: Value) -> Result<String, String> {
    let issues = problems(text);
    if !issues.is_empty() {
        return Err(format!(
            "the file is already invalid: {}",
            issues.join("; ")
        ));
    }
    let mut list = serde_json::from_str::<Vec<Value>>(text).map_err(|e| e.to_string())?;
    list.push(entry);
    let out = serde_json::to_string_pretty(&list).map_err(|e| e.to_string())? + "\n";
    let issues = problems(&out);
    if issues.is_empty() {
        Ok(out)
    } else {
        Err(issues.join("; "))
    }
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let Some((sub, rest)) = args.split_first() else {
        return Err(format!("check or add?\n\n{HELP}"));
    };
    if sub == "-h" || sub == "--help" {
        println!("{HELP}");
        return Ok(0);
    }
    let mut file = PathBuf::from(DEFAULT_FILE);
    let (mut affected, mut fixed, mut severity, mut url, mut summary, mut id) =
        (None, None, None, None, None, None);
    let mut it = rest.iter();
    while let Some(arg) = it.next() {
        let mut value = |what: &str| it.next().cloned().ok_or(format!("{what} needs a value"));
        match arg.as_str() {
            "--file" => file = PathBuf::from(value("--file")?),
            "--affected" => affected = Some(value("--affected")?),
            "--fixed" => fixed = Some(value("--fixed")?),
            "--severity" => severity = Some(value("--severity")?),
            "--url" => url = Some(value("--url")?),
            "--summary" => summary = Some(value("--summary")?),
            "--id" => id = Some(value("--id")?),
            other => return Err(format!("unknown argument '{other}'\n\n{HELP}")),
        }
    }
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    match sub.as_str() {
        "check" => {
            let issues = problems(&text);
            if issues.is_empty() {
                println!("advisory: {} is valid", file.display());
                Ok(0)
            } else {
                for issue in &issues {
                    eprintln!("advisory: {issue}");
                }
                Ok(1)
            }
        }
        "add" => {
            let list = serde_json::from_str::<Vec<Value>>(&text).map_err(|e| e.to_string())?;
            let id = id.unwrap_or_else(|| {
                next_id(&list, crate::lock_age::year_of(crate::lock_age::now_secs()))
            });
            let mut entry = json!({
                "id": id,
                "affected": affected.ok_or("--affected is required")?,
                "severity": severity.ok_or("--severity is required")?,
                "url": url.ok_or("--url is required")?,
                "summary": summary.ok_or("--summary is required")?,
            });
            if let Some(fixed) = fixed {
                entry["fixed"] = Value::String(fixed);
            }
            let out = add_entry(&text, entry)?;
            std::fs::write(&file, out).map_err(|e| format!("{}: {e}", file.display()))?;
            println!("advisory: added {id} to {}", file.display());
            Ok(0)
        }
        other => Err(format!("unknown subcommand '{other}'\n\n{HELP}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str) -> Value {
        json!({"id": id, "affected": ">=1.2.0, <1.4.3", "fixed": "1.4.3", "severity": "high",
               "url": "https://example.org/a", "summary": "A flaw."})
    }

    #[test]
    fn the_empty_list_is_valid() {
        assert!(problems("[]").is_empty());
    }

    #[test]
    fn a_good_entry_passes_and_bad_ones_are_named() {
        let good = serde_json::to_string(&json!([entry("GPSA-2026-001")])).unwrap();
        assert!(problems(&good).is_empty());

        let mut bad = entry("GPSA-26-1");
        bad["affected"] = json!("everything");
        bad["fixed"] = json!("soon");
        bad["severity"] = json!("meh");
        bad["url"] = json!("http://example.org");
        bad["summary"] = json!(" ");
        let issues = problems(&serde_json::to_string(&json!([bad])).unwrap());
        assert_eq!(issues.len(), 6, "{issues:?}");
    }

    #[test]
    fn duplicate_ids_are_refused() {
        let text = serde_json::to_string(&json!([entry("GPSA-2026-001"), entry("GPSA-2026-001")]))
            .unwrap();
        assert!(problems(&text).iter().any(|p| p.contains("duplicate")));
    }

    #[test]
    fn add_picks_the_next_id_and_validates() {
        let list = vec![entry("GPSA-2026-001"), entry("GPSA-2025-009")];
        assert_eq!(next_id(&list, 2026), "GPSA-2026-002");
        assert_eq!(next_id(&list, 2027), "GPSA-2027-001");

        let out = add_entry("[]", entry("GPSA-2026-001")).unwrap();
        assert!(problems(&out).is_empty());
        assert!(
            add_entry(&out, entry("GPSA-2026-001")).is_err(),
            "duplicate"
        );
    }
}
