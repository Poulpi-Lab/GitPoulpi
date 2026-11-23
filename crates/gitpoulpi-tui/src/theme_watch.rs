//! Notices the theme being edited from outside: `config.toml` (a
//! `gitpoulpi config set ui.theme ...` from a shell) or the active user
//! theme file `<config>/themes/<slug>.toml` rewritten while the window runs.
//!
//! Nothing else re-reads them (only closing Settings does), so without this a
//! running window kept the old colours until restart. Same shape as
//! `workspace_ui::watch`: a cheap fingerprint (mtime, length) of the two files
//! looked at on the loop's tick at most once per [`EVERY`], that is two `stat`
//! calls and never a read, so a frame is never held up. A moved fingerprint
//! starts the real re-read (config, theme files, install) on a thread, which
//! then pokes the loop. The first look only records. The next look is also a
//! registered deadline ([`next_deadline`]) because the loop idles in `poll`.

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

const EVERY: Duration = Duration::from_millis(750);

type Stat = Option<(SystemTime, u64)>;
type Fingerprint = (Stat, Stat);

#[derive(Default)]
struct Watch {
    seen: Option<Fingerprint>,
    checked: Option<Instant>,
}

impl Watch {
    fn step(&mut self, now: Instant, stat: impl FnOnce() -> Fingerprint) -> bool {
        if self
            .checked
            .is_some_and(|at| now.duration_since(at) < EVERY)
        {
            return false;
        }
        self.checked = Some(now);
        let fingerprint = stat();
        let changed = self.seen.is_some_and(|before| before != fingerprint);
        self.seen = Some(fingerprint);
        changed
    }

    fn deadline(&self) -> Option<Instant> {
        self.checked.map(|at| at + EVERY)
    }
}

static STATE: Mutex<Option<Watch>> = Mutex::new(None);
static ACTIVE_SLUG: Mutex<String> = Mutex::new(String::new());

/// Records the slug whose file is watched (called when a theme is installed
/// from the config).
pub(crate) fn set_active_slug(slug: &str) {
    let mut guard = ACTIVE_SLUG.lock().unwrap_or_else(|e| e.into_inner());
    if *guard != slug {
        *guard = slug.to_string();
    }
}

fn stat_of(path: &Path) -> Stat {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// The two files' fingerprint: `config.toml` and `themes/<slug>.toml`.
fn fingerprint(paths: &gitpoulpi_config::ConfigPaths, slug: &str) -> Fingerprint {
    (
        stat_of(&paths.config_file()),
        stat_of(&paths.themes_dir().join(format!("{slug}.toml"))),
    )
}

/// When the loop must look in again.
pub(crate) fn next_deadline() -> Option<Instant> {
    STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .and_then(Watch::deadline)
}

/// One loop iteration: a change re-reads and installs the theme on a thread.
pub(crate) fn poll() {
    let changed = {
        let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
        let watch = guard.get_or_insert_with(Watch::default);
        watch.step(Instant::now(), || {
            let slug = ACTIVE_SLUG
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            gitpoulpi_config::ConfigPaths::discover()
                .map(|p| fingerprint(&p, &slug))
                .unwrap_or_default()
        })
    };
    if changed {
        std::thread::spawn(|| {
            if let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() {
                crate::install_theme_from_config(settings.config().ui.theme_slug());
                crate::redraw::poke();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: u64, len: u64) -> Stat {
        Some((SystemTime::UNIX_EPOCH + Duration::from_secs(secs), len))
    }

    #[test]
    fn the_first_look_records_and_either_file_moving_is_a_change() {
        let mut w = Watch::default();
        let t0 = Instant::now();
        assert!(!w.step(t0, || (at(1, 1), at(1, 1))));
        assert!(!w.step(t0 + Duration::from_millis(10), || panic!("too early")));
        assert!(!w.step(t0 + EVERY, || (at(1, 1), at(1, 1))));
        assert!(w.step(t0 + EVERY * 2, || (at(2, 1), at(1, 1))), "config");
        assert!(w.step(t0 + EVERY * 3, || (at(2, 1), at(3, 9))), "theme");
        assert!(w.step(t0 + EVERY * 4, || (at(2, 1), None)), "theme deleted");
    }

    #[test]
    fn rewriting_the_active_theme_file_moves_the_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        let paths = gitpoulpi_config::ConfigPaths::for_dir(dir.path().to_path_buf());
        std::fs::create_dir_all(paths.themes_dir()).unwrap();
        let file = paths.themes_dir().join("mine.toml");
        std::fs::write(&file, "base = \"dark\"\n").unwrap();
        let before = fingerprint(&paths, "mine");
        std::fs::write(&file, "base = \"dark\"\n[ui]\naccent = \"#00ff00\"\n").unwrap();
        assert_ne!(fingerprint(&paths, "mine"), before);
        // Another slug's file is not this theme's business.
        assert_eq!(fingerprint(&paths, "other").1, None);
    }
}
