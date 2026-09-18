//! The disk cache: one file per author hash, under the OS cache directory
//! (`gitpoulpi_config::ConfigPaths::cache_dir`), so a fetched avatar
//! survives a restart and a repeated failure is not retried every frame.
//!
//! Three kinds of entry, distinguished by extension rather than by a
//! sidecar file (one `stat` per lookup, not two):
//!
//! - `<hash>.img` -- the raw bytes of a successfully fetched image (PNG or
//!   JPEG; [`super::decode`] sniffs which).
//! - `<hash>.none` -- an empty marker: no source had a picture for this
//!   author (Gravatar's `?d=404` answered 404, GitHub knows no account for
//!   the email). Without it, an author with no avatar (the common case)
//!   would be re-requested on every Log refresh, once per visible row.
//!   **Only a definite "none" is written.** A failed request (no network, a
//!   TLS error, a 5xx, GitHub's rate limit) is not, or a TLS failure would
//!   cache "no avatar" for everybody for six hours.
//! - `<hash>.gh` -- the GitHub REST answer for this author's email (see
//!   `super::github`): the `avatar_url`, or empty for "GitHub links no
//!   account to this email". Kept apart from the two above and for longer
//!   ([`GITHUB_HIT_TTL`]), because each one cost a call against a
//!   60-an-hour unauthenticated budget.
//!
//! Freshness is the file's own mtime against a *caller-supplied* clock,
//! never `SystemTime::now()` read inside this module, so every case is a
//! pure comparison a test drives by choosing `now`.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// A positive result is kept for a day: long enough that scrolling the
/// Log all afternoon does not re-fetch anything, short enough that an
/// author who sets up a Gravatar shows up within a session or two.
pub(crate) const HIT_TTL: Duration = Duration::from_secs(24 * 60 * 60);
/// A negative result (no Gravatar, or the fetch failed) is kept for six
/// hours - long enough to stop a hopeless retry storm across a busy
/// session, short enough that a transient network failure heals itself
/// well within a working day rather than needing a cache clear.
pub(crate) const NEGATIVE_TTL: Duration = Duration::from_secs(6 * 60 * 60);

/// A GitHub `avatar_url` for an email is kept a week: accounts rarely
/// change their picture URL (it is keyed by account id), and every
/// re-ask spends the rate limit.
pub(crate) const GITHUB_HIT_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);
/// "GitHub links no account to this email" is kept a day: an author may
/// add the address to their account, but not often.
pub(crate) const GITHUB_NONE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// [`AvatarCache::get_github`]'s answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GitHubLookup {
    Url(String),
    NoAccount,
    /// Never asked, or the answer expired: ask GitHub.
    Miss,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Lookup {
    /// A decodable image is on disk and still fresh.
    Hit(Vec<u8>),
    /// This author was looked up before and had no avatar, or the fetch
    /// failed, and that answer is still fresh - do not fetch again.
    Negative,
    /// Nothing usable on disk: never looked up, or what is there expired.
    /// The caller should fetch.
    Miss,
}

pub(crate) struct AvatarCache {
    dir: PathBuf,
}

impl AvatarCache {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn image_path(&self, hash: &str) -> PathBuf {
        self.dir.join(format!("{hash}.img"))
    }

    fn miss_path(&self, hash: &str) -> PathBuf {
        self.dir.join(format!("{hash}.none"))
    }

    fn github_path(&self, hash: &str) -> PathBuf {
        self.dir.join(format!("{hash}.gh"))
    }

    /// What GitHub said about `hash`'s email, as of `now`.
    pub(crate) fn get_github(&self, hash: &str, now: SystemTime) -> GitHubLookup {
        let path = self.github_path(hash);
        let Ok(mtime) = std::fs::metadata(&path).and_then(|m| m.modified()) else {
            return GitHubLookup::Miss;
        };
        let Ok(url) = std::fs::read_to_string(&path) else {
            return GitHubLookup::Miss;
        };
        let url = url.trim();
        if url.is_empty() {
            if is_fresh(mtime, now, GITHUB_NONE_TTL) {
                return GitHubLookup::NoAccount;
            }
        } else if is_fresh(mtime, now, GITHUB_HIT_TTL) {
            return GitHubLookup::Url(url.to_string());
        }
        GitHubLookup::Miss
    }

    /// Records GitHub's answer: `Some(avatar_url)`, or `None` for "no
    /// account linked to this email".
    pub(crate) fn put_github(&self, hash: &str, avatar_url: Option<&str>) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.github_path(hash), avatar_url.unwrap_or(""))
    }

    /// Looks up `hash` as of `now`. At most one file's worth of I/O per call
    /// (image *or* marker, whichever `stat`s first) plus the image bytes on a
    /// hit.
    pub(crate) fn get(&self, hash: &str, now: SystemTime) -> Lookup {
        if let Some(bytes) = fresh_file(&self.image_path(hash), now, HIT_TTL) {
            return Lookup::Hit(bytes);
        }
        if fresh_marker(&self.miss_path(hash), now, NEGATIVE_TTL) {
            return Lookup::Negative;
        }
        Lookup::Miss
    }

    /// Records a successful fetch. Clears any stale negative marker so a
    /// later-arriving avatar is not shadowed by an old miss.
    pub(crate) fn put_hit(&self, hash: &str, bytes: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.image_path(hash), bytes)?;
        let _ = std::fs::remove_file(self.miss_path(hash));
        Ok(())
    }

    /// Records "no avatar" or "fetch failed" - the entry [`Lookup::Miss`]
    /// stops being returned for.
    pub(crate) fn put_negative(&self, hash: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.miss_path(hash), [])?;
        Ok(())
    }
}

/// `true` when `mtime + ttl` has not yet passed `now`: the arithmetic every
/// expiry case reduces to, separate so it is testable without a filesystem.
fn is_fresh(mtime: SystemTime, now: SystemTime, ttl: Duration) -> bool {
    match now.duration_since(mtime) {
        // `now` before `mtime` (a clock that moved backwards, or a file
        // written by a future-dated test) is treated as fresh rather than as
        // an error: refusing to serve a cache entry over a clock's wobble
        // would defeat the point of caching.
        Err(_) => true,
        Ok(age) => age <= ttl,
    }
}

fn fresh_file(path: &Path, now: SystemTime, ttl: Duration) -> Option<Vec<u8>> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?;
    if is_fresh(mtime, now, ttl) {
        std::fs::read(path).ok()
    } else {
        None
    }
}

fn fresh_marker(path: &Path, now: SystemTime, ttl: Duration) -> bool {
    std::fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .is_some_and(|mtime| is_fresh(mtime, now, ttl))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache() -> (tempfile::TempDir, AvatarCache) {
        let dir = tempfile::tempdir().unwrap();
        let cache = AvatarCache::new(dir.path().join("avatars"));
        (dir, cache)
    }

    #[test]
    fn a_hash_never_looked_up_is_a_miss() {
        let (_dir, cache) = cache();
        assert_eq!(cache.get("deadbeef", SystemTime::now()), Lookup::Miss);
    }

    #[test]
    fn a_stored_image_is_returned_byte_for_byte() {
        let (_dir, cache) = cache();
        cache.put_hit("deadbeef", b"\x89PNG-ish-bytes").unwrap();
        assert_eq!(
            cache.get("deadbeef", SystemTime::now()),
            Lookup::Hit(b"\x89PNG-ish-bytes".to_vec())
        );
    }

    #[test]
    fn a_negative_result_is_remembered() {
        let (_dir, cache) = cache();
        cache.put_negative("deadbeef").unwrap();
        assert_eq!(cache.get("deadbeef", SystemTime::now()), Lookup::Negative);
    }

    /// Red if the TTL assertion's `just_before` uses `+ HIT_TTL + 1s` instead
    /// of `- 1s`. A `<=` to `<` mutant is not distinguishable here (the two
    /// `now` values never land exactly on the boundary); the operator stays
    /// `<=` because "fresh up to and including the TTL" is the intended
    /// meaning.
    #[test]
    fn a_hit_expires_after_its_ttl() {
        let (_dir, cache) = cache();
        cache.put_hit("deadbeef", b"bytes").unwrap();
        let just_before = SystemTime::now() + HIT_TTL - Duration::from_secs(1);
        assert_eq!(
            cache.get("deadbeef", just_before),
            Lookup::Hit(b"bytes".to_vec()),
            "still inside the TTL"
        );
        let just_after = SystemTime::now() + HIT_TTL + Duration::from_secs(5);
        assert_eq!(
            cache.get("deadbeef", just_after),
            Lookup::Miss,
            "past the TTL, must be treated as absent so the caller refetches"
        );
    }

    #[test]
    fn a_negative_result_expires_on_its_own_shorter_ttl() {
        let (_dir, cache) = cache();
        cache.put_negative("deadbeef").unwrap();
        let just_after_negative_but_not_hit_ttl =
            SystemTime::now() + NEGATIVE_TTL + Duration::from_secs(5);
        assert_eq!(
            cache.get("deadbeef", just_after_negative_but_not_hit_ttl),
            Lookup::Miss
        );
    }

    #[test]
    fn a_later_hit_clears_an_earlier_negative_marker() {
        let (_dir, cache) = cache();
        cache.put_negative("deadbeef").unwrap();
        cache.put_hit("deadbeef", b"bytes").unwrap();
        // Not `Negative` even though the marker existed a moment ago: an
        // author who sets up a Gravatar mid-session must not stay stuck
        // behind their own old negative result until it expires.
        assert_eq!(
            cache.get("deadbeef", SystemTime::now()),
            Lookup::Hit(b"bytes".to_vec())
        );
    }

    #[test]
    fn different_hashes_do_not_collide() {
        let (_dir, cache) = cache();
        cache.put_hit("aaaa", b"one").unwrap();
        cache.put_hit("bbbb", b"two").unwrap();
        assert_eq!(
            cache.get("aaaa", SystemTime::now()),
            Lookup::Hit(b"one".to_vec())
        );
        assert_eq!(
            cache.get("bbbb", SystemTime::now()),
            Lookup::Hit(b"two".to_vec())
        );
    }

    /// Red if the two TTLs in `get_github` are swapped (the negative answer
    /// then lives a week).
    #[test]
    fn a_github_answer_is_remembered_and_expires_on_its_own_ttl() {
        let (_dir, cache) = cache();
        let now = SystemTime::now();
        assert_eq!(cache.get_github("aaaa", now), GitHubLookup::Miss);
        cache
            .put_github(
                "aaaa",
                Some("https://avatars.githubusercontent.com/u/1?v=4"),
            )
            .unwrap();
        cache.put_github("bbbb", None).unwrap();
        assert_eq!(
            cache.get_github("aaaa", now),
            GitHubLookup::Url("https://avatars.githubusercontent.com/u/1?v=4".into())
        );
        assert_eq!(cache.get_github("bbbb", now), GitHubLookup::NoAccount);
        let two_days = now + Duration::from_secs(2 * 24 * 60 * 60);
        assert_eq!(cache.get_github("bbbb", two_days), GitHubLookup::Miss);
        assert!(matches!(
            cache.get_github("aaaa", two_days),
            GitHubLookup::Url(_)
        ));
        let eight_days = now + Duration::from_secs(8 * 24 * 60 * 60);
        assert_eq!(cache.get_github("aaaa", eight_days), GitHubLookup::Miss);
    }

    #[test]
    fn a_clock_that_appears_to_move_backwards_still_serves_the_entry() {
        let (_dir, cache) = cache();
        cache.put_hit("deadbeef", b"bytes").unwrap();
        let earlier = SystemTime::now() - Duration::from_secs(60);
        assert_eq!(
            cache.get("deadbeef", earlier),
            Lookup::Hit(b"bytes".to_vec())
        );
    }
}
