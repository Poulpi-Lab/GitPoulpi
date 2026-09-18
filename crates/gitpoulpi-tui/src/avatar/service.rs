//! Fetching avatars off the input thread, with a disk cache in front of
//! the network and an in-memory cache in front of the disk.
//!
//! Same shape as `crate::clone_task` and `crate::repo_probe`: work that must
//! not block a frame, reported back on a channel the render loop drains with
//! `try_recv`. Here there can be **many** small jobs in flight at once (one
//! per distinct author visible in the Log), so this owns a background worker
//! with **bounded concurrency** instead of one thread per call.
//! **bounded concurrency** instead of one thread per call.
//!
//! Which sources are tried, in which order, and what is remembered about
//! each answer is `super::resolve`'s; this file is only the threading.
//!
//! ## Where each guarantee lives
//!
//! - **Off the input thread, never blocking a frame**: every network call,
//!   disk read and decode, and the GitHub token lookup (which may run `git
//!   credential fill`), happens on [`AvatarService::spawn`]'s background
//!   thread and runtime; the render loop only calls
//!   [`AvatarService::status`] / [`AvatarService::pictures`] (`HashMap`
//!   lookups, occasionally an unbounded channel `send`) and
//!   [`AvatarService::poll`] (an `mpsc::try_recv` loop), none of which can
//!   block.
//! - **Bounded concurrency and timeouts**: `resolve::ReqwestFetch`.
//! - **No retry storms**: there is no retry. A definite "none" is cached
//!   on disk; any answer, including "could not tell" (no network, a rate
//!   limit), is kept in memory for the rest of the session, so an author
//!   is looked up at most once per session.
//! - **Lazy**: an author is looked up the first time a row showing them
//!   is drawn -- only what is on screen costs anything.

use crate::redraw::NotifySend as _;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use super::bitbucket::BitbucketRepo;
use super::cache::AvatarCache;
use super::github::GitHubRepo;
use super::gitlab::GitLabHost;
use super::gravatar::email_hash;
use super::identicon;
use super::resolve::{self, Context, Fetch, Request, Resolution, TokenProvider};
use crate::terminal_image::Bitmap;
use gitpoulpi_config::{AvatarFallback, AvatarSources};

/// What is known about one author's avatar right now, from the render loop's
/// point of view. Never blocks to produce; a lookup not yet answered reads as
/// [`AvatarStatus::Loading`] and a request is kicked off (once) in the
/// background.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AvatarStatus<'a> {
    /// A decoded bitmap is ready, already resized to the caller's cell
    /// box.
    Ready(&'a Arc<Bitmap>),
    /// A fetch is in flight, or has not been asked for yet (asking is
    /// what this variant's caller triggers).
    Loading,
    /// No source has a picture for this author, or none could answer.
    /// The caller's initials fallback applies.
    Unavailable,
}

/// One row's question: whose picture, and a commit of theirs (what the
/// GitHub API is asked about).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AvatarQuery<'a> {
    pub(crate) email: &'a str,
    pub(crate) sha: &'a str,
}

enum WorkerEvent {
    Ready { hash: String, bitmap: Bitmap },
    Unavailable { hash: String },
}

/// Owns the background worker thread and every cache layer in front of
/// it. One instance per running TUI window, created alongside
/// `graph::GraphImage` (see `event_loop`).
pub(crate) struct AvatarService {
    jobs_tx: std_mpsc::Sender<Request>,
    events_rx: std_mpsc::Receiver<WorkerEvent>,
    /// Decoded bitmaps and negative results already known this session,
    /// keyed by Gravatar hash so two authors who happen to share an
    /// email (a bot committing as several names) share one entry.
    memory: HashMap<String, MemoryEntry>,
    /// Hashes a job has been sent for but no event has come back for yet -
    /// so `status` does not queue the same author twice while their
    /// first request is still in flight.
    inflight: HashSet<String>,
    /// Email -> Gravatar hash, memoised. The Log asks about every visible
    /// row on every frame (`pictures`), and a SHA-256 per row per frame
    /// would be work repeated for nothing.
    hashes: HashMap<String, String>,
    /// The window's github.com repository, from its remotes
    /// (`set_github_repo`). Copied into each job as it is sent.
    github_repo: Option<GitHubRepo>,
    /// Connected hosting (GitLab host, GitHub token) and the source order
    /// (`ui.avatar_sources`), copied into each job as it is sent.
    hosting: resolve::Hosting,
    /// `ui.avatar_fallback`.
    fallback: AvatarFallback,
    /// The identicon picture per author (Gravatar hash) whose lookup found
    /// nothing, generated on first use: a few thousand pixels of local
    /// arithmetic, cached, no I/O.
    identicons: HashMap<String, Arc<Bitmap>>,
    /// The picture box, as the resolver's.
    cell_px: u32,
    /// The resolver's own copy of `cell_px`, changed by `set_cell_px`.
    cell_px_shared: Arc<std::sync::atomic::AtomicU32>,
}

enum MemoryEntry {
    Ready(Arc<Bitmap>),
    Unavailable,
}

impl AvatarService {
    /// Starts the background worker over the real network. `cell_px` is the
    /// pixel box (already supersampled, matching
    /// `terminal_image::supersample`) every fetched avatar is downscaled to,
    /// decided once so every avatar in a session is the same size and the
    /// downscale cost is paid once per author. `github_token` is called at
    /// most once, on the worker, the first time the GitHub API is about to be
    /// asked.
    pub(crate) fn spawn(
        cache_dir: PathBuf,
        cell_px: u32,
        github_token: TokenProvider,
        gitlab_tokens: resolve::GitLabTokens,
    ) -> AvatarService {
        // Without `avatar-fetch` there is no HTTP client in the binary: the
        // pipeline runs over `Offline` (disk cache, then identicon/initials).
        #[cfg(feature = "avatar-fetch")]
        let fetch: Arc<dyn Fetch> = match resolve::ReqwestFetch::new() {
            Some(fetch) => Arc::new(fetch),
            // No HTTP client could be built (no TLS backend?): every
            // author falls back to the identicon or initials, and the
            // cache still serves.
            None => Arc::new(Offline),
        };
        #[cfg(not(feature = "avatar-fetch"))]
        let fetch: Arc<dyn Fetch> = Arc::new(Offline);
        Self::spawn_full(cache_dir, cell_px, fetch, github_token, gitlab_tokens)
    }

    /// [`AvatarService::spawn`] over any [`Fetch`] - the tests' canned
    /// network.
    #[cfg(test)]
    pub(crate) fn spawn_with(
        cache_dir: PathBuf,
        cell_px: u32,
        fetch: Arc<dyn Fetch>,
        github_token: TokenProvider,
    ) -> AvatarService {
        Self::spawn_full(cache_dir, cell_px, fetch, github_token, Box::new(|_| None))
    }

    /// [`AvatarService::spawn_with`] with GitLab tokens too.
    pub(crate) fn spawn_full(
        cache_dir: PathBuf,
        cell_px: u32,
        fetch: Arc<dyn Fetch>,
        github_token: TokenProvider,
        gitlab_tokens: resolve::GitLabTokens,
    ) -> AvatarService {
        let (jobs_tx, jobs_rx) = std_mpsc::channel::<Request>();
        let (events_tx, events_rx) = std_mpsc::channel::<WorkerEvent>();
        let ctx = Context::new(AvatarCache::new(cache_dir), fetch, cell_px, github_token)
            .with_gitlab_tokens(gitlab_tokens);
        let cell_px_shared = ctx.cell_px_handle();
        std::thread::spawn(move || worker_main(ctx, jobs_rx, events_tx));
        AvatarService {
            jobs_tx,
            events_rx,
            memory: HashMap::new(),
            inflight: HashSet::new(),
            hashes: HashMap::new(),
            github_repo: None,
            hosting: resolve::Hosting::default(),
            fallback: AvatarFallback::default(),
            identicons: HashMap::new(),
            cell_px,
            cell_px_shared,
        }
    }

    /// The terminal's zoom changed, so the on-screen size of a picture did:
    /// decode from now on at `cell_px`, and forget the pictures decoded at
    /// the old size so they are decoded again (from the disk cache, which
    /// keeps the large original). No-op when the box is unchanged.
    pub(crate) fn set_cell_px(&mut self, cell_px: u32) {
        if cell_px == self.cell_px {
            return;
        }
        self.cell_px = cell_px;
        self.cell_px_shared
            .store(cell_px, std::sync::atomic::Ordering::Relaxed);
        self.memory
            .retain(|_, entry| !matches!(entry, MemoryEntry::Ready(_)));
        self.identicons.clear();
    }

    /// Makes `email`'s picture ready in memory, as if the worker had decoded
    /// it: no job, no network. For tests of what is drawn with a picture.
    #[cfg(test)]
    pub(crate) fn seed_picture(&mut self, email: &str, bitmap: Bitmap) {
        let hash = email_hash(email);
        self.hashes.insert(email.to_string(), hash.clone());
        self.memory
            .insert(hash, MemoryEntry::Ready(Arc::new(bitmap)));
    }

    /// The picture box in use, in pixels.
    #[cfg(test)]
    pub(crate) fn cell_px(&self) -> u32 {
        self.cell_px
    }

    /// The connected hosting the resolver may ask (`gitlab`: a connected
    /// GitLab instance the repository's remote points at;
    /// `github_connected`: a github.com integration exists). Authors
    /// already answered keep their answer for the session.
    pub(crate) fn set_hosting(&mut self, gitlab: Option<GitLabHost>, github_connected: bool) {
        self.hosting.gitlab = gitlab;
        self.hosting.github_connected = github_connected;
    }

    /// The Bitbucket Cloud repository jobs sent from now on may ask the
    /// commits API about (`None`: no bitbucket.org remote, or no enabled
    /// Bitbucket integration).
    pub(crate) fn set_bitbucket(&mut self, bitbucket: Option<BitbucketRepo>) {
        self.hosting.bitbucket = bitbucket;
    }

    /// `ui.avatar_sources`. A change forgets the authors that had no
    /// picture, so the new order is tried for them.
    pub(crate) fn set_sources(&mut self, sources: AvatarSources) {
        if self.hosting.sources != sources {
            self.hosting.sources = sources;
            self.memory
                .retain(|_, entry| matches!(entry, MemoryEntry::Ready(_)));
            self.identicons.clear();
        }
    }

    /// `ui.avatar_fallback`.
    pub(crate) fn set_fallback(&mut self, fallback: AvatarFallback) {
        self.fallback = fallback;
    }

    /// The identicon picture for `hash`, when the fallback is the
    /// identicon and one has been generated.
    fn identicon(&self, hash: &str) -> Option<&Arc<Bitmap>> {
        (self.fallback == AvatarFallback::Identicon)
            .then(|| self.identicons.get(hash))
            .flatten()
    }

    /// Makes sure `email`'s identicon exists when the lookup for `hash`
    /// found nothing and the fallback is the identicon.
    fn ensure_identicon(&mut self, hash: &str, email: &str) {
        if self.fallback == AvatarFallback::Identicon
            && !self.inflight.contains(hash)
            && matches!(self.memory.get(hash), Some(MemoryEntry::Unavailable))
            && !self.identicons.contains_key(hash)
        {
            self.identicons.insert(
                hash.to_string(),
                Arc::new(identicon::bitmap(email, self.cell_px)),
            );
        }
    }

    /// The github.com repository jobs sent from now on may ask the
    /// GitHub API about (`None`: no github.com remote). Authors already
    /// answered keep their answer for the session.
    pub(crate) fn set_github_repo(&mut self, repo: Option<GitHubRepo>) {
        self.github_repo = repo;
    }

    /// Drains whatever the worker has finished since the last call.
    /// `O(events since last poll)`, never blocks - call once per frame,
    /// the same way `clone_task`/`repo_probe`'s receivers are drained.
    pub(crate) fn poll(&mut self) {
        while let Ok(event) = self.events_rx.try_recv() {
            match event {
                WorkerEvent::Ready { hash, bitmap } => {
                    self.inflight.remove(&hash);
                    self.memory
                        .insert(hash, MemoryEntry::Ready(Arc::new(bitmap)));
                }
                WorkerEvent::Unavailable { hash } => {
                    self.inflight.remove(&hash);
                    self.memory.insert(hash, MemoryEntry::Unavailable);
                }
            }
        }
    }

    /// The status for `query`'s author, kicking off a background fetch
    /// the first time it is asked about. Cheap enough to call every
    /// frame: a hit is one `HashMap` lookup, a miss sends one small job
    /// over an unbounded `std::sync::mpsc` channel (never blocks the
    /// sender) and returns [`AvatarStatus::Loading`] immediately.
    pub(crate) fn status(&mut self, query: AvatarQuery<'_>) -> AvatarStatus<'_> {
        let hash = self.request(query.email, &[query.sha]);
        if self.inflight.contains(&hash) {
            return AvatarStatus::Loading;
        }
        self.ensure_identicon(&hash, query.email);
        match self.memory.get(&hash) {
            Some(MemoryEntry::Ready(bitmap)) => AvatarStatus::Ready(bitmap),
            Some(MemoryEntry::Unavailable) => match self.identicon(&hash) {
                Some(bitmap) => AvatarStatus::Ready(bitmap),
                None => AvatarStatus::Unavailable,
            },
            None => AvatarStatus::Loading,
        }
    }

    /// The Log's question, for every visible row at once: each author's
    /// decoded picture and its identity (the Gravatar hash) when ready,
    /// `None` while loading or when there is none. Asks for every author not
    /// known yet, like [`AvatarService::status`]: memory only, never a disk
    /// read or a network call on this thread.
    ///
    /// An author seen for the first time is sent with every sha of theirs in
    /// `rows`, oldest first: the GitHub API is asked about the oldest visible
    /// one first, the one most likely to be pushed.
    ///
    /// One call for all rows because the answers borrow from `self`, so a
    /// caller collecting them one `&mut self` call at a time could not hold
    /// two at once.
    pub(crate) fn pictures<'s>(
        &'s mut self,
        rows: &[AvatarQuery<'_>],
    ) -> Vec<Option<(&'s str, &'s Arc<Bitmap>)>> {
        let hashes: Vec<String> = rows
            .iter()
            .map(|row| {
                let shas: Vec<&str> = rows
                    .iter()
                    .rev()
                    .filter(|other| other.email.eq_ignore_ascii_case(row.email))
                    .map(|other| other.sha)
                    .collect();
                let hash = self.request(row.email, &shas);
                self.ensure_identicon(&hash, row.email);
                hash
            })
            .collect();
        let this: &'s AvatarService = self;
        hashes
            .iter()
            .map(|hash| match this.memory.get_key_value(hash) {
                Some((hash, MemoryEntry::Ready(bitmap))) => Some((hash.as_str(), bitmap)),
                Some((hash, MemoryEntry::Unavailable)) => {
                    this.identicon(hash).map(|bitmap| (hash.as_str(), bitmap))
                }
                None => None,
            })
            .collect()
    }

    /// `email`'s Gravatar hash, sending a job for it the first time it is
    /// neither known nor in flight.
    fn request(&mut self, email: &str, shas: &[&str]) -> String {
        let hash = match self.hashes.get(email) {
            Some(hash) => hash.clone(),
            None => {
                let hash = email_hash(email);
                self.hashes.insert(email.to_string(), hash.clone());
                hash
            }
        };
        if !self.inflight.contains(&hash) && !self.memory.contains_key(&hash) {
            self.inflight.insert(hash.clone());
            let mut unique: Vec<String> = Vec::new();
            for sha in shas {
                if !sha.is_empty() && !unique.iter().any(|s| s == sha) {
                    unique.push(sha.to_string());
                }
            }
            unique.truncate(resolve::MAX_SHAS_PER_AUTHOR);
            // `send` only fails if the worker thread panicked, in which
            // case there is no receiver left to notice and nothing this
            // call can do beyond not panicking itself -- the author
            // simply stays on the initials fallback for the rest of the
            // session.
            let _ = self.jobs_tx.send(Request {
                hash: hash.clone(),
                email: email.to_string(),
                shas: unique,
                repo: self.github_repo.clone(),
                hosting: self.hosting.clone(),
            });
        }
        hash
    }
}

/// The network when no HTTP client could be built: every request fails.
struct Offline;

impl Fetch for Offline {
    fn get<'a>(&'a self, _url: &'a str, _target: resolve::Target<'a>) -> resolve::FetchFuture<'a> {
        Box::pin(async { None })
    }
}

fn worker_main(
    ctx: Context,
    jobs_rx: std_mpsc::Receiver<Request>,
    events_tx: std_mpsc::Sender<WorkerEvent>,
) {
    // Multi-threaded on purpose: a current-thread runtime only progresses
    // while something on *this* thread is inside `block_on`, and this
    // thread sits in a blocking `jobs_rx.recv()` between jobs
    // (`std::sync::mpsc` has no async receiver to `select!` against). A
    // multi-thread runtime's own workers drive spawned tasks independently,
    // so `runtime.spawn` below runs immediately.
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        // Nothing to report to and no frame to block; every author falls
        // back to initials for the rest of the session.
        Err(_) => return,
    };
    let ctx = Arc::new(ctx);

    // Every job becomes its own task, so many distinct authors are in
    // flight *concurrently* rather than one after another -- the fetcher's
    // semaphore, not this loop, bounds how many reach the network at once.
    while let Ok(job) = jobs_rx.recv() {
        let ctx = Arc::clone(&ctx);
        let events_tx = events_tx.clone();
        runtime.spawn(async move {
            let event = match resolve::resolve(&job, &ctx, SystemTime::now()).await {
                Resolution::Found(bitmap) => WorkerEvent::Ready {
                    hash: job.hash,
                    bitmap,
                },
                Resolution::None | Resolution::Unknown => {
                    WorkerEvent::Unavailable { hash: job.hash }
                }
            };
            let _ = events_tx.send_notify(event);
        });
    }
    // `jobs_rx.recv()` returned `Err`: every `Job` sender is gone, which
    // means the `AvatarService` (and the TUI it belonged to) was dropped.
    // Give whatever tasks are still in flight a bounded window to finish
    // writing to the disk cache rather than aborting them mid-write.
    runtime.shutdown_timeout(Duration::from_secs(2));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avatar::resolve::tests::{png_of, Canned};

    fn no_token() -> TokenProvider {
        Box::new(|| None)
    }

    fn wait_for<T>(mut probe: impl FnMut() -> Option<T>) -> T {
        // Generous on purpose: each test spawns a real OS thread plus a
        // 2-worker Tokio runtime, and under a full parallel `cargo test` load
        // that setup can be scheduled well behind where it would land alone.
        // The probe returns the instant the picture arrives, so a generous
        // ceiling costs nothing on the common path.
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(found) = probe() {
                return found;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "the picture never arrived"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn row<'a>(email: &'a str, sha: &'a str) -> AvatarQuery<'a> {
        AvatarQuery { email, sha }
    }

    /// The Log's path: a disk-cached author reads as "no picture yet" on the
    /// first frame (the frame never waits on the disk), then as the picture
    /// once the worker has read and decoded it; an author the cache knows has
    /// none stays on the initials. Both come from the cache: the canned
    /// network has no rules, so any request would fail.
    ///
    /// Red if `pictures` answers `None` for every hash (the poll loop times
    /// out).
    #[test]
    fn a_cached_author_becomes_a_picture_without_blocking_the_frame() {
        let dir = tempfile::tempdir().unwrap();
        let cache = AvatarCache::new(dir.path().to_path_buf());
        cache
            .put_hit(&email_hash("ada@example.com"), &png_of((200, 10, 10)))
            .unwrap();
        cache
            .put_negative(&email_hash("nobody@example.com"))
            .unwrap();
        let fetch = Arc::new(Canned::default());
        let mut service =
            AvatarService::spawn_with(dir.path().to_path_buf(), 16, fetch.clone(), no_token());
        service.set_fallback(AvatarFallback::Initials);

        let rows = [
            row("ada@example.com", "a1"),
            row("nobody@example.com", "n1"),
        ];
        let first = service.pictures(&rows);
        assert!(
            first.iter().all(Option::is_none),
            "the first frame answered from the disk"
        );

        wait_for(|| {
            service.poll();
            let now = service.pictures(&rows);
            let (hash, bitmap) = now[0]?;
            assert_eq!(hash, email_hash("ada@example.com"));
            assert_eq!((bitmap.width, bitmap.height), (16, 16));
            assert_eq!(bitmap.pixel(8, 8), Some((200, 10, 10)));
            assert!(now[1].is_none(), "a known miss must stay on the initials");
            Some(())
        });
        assert_eq!(fetch.calls.lock().unwrap().len(), 0);
    }

    /// A zoom changes the picture box: pictures decoded for the old box
    /// are forgotten and come back from the disk cache at the new size.
    #[test]
    fn a_new_picture_box_decodes_the_cached_author_again_at_that_size() {
        let dir = tempfile::tempdir().unwrap();
        let cache = AvatarCache::new(dir.path().to_path_buf());
        cache
            .put_hit(&email_hash("ada@example.com"), &png_of((200, 10, 10)))
            .unwrap();
        let fetch = Arc::new(Canned::default());
        let mut service =
            AvatarService::spawn_with(dir.path().to_path_buf(), 16, fetch, no_token());
        let rows = [row("ada@example.com", "a1")];
        let first = wait_for(|| {
            service.poll();
            service.pictures(&rows)[0].map(|(_, b)| b.width)
        });
        assert_eq!(first, 16);
        service.set_cell_px(32);
        assert_eq!(service.cell_px(), 32);
        let second = wait_for(|| {
            service.poll();
            service.pictures(&rows)[0].map(|(_, b)| b.width)
        });
        assert_eq!(second, 32, "the picture was not decoded at the new box");
    }

    /// An author no source knows gets the identicon as a picture (the
    /// default fallback), the same one every time; with `initials` they
    /// get none and the caller's badge shows.
    #[test]
    fn an_author_without_a_picture_gets_the_identicon() {
        let dir = tempfile::tempdir().unwrap();
        let cache = AvatarCache::new(dir.path().to_path_buf());
        cache
            .put_negative(&email_hash("nobody@example.com"))
            .unwrap();
        let fetch = Arc::new(Canned::default());
        let mut service =
            AvatarService::spawn_with(dir.path().to_path_buf(), 40, fetch, no_token());
        let rows = [row("nobody@example.com", "n1")];
        let picture = wait_for(|| {
            service.poll();
            service.pictures(&rows)[0].map(|(_, b)| b.clone())
        });
        assert_eq!((picture.width, picture.height), (40, 40));
        assert_eq!(
            picture.data,
            identicon::bitmap("nobody@example.com", 40).data
        );
        assert!(matches!(
            service.status(AvatarQuery {
                email: "nobody@example.com",
                sha: "n1"
            }),
            AvatarStatus::Ready(_)
        ));
        service.set_fallback(AvatarFallback::Initials);
        assert!(service.pictures(&rows)[0].is_none());
    }

    /// End to end over the canned network: no Gravatar, a github.com
    /// remote set on the service, and the author's picture lands --
    /// asked about the *oldest* of their visible commits first.
    #[test]
    #[cfg(feature = "github")]
    fn a_github_avatar_arrives_through_the_service() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on("gravatar.com", 404, b"")
                .on(
                    "api.github.com/repos/o/r/commits/old",
                    200,
                    br#"{"author":{"avatar_url":"https://avatars.githubusercontent.com/u/4?v=4"}}"#,
                )
                .on("avatars.githubusercontent.com/u/4", 200, &png_of((4, 4, 4))),
        );
        let mut service =
            AvatarService::spawn_with(dir.path().to_path_buf(), 16, fetch.clone(), no_token());
        service.set_github_repo(Some(GitHubRepo {
            owner: "o".into(),
            name: "r".into(),
        }));
        let rows = [
            row("me@example.com", "new"),
            row("other@example.com", "x"),
            row("me@example.com", "old"),
        ];
        let color = wait_for(|| {
            service.poll();
            service.pictures(&rows)[0].and_then(|(_, b)| b.pixel(8, 8))
        });
        assert_eq!(color, (4, 4, 4));
        // `me@example.com` and `other@example.com` are two independent jobs
        // on two worker threads (`worker_threads(2)` in `worker_main`) with no
        // ordering guarantee. The oldest-first guarantee under test is scoped
        // to *one* author's own shas, so the assertion narrows to
        // `me@example.com`'s two candidate URLs (`/commits/old`,
        // `/commits/new`) and ignores the unrelated job.
        let api_calls: Vec<String> = fetch
            .calls
            .lock()
            .unwrap()
            .iter()
            .map(|(u, _)| u.clone())
            .filter(|u| u.contains("/commits/old") || u.contains("/commits/new"))
            .collect();
        assert!(
            api_calls.first().is_some_and(|u| u.ends_with("/old")),
            "{api_calls:?}"
        );
    }
}
