//! The folder of notes: reading, creating and editing issues.
//!
//! No TUI, no network. Every write is atomic (a temp file next to the note,
//! then a rename) and touches only the frontmatter keys GitPoulpi owns; the
//! body and every other property stay byte for byte (see
//! [`super::frontmatter`]).

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use gitpoulpi_config::LocalMarkdownSettings;

use super::frontmatter::{Document, Frontmatter};
use crate::{Error, Result};

/// One note read as an issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub id: u64,
    pub path: PathBuf,
    pub title: String,
    pub status: String,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub created: String,
    /// The `updated` property, else the file's modification time.
    pub updated: String,
}

/// What a new issue is made of.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewNote {
    pub title: String,
    pub body: String,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    /// Empty: the first status of the configured order.
    pub status: String,
}

/// A change to an existing issue; `None` leaves a property alone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Patch {
    pub title: Option<String>,
    pub status: Option<String>,
    pub assignees: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    /// `Some(None)` clears it.
    pub milestone: Option<Option<String>>,
}

/// Every value already used in the folder, for the pickers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsedValues {
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub milestones: Vec<String>,
    /// The configured order first, then the others in order of appearance.
    pub statuses: Vec<String>,
}

pub struct Store {
    dir: PathBuf,
    settings: LocalMarkdownSettings,
}

fn local(msg: impl Into<String>) -> Error {
    Error::Local(msg.into())
}

/// `12-crash-on-empty-name.md` gives 12.
pub fn id_from_file_name(name: &str) -> Option<u64> {
    let stem = name
        .strip_suffix(".md")
        .or_else(|| name.strip_suffix(".MD"))?;
    let digits: String = stem.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() || digits.len() > 9 {
        return None;
    }
    // `2026-10-14 standup` is a date, not issue 2026.
    let rest = &stem[digits.len()..];
    if !(rest.is_empty() || rest.starts_with(['-', '_', ' '])) {
        return None;
    }
    let mut after = rest.chars().skip(1);
    if digits.len() == 4
        && rest.starts_with('-')
        && after.next().is_some_and(|c| c.is_ascii_digit())
        && after.next().is_some_and(|c| c.is_ascii_digit())
        && after.next() == Some('-')
    {
        return None;
    }
    digits.parse().ok()
}

/// A file-name slug: lower case, letters and digits, single hyphens.
pub fn slug(title: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in title.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    let out = out.trim_end_matches('-');
    let cut: String = out.chars().take(50).collect();
    let cut = cut.trim_end_matches('-');
    if cut.is_empty() {
        "issue".to_string()
    } else {
        cut.to_string()
    }
}

/// Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `2026-10-14T09:30:00` (UTC) for a unix time.
pub fn iso_datetime(unix: i64) -> String {
    let (y, m, d) = civil(unix.div_euclid(86_400));
    let secs = unix.rem_euclid(86_400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}",
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn mtime_unix(path: &Path) -> i64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn read_issue(path: &Path) -> Option<(Issue, Document)> {
    let name = path.file_name()?.to_str()?;
    let text = std::fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&text).into_owned();
    let doc = Document::parse(&text);
    let empty = Frontmatter::default();
    let front = doc.front.as_ref().unwrap_or(&empty);
    let id = front
        .scalar("id")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .or_else(|| id_from_file_name(name))?;
    let title = front
        .scalar("title")
        .filter(|t| !t.trim().is_empty())
        .or_else(|| {
            doc.body
                .lines()
                .find_map(|l| l.strip_prefix("# ").map(|t| t.trim().to_string()))
        })
        .unwrap_or_else(|| {
            let stem = name.trim_end_matches(".md").trim_end_matches(".MD");
            let rest = stem.trim_start_matches(|c: char| c.is_ascii_digit());
            rest.trim_start_matches(['-', '_', ' ']).replace('-', " ")
        });
    let milestone = front.scalar("milestone").filter(|m| !m.trim().is_empty());
    let issue = Issue {
        id,
        path: path.to_path_buf(),
        title,
        status: front
            .scalar("status")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "todo".to_string()),
        assignees: front.list("assignees"),
        labels: front.list("labels"),
        milestone,
        created: front.scalar("created").unwrap_or_default(),
        updated: front
            .scalar("updated")
            .unwrap_or_else(|| iso_datetime(mtime_unix(path))),
    };
    Some((issue, doc))
}

impl Store {
    pub fn new(dir: impl Into<PathBuf>, settings: LocalMarkdownSettings) -> Self {
        Self {
            dir: dir.into(),
            settings,
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn settings(&self) -> &LocalMarkdownSettings {
        &self.settings
    }

    /// The `.md` files of the folder (not recursive, no hidden files), by name.
    fn note_paths(&self) -> Result<Vec<PathBuf>> {
        let entries = std::fs::read_dir(&self.dir)
            .map_err(|e| local(format!("cannot read `{}`: {e}", self.dir.display())))?;
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("md"))
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| !n.starts_with('.'))
            })
            .collect();
        paths.sort();
        Ok(paths)
    }

    /// Every note that reads as an issue, by id. A second note claiming an
    /// id already taken is skipped (the first file name wins).
    pub fn list(&self) -> Result<Vec<Issue>> {
        let mut seen = HashSet::new();
        let mut out: Vec<Issue> = self
            .note_paths()?
            .iter()
            .filter_map(|p| read_issue(p))
            .map(|(i, _)| i)
            .filter(|i| seen.insert(i.id))
            .collect();
        out.sort_by_key(|i| i.id);
        Ok(out)
    }

    pub fn get(&self, id: u64) -> Result<Issue> {
        self.list()?
            .into_iter()
            .find(|i| i.id == id)
            .ok_or_else(|| local(format!("no issue {id} in `{}`", self.dir.display())))
    }

    /// The next free id: the highest in the folder (frontmatter or file
    /// name, so a note GitPoulpi cannot read as an issue still counts) plus
    /// one.
    pub fn next_id(&self) -> Result<u64> {
        let mut max = 0;
        for path in self.note_paths()? {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                max = max.max(id_from_file_name(name).unwrap_or(0));
            }
            if let Some((issue, _)) = read_issue(&path) {
                max = max.max(issue.id);
            }
        }
        Ok(max + 1)
    }

    /// The values already used, for the pickers (case-insensitive unique,
    /// first spelling wins).
    pub fn used_values(&self) -> Result<UsedValues> {
        let issues = self.list()?;
        let unique = |all: Vec<String>| {
            let mut seen = BTreeSet::new();
            let mut out = Vec::new();
            for v in all {
                if seen.insert(v.to_lowercase()) {
                    out.push(v);
                }
            }
            out.sort_by_key(|v| v.to_lowercase());
            out
        };
        let mut statuses = self.settings.status_order();
        for issue in &issues {
            if !statuses
                .iter()
                .any(|s| s.eq_ignore_ascii_case(&issue.status))
            {
                statuses.push(issue.status.clone());
            }
        }
        Ok(UsedValues {
            assignees: unique(issues.iter().flat_map(|i| i.assignees.clone()).collect()),
            labels: unique(issues.iter().flat_map(|i| i.labels.clone()).collect()),
            milestones: unique(issues.iter().filter_map(|i| i.milestone.clone()).collect()),
            statuses,
        })
    }

    /// Writes a new note `<id>-<slug>.md` and returns it.
    pub fn create(&self, new: &NewNote) -> Result<Issue> {
        let title = new.title.trim();
        if title.is_empty() {
            return Err(local("an issue needs a title"));
        }
        if !self.dir.is_dir() {
            return Err(local(format!("`{}` is not a folder", self.dir.display())));
        }
        let status = if new.status.trim().is_empty() {
            self.settings.status_order().remove(0)
        } else {
            new.status.trim().to_string()
        };
        let now = unix_now();
        let eol = "\n";
        let mut front = Frontmatter::default();
        let mut id = self.next_id()?;
        let mut path;
        loop {
            path = self.dir.join(format!("{id}-{}.md", slug(title)));
            if !path.exists() {
                break;
            }
            id += 1;
        }
        front.set_number("id", id, eol);
        front.set_scalar("title", title, eol);
        front.set_scalar("status", &status, eol);
        front.set_list("assignees", &new.assignees, eol);
        front.set_list("labels", &new.labels, eol);
        front.set_scalar("milestone", new.milestone.as_deref().unwrap_or(""), eol);
        front.set_scalar("created", &iso_datetime(now)[..10], eol);
        front.set_scalar("updated", &iso_datetime(now), eol);
        let mut doc = Document::parse("");
        *doc.ensure_front() = front;
        let mut text = doc.render();
        if !new.body.is_empty() {
            text.push_str(new.body.trim_end_matches(['\r', '\n']));
            text.push('\n');
        }
        write_atomic(&path, &text)?;
        read_issue(&path)
            .map(|(i, _)| i)
            .ok_or_else(|| local("the new note could not be read back"))
    }

    /// Applies `patch` to issue `id`.
    pub fn update(&self, id: u64, patch: &Patch) -> Result<Issue> {
        let issue = self.get(id)?;
        let text = std::fs::read(&issue.path)
            .map_err(|e| local(format!("cannot read `{}`: {e}", issue.path.display())))?;
        let mut doc = Document::parse(&String::from_utf8_lossy(&text));
        let eol = doc.eol;
        let had_front = doc.front.is_some();
        let front = doc.ensure_front();
        if !had_front {
            front.set_number("id", id, eol);
            front.set_scalar("title", &issue.title, eol);
        } else if front.scalar("id").is_none() {
            front.set_number("id", id, eol);
        }
        if let Some(title) = &patch.title {
            front.set_scalar("title", title, eol);
        }
        if let Some(status) = &patch.status {
            front.set_scalar("status", status, eol);
        }
        if let Some(list) = &patch.assignees {
            front.set_list("assignees", list, eol);
        }
        if let Some(list) = &patch.labels {
            front.set_list("labels", list, eol);
        }
        if let Some(milestone) = &patch.milestone {
            front.set_scalar("milestone", milestone.as_deref().unwrap_or(""), eol);
        }
        front.set_scalar("updated", &iso_datetime(unix_now()), eol);
        write_atomic(&issue.path, &doc.render())?;
        read_issue(&issue.path)
            .map(|(i, _)| i)
            .ok_or_else(|| local("the note could not be read back"))
    }

    /// Cheap change detector: a hash of every note's name, size and
    /// modification time. Reads directory metadata only.
    pub fn fingerprint(dir: &Path) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut feed = |bytes: &[u8]| {
            for b in bytes {
                hash ^= u64::from(*b);
                hash = hash.wrapping_mul(0x0100_0000_01b3);
            }
        };
        let Ok(entries) = std::fs::read_dir(dir) else {
            return 0;
        };
        let mut rows: Vec<(String, u64, i64)> = entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                if !name.to_ascii_lowercase().ends_with(".md") {
                    return None;
                }
                let meta = e.metadata().ok()?;
                let modified = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i64)
                    .unwrap_or(0);
                Some((name, meta.len(), modified))
            })
            .collect();
        rows.sort();
        for (name, len, modified) in rows {
            feed(name.as_bytes());
            feed(&len.to_le_bytes());
            feed(&modified.to_le_bytes());
        }
        hash
    }
}

/// Temp file next to `path`, then a rename: a reader (Obsidian's file
/// watcher, a `git add`) never sees half a note.
pub fn write_atomic(path: &Path, text: &str) -> Result<()> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| local("bad note path"))?;
    let tmp = path.with_file_name(format!(".{name}.gitpoulpi-tmp"));
    std::fs::write(&tmp, text.as_bytes())
        .map_err(|e| local(format!("cannot write `{}`: {e}", tmp.display())))?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        local(format!("cannot replace `{}`: {e}", path.display()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(dir: &Path) -> Store {
        Store::new(dir, LocalMarkdownSettings::default())
    }

    const UNKNOWN: &str = "---\nid: 7\ntitle: Seven\nstatus: doing\ncssclasses: wide\nlabels: [bug]\nrating: 4.5\n---\nBody stays.\n\n- [ ] item\n";

    #[test]
    fn ids_are_max_plus_one_and_file_names_count() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(dir.path());
        assert_eq!(s.next_id().unwrap(), 1);
        std::fs::write(dir.path().join("7-seven.md"), UNKNOWN).unwrap();
        std::fs::write(dir.path().join("20-raw.md"), "no frontmatter\n").unwrap();
        std::fs::write(dir.path().join("2026-10-14 standup.md"), "x\n").unwrap();
        assert_eq!(s.next_id().unwrap(), 21);
        let made = s
            .create(&NewNote {
                title: "Crash on empty name!".into(),
                body: "Steps".into(),
                labels: vec!["bug".into()],
                ..NewNote::default()
            })
            .unwrap();
        assert_eq!(made.id, 21);
        assert_eq!(
            made.path.file_name().unwrap().to_str().unwrap(),
            "21-crash-on-empty-name.md"
        );
        assert_eq!(made.status, "todo");
        assert_eq!(made.labels, ["bug"]);
        let text = std::fs::read_to_string(&made.path).unwrap();
        assert!(text.starts_with("---\nid: 21\ntitle: Crash on empty name!\nstatus: todo\nassignees: []\nlabels: [bug]\nmilestone:\ncreated: "));
        assert!(text.ends_with("---\nSteps\n"));
    }

    #[test]
    fn an_edit_keeps_unknown_keys_and_the_body_byte_for_byte() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(dir.path());
        let path = dir.path().join("7-seven.md");
        std::fs::write(&path, UNKNOWN).unwrap();
        let after = s
            .update(
                7,
                &Patch {
                    status: Some("done".into()),
                    labels: Some(vec!["bug".into(), "ui".into()]),
                    ..Patch::default()
                },
            )
            .unwrap();
        assert_eq!(after.status, "done");
        let text = std::fs::read_to_string(&path).unwrap();
        let want_head = "---\nid: 7\ntitle: Seven\nstatus: done\ncssclasses: wide\nlabels: [bug, ui]\nrating: 4.5\nupdated: ";
        assert!(text.starts_with(want_head), "{text}");
        assert!(text.ends_with("---\nBody stays.\n\n- [ ] item\n"));
        // No temp file is left behind.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn crlf_notes_stay_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(dir.path());
        let path = dir.path().join("3-x.md");
        std::fs::write(
            &path,
            UNKNOWN.replace('\n', "\r\n").replace("id: 7", "id: 3"),
        )
        .unwrap();
        s.update(
            3,
            &Patch {
                assignees: Some(vec!["kai".into()]),
                ..Patch::default()
            },
        )
        .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("assignees: [kai]\r\n"));
        assert!(!text.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn reading_is_lenient() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(dir.path());
        // No id in the frontmatter: the file name gives it. No status: todo.
        std::fs::write(dir.path().join("5-from-name.md"), "---\ntitle: T\n---\nb\n").unwrap();
        // No frontmatter at all: title from the heading.
        std::fs::write(dir.path().join("6-x.md"), "# Heading title\ntext\n").unwrap();
        // Not an issue.
        std::fs::write(dir.path().join("readme.md"), "hello\n").unwrap();
        std::fs::write(dir.path().join(".hidden.md"), "---\nid: 9\n---\n").unwrap();
        // A duplicate id: the first file name wins.
        std::fs::write(dir.path().join("5-zzz.md"), "---\nid: 5\ntitle: dup\n---\n").unwrap();
        let all = s.list().unwrap();
        let ids: Vec<u64> = all.iter().map(|i| i.id).collect();
        assert_eq!(ids, [5, 6]);
        assert_eq!(all[0].status, "todo");
        assert_eq!(all[0].title, "T");
        assert_eq!(all[1].title, "Heading title");
        // Editing a note with no block adds one, keeps the body.
        s.update(
            6,
            &Patch {
                status: Some("doing".into()),
                ..Patch::default()
            },
        )
        .unwrap();
        let text = std::fs::read_to_string(dir.path().join("6-x.md")).unwrap();
        assert!(text.starts_with("---\nid: 6\ntitle: Heading title\nstatus: doing\nupdated: "));
        assert!(text.ends_with("---\n# Heading title\ntext\n"));
    }

    #[test]
    fn used_values_feed_the_pickers() {
        let dir = tempfile::tempdir().unwrap();
        let s = store(dir.path());
        std::fs::write(dir.path().join("7-seven.md"), UNKNOWN).unwrap();
        s.create(&NewNote {
            title: "b".into(),
            labels: vec!["Bug".into(), "ui".into()],
            assignees: vec!["kai".into()],
            milestone: Some("v0.1".into()),
            status: "review".into(),
            ..NewNote::default()
        })
        .unwrap();
        let v = s.used_values().unwrap();
        assert_eq!(v.labels, ["bug", "ui"]);
        assert_eq!(v.assignees, ["kai"]);
        assert_eq!(v.milestones, ["v0.1"]);
        assert_eq!(v.statuses, ["todo", "doing", "done", "review"]);
    }

    #[test]
    fn the_fingerprint_moves_with_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let before = Store::fingerprint(dir.path());
        std::fs::write(dir.path().join("1-a.md"), "x").unwrap();
        let one = Store::fingerprint(dir.path());
        assert_ne!(before, one);
        std::fs::write(dir.path().join("1-a.md"), "xy").unwrap();
        assert_ne!(one, Store::fingerprint(dir.path()));
        std::fs::write(dir.path().join("notes.txt"), "ignored").unwrap();
        assert_eq!(
            Store::fingerprint(dir.path()),
            Store::fingerprint(dir.path())
        );
    }

    #[test]
    fn slugs_and_dates() {
        assert_eq!(slug("  Fix: the Thing / now!! "), "fix-the-thing-now");
        assert_eq!(slug("***"), "issue");
        assert_eq!(slug("Été à Paris"), "été-à-paris");
        assert_eq!(iso_datetime(0), "1970-01-01T00:00:00");
        assert_eq!(iso_datetime(1_792_000_000), "2026-10-14T17:46:40");
        assert_eq!(id_from_file_name("12-x.md"), Some(12));
        assert_eq!(id_from_file_name("12.md"), Some(12));
        assert_eq!(id_from_file_name("12abc.md"), None);
        assert_eq!(id_from_file_name("x.md"), None);
    }
}
