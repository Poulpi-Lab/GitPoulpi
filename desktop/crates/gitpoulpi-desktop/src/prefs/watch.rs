//! Noticing an edit of the settings files made from outside (another frontend, `gitpoulpi config set`,
//! a text editor), the way the terminal UI's `theme_watch.rs` does: a cheap fingerprint (modification
//! time and length of each file, two `stat` calls and never a read) compared on a timer. A moved
//! fingerprint starts the real re-read, which runs off the UI thread.

use std::path::Path;
use std::time::SystemTime;

use gitpoulpi_config::ConfigPaths;
use gitpoulpi_session::RecentRepos;

type Stat = Option<(SystemTime, u64)>;

/// `config.toml`, `desktop.toml`, `recent.toml`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fingerprint([Stat; 3]);

fn stat_of(path: &Path) -> Stat {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

impl Fingerprint {
    pub fn of(paths: &ConfigPaths) -> Self {
        Self([
            stat_of(&paths.config_file()),
            stat_of(&paths.desktop_file()),
            stat_of(&RecentRepos::file_in(paths.config_dir())),
        ])
    }
}

/// What was last seen.
#[derive(Debug, Default)]
pub struct Watch {
    seen: Option<Fingerprint>,
}

impl Watch {
    /// Whether `now` differs from the last look. The first look only records.
    pub fn changed(&mut self, now: Fingerprint) -> bool {
        let changed = self.seen.is_some_and(|before| before != now);
        self.seen = Some(now);
        changed
    }

    /// Records `now` without calling it a change: this process wrote the files itself.
    pub fn rebase(&mut self, now: Fingerprint) {
        self.seen = Some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn at(secs: u64, len: u64) -> Stat {
        Some((SystemTime::UNIX_EPOCH + Duration::from_secs(secs), len))
    }

    #[test]
    fn the_first_look_records_and_any_file_moving_is_a_change() {
        let mut w = Watch::default();
        assert!(!w.changed(Fingerprint([at(1, 1), None, None])));
        assert!(!w.changed(Fingerprint([at(1, 1), None, None])));
        assert!(
            w.changed(Fingerprint([at(2, 1), None, None])),
            "config.toml"
        );
        assert!(
            w.changed(Fingerprint([at(2, 1), at(3, 9), None])),
            "desktop.toml"
        );
        assert!(w.changed(Fingerprint([at(2, 1), None, None])), "deleted");
        assert!(
            w.changed(Fingerprint([at(2, 1), None, at(5, 5)])),
            "recent.toml"
        );
    }

    #[test]
    fn our_own_write_is_not_a_change() {
        let mut w = Watch::default();
        w.changed(Fingerprint([at(1, 1), None, None]));
        w.rebase(Fingerprint([at(2, 8), None, None]));
        assert!(!w.changed(Fingerprint([at(2, 8), None, None])));
    }
}
