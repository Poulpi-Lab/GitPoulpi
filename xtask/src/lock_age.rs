//! `cargo run -p xtask -- lock-age --base <ref> --min-days 2`: refuses a
//! `Cargo.lock` that pulls in a crates.io release younger than the minimum
//! age, so a freshly published (and possibly hijacked) version is never taken
//! the day it appears. Dependabot has the same rule as its `cooldown`; this
//! covers a manual `cargo update`.
//!
//! 1. diff `Cargo.lock` against `git show <base>:Cargo.lock`;
//! 2. for every added or changed crates.io entry ask the crates.io API for
//!    `created_at`;
//! 3. fail with the list of entries younger than the minimum.
//!
//! Escape hatch for an urgent fix: a commit trailer in the range
//! `<base>..HEAD`, `Lock-Age-Override: <name>@<version> <reason>` (the reason
//! is mandatory). The logic is pure over two lockfile strings and an injected
//! date lookup; only [`crates_io_created_at`] touches the network.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const HELP: &str = "\
Usage: cargo run -p xtask -- lock-age --base <ref> [--min-days N] [--head <file>]

Fails when Cargo.lock adds or changes a crates.io crate whose version was
published less than N days ago (default 2). <ref> is a git ref whose
Cargo.lock is the baseline. --head names the lock file to check (default
Cargo.lock). A commit trailer between <ref> and HEAD,
`Lock-Age-Override: name@version reason`, waives one entry.";

/// The registry source string Cargo writes for crates.io.
const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";

/// A crate version taken from the registry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub name: String,
    pub version: String,
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// A new entry that is younger than the minimum age.
#[derive(Debug, PartialEq, Eq)]
pub struct Young {
    pub entry: Entry,
    pub age_secs: i64,
}

/// Every crates.io `(name, version)` of a `Cargo.lock`.
pub fn registry_entries(lock: &str) -> Result<BTreeSet<Entry>, String> {
    let doc: toml::Value = toml::from_str(lock).map_err(|e| format!("Cargo.lock: {e}"))?;
    let mut out = BTreeSet::new();
    for package in doc
        .get("package")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
    {
        let field = |key: &str| package.get(key).and_then(|v| v.as_str());
        if field("source") != Some(CRATES_IO) {
            continue;
        }
        if let (Some(name), Some(version)) = (field("name"), field("version")) {
            out.insert(Entry {
                name: name.to_string(),
                version: version.to_string(),
            });
        }
    }
    Ok(out)
}

/// Entries of `head` that `base` does not have: added crates and new versions.
pub fn added_entries(base: &str, head: &str) -> Result<Vec<Entry>, String> {
    let before = registry_entries(base)?;
    let after = registry_entries(head)?;
    Ok(after.difference(&before).cloned().collect())
}

/// `Lock-Age-Override: name@version reason` trailers of a commit log. A
/// trailer without a reason does not count.
pub fn overrides(log: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in log.lines() {
        let Some(rest) = line.trim().strip_prefix("Lock-Age-Override:") else {
            continue;
        };
        let mut words = rest.split_whitespace();
        let (Some(target), Some(_reason)) = (words.next(), words.next()) else {
            continue;
        };
        out.insert(target.to_string());
    }
    out
}

/// The entries younger than `min_days`. `lookup` returns the publish time of
/// an entry in seconds since the Unix epoch; its error fails the whole check
/// (an age that cannot be established is not assumed to be fine).
pub fn too_young(
    entries: &[Entry],
    min_days: i64,
    now: i64,
    waived: &BTreeSet<String>,
    lookup: &mut dyn FnMut(&Entry) -> Result<i64, String>,
) -> Result<Vec<Young>, String> {
    let mut young = Vec::new();
    for entry in entries {
        if waived.contains(&entry.to_string()) {
            continue;
        }
        let created = lookup(entry)?;
        let age_secs = now - created;
        if age_secs < min_days * 86_400 {
            young.push(Young {
                entry: entry.clone(),
                age_secs,
            });
        }
    }
    Ok(young)
}

/// Seconds since the Unix epoch of an RFC 3339 timestamp such as
/// `2026-11-05T12:34:56.123456+00:00` (what crates.io returns).
pub fn parse_rfc3339(text: &str) -> Result<i64, String> {
    let bad = || format!("not an RFC 3339 timestamp: {text}");
    let (date, time) = text.split_once(['T', 't', ' ']).ok_or_else(bad)?;
    let mut d = date.split('-');
    let (y, m, day) = (
        d.next().and_then(|v| v.parse::<i64>().ok()),
        d.next().and_then(|v| v.parse::<i64>().ok()),
        d.next().and_then(|v| v.parse::<i64>().ok()),
    );
    let (Some(y), Some(m), Some(day)) = (y, m, day) else {
        return Err(bad());
    };
    if !(1..=12).contains(&m) || !(1..=31).contains(&day) {
        return Err(bad());
    }
    // Split the time from its zone: Z, +hh:mm or -hh:mm.
    let (clock, offset_secs) = if let Some(c) = time.strip_suffix(['Z', 'z']) {
        (c, 0)
    } else if let Some(i) = time.rfind(['+', '-']) {
        let (c, z) = time.split_at(i);
        let sign = if z.starts_with('-') { -1 } else { 1 };
        let (zh, zm) = z[1..].split_once(':').ok_or_else(bad)?;
        let secs = zh.parse::<i64>().map_err(|_| bad())? * 3600
            + zm.parse::<i64>().map_err(|_| bad())? * 60;
        (c, sign * secs)
    } else {
        return Err(bad());
    };
    let clock = clock.split('.').next().unwrap_or(clock);
    let mut t = clock.split(':');
    let (h, mi, s) = (
        t.next().and_then(|v| v.parse::<i64>().ok()),
        t.next().and_then(|v| v.parse::<i64>().ok()),
        t.next().and_then(|v| v.parse::<i64>().ok()),
    );
    let (Some(h), Some(mi), Some(s)) = (h, mi, s) else {
        return Err(bad());
    };
    Ok(days_from_civil(y, m, day) * 86_400 + h * 3600 + mi * 60 + s - offset_secs)
}

/// Days since 1970-01-01 of a proleptic Gregorian date.
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The calendar year of a Unix timestamp (UTC).
pub fn year_of(epoch_secs: i64) -> i64 {
    let z = epoch_secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    yoe + era * 400 + i64::from(m <= 2)
}

/// Seconds since the Unix epoch, now.
pub fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// The real lookup: `GET https://crates.io/api/v1/crates/<name>/<version>`.
pub fn crates_io_created_at(entry: &Entry) -> Result<i64, String> {
    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}",
        entry.name, entry.version
    );
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .user_agent("gitpoulpi-xtask lock-age (+https://github.com/Poulpi-Lab/gitpoulpi)")
        .build()
        .into();
    let body: serde_json::Value = agent
        .get(&url)
        .call()
        .map_err(|e| format!("{entry}: crates.io lookup failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("{entry}: unreadable crates.io answer: {e}"))?;
    let created = body["version"]["created_at"]
        .as_str()
        .ok_or_else(|| format!("{entry}: crates.io answer has no created_at"))?;
    // crates.io asks for at most one request per second.
    std::thread::sleep(Duration::from_millis(1100));
    parse_rfc3339(created)
}

fn git(args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let mut base = None;
    let mut min_days = 2_i64;
    let mut head = "Cargo.lock".to_string();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return Ok(0);
            }
            "--base" => base = it.next().cloned(),
            "--min-days" => {
                min_days = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--min-days needs a number")?;
            }
            "--head" => head = it.next().cloned().ok_or("--head needs a file")?,
            other => return Err(format!("unknown argument '{other}'\n\n{HELP}")),
        }
    }
    let base = base.ok_or("--base <ref> is required")?;
    let base_lock = git(&["show", &format!("{base}:Cargo.lock")])?;
    let head_lock =
        std::fs::read_to_string(Path::new(&head)).map_err(|e| format!("{head}: {e}"))?;
    let entries = added_entries(&base_lock, &head_lock)?;
    if entries.is_empty() {
        println!("lock-age: no new crates.io versions against {base}");
        return Ok(0);
    }
    // A shallow checkout has no range to read; the trailers are then simply absent.
    let log = git(&["log", "--format=%B", &format!("{base}..HEAD")]).unwrap_or_default();
    let waived = overrides(&log);
    let mut lookup = |e: &Entry| -> Result<i64, String> { crates_io_created_at(e) };
    let young = too_young(&entries, min_days, now_secs(), &waived, &mut lookup)?;
    if young.is_empty() {
        println!(
            "lock-age: {} new crates.io version(s) against {base}, none younger than {min_days} day(s)",
            entries.len()
        );
        return Ok(0);
    }
    eprintln!("lock-age: younger than {min_days} day(s):");
    for y in &young {
        eprintln!("  {} published {} hour(s) ago", y.entry, y.age_secs / 3600);
    }
    eprintln!(
        "wait, or waive an urgent one with a commit trailer: \
         `Lock-Age-Override: name@version <reason>`"
    );
    Ok(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const NOW: i64 = 1_793_000_000; // 2026-10-26 in seconds, any fixed value works

    fn lock(packages: &[(&str, &str, bool)]) -> String {
        let mut out = String::from("version = 4\n");
        for (name, version, registry) in packages {
            out.push_str(&format!(
                "\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\n"
            ));
            if *registry {
                out.push_str(&format!("source = \"{CRATES_IO}\"\n"));
            }
        }
        out
    }

    fn entry(name: &str, version: &str) -> Entry {
        Entry {
            name: name.into(),
            version: version.into(),
        }
    }

    #[test]
    fn only_added_or_changed_registry_entries_are_listed() {
        let base = lock(&[
            ("serde", "1.0.1", true),
            ("old", "1.0.0", true),
            ("local", "0.1.0", false),
        ]);
        let head = lock(&[
            ("serde", "1.0.2", true),
            ("old", "1.0.0", true),
            ("fresh", "0.3.0", true),
            ("local", "0.2.0", false),
        ]);
        assert_eq!(
            added_entries(&base, &head).unwrap(),
            vec![entry("fresh", "0.3.0"), entry("serde", "1.0.2")]
        );
    }

    #[test]
    fn a_young_entry_is_reported_and_an_old_one_is_not() {
        let entries = vec![entry("young", "1.0.0"), entry("old", "2.0.0")];
        let table: BTreeMap<String, i64> = [
            ("young@1.0.0".to_string(), NOW - 3600),
            ("old@2.0.0".to_string(), NOW - 10 * 86_400),
        ]
        .into();
        let mut lookup = |e: &Entry| {
            table
                .get(&e.to_string())
                .copied()
                .ok_or("missing".to_string())
        };
        let young = too_young(&entries, 2, NOW, &BTreeSet::new(), &mut lookup).unwrap();
        assert_eq!(
            young,
            vec![Young {
                entry: entry("young", "1.0.0"),
                age_secs: 3600
            }]
        );
    }

    #[test]
    fn exactly_the_minimum_age_passes() {
        let entries = vec![entry("edge", "1.0.0")];
        let mut lookup = |_: &Entry| Ok(NOW - 2 * 86_400);
        assert!(too_young(&entries, 2, NOW, &BTreeSet::new(), &mut lookup)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn a_failed_lookup_fails_the_check() {
        let entries = vec![entry("x", "1.0.0")];
        let mut lookup = |_: &Entry| Err("offline".to_string());
        assert!(too_young(&entries, 2, NOW, &BTreeSet::new(), &mut lookup).is_err());
    }

    #[test]
    fn an_override_trailer_with_a_reason_waives_the_entry() {
        let log = "fix: bump\n\nLock-Age-Override: young@1.0.0 security fix for RUSTSEC-1\n\nLock-Age-Override: bare@1.0.0\n";
        let waived = overrides(log);
        assert!(waived.contains("young@1.0.0"));
        assert!(
            !waived.contains("bare@1.0.0"),
            "a trailer without a reason does not count"
        );
        let entries = vec![entry("young", "1.0.0")];
        let mut lookup = |_: &Entry| Ok(NOW);
        assert!(too_young(&entries, 2, NOW, &waived, &mut lookup)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn timestamps_parse_with_zones() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(
            parse_rfc3339("2000-03-01T00:00:00.123456+00:00").unwrap(),
            951_868_800
        );
        assert_eq!(
            parse_rfc3339("2000-03-01T01:00:00+01:00").unwrap(),
            951_868_800
        );
        assert!(parse_rfc3339("yesterday").is_err());
    }

    #[test]
    fn the_year_of_an_epoch() {
        assert_eq!(year_of(0), 1970);
        assert_eq!(year_of(951_868_800), 2000);
        assert_eq!(year_of(1_793_000_000), 2026);
    }
}
