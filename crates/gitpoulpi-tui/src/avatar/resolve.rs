//! Finding one author's picture: which sources, in which order, and what
//! is remembered about each answer. Runs on the avatar worker's runtime
//! (`service`), never on the input thread.
//!
//! ## The order, and why
//!
//! 1. **The disk cache** (`cache`): a picture or a definite "none" still
//!    fresh answers without any network.
//! 2. **A GitHub noreply address** (`github::noreply_avatar_url`): the
//!    address names the account, so the picture URL is known without an
//!    API call -- and Gravatar cannot know a noreply address anyway.
//! 3. **Gravatar** (`gravatar`, `d=404`): free and not rate limited.
//! 4. **The GitHub REST commits API** (`github`), only in a repository
//!    with a github.com remote and only for a commit sha by this author:
//!    last, because it is the one source with a budget (60 calls an hour
//!    unauthenticated, 5000 with a token). One call per author, its
//!    answer cached on disk for a week (a day for "no account").
//!
//! ## Only a definite answer is remembered
//!
//! A source that answers 404 (no Gravatar, no account) is a "none". A
//! source that fails (no network, TLS, a timeout, a 5xx, GitHub's rate
//! limit, a repository the API cannot see) is "unknown", and a lookup with
//! any unknown in it writes **nothing** to disk: the author stays on the
//! initials for this session and is asked again next time. Caching a
//! failure as "none" would make a TLS error look like "nobody has an
//! avatar" for hours.
//!
//! ## Testable without a network
//!
//! Every request goes through [`Fetch`], which the service fills with
//! [`ReqwestFetch`] and the tests with a canned table.

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;
#[cfg(any(feature = "avatar-fetch", test))]
use std::time::{Duration, UNIX_EPOCH};

use super::bitbucket::{self, Auth, BitbucketRepo};
use super::cache::{AvatarCache, GitHubLookup, Lookup};
use super::github::{self, GitHubRepo};
use super::gitlab::{self, GitLabHost};
use super::gravatar;
use crate::image_decode as decode;
use crate::terminal_image::Bitmap;
use gitpoulpi_config::AvatarSources;

/// A hung DNS lookup or a slow host must not occupy a permit
/// indefinitely; the Log has already moved on by then.
#[cfg(feature = "avatar-fetch")]
pub(crate) const FETCH_TIMEOUT: Duration = Duration::from_secs(5);

/// However many Log rows are visible, the ceiling on simultaneous
/// outbound requests - deliberately small (third-party hosts whose
/// limits this project does not control).
#[cfg(feature = "avatar-fetch")]
const MAX_CONCURRENT_FETCHES: usize = 4;

/// How many of an author's commits the GitHub API is asked about before
/// giving up: a commit that was never pushed answers 422, and the
/// author's newest commits - the ones on screen first - are the likely
/// unpushed ones.
pub(crate) const MAX_SHAS_PER_AUTHOR: usize = 3;

/// When GitHub says "rate limited" without saying until when.
#[cfg(any(feature = "avatar-fetch", test))]
const DEFAULT_RATE_LIMIT_PAUSE: Duration = Duration::from_secs(60 * 60);

/// What one request is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target<'a> {
    /// A picture: no credentials, whatever image host the URL names.
    Image,
    /// A GitHub REST call: the JSON media type and API version headers,
    /// and the token when there is one. The token is only ever sent to
    /// `api.github.com`, never to an image host.
    GitHubApi { token: Option<&'a str> },
    /// A GitLab REST call (JSON, bearer token when there is one). The
    /// token only ever goes to the connected host the URL was built from.
    GitLabApi { token: Option<&'a str> },
    /// A Bitbucket Cloud REST call (JSON). The credentials only ever go to
    /// `api.bitbucket.org`, the host the URL is built on.
    BitbucketApi { auth: Option<&'a Auth> },
}

/// A response, reduced to what the pipeline reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Fetched {
    pub(crate) status: u16,
    pub(crate) body: Vec<u8>,
    /// Set when the answer is GitHub's rate limit (see
    /// [`rate_limited_until`]).
    pub(crate) rate_limited_until: Option<SystemTime>,
}

pub(crate) type FetchFuture<'a> = Pin<Box<dyn Future<Output = Option<Fetched>> + Send + 'a>>;

/// The network, as the pipeline sees it. `None` is a transport failure
/// (no connection, TLS, timeout): "unknown", never "none".
pub(crate) trait Fetch: Send + Sync {
    fn get<'a>(&'a self, url: &'a str, target: Target<'a>) -> FetchFuture<'a>;
}

/// Where the GitHub token comes from, asked at most once per session and
/// only when the GitHub API is actually about to be called (it may run
/// `git credential fill`).
pub(crate) type TokenProvider = Box<dyn FnOnce() -> Option<String> + Send>;

/// One author to find a picture for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Request {
    /// The Gravatar hash of the email: the cache key and the identity.
    pub(crate) hash: String,
    pub(crate) email: String,
    /// Commits by this author, in the order to ask GitHub about them.
    pub(crate) shas: Vec<String>,
    /// The window's github.com repository, if it has one.
    pub(crate) repo: Option<GitHubRepo>,
    /// The connected hosting integrations and the source order
    /// (`ui.avatar_sources`).
    pub(crate) hosting: Hosting,
}

/// What the resolver knows about the window's hosting, beyond the
/// github.com repository.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Hosting {
    /// A connected GitLab instance the repository's remote points at.
    pub(crate) gitlab: Option<GitLabHost>,
    /// The window's Bitbucket Cloud repository, when an enabled Bitbucket
    /// integration is connected.
    pub(crate) bitbucket: Option<BitbucketRepo>,
    /// A github.com integration is connected (its token is available, so
    /// the user search is allowed after the commits API).
    pub(crate) github_connected: bool,
    pub(crate) sources: AvatarSources,
}

impl Hosting {
    /// Whether an integration matching the repository's host is
    /// connected: the hosting API goes before Gravatar when the order
    /// says so.
    fn connected(&self, repo: &Option<GitHubRepo>) -> bool {
        self.gitlab.is_some()
            || self.bitbucket.is_some()
            || (self.github_connected && repo.is_some())
    }
}

/// Prefix of the on-disk key a GitLab answer is remembered under, so a
/// GitHub "no account" never hides a GitLab account (same email, other
/// host).
const GITLAB_KEY_PREFIX: &str = "gl-";

/// The same, for Bitbucket Cloud.
const BITBUCKET_KEY_PREFIX: &str = "bb-";

/// What a lookup concluded.
#[derive(Debug)]
pub(crate) enum Resolution {
    /// Decoded and downscaled to the service's cell box.
    Found(Bitmap),
    /// Every source answered "none"; remembered on disk.
    None,
    /// Some source could not answer; nothing remembered.
    Unknown,
}

/// The sources [`resolve`] tries, in order, after the disk cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Source {
    /// A picture URL known up front: a noreply address, or Gravatar.
    Picture(String),
    /// Ask the GitHub commits API which account this author is.
    /// Also, with a token, the user search by email.
    GitHubCommits { repo: GitHubRepo, shas: Vec<String> },
    /// Ask the connected GitLab instance which account owns this email.
    GitLab { host: GitLabHost },
    /// Ask the Bitbucket Cloud commits API which account authored one of
    /// these commits.
    BitbucketCommits {
        repo: Box<BitbucketRepo>,
        shas: Vec<String>,
    },
}

/// The source order for `req` (see the module header for why, and
/// [`AvatarSources`] for the presets).
pub(crate) fn plan(req: &Request, size: u32) -> Vec<Source> {
    let order = req.hosting.sources;
    let mut sources = Vec::with_capacity(4);
    // GitHub and GitLab as avatar sources need their provider built
    // (custom builds); Gravatar and the initials badge never do.
    if crate::build_features::GITHUB {
        if let Some(url) = github::noreply_avatar_url(&req.email, size) {
            sources.push(Source::Picture(url));
        }
    }
    let gravatar = order
        .uses_gravatar()
        .then(|| Source::Picture(gravatar::avatar_url(&req.email, size)));
    let mut hosting = Vec::new();
    if order.uses_hosting() {
        if let Some(repo) = req.repo.as_ref().filter(|_| crate::build_features::GITHUB) {
            if !req.shas.is_empty() || req.hosting.github_connected {
                hosting.push(Source::GitHubCommits {
                    repo: repo.clone(),
                    shas: req.shas.iter().take(MAX_SHAS_PER_AUTHOR).cloned().collect(),
                });
            }
        }
        if let Some(host) = req
            .hosting
            .gitlab
            .as_ref()
            .filter(|_| crate::build_features::GITLAB)
        {
            hosting.push(Source::GitLab { host: host.clone() });
        }
        if let Some(repo) = req
            .hosting
            .bitbucket
            .as_ref()
            .filter(|_| crate::build_features::BITBUCKET)
            .filter(|_| !req.shas.is_empty())
        {
            hosting.push(Source::BitbucketCommits {
                repo: Box::new(repo.clone()),
                shas: req.shas.iter().take(MAX_SHAS_PER_AUTHOR).cloned().collect(),
            });
        }
    }
    // With no matching integration connected, `hosting_gravatar` keeps
    // the pre-integration order (Gravatar first, the unauthenticated
    // GitHub API last).
    if order.hosting_first() && req.hosting.connected(&req.repo) {
        sources.extend(hosting);
        sources.extend(gravatar);
    } else {
        sources.extend(gravatar);
        sources.extend(hosting);
    }
    sources
}

/// What GitHub is known not to answer this session.
#[derive(Debug, Default)]
struct GitHubSession {
    /// A rate limit is in force until then.
    blocked_until: Option<SystemTime>,
    /// Repositories (by slug) the API answered 401/403/404 for: private
    /// without a (good) token. Not asked again this session.
    unreachable: HashSet<String>,
    /// Bitbucket's rolling-hour rate limit is in force until then.
    bitbucket_blocked_until: Option<SystemTime>,
}

/// How many pixels to ask the host for, given the picture box every avatar
/// ends up in (`Context::cell_px`).
///
/// **Several times the box, not the box.** The picture is averaged down to
/// the box ([`decode::downscale`]), and an average of a larger source is
/// what makes a face at 19 px read as a face instead of a smudge; asking for
/// exactly the box size would hand us a picture the terminal resamples a
/// second time. At least 128 px (Gravatar/GitHub both serve those from
/// cache), at most 460 (GitHub's cap, `github::sized`).
pub(crate) fn fetch_size(box_px: u32) -> u32 {
    (box_px * 3).clamp(128, 460)
}

/// Everything every lookup shares: one per worker.
pub(crate) struct Context {
    pub(crate) cache: AvatarCache,
    pub(crate) fetch: Arc<dyn Fetch>,
    /// The square pixel box every picture is downscaled to: the graph
    /// node's picture diameter in bitmap pixels. Shared with the
    /// `AvatarService`, which changes it when the terminal's zoom does.
    cell_px: Arc<std::sync::atomic::AtomicU32>,
    token_provider: Mutex<Option<TokenProvider>>,
    token: OnceLock<Option<String>>,
    /// The GitLab token for a host (`base`), asked once per host.
    gitlab_tokens: GitLabTokens,
    gitlab_token_cache: Mutex<std::collections::HashMap<String, Option<String>>>,
    session: Mutex<GitHubSession>,
    /// Credentials of a Bitbucket integration (by id), asked once each.
    bitbucket_auth: BitbucketAuths,
    bitbucket_auth_cache: Mutex<std::collections::HashMap<String, Option<Auth>>>,
}

/// Where a Bitbucket integration's credentials come from, asked once per
/// integration, on the worker.
pub(crate) type BitbucketAuths =
    Box<dyn Fn(&gitpoulpi_config::Integration) -> Option<Auth> + Send + Sync>;

/// Where a GitLab host's token comes from (the integration whose URL is
/// that host), asked once per host, on the worker.
pub(crate) type GitLabTokens = Box<dyn Fn(&str) -> Option<String> + Send + Sync>;

impl Context {
    pub(crate) fn new(
        cache: AvatarCache,
        fetch: Arc<dyn Fetch>,
        cell_px: u32,
        token: TokenProvider,
    ) -> Context {
        Context {
            cache,
            fetch,
            cell_px: Arc::new(std::sync::atomic::AtomicU32::new(cell_px)),
            token_provider: Mutex::new(Some(token)),
            token: OnceLock::new(),
            gitlab_tokens: Box::new(|_| None),
            gitlab_token_cache: Mutex::new(Default::default()),
            session: Mutex::new(GitHubSession::default()),
            bitbucket_auth: Box::new(bitbucket::auth_of),
            bitbucket_auth_cache: Mutex::new(Default::default()),
        }
    }

    /// The current picture box, in pixels.
    pub(crate) fn cell_px(&self) -> u32 {
        self.cell_px.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// A handle that changes [`Context::cell_px`] from another thread.
    pub(crate) fn cell_px_handle(&self) -> Arc<std::sync::atomic::AtomicU32> {
        Arc::clone(&self.cell_px)
    }

    pub(crate) fn with_gitlab_tokens(mut self, tokens: GitLabTokens) -> Context {
        self.gitlab_tokens = tokens;
        self
    }

    #[cfg(all(test, feature = "bitbucket"))]
    pub(crate) fn with_bitbucket_auth(mut self, auths: BitbucketAuths) -> Context {
        self.bitbucket_auth = auths;
        self
    }

    fn bitbucket_auth(&self, integration: &gitpoulpi_config::Integration) -> Option<Auth> {
        let mut cache = self
            .bitbucket_auth_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        cache
            .entry(integration.id.clone())
            .or_insert_with(|| (self.bitbucket_auth)(integration))
            .clone()
    }

    fn gitlab_token(&self, base: &str) -> Option<String> {
        let mut cache = self
            .gitlab_token_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        cache
            .entry(base.to_string())
            .or_insert_with(|| (self.gitlab_tokens)(base).filter(|t| !t.trim().is_empty()))
            .clone()
    }

    fn token(&self) -> Option<&str> {
        self.token
            .get_or_init(|| {
                let provider = self.token_provider.lock().ok()?.take()?;
                provider().filter(|t| !t.trim().is_empty())
            })
            .as_deref()
    }

    fn session(&self) -> std::sync::MutexGuard<'_, GitHubSession> {
        // A poisoned lock only means another lookup panicked mid-update;
        // the data (a timestamp and a set) is still usable.
        self.session.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Finds `req`'s picture: cache, then [`plan`]'s sources in order.
pub(crate) async fn resolve(req: &Request, ctx: &Context, now: SystemTime) -> Resolution {
    match ctx.cache.get(&req.hash, now) {
        Lookup::Hit(bytes) => {
            if let Some(bitmap) = decode::decode(&bytes) {
                return Resolution::Found(decode::downscale(&bitmap, ctx.cell_px(), ctx.cell_px()));
            }
            // A corrupt cache file: fetch it again.
        }
        Lookup::Negative => return Resolution::None,
        Lookup::Miss => {}
    }

    let mut unknown = false;
    for source in plan(req, fetch_size(ctx.cell_px())) {
        let url = match source {
            Source::Picture(url) => url,
            Source::GitHubCommits { repo, shas } => {
                match github_avatar_url(req, &repo, &shas, ctx, now).await {
                    Answer::Url(url) => github::sized(&url, fetch_size(ctx.cell_px())),
                    Answer::NoAccount => continue,
                    Answer::Unknown => {
                        unknown = true;
                        continue;
                    }
                }
            }
            Source::BitbucketCommits { repo, shas } => {
                match bitbucket_avatar_url(req, &repo, &shas, ctx, now).await {
                    Answer::Url(url) => url,
                    Answer::NoAccount => continue,
                    Answer::Unknown => {
                        unknown = true;
                        continue;
                    }
                }
            }
            Source::GitLab { host } => match gitlab_avatar_url(req, &host, ctx, now).await {
                Answer::Url(url) => url,
                Answer::NoAccount => continue,
                Answer::Unknown => {
                    unknown = true;
                    continue;
                }
            },
        };
        match ctx.fetch.get(&url, Target::Image).await {
            Some(f) if (200..300).contains(&f.status) => {
                if let Some(bitmap) = decode::decode(&f.body) {
                    let _ = ctx.cache.put_hit(&req.hash, &f.body);
                    return Resolution::Found(decode::downscale(
                        &bitmap,
                        ctx.cell_px(),
                        ctx.cell_px(),
                    ));
                }
                // A 200 that is not a PNG/JPEG (an HTML error page, a
                // WebP): nothing usable from this source.
            }
            Some(f) if f.status == 404 || f.status == 410 => {}
            _ => unknown = true,
        }
    }
    if unknown {
        Resolution::Unknown
    } else {
        let _ = ctx.cache.put_negative(&req.hash);
        Resolution::None
    }
}

enum Answer {
    Url(String),
    NoAccount,
    Unknown,
}

/// The author's GitHub `avatar_url`: the disk cache, else one API call
/// per sha until one is a commit GitHub knows.
async fn github_avatar_url(
    req: &Request,
    repo: &GitHubRepo,
    shas: &[String],
    ctx: &Context,
    now: SystemTime,
) -> Answer {
    match ctx.cache.get_github(&req.hash, now) {
        GitHubLookup::Url(url) => return Answer::Url(url),
        GitHubLookup::NoAccount => return Answer::NoAccount,
        GitHubLookup::Miss => {}
    }
    let slug = repo.slug();
    {
        let session = ctx.session();
        if session.blocked_until.is_some_and(|until| until > now) {
            return Answer::Unknown;
        }
        if session.unreachable.contains(&slug) {
            return Answer::Unknown;
        }
    }
    let token = ctx.token();
    for sha in shas {
        let url = github::commit_api_url(repo, sha);
        let Some(response) = ctx.fetch.get(&url, Target::GitHubApi { token }).await else {
            return Answer::Unknown;
        };
        if let Some(until) = response.rate_limited_until {
            ctx.session().blocked_until = Some(until);
            return Answer::Unknown;
        }
        match response.status {
            200 => match github::author_avatar_url(&response.body) {
                Some(url) => {
                    let _ = ctx.cache.put_github(&req.hash, Some(&url));
                    return Answer::Url(url);
                }
                None => {
                    // GitHub linked no account to this commit's email.
                    // With a connected token, the user search may still
                    // know the address; without, that is the answer.
                    if !search_allowed(req, token) {
                        let _ = ctx.cache.put_github(&req.hash, None);
                        return Answer::NoAccount;
                    }
                    return match github_user_search(req, ctx, token).await {
                        Answer::Url(url) => Answer::Url(url),
                        Answer::NoAccount => {
                            let _ = ctx.cache.put_github(&req.hash, None);
                            Answer::NoAccount
                        }
                        Answer::Unknown => Answer::Unknown,
                    };
                }
            },
            // "No commit found for SHA": never pushed. Try the next one.
            422 => continue,
            401 | 403 | 404 => {
                ctx.session().unreachable.insert(slug);
                return Answer::Unknown;
            }
            _ => return Answer::Unknown,
        }
    }
    // No commit of theirs is on GitHub (or none was offered): a connected
    // token can still find them by email.
    if search_allowed(req, token) {
        if let Answer::Url(url) = github_user_search(req, ctx, token).await {
            return Answer::Url(url);
        }
    }
    Answer::Unknown
}

/// The user search needs a connected github.com integration's token.
fn search_allowed(req: &Request, token: Option<&str>) -> bool {
    req.hosting.github_connected && token.is_some()
}

/// `GET /search/users?q=<email> in:email` -> the first account's picture.
/// A definite `NoAccount` on 200 with no hit; rate limits are shared with
/// the commits API's.
async fn github_user_search(req: &Request, ctx: &Context, token: Option<&str>) -> Answer {
    let url = github::search_users_url(&req.email);
    let Some(response) = ctx.fetch.get(&url, Target::GitHubApi { token }).await else {
        return Answer::Unknown;
    };
    if let Some(until) = response.rate_limited_until {
        ctx.session().blocked_until = Some(until);
        return Answer::Unknown;
    }
    match response.status {
        200 => match github::search_avatar_url(&response.body) {
            Some(url) => {
                let _ = ctx.cache.put_github(&req.hash, Some(&url));
                Answer::Url(url)
            }
            None => Answer::NoAccount,
        },
        _ => Answer::Unknown,
    }
}

/// The author's account picture on the connected GitLab host: the disk
/// cache (under a `gl-` key), then `/avatar?email=`, then, with a token,
/// the users API.
async fn gitlab_avatar_url(
    req: &Request,
    host: &GitLabHost,
    ctx: &Context,
    now: SystemTime,
) -> Answer {
    let key = format!("{GITLAB_KEY_PREFIX}{}", req.hash);
    match ctx.cache.get_github(&key, now) {
        GitHubLookup::Url(url) => return Answer::Url(url),
        GitHubLookup::NoAccount => return Answer::NoAccount,
        GitHubLookup::Miss => {}
    }
    if ctx.session().unreachable.contains(&host.base) {
        return Answer::Unknown;
    }
    let token = ctx.gitlab_token(&host.base);
    let token = token.as_deref();
    let size = fetch_size(ctx.cell_px());
    let mut urls = vec![gitlab::avatar_api_url(host, &req.email, size)];
    if token.is_some() {
        urls.push(gitlab::users_search_url(host, &req.email));
    }
    for url in urls {
        let Some(response) = ctx.fetch.get(&url, Target::GitLabApi { token }).await else {
            return Answer::Unknown;
        };
        match response.status {
            200 => {
                if let Some(picture) = gitlab::avatar_url(host, &response.body) {
                    let _ = ctx.cache.put_github(&key, Some(&picture));
                    return Answer::Url(picture);
                }
            }
            401 | 403 => {
                ctx.session().unreachable.insert(host.base.clone());
                return Answer::Unknown;
            }
            404 => {}
            _ => return Answer::Unknown,
        }
    }
    let _ = ctx.cache.put_github(&key, None);
    Answer::NoAccount
}

/// The author's Bitbucket Cloud account picture: the disk cache (under a
/// `bb-` key), else one commits API call per sha until one is a commit
/// Bitbucket knows. Anonymous when no token is stored (public repositories
/// only); a 401/403/404 marks the repository unreachable for the session
/// and a 429 pauses Bitbucket until it resets.
async fn bitbucket_avatar_url(
    req: &Request,
    repo: &BitbucketRepo,
    shas: &[String],
    ctx: &Context,
    now: SystemTime,
) -> Answer {
    let key = format!("{BITBUCKET_KEY_PREFIX}{}", req.hash);
    match ctx.cache.get_github(&key, now) {
        GitHubLookup::Url(url) => return Answer::Url(url),
        GitHubLookup::NoAccount => return Answer::NoAccount,
        GitHubLookup::Miss => {}
    }
    {
        let session = ctx.session();
        if session
            .bitbucket_blocked_until
            .is_some_and(|until| until > now)
            || session.unreachable.contains(&repo.key())
        {
            return Answer::Unknown;
        }
    }
    let auth = ctx.bitbucket_auth(&repo.integration);
    for sha in shas {
        let url = bitbucket::commit_api_url(repo, sha);
        let Some(response) = ctx
            .fetch
            .get(
                &url,
                Target::BitbucketApi {
                    auth: auth.as_ref(),
                },
            )
            .await
        else {
            return Answer::Unknown;
        };
        if let Some(until) = response.rate_limited_until {
            ctx.session().bitbucket_blocked_until = Some(until);
            return Answer::Unknown;
        }
        match response.status {
            200 => {
                return match bitbucket::author_avatar_url(&response.body) {
                    Some(picture) => {
                        let _ = ctx.cache.put_github(&key, Some(&picture));
                        Answer::Url(picture)
                    }
                    // Bitbucket linked no account to this commit's email.
                    None => {
                        let _ = ctx.cache.put_github(&key, None);
                        Answer::NoAccount
                    }
                };
            }
            // Either never pushed, or the repository is not visible: only
            // a repeated 404 across every sha says the latter, and then
            // asking again this session would not help either.
            404 => continue,
            401 | 403 => {
                ctx.session().unreachable.insert(repo.key());
                return Answer::Unknown;
            }
            _ => return Answer::Unknown,
        }
    }
    Answer::Unknown
}

/// Whether a response is GitHub's rate limit, and until when: a 403 or
/// 429 with `x-ratelimit-remaining: 0` or a `retry-after`. The reset is
/// `x-ratelimit-reset` (epoch seconds), else `retry-after` (seconds),
/// else an hour.
#[cfg(any(feature = "avatar-fetch", test))]
pub(crate) fn rate_limited_until(
    status: u16,
    remaining: Option<&str>,
    reset: Option<&str>,
    retry_after: Option<&str>,
    now: SystemTime,
) -> Option<SystemTime> {
    if status != 403 && status != 429 {
        return None;
    }
    let exhausted = remaining.map(str::trim) == Some("0");
    let retry_after = retry_after.and_then(|v| v.trim().parse::<u64>().ok());
    if !exhausted && retry_after.is_none() {
        return None;
    }
    if let Some(reset) = reset.and_then(|v| v.trim().parse::<u64>().ok()) {
        return Some(UNIX_EPOCH + Duration::from_secs(reset));
    }
    Some(now + retry_after.map_or(DEFAULT_RATE_LIMIT_PAUSE, Duration::from_secs))
}

/// [`Fetch`] over `reqwest`, at most [`MAX_CONCURRENT_FETCHES`] requests
/// in flight, each bounded by [`FETCH_TIMEOUT`] on top of the client's
/// own timeout (so a permit is never held by one hung connection).
#[cfg(feature = "avatar-fetch")]
pub(crate) struct ReqwestFetch {
    client: reqwest::Client,
    permits: tokio::sync::Semaphore,
}

#[cfg(feature = "avatar-fetch")]
impl ReqwestFetch {
    pub(crate) fn new() -> Option<ReqwestFetch> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("gitpoulpi/", env!("CARGO_PKG_VERSION")))
            .timeout(FETCH_TIMEOUT)
            .build()
            .ok()?;
        Some(ReqwestFetch {
            client,
            permits: tokio::sync::Semaphore::new(MAX_CONCURRENT_FETCHES),
        })
    }
}

#[cfg(feature = "avatar-fetch")]
impl Fetch for ReqwestFetch {
    fn get<'a>(&'a self, url: &'a str, target: Target<'a>) -> FetchFuture<'a> {
        Box::pin(async move {
            let _permit = self.permits.acquire().await.ok()?;
            if gitpoulpi_hosting::http::external_host_refusal(url).is_some() {
                return None;
            }
            let mut request = self.client.get(url);
            if let Target::BitbucketApi { auth } = target {
                request = request.header("Accept", "application/json"); // i18n-english
                request = match auth {
                    Some(Auth::Basic { user, token }) => request.basic_auth(user, Some(token)),
                    Some(Auth::Bearer(token)) => request.bearer_auth(token),
                    None => request,
                };
            }
            if let Target::GitHubApi { token } = target {
                request = request
                    .header("Accept", "application/vnd.github+json") // i18n-english
                    .header("X-GitHub-Api-Version", "2022-11-28");
                if let Some(token) = token {
                    request = request.bearer_auth(token);
                }
            }
            if let Target::GitLabApi { token } = target {
                request = request.header("Accept", "application/json"); // i18n-english
                if let Some(token) = token {
                    request = request.bearer_auth(token);
                }
            }
            let response = tokio::time::timeout(FETCH_TIMEOUT, request.send())
                .await
                .ok()?
                .ok()?;
            let status = response.status().as_u16();
            let header = |name: &str| {
                response
                    .headers()
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string)
            };
            let rate_limited_until = rate_limited_until(
                status,
                header("x-ratelimit-remaining").as_deref(),
                header("x-ratelimit-reset").as_deref(),
                header("retry-after").as_deref(),
                SystemTime::now(),
            );
            let body = tokio::time::timeout(FETCH_TIMEOUT, response.bytes())
                .await
                .ok()?
                .ok()?;
            Some(Fetched {
                status,
                body: body.to_vec(),
                rate_limited_until,
            })
        })
    }
}

#[cfg(test)]
#[cfg_attr(not(any(feature = "github", feature = "gitlab")), allow(dead_code))]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn png_of(fill: (u8, u8, u8)) -> Vec<u8> {
        let bitmap = Bitmap::new(8, 8, fill);
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, bitmap.width, bitmap.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&bitmap.data).unwrap();
        drop(writer);
        out
    }

    /// A canned network: the first rule whose fragment the URL contains
    /// answers; anything else is a transport failure. Records every call
    /// with whether it carried a token.
    #[derive(Default)]
    pub(crate) struct Canned {
        rules: Vec<(String, Fetched)>,
        pub(crate) calls: Mutex<Vec<(String, Option<String>)>>,
    }

    impl Canned {
        pub(crate) fn on(mut self, fragment: &str, status: u16, body: &[u8]) -> Canned {
            self.rules.push((
                fragment.to_string(),
                Fetched {
                    status,
                    body: body.to_vec(),
                    rate_limited_until: None,
                },
            ));
            self
        }

        #[cfg_attr(not(feature = "github"), allow(dead_code))]
        fn rate_limited(mut self, fragment: &str, until: SystemTime) -> Canned {
            self.rules.push((
                fragment.to_string(),
                Fetched {
                    status: 403,
                    body: Vec::new(),
                    rate_limited_until: Some(until),
                },
            ));
            self
        }

        pub(crate) fn calls_to(&self, fragment: &str) -> usize {
            self.calls
                .lock()
                .unwrap()
                .iter()
                .filter(|(url, _)| url.contains(fragment))
                .count()
        }
    }

    impl Fetch for Canned {
        fn get<'a>(&'a self, url: &'a str, target: Target<'a>) -> FetchFuture<'a> {
            let token = match target {
                Target::GitHubApi { token } | Target::GitLabApi { token } => {
                    token.map(str::to_string)
                }
                Target::BitbucketApi { auth } => auth.map(|a| format!("{a:?}")),
                Target::Image => None,
            };
            self.calls.lock().unwrap().push((url.to_string(), token));
            let answer = self
                .rules
                .iter()
                .find(|(fragment, _)| url.contains(fragment.as_str()))
                .map(|(_, fetched)| fetched.clone());
            Box::pin(async move { answer })
        }
    }

    #[cfg_attr(not(feature = "github"), allow(dead_code))]
    const REPO_URL: &str = "https://api.github.com/repos/o/r/commits/";

    fn repo() -> GitHubRepo {
        GitHubRepo {
            owner: "o".into(),
            name: "r".into(),
        }
    }

    fn request(email: &str, shas: &[&str], repo: Option<GitHubRepo>) -> Request {
        Request {
            hash: gravatar::email_hash(email),
            email: email.into(),
            shas: shas.iter().map(|s| s.to_string()).collect(),
            repo,
            hosting: Hosting::default(),
        }
    }

    fn context(dir: &std::path::Path, fetch: Arc<Canned>, token: Option<&str>) -> Context {
        let token = token.map(str::to_string);
        Context::new(
            AvatarCache::new(dir.to_path_buf()),
            fetch,
            16,
            Box::new(move || token),
        )
    }

    fn run<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(future)
    }

    fn found_color(resolution: &Resolution) -> Option<(u8, u8, u8)> {
        match resolution {
            Resolution::Found(bitmap) => bitmap.pixel(8, 8),
            _ => None,
        }
    }

    /// Red if the GitHub API is moved ahead of Gravatar in `plan`, or
    /// Gravatar ahead of the noreply URL.
    #[test]
    #[cfg(feature = "github")]
    fn the_source_order_is_noreply_then_gravatar_then_the_github_api() {
        let noreply = request("7+me@users.noreply.github.com", &["abc"], Some(repo()));
        let order = plan(&noreply, 16);
        assert!(
            matches!(&order[0], Source::Picture(u) if u.contains("avatars.githubusercontent.com/u/7"))
        );
        assert!(matches!(&order[1], Source::Picture(u) if u.contains("gravatar.com")));
        assert!(matches!(&order[2], Source::GitHubCommits { .. }));

        let plain = request("ada@example.com", &["abc"], Some(repo()));
        let order = plan(&plain, 16);
        assert_eq!(order.len(), 2);
        assert!(matches!(&order[0], Source::Picture(u) if u.contains("gravatar.com")));

        // No github.com remote, or no commit to ask about: Gravatar only.
        assert_eq!(
            plan(&request("ada@example.com", &["abc"], None), 16).len(),
            1
        );
        assert_eq!(
            plan(&request("ada@example.com", &[], Some(repo())), 16).len(),
            1
        );
    }

    #[test]
    fn a_gravatar_hit_never_spends_the_github_budget() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(Canned::default().on("gravatar.com", 200, &png_of((1, 2, 3))));
        let ctx = context(dir.path(), fetch.clone(), None);
        let req = request("ada@example.com", &["abc"], Some(repo()));
        let got = run(resolve(&req, &ctx, SystemTime::now()));
        assert_eq!(found_color(&got), Some((1, 2, 3)));
        assert_eq!(fetch.calls_to("api.github.com"), 0);
    }

    /// Once Gravatar has nothing: the commit's GitHub author, cached per
    /// email, then the picture at `avatar_url`.
    #[test]
    #[cfg(feature = "github")]
    fn with_no_gravatar_the_github_commit_author_supplies_the_picture() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on("gravatar.com", 404, b"")
                .on(
                    REPO_URL,
                    200,
                    br#"{"author":{"avatar_url":"https://avatars.githubusercontent.com/u/9?v=4"}}"#,
                )
                .on("avatars.githubusercontent.com/u/9", 200, &png_of((9, 9, 9))),
        );
        let ctx = context(dir.path(), fetch.clone(), Some("tok"));
        let req = request("me@example.com", &["abc"], Some(repo()));
        let got = run(resolve(&req, &ctx, SystemTime::now()));
        assert_eq!(found_color(&got), Some((9, 9, 9)));
        let calls = fetch.calls.lock().unwrap().clone();
        let api: Vec<_> = calls.iter().filter(|(u, _)| u.contains(REPO_URL)).collect();
        assert_eq!(api.len(), 1);
        assert_eq!(
            api[0].1.as_deref(),
            Some("tok"),
            "the token goes to the API"
        );
        assert!(
            calls
                .iter()
                .filter(|(u, _)| !u.contains("api.github.com"))
                .all(|(_, token)| token.is_none()),
            "the token went to an image host: {calls:?}"
        );
        assert!(calls
            .iter()
            .any(|(u, _)| u.ends_with(&format!("?v=4&s={}", fetch_size(16)))));
        assert_eq!(
            ctx.cache.get_github(&req.hash, SystemTime::now()),
            GitHubLookup::Url("https://avatars.githubusercontent.com/u/9?v=4".into())
        );
    }

    /// Red if `put_negative` is written on every failed lookup.
    #[test]
    fn a_failed_request_is_not_remembered_as_no_avatar() {
        let dir = tempfile::tempdir().unwrap();
        // No rules: every request is a transport failure (a TLS error).
        let fetch = Arc::new(Canned::default());
        let ctx = context(dir.path(), fetch, None);
        let req = request("me@example.com", &[], None);
        assert!(matches!(
            run(resolve(&req, &ctx, SystemTime::now())),
            Resolution::Unknown
        ));
        assert_eq!(ctx.cache.get(&req.hash, SystemTime::now()), Lookup::Miss);
    }

    #[test]
    #[cfg(feature = "github")]
    fn a_definite_none_everywhere_is_remembered() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(Canned::default().on("gravatar.com", 404, b"").on(
            REPO_URL,
            200,
            br#"{"author":null}"#,
        ));
        let ctx = context(dir.path(), fetch.clone(), None);
        let req = request("me@example.com", &["abc"], Some(repo()));
        assert!(matches!(
            run(resolve(&req, &ctx, SystemTime::now())),
            Resolution::None
        ));
        assert_eq!(
            ctx.cache.get(&req.hash, SystemTime::now()),
            Lookup::Negative
        );
        assert_eq!(
            ctx.cache.get_github(&req.hash, SystemTime::now()),
            GitHubLookup::NoAccount
        );
    }

    /// A commit never pushed answers 422; the next sha by the same author
    /// is asked instead, and no more than `MAX_SHAS_PER_AUTHOR`.
    #[test]
    #[cfg(feature = "github")]
    fn an_unpushed_commit_moves_on_to_the_authors_next_commit() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on("gravatar.com", 404, b"")
                .on("commits/local1", 422, b"")
                .on(
                    "commits/pushed",
                    200,
                    br#"{"author":{"avatar_url":"https://avatars.githubusercontent.com/u/5?v=4"}}"#,
                )
                .on("avatars.githubusercontent.com/u/5", 200, &png_of((5, 5, 5))),
        );
        let ctx = context(dir.path(), fetch.clone(), None);
        let req = request("me@example.com", &["local1", "pushed"], Some(repo()));
        assert_eq!(
            found_color(&run(resolve(&req, &ctx, SystemTime::now()))),
            Some((5, 5, 5))
        );

        let many = request(
            "other@example.com",
            &["local1", "local1", "local1", "local1", "local1"],
            Some(repo()),
        );
        let before = fetch.calls_to("commits/local1");
        let _ = run(resolve(&many, &ctx, SystemTime::now()));
        assert_eq!(
            fetch.calls_to("commits/local1") - before,
            MAX_SHAS_PER_AUTHOR
        );
    }

    /// Red if `blocked_until` is ignored (the second author calls the API
    /// again).
    #[test]
    #[cfg(feature = "github")]
    fn a_rate_limit_stops_every_api_call_until_it_resets() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        let fetch = Arc::new(
            Canned::default()
                .on("gravatar.com", 404, b"")
                .rate_limited(REPO_URL, now + Duration::from_secs(600)),
        );
        let ctx = context(dir.path(), fetch.clone(), None);
        let first = request("a@example.com", &["abc"], Some(repo()));
        assert!(matches!(
            run(resolve(&first, &ctx, now)),
            Resolution::Unknown
        ));
        let second = request("b@example.com", &["abc"], Some(repo()));
        assert!(matches!(
            run(resolve(&second, &ctx, now)),
            Resolution::Unknown
        ));
        assert_eq!(fetch.calls_to(REPO_URL), 1, "asked again while limited");
        // Nothing remembered on disk: both are asked again next session.
        assert_eq!(ctx.cache.get(&second.hash, now), Lookup::Miss);
        assert_eq!(ctx.cache.get_github(&second.hash, now), GitHubLookup::Miss);
    }

    /// A private repository without a token answers 404. It is not asked
    /// again this session, and nobody is recorded as having no avatar
    /// because of it.
    #[test]
    #[cfg(feature = "github")]
    fn a_repository_the_api_cannot_see_is_asked_once_per_session() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(Canned::default().on("gravatar.com", 404, b"").on(
            REPO_URL,
            404,
            br#"{"message":"Not Found"}"#,
        ));
        let ctx = context(dir.path(), fetch.clone(), None);
        let now = SystemTime::now();
        for email in ["a@example.com", "b@example.com", "c@example.com"] {
            let req = request(email, &["abc"], Some(repo()));
            assert!(matches!(run(resolve(&req, &ctx, now)), Resolution::Unknown));
            assert_eq!(ctx.cache.get(&req.hash, now), Lookup::Miss);
        }
        assert_eq!(fetch.calls_to(REPO_URL), 1);
    }

    #[test]
    #[cfg(feature = "github")]
    fn a_cached_github_answer_costs_no_api_call() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(Canned::default().on("gravatar.com", 404, b"").on(
            "avatars.githubusercontent.com/u/3",
            200,
            &png_of((3, 3, 3)),
        ));
        let ctx = context(dir.path(), fetch.clone(), None);
        let req = request("me@example.com", &["abc"], Some(repo()));
        ctx.cache
            .put_github(
                &req.hash,
                Some("https://avatars.githubusercontent.com/u/3?v=4"),
            )
            .unwrap();
        assert_eq!(
            found_color(&run(resolve(&req, &ctx, SystemTime::now()))),
            Some((3, 3, 3))
        );
        assert_eq!(fetch.calls_to("api.github.com"), 0);
    }

    #[test]
    #[cfg(feature = "github")]
    fn a_noreply_address_is_fetched_directly() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(Canned::default().on(
            "avatars.githubusercontent.com/u/77",
            200,
            &png_of((7, 7, 7)),
        ));
        let ctx = context(dir.path(), fetch.clone(), None);
        let req = request("77+me@users.noreply.github.com", &["abc"], Some(repo()));
        assert_eq!(
            found_color(&run(resolve(&req, &ctx, SystemTime::now()))),
            Some((7, 7, 7))
        );
        assert_eq!(
            fetch.calls.lock().unwrap().len(),
            1,
            "one request, no Gravatar"
        );
    }

    #[test]
    fn the_rate_limit_headers_are_read_as_documented() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000);
        assert_eq!(
            rate_limited_until(403, Some("0"), Some("1600"), None, now),
            Some(UNIX_EPOCH + Duration::from_secs(1_600))
        );
        assert_eq!(
            rate_limited_until(429, None, None, Some("30"), now),
            Some(now + Duration::from_secs(30))
        );
        assert_eq!(
            rate_limited_until(403, Some("0"), None, None, now),
            Some(now + DEFAULT_RATE_LIMIT_PAUSE)
        );
        // A 403 that is not the rate limit (no access), and a success.
        assert_eq!(
            rate_limited_until(403, Some("41"), Some("1600"), None, now),
            None
        );
        assert_eq!(
            rate_limited_until(200, Some("0"), Some("1600"), None, now),
            None
        );
    }

    // ---- ui.avatar_sources: the order, with connected hosting ----

    fn gitlab_host() -> GitLabHost {
        GitLabHost {
            base: "https://git.example.com".into(),
        }
    }

    fn with_hosting(mut req: Request, hosting: Hosting) -> Request {
        req.hosting = hosting;
        req
    }

    fn kinds(plan: &[Source]) -> Vec<&'static str> {
        plan.iter()
            .map(|s| match s {
                Source::Picture(u) if u.contains("gravatar.com") => "gravatar",
                Source::Picture(_) => "noreply",
                Source::GitHubCommits { .. } => "github",
                Source::GitLab { .. } => "gitlab",
                Source::BitbucketCommits { .. } => "bitbucket",
            })
            .collect()
    }

    /// Red if `hosting_first` is ignored in `plan` (always Gravatar first).
    #[test]
    #[cfg(all(feature = "github", feature = "gitlab"))]
    fn a_connected_integration_puts_hosting_before_gravatar() {
        let base = request("ada@example.com", &["abc"], Some(repo()));
        let connected = |sources| {
            with_hosting(
                base.clone(),
                Hosting {
                    gitlab: Some(gitlab_host()),
                    bitbucket: None,
                    github_connected: true,
                    sources,
                },
            )
        };
        assert_eq!(
            kinds(&plan(&connected(AvatarSources::HostingGravatar), 16)),
            ["github", "gitlab", "gravatar"]
        );
        assert_eq!(
            kinds(&plan(&connected(AvatarSources::GravatarHosting), 16)),
            ["gravatar", "github", "gitlab"]
        );
        assert_eq!(
            kinds(&plan(&connected(AvatarSources::Gravatar), 16)),
            ["gravatar"]
        );
        assert_eq!(
            kinds(&plan(&connected(AvatarSources::Hosting), 16)),
            ["github", "gitlab"]
        );
        // Nothing connected: `hosting_gravatar` is today's order.
        assert_eq!(kinds(&plan(&base, 16)), ["gravatar", "github"]);
        // A noreply address still leads.
        let noreply = with_hosting(
            request("7+me@users.noreply.github.com", &[], None),
            Hosting::default(),
        );
        assert_eq!(kinds(&plan(&noreply, 16)), ["noreply", "gravatar"]);
    }

    /// A GitLab host is asked only when one was connected for the remote.
    #[test]
    fn gitlab_is_never_planned_without_a_connected_host() {
        let plain = request("ada@example.com", &[], None);
        assert_eq!(kinds(&plan(&plain, 16)), ["gravatar"]);
    }

    fn hosting_first(gitlab: bool) -> Hosting {
        Hosting {
            gitlab: gitlab.then(gitlab_host),
            bitbucket: None,
            github_connected: !gitlab,
            sources: AvatarSources::HostingGravatar,
        }
    }

    #[test]
    #[cfg(feature = "github")]
    fn the_connected_github_account_answers_before_gravatar_is_asked() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on(
                    REPO_URL,
                    200,
                    br#"{"author":{"avatar_url":"https://avatars.githubusercontent.com/u/9?v=4"}}"#,
                )
                .on("avatars.githubusercontent.com/u/9", 200, &png_of((9, 9, 9)))
                .on("gravatar.com", 200, &png_of((1, 1, 1))),
        );
        let ctx = context(dir.path(), fetch.clone(), Some("tok"));
        let req = with_hosting(
            request("ada@example.com", &["abc"], Some(repo())),
            hosting_first(false),
        );
        let resolution = run(resolve(&req, &ctx, SystemTime::now()));
        assert_eq!(found_color(&resolution), Some((9, 9, 9)));
        assert_eq!(fetch.calls_to("gravatar.com"), 0, "Gravatar never asked");
    }

    /// Red if the `search_allowed` token test is dropped (the search runs
    /// without a connected token).
    #[test]
    #[cfg(feature = "github")]
    fn a_connected_token_finds_the_account_by_email_when_the_commit_has_none() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on(REPO_URL, 200, br#"{"author":null}"#)
                .on(
                    "api.github.com/search/users",
                    200,
                    br#"{"items":[{"avatar_url":"https://avatars.githubusercontent.com/u/5?v=4"}]}"#,
                )
                .on("avatars.githubusercontent.com/u/5", 200, &png_of((5, 5, 5)))
                .on("gravatar.com", 404, b""),
        );
        let ctx = context(dir.path(), fetch.clone(), Some("tok"));
        let req = with_hosting(
            request("ada@example.com", &["abc"], Some(repo())),
            hosting_first(false),
        );
        let resolution = run(resolve(&req, &ctx, SystemTime::now()));
        assert_eq!(found_color(&resolution), Some((5, 5, 5)));
        assert_eq!(fetch.calls_to("search/users"), 1);

        // Without a token the search is never sent.
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on(REPO_URL, 200, br#"{"author":null}"#)
                .on("gravatar.com", 404, b""),
        );
        let ctx = context(dir.path(), fetch.clone(), None);
        let resolution = run(resolve(&req, &ctx, SystemTime::now()));
        assert!(matches!(resolution, Resolution::None));
        assert_eq!(fetch.calls_to("search/users"), 0);
    }

    #[cfg(feature = "gitlab")]
    fn gitlab_ctx(
        dir: &std::path::Path,
        fetch: Arc<Canned>,
        token: Option<&'static str>,
    ) -> Context {
        context(dir, fetch, None).with_gitlab_tokens(Box::new(move |base| {
            assert_eq!(base, "https://git.example.com");
            token.map(str::to_string)
        }))
    }

    #[test]
    #[cfg(feature = "gitlab")]
    fn a_connected_gitlab_host_supplies_the_picture_before_gravatar() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on(
                    "git.example.com/api/v4/avatar",
                    200,
                    br#"{"avatar_url":"https://git.example.com/uploads/u.png"}"#,
                )
                .on("git.example.com/uploads/u.png", 200, &png_of((7, 7, 7)))
                .on("gravatar.com", 200, &png_of((1, 1, 1))),
        );
        let ctx = gitlab_ctx(dir.path(), fetch.clone(), Some("glpat"));
        let req = with_hosting(request("ada@example.com", &[], None), hosting_first(true));
        let resolution = run(resolve(&req, &ctx, SystemTime::now()));
        assert_eq!(found_color(&resolution), Some((7, 7, 7)));
        assert_eq!(fetch.calls_to("gravatar.com"), 0);
        // The token went to the connected host only.
        let calls = fetch.calls.lock().unwrap();
        assert!(calls
            .iter()
            .any(|(u, t)| u.contains("/api/v4/avatar") && t.as_deref() == Some("glpat")));
        assert!(calls
            .iter()
            .filter(|(u, _)| u.contains("uploads"))
            .all(|(_, t)| t.is_none()));
    }

    /// GitLab answers an unknown address with a Gravatar URL: the order
    /// moves on to Gravatar itself, then the fallback.
    #[test]
    #[cfg(feature = "gitlab")]
    fn gitlabs_gravatar_stand_in_falls_through_to_gravatar_then_none() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on(
                    "api/v4/avatar",
                    200,
                    br#"{"avatar_url":"https://www.gravatar.com/avatar/x?d=identicon"}"#,
                )
                .on("gravatar.com/avatar", 404, b""),
        );
        let ctx = gitlab_ctx(dir.path(), fetch.clone(), None);
        let req = with_hosting(request("ada@example.com", &[], None), hosting_first(true));
        let resolution = run(resolve(&req, &ctx, SystemTime::now()));
        assert!(matches!(resolution, Resolution::None), "{resolution:?}");
        // Asked in the order: the host, then Gravatar.
        let calls = fetch.calls.lock().unwrap();
        assert!(calls[0].0.contains("api/v4/avatar"), "{calls:?}");
        assert!(calls[1].0.contains("gravatar.com"), "{calls:?}");
    }

    #[cfg(feature = "bitbucket")]
    const BB_COMMIT: &str = "api.bitbucket.org/2.0/repositories/ws/repo/commit/";

    #[cfg(feature = "bitbucket")]
    fn bitbucket_repo() -> BitbucketRepo {
        let mut integration = gitpoulpi_config::Integration::new(
            "bb",
            gitpoulpi_config::IntegrationKind::Bitbucket,
            "https://bitbucket.org",
        );
        integration.account = "me@x.test".into();
        BitbucketRepo {
            workspace: "ws".into(),
            slug: "repo".into(),
            integration,
        }
    }

    #[cfg(feature = "bitbucket")]
    fn bitbucket_hosting() -> Hosting {
        Hosting {
            bitbucket: Some(bitbucket_repo()),
            sources: AvatarSources::HostingGravatar,
            ..Hosting::default()
        }
    }

    #[cfg(feature = "bitbucket")]
    fn bitbucket_ctx(dir: &std::path::Path, fetch: Arc<Canned>) -> Context {
        context(dir, fetch, None).with_bitbucket_auth(Box::new(|_| {
            Some(Auth::Basic {
                user: "me@x.test".into(),
                token: "tok".into(),
            })
        }))
    }

    #[cfg(feature = "bitbucket")]
    const BB_LINKED: &[u8] = br#"{"author":{"raw":"Ada <ada@example.com>","user":{
        "links":{"avatar":{"href":"https://bitbucket.org/account/ada/avatar/32/"}}}}}"#;

    #[test]
    #[cfg(feature = "bitbucket")]
    fn bitbucket_is_planned_only_for_a_connected_repository_with_commits() {
        let none = request("ada@example.com", &["abc"], None);
        assert_eq!(kinds(&plan(&none, 16)), ["gravatar"]);
        let connected = with_hosting(
            request("ada@example.com", &["abc"], None),
            bitbucket_hosting(),
        );
        assert_eq!(kinds(&plan(&connected, 16)), ["bitbucket", "gravatar"]);
        // No commit to ask about: nothing to plan.
        let no_commits = with_hosting(request("ada@example.com", &[], None), bitbucket_hosting());
        assert_eq!(kinds(&plan(&no_commits, 16)), ["gravatar"]);
        // At most MAX_SHAS_PER_AUTHOR commits are asked about.
        let many = with_hosting(
            request("ada@example.com", &["1", "2", "3", "4", "5"], None),
            bitbucket_hosting(),
        );
        match &plan(&many, 16)[0] {
            Source::BitbucketCommits { shas, .. } => assert_eq!(shas.len(), MAX_SHAS_PER_AUTHOR),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    #[cfg(feature = "bitbucket")]
    fn the_linked_account_picture_is_fetched_with_credentials_for_the_api_only() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on(BB_COMMIT, 200, BB_LINKED)
                .on("bitbucket.org/account/ada/avatar", 200, &png_of((4, 5, 6)))
                .on("gravatar.com", 200, &png_of((1, 1, 1))),
        );
        let ctx = bitbucket_ctx(dir.path(), fetch.clone());
        let req = with_hosting(
            request("ada@example.com", &["abc"], None),
            bitbucket_hosting(),
        );
        let resolution = run(resolve(&req, &ctx, SystemTime::now()));
        assert_eq!(found_color(&resolution), Some((4, 5, 6)));
        assert_eq!(fetch.calls_to("gravatar.com"), 0);
        let calls = fetch.calls.lock().unwrap();
        assert!(calls.iter().any(
            |(u, t)| u.contains(BB_COMMIT) && t.as_deref().is_some_and(|t| t.contains("Basic"))
        ));
        assert!(calls
            .iter()
            .filter(|(u, _)| u.contains("avatar"))
            .all(|(_, t)| t.is_none()));
    }

    /// A definite "no account" is remembered on disk and not asked again.
    #[test]
    #[cfg(feature = "bitbucket")]
    fn an_author_without_an_account_is_remembered_and_falls_through() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(
            Canned::default()
                .on(
                    BB_COMMIT,
                    200,
                    br#"{"author":{"raw":"Ada <ada@example.com>"}}"#,
                )
                .on("gravatar.com", 200, &png_of((2, 2, 2))),
        );
        let ctx = bitbucket_ctx(dir.path(), fetch.clone());
        let req = with_hosting(
            request("ada@example.com", &["abc"], None),
            bitbucket_hosting(),
        );
        let first = run(resolve(&req, &ctx, SystemTime::now()));
        assert_eq!(found_color(&first), Some((2, 2, 2)));
        // Same author in another session: Bitbucket's answer is cached.
        let again = bitbucket_ctx(dir.path(), fetch.clone());
        let other = Request {
            hash: format!("{}-2", req.hash),
            ..req.clone()
        };
        let _ = run(resolve(&other, &again, SystemTime::now()));
        let _ = run(resolve(&req, &again, SystemTime::now()));
        assert_eq!(
            fetch.calls_to(BB_COMMIT),
            2,
            "the cached author is not asked again"
        );
    }

    #[test]
    #[cfg(feature = "bitbucket")]
    fn an_unpushed_commit_tries_the_next_sha_and_a_rate_limit_pauses_bitbucket() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        let fetch = Arc::new(
            Canned::default()
                .on("/commit/aaa", 404, b"")
                .on("/commit/bbb", 200, BB_LINKED)
                .on("bitbucket.org/account/ada/avatar", 200, &png_of((3, 3, 3))),
        );
        let ctx = bitbucket_ctx(dir.path(), fetch.clone());
        let req = with_hosting(
            request("ada@example.com", &["aaa", "bbb"], None),
            bitbucket_hosting(),
        );
        assert_eq!(found_color(&run(resolve(&req, &ctx, now))), Some((3, 3, 3)));
        assert_eq!(fetch.calls_to("/commit/"), 2);

        let limited =
            Arc::new(Canned::default().rate_limited(BB_COMMIT, now + Duration::from_secs(600)));
        let dir2 = tempfile::tempdir().unwrap();
        let ctx = bitbucket_ctx(dir2.path(), limited.clone());
        let a = with_hosting(
            request("a@example.com", &["aaa"], None),
            bitbucket_hosting(),
        );
        let b = with_hosting(
            request("b@example.com", &["bbb"], None),
            bitbucket_hosting(),
        );
        assert!(!matches!(run(resolve(&a, &ctx, now)), Resolution::Found(_)));
        assert!(!matches!(run(resolve(&b, &ctx, now)), Resolution::Found(_)));
        assert_eq!(limited.calls_to(BB_COMMIT), 1, "paused until the reset");
    }

    #[test]
    #[cfg(feature = "bitbucket")]
    fn an_unreachable_repository_is_not_asked_again_this_session() {
        let dir = tempfile::tempdir().unwrap();
        let fetch = Arc::new(Canned::default().on(BB_COMMIT, 403, b""));
        let ctx = bitbucket_ctx(dir.path(), fetch.clone());
        for email in ["a@example.com", "b@example.com"] {
            let req = with_hosting(request(email, &["abc"], None), bitbucket_hosting());
            let resolution = run(resolve(&req, &ctx, SystemTime::now()));
            // Nothing definite is remembered about the author.
            assert!(matches!(resolution, Resolution::Unknown), "{resolution:?}");
        }
        assert_eq!(fetch.calls_to(BB_COMMIT), 1);
    }
}
