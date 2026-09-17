//! Conditional requests and rate-limit back-off, as a transport decorator.
//!
//! [`ConditionalTransport`] wraps any [`HttpTransport`] and:
//!
//! * remembers the `ETag` of every successful GET and sends it back as
//!   `If-None-Match`; a `304` is answered from the cached body (so callers
//!   see a normal 200) and, on GitHub, costs no rate limit;
//! * reads each host's rate-limit headers -- interpreted per API family,
//!   because the six products do not mean the same thing by them (see
//!   [`rate_observation`]) -- and, once a host says "none left", refuses
//!   further requests to it until the reset time instead of hammering it
//!   ([`Error::RateLimited`]);
//! * exposes a short human note ([`ConditionalTransport::rate_note`]) when
//!   a host is close to its limit, for the TUI's status line.
//!
//! Every client is rebuilt per refresh in the TUI, so the cache lives here
//! (one shared instance), not in the clients.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use crate::http::{HttpRequest, HttpResponse, HttpTransport, Method};
use crate::{Error, Result};
use gitpoulpi_config::ApiFamily;

/// Below this many calls left, [`ConditionalTransport::rate_note`] speaks up.
const LOW_WATER: u64 = 20;

/// A bound on cached bodies, so a long session cannot grow without limit.
/// The Radar asks about five lists per repository, so 64 was gone after a
/// dozen repositories; when full, the entry untouched the longest goes, not
/// every entry.
const MAX_ENTRIES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RateState {
    remaining: u64,
    /// Unix seconds when the window resets (or the `Retry-After` deadline).
    reset_at: u64,
    /// The host said it is close to its limit without giving a count
    /// (Bitbucket `X-RateLimit-NearLimit`).
    near: bool,
}

/// What one answer says about the host's budget, per family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RateObservation {
    pub remaining: u64,
    pub reset_at: u64,
    pub near: bool,
    /// Why the host is throttling, when it says (`X-RateLimit-Resource`,
    /// `RateLimit-Reason`), for the status note.
    pub reason: Option<String>,
}

struct Cached {
    etag: String,
    response: HttpResponse,
    /// When it was stored or last confirmed (`304`): the smallest goes first.
    used: u64,
}

pub struct ConditionalTransport {
    inner: Arc<dyn HttpTransport>,
    now: fn() -> u64,
    cache: Mutex<HashMap<String, Cached>>,
    rates: Mutex<BTreeMap<String, RateState>>,
    /// Why a host last throttled us (shown in the note).
    reasons: Mutex<BTreeMap<String, String>>,
    /// Hosts whose family is known from a configured profile rather than
    /// from the public API host names.
    families: Mutex<HashMap<String, ApiFamily>>,
    hits: std::sync::atomic::AtomicU64,
    /// Ticks once per cache write or confirmation; orders `Cached::used`.
    tick: std::sync::atomic::AtomicU64,
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn host_of(url: &str) -> String {
    url.split("://")
        .nth(1)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn header<'a>(response: &'a HttpResponse, name: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim())
}

/// The family a public API host name belongs to. Anything unknown is read
/// the GitHub/GitLab way (see [`ConditionalTransport::register_host`] for
/// self-hosted instances).
pub(crate) fn family_of_public_host(host: &str) -> ApiFamily {
    let host = host.split(':').next().unwrap_or(host);
    if host == "dev.azure.com" || host.ends_with(".visualstudio.com") {
        ApiFamily::AzureDevops
    } else if host == "bitbucket.org" || host.ends_with(".bitbucket.org") {
        ApiFamily::Bitbucket
    } else if host == "trello.com" || host.ends_with(".trello.com") {
        ApiFamily::Trello
    } else if host.ends_with(".atlassian.net") || host == "api.atlassian.com" {
        ApiFamily::Jira
    } else if host == "linear.app" || host.ends_with(".linear.app") {
        ApiFamily::Linear
    } else if host.ends_with(".youtrack.cloud") || host.ends_with(".myjetbrains.com") {
        ApiFamily::YouTrack
    } else if host == "codeberg.org" || host == "gitea.com" {
        ApiFamily::Gitea
    } else if host == "gitlab.com" || host.ends_with(".gitlab.com") {
        ApiFamily::Gitlab
    } else {
        ApiFamily::Github
    }
}

fn number(response: &HttpResponse, names: &[&str]) -> Option<u64> {
    names
        .iter()
        .find_map(|n| header(response, n))
        .and_then(|v| v.parse::<u64>().ok())
}

/// Reads one answer's rate-limit headers the way its family means them.
///
/// * GitHub / GitLab: `x-ratelimit-*` / `ratelimit-*` call counts, plus
///   `Retry-After` on a 429/403.
/// * Azure DevOps: `X-RateLimit-Remaining` counts TSTUs (a cost unit), not
///   calls, so it is never read as a budget; only `Retry-After` on a 429
///   blocks, and `X-RateLimit-Resource` is kept as the reason.
/// * Bitbucket: `X-RateLimit-NearLimit: true` is the only early warning;
///   a 429 blocks (default one minute).
/// * Jira: a 429 with `Retry-After` blocks until then; `RateLimit-Reason`
///   is the reason. No count is read.
/// * Linear: `x-ratelimit-requests-remaining` is the call count and
///   `x-ratelimit-requests-reset` the epoch second the bucket refills.
/// * YouTrack, Redmine, Gerrit: a 429 with `Retry-After` blocks; no count is read.
/// * Trello: `x-rate-limit-api-token-remaining` is the call count; the
///   bucket refills after `x-rate-limit-api-token-interval-ms`.
pub(crate) fn rate_observation(
    family: ApiFamily,
    response: &HttpResponse,
    now: u64,
) -> Option<RateObservation> {
    let retry_after = number(response, &["retry-after"]);
    let throttled = is_rate_limited_for(family, response);
    let blocked = |default_wait: u64, reason: Option<&str>| RateObservation {
        remaining: 0,
        reset_at: now + retry_after.unwrap_or(default_wait),
        near: false,
        reason: reason.map(str::to_string),
    };
    match family {
        ApiFamily::Github | ApiFamily::Gitlab => {
            let remaining = number(response, &["x-ratelimit-remaining", "ratelimit-remaining"]);
            let reset = number(response, &["x-ratelimit-reset", "ratelimit-reset"]);
            match (remaining, reset, retry_after) {
                (_, _, Some(secs)) if response.status == 429 || response.status == 403 => {
                    Some(RateObservation {
                        remaining: 0,
                        reset_at: now + secs,
                        near: false,
                        reason: None,
                    })
                }
                (Some(rem), Some(reset), _) => Some(RateObservation {
                    remaining: rem,
                    reset_at: reset,
                    near: false,
                    reason: None,
                }),
                (Some(rem), None, _) => Some(RateObservation {
                    remaining: rem,
                    reset_at: now + 60,
                    near: false,
                    reason: None,
                }),
                _ => None,
            }
        }
        ApiFamily::AzureDevops => {
            throttled.then(|| blocked(60, header(response, "x-ratelimit-resource")))
        }
        ApiFamily::Jira => throttled.then(|| blocked(60, header(response, "ratelimit-reason"))),
        // No headers of their own: a rate limit is a proxy's or the
        // instance's 429 (Codeberg's anti-abuse layer), with `Retry-After`.
        ApiFamily::Gitea => throttled.then(|| blocked(60, None)),
        ApiFamily::YouTrack | ApiFamily::Redmine | ApiFamily::Gerrit => {
            throttled.then(|| blocked(60, None))
        }
        ApiFamily::Linear => {
            if throttled {
                return Some(blocked(60, None));
            }
            let remaining = number(response, &["x-ratelimit-requests-remaining"])?;
            let reset = number(response, &["x-ratelimit-requests-reset"]).unwrap_or(now + 60);
            Some(RateObservation {
                remaining,
                reset_at: reset,
                near: false,
                reason: None,
            })
        }
        ApiFamily::Bitbucket => {
            if throttled {
                Some(blocked(60, None))
            } else if header(response, "x-ratelimit-nearlimit")
                .is_some_and(|v| v.eq_ignore_ascii_case("true"))
            {
                Some(RateObservation {
                    remaining: LOW_WATER,
                    reset_at: now + 60,
                    near: true,
                    reason: None,
                })
            } else {
                None
            }
        }
        ApiFamily::Trello => {
            let interval = number(response, &["x-rate-limit-api-token-interval-ms"])
                .map(|ms| ms.div_ceil(1000))
                .unwrap_or(10);
            if throttled {
                return Some(RateObservation {
                    remaining: 0,
                    reset_at: now + retry_after.unwrap_or(interval),
                    near: false,
                    reason: None,
                });
            }
            number(response, &["x-rate-limit-api-token-remaining"]).map(|remaining| {
                RateObservation {
                    remaining,
                    reset_at: now + interval,
                    near: false,
                    reason: None,
                }
            })
        }
        // A folder has no rate limit.
        ApiFamily::LocalMarkdown => None,
    }
}

/// Reads both hosts' spellings: GitHub `x-ratelimit-*`, GitLab
/// `ratelimit-*`, plus `Retry-After` (seconds) on a throttled answer.
pub(crate) fn rate_state(response: &HttpResponse, now: u64) -> Option<(u64, u64)> {
    rate_observation(ApiFamily::Github, response, now).map(|o| (o.remaining, o.reset_at))
}

/// Is this answer the host's "slow down" (as opposed to a scope problem)?
pub(crate) fn is_rate_limited(response: &HttpResponse) -> bool {
    is_rate_limited_for(ApiFamily::Github, response)
}

/// [`is_rate_limited`] for one family: Azure DevOps, Jira, Bitbucket and
/// Trello only ever say it with a 429 (a 403 there is a permission).
pub(crate) fn is_rate_limited_for(family: ApiFamily, response: &HttpResponse) -> bool {
    match family {
        ApiFamily::Github | ApiFamily::Gitlab => match response.status {
            429 => true,
            403 => {
                header(response, "x-ratelimit-remaining") == Some("0")
                    || header(response, "retry-after").is_some()
            }
            _ => false,
        },
        _ => response.status == 429,
    }
}

pub(crate) fn wait_text(secs: u64) -> String {
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs.div_ceil(60)),
        _ => format!("{}h", secs.div_ceil(3600)),
    }
}

impl ConditionalTransport {
    pub fn new(inner: Arc<dyn HttpTransport>) -> Self {
        Self::with_clock(inner, unix_now)
    }

    /// Like [`Self::new`] with an injected clock, for tests.
    pub fn with_clock(inner: Arc<dyn HttpTransport>, now: fn() -> u64) -> Self {
        Self {
            inner,
            now,
            cache: Mutex::new(HashMap::new()),
            rates: Mutex::new(BTreeMap::new()),
            reasons: Mutex::new(BTreeMap::new()),
            families: Mutex::new(HashMap::new()),
            hits: Default::default(),
            tick: Default::default(),
        }
    }

    /// Tells the transport which family a (self-hosted) host belongs to, so
    /// its headers are read the right way: call it for each configured
    /// profile. Public API hosts need no registration.
    pub fn register_host(&self, host: &str, family: ApiFamily) {
        self.families
            .lock()
            .unwrap()
            .insert(host.to_ascii_lowercase(), family);
    }

    fn family_of(&self, host: &str) -> ApiFamily {
        self.families
            .lock()
            .unwrap()
            .get(host)
            .copied()
            .unwrap_or_else(|| family_of_public_host(host))
    }

    /// How many requests were answered `304 Not Modified` so far.
    pub fn not_modified_count(&self) -> u64 {
        self.hits.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// A short status-line message when some host is low on (or out of)
    /// calls, else `None`.
    pub fn rate_note(&self) -> Option<String> {
        let now = (self.now)();
        let rates = self.rates.lock().unwrap();
        let reasons = self.reasons.lock().unwrap();
        rates
            .iter()
            .filter(|(_, s)| s.reset_at > now && s.remaining <= LOW_WATER)
            .map(|(host, s)| {
                let wait = wait_text(s.reset_at - now);
                if s.remaining == 0 {
                    match reasons.get(host) {
                        Some(why) => {
                            format!("{host}: rate limit reached ({why}), paused for {wait}")
                        }
                        None => format!("{host}: rate limit reached, paused for {wait}"),
                    }
                } else if s.near {
                    format!("{host}: rate limit nearly reached, resets in {wait}")
                } else {
                    format!(
                        "{host}: rate limit low ({} left), resets in {wait}",
                        s.remaining
                    )
                }
            })
            .next()
    }

    fn blocked(&self, host: &str) -> Option<u64> {
        let now = (self.now)();
        let rates = self.rates.lock().unwrap();
        rates
            .get(host)
            .filter(|s| s.remaining == 0 && s.reset_at > now)
            .map(|s| s.reset_at - now)
    }

    fn cache_key(request: &HttpRequest) -> String {
        let auth = request
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        format!("{auth}\n{}", request.url)
    }
}

#[async_trait::async_trait]
impl HttpTransport for ConditionalTransport {
    async fn send(&self, mut request: HttpRequest) -> Result<HttpResponse> {
        let host = host_of(&request.url);
        let family = self.family_of(&host);
        let is_get = request.method == Method::Get;
        if let Some(wait) = self.blocked(&host) {
            return Err(Error::RateLimited {
                host,
                retry_in_secs: wait,
            });
        }
        let key = Self::cache_key(&request);
        if is_get {
            if let Some(c) = self.cache.lock().unwrap().get(&key) {
                request
                    .headers
                    .push(("If-None-Match".to_string(), c.etag.clone()));
            }
        }
        let response = self.inner.send(request).await?;
        let now = (self.now)();
        let observed = rate_observation(family, &response, now);
        match &observed {
            Some(o) => {
                self.rates.lock().unwrap().insert(
                    host.clone(),
                    RateState {
                        remaining: o.remaining,
                        reset_at: o.reset_at,
                        near: o.near,
                    },
                );
                match &o.reason {
                    Some(why) => {
                        self.reasons
                            .lock()
                            .unwrap()
                            .insert(host.clone(), why.clone());
                    }
                    None => {
                        self.reasons.lock().unwrap().remove(&host);
                    }
                }
            }
            // A calm answer from a family that reports nothing when calm
            // ends an earlier warning; GitHub/GitLab keep their old rule
            // (a header-less answer leaves the last count alone).
            None if !matches!(family, ApiFamily::Github | ApiFamily::Gitlab)
                && !is_rate_limited_for(family, &response) =>
            {
                self.rates.lock().unwrap().remove(&host);
                self.reasons.lock().unwrap().remove(&host);
            }
            None => {}
        }
        if is_rate_limited_for(family, &response) {
            let wait = observed
                .as_ref()
                .map(|o| o.reset_at.saturating_sub(now))
                .unwrap_or(60)
                .max(1);
            // Make sure the back-off sticks even if the host sent no
            // remaining-count header.
            self.rates.lock().unwrap().insert(
                host.clone(),
                RateState {
                    remaining: 0,
                    reset_at: now + wait,
                    near: false,
                },
            );
            return Err(Error::RateLimited {
                host,
                retry_in_secs: wait,
            });
        }
        if is_get && response.status == 304 {
            if let Some(c) = self.cache.lock().unwrap().get_mut(&key) {
                self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                c.used = self.tick.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let mut cached = c.response.clone();
                // Keep the fresh pagination/rate headers, serve the old body.
                for (k, v) in response.headers {
                    cached.headers.insert(k, v);
                }
                return Ok(cached);
            }
        }
        if is_get && response.status == 200 {
            if let Some(etag) = header(&response, "etag") {
                let mut cache = self.cache.lock().unwrap();
                if cache.len() >= MAX_ENTRIES && !cache.contains_key(&key) {
                    let oldest = cache
                        .iter()
                        .min_by_key(|(_, c)| c.used)
                        .map(|(k, _)| k.clone());
                    if let Some(oldest) = oldest {
                        cache.remove(&oldest);
                    }
                }
                cache.insert(
                    key,
                    Cached {
                        etag: etag.to_string(),
                        response: response.clone(),
                        used: self.tick.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                    },
                );
            }
        }
        Ok(response)
    }
}

#[cfg(feature = "network")]
static SHARED: std::sync::OnceLock<Arc<ConditionalTransport>> = std::sync::OnceLock::new();

/// The process-wide caching network transport (created on first use).
#[cfg(feature = "network")]
pub fn shared_network_transport() -> Result<Arc<ConditionalTransport>> {
    if let Some(t) = SHARED.get() {
        return Ok(t.clone());
    }
    let inner = Arc::new(crate::http::ReqwestTransport::new()?);
    let t = Arc::new(ConditionalTransport::new(inner));
    Ok(SHARED.get_or_init(|| t).clone())
}

/// The shared transport's rate note, without creating the transport.
#[cfg(feature = "network")]
pub fn shared_rate_note() -> Option<String> {
    SHARED.get().and_then(|t| t.rate_note())
}

#[cfg(not(feature = "network"))]
pub fn shared_rate_note() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // Per test thread, so parallel tests cannot move each other's clock.
    thread_local! {
        static NOW: Cell<u64> = const { Cell::new(1_000) };
    }
    fn clock() -> u64 {
        NOW.with(Cell::get)
    }

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(f)
    }

    /// Scripted transport: pops answers in order and records requests.
    struct Script {
        answers: Mutex<Vec<HttpResponse>>,
        seen: Mutex<Vec<HttpRequest>>,
    }
    impl Script {
        fn new(answers: Vec<HttpResponse>) -> Arc<Self> {
            Arc::new(Self {
                answers: Mutex::new(answers),
                seen: Default::default(),
            })
        }
    }
    #[async_trait::async_trait]
    impl HttpTransport for Script {
        async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
            self.seen.lock().unwrap().push(request);
            Ok(self.answers.lock().unwrap().remove(0))
        }
    }

    fn resp(status: u16, body: &str, headers: &[(&str, &str)]) -> HttpResponse {
        HttpResponse {
            status,
            headers: headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            body: body.to_string(),
        }
    }

    #[test]
    fn a_304_is_served_from_the_cached_body_and_sends_if_none_match() {
        let inner = Script::new(vec![
            resp(200, "[1]", &[("etag", "\"abc\"")]),
            resp(304, "", &[]),
        ]);
        let t = ConditionalTransport::with_clock(inner.clone(), clock);
        let url = "https://api.github.com/repos/a/b/pulls?state=open";
        let first = block_on(t.send(HttpRequest::get(url))).unwrap();
        assert_eq!(first.body, "[1]");
        let second = block_on(t.send(HttpRequest::get(url))).unwrap();
        assert_eq!(second.status, 200);
        assert_eq!(second.body, "[1]");
        assert_eq!(t.not_modified_count(), 1);
        let seen = inner.seen.lock().unwrap();
        assert!(seen[0].headers.iter().all(|(k, _)| k != "If-None-Match"));
        assert!(seen[1]
            .headers
            .contains(&("If-None-Match".to_string(), "\"abc\"".to_string())));
    }

    #[test]
    fn a_full_cache_drops_its_oldest_entry_not_all_of_them() {
        let urls: Vec<String> = (0..MAX_ENTRIES + 5)
            .map(|i| format!("https://api.github.com/repos/a/b{i}/pulls"))
            .collect();
        let mut answers: Vec<HttpResponse> = urls
            .iter()
            .map(|_| resp(200, "[]", &[("etag", "\"e\"")]))
            .collect();
        // The probes below: a recent entry and a long-evicted one.
        answers.push(resp(304, "", &[]));
        answers.push(resp(200, "[]", &[("etag", "\"e\"")]));
        let inner = Script::new(answers);
        let t = ConditionalTransport::with_clock(inner.clone(), clock);
        for url in &urls {
            block_on(t.send(HttpRequest::get(url))).unwrap();
        }
        // An entry from the first half survived (the old behaviour cleared
        // everything the moment the cache filled).
        block_on(t.send(HttpRequest::get(&urls[10]))).unwrap();
        block_on(t.send(HttpRequest::get(&urls[0]))).unwrap();
        let seen = inner.seen.lock().unwrap();
        let conditional = |i: usize| {
            seen[i]
                .headers
                .iter()
                .any(|(k, _)| k.eq_ignore_ascii_case("If-None-Match"))
        };
        assert!(conditional(urls.len()), "entry 10 is still cached");
        assert!(!conditional(urls.len() + 1), "entry 0 was the oldest");
    }

    #[test]
    fn an_exhausted_host_is_not_hammered_until_its_reset() {
        NOW.with(|n| n.set(1_000));
        let inner = Script::new(vec![resp(
            200,
            "[]",
            &[
                ("x-ratelimit-remaining", "0"),
                ("x-ratelimit-reset", "1300"),
            ],
        )]);
        let t = ConditionalTransport::with_clock(inner.clone(), clock);
        let url = "https://api.github.com/user";
        block_on(t.send(HttpRequest::get(url))).unwrap();
        let err = block_on(t.send(HttpRequest::get(url))).unwrap_err();
        assert!(matches!(
            err,
            Error::RateLimited {
                retry_in_secs: 300,
                ..
            }
        ));
        assert_eq!(inner.seen.lock().unwrap().len(), 1, "no second request");
        assert!(t.rate_note().unwrap().contains("paused for 5m"));
        // After the reset it asks again.
        NOW.with(|n| n.set(1_301));
        inner
            .answers
            .lock()
            .unwrap()
            .push(resp(200, "[]", &[("x-ratelimit-remaining", "4999")]));
        assert!(block_on(t.send(HttpRequest::get(url))).is_ok());
        assert!(t.rate_note().is_none());
    }

    #[test]
    fn a_low_count_warns_but_still_sends() {
        NOW.with(|n| n.set(2_000));
        let inner = Script::new(vec![resp(
            200,
            "[]",
            &[("ratelimit-remaining", "7"), ("ratelimit-reset", "2600")],
        )]);
        let t = ConditionalTransport::with_clock(inner, clock);
        block_on(t.send(HttpRequest::get("https://gitlab.example.com/api/v4/x"))).unwrap();
        let note = t.rate_note().unwrap();
        assert!(
            note.contains("gitlab.example.com") && note.contains("7 left"),
            "{note}"
        );
    }

    #[test]
    fn a_429_with_retry_after_becomes_a_rate_limit_error() {
        NOW.with(|n| n.set(3_000));
        let inner = Script::new(vec![resp(429, "slow down", &[("retry-after", "30")])]);
        let t = ConditionalTransport::with_clock(inner, clock);
        let err = block_on(t.send(HttpRequest::get("https://gitlab.com/api/v4/x"))).unwrap_err();
        assert!(matches!(
            err,
            Error::RateLimited {
                retry_in_secs: 30,
                ..
            }
        ));
    }

    #[test]
    fn a_plain_403_stays_a_scope_problem() {
        let r = resp(403, "{}", &[("x-ratelimit-remaining", "100")]);
        assert!(!is_rate_limited(&r));
        assert!(is_rate_limited(&resp(
            403,
            "{}",
            &[("x-ratelimit-remaining", "0")]
        )));
    }

    #[test]
    fn azure_tstu_counts_never_trip_the_low_water_note() {
        NOW.with(|n| n.set(4_000));
        let inner = Script::new(vec![
            resp(
                200,
                "{}",
                &[
                    ("X-RateLimit-Remaining", "3"),
                    ("X-RateLimit-Resource", "git"),
                    ("X-RateLimit-Reset", "4100"),
                ],
            ),
            resp(200, "{}", &[("X-RateLimit-Remaining", "0")]),
        ]);
        let t = ConditionalTransport::with_clock(inner.clone(), clock);
        let url = "https://dev.azure.com/org/_apis/projects";
        block_on(t.send(HttpRequest::get(url))).unwrap();
        assert!(t.rate_note().is_none(), "TSTUs are not calls");
        // Even "0 left" in TSTUs does not stop the next call.
        block_on(t.send(HttpRequest::get(url))).unwrap();
        assert!(t.rate_note().is_none());
    }

    #[test]
    fn azure_backs_off_on_429_and_names_the_resource() {
        NOW.with(|n| n.set(4_000));
        let inner = Script::new(vec![resp(
            429,
            "",
            &[("Retry-After", "45"), ("X-RateLimit-Resource", "git")],
        )]);
        let t = ConditionalTransport::with_clock(inner.clone(), clock);
        let url = "https://dev.azure.com/org/_apis/projects";
        let err = block_on(t.send(HttpRequest::get(url))).unwrap_err();
        assert!(matches!(
            err,
            Error::RateLimited {
                retry_in_secs: 45,
                ..
            }
        ));
        let note = t.rate_note().unwrap();
        assert!(
            note.contains("dev.azure.com") && note.contains("git"),
            "{note}"
        );
        assert!(block_on(t.send(HttpRequest::get(url))).is_err());
        assert_eq!(
            inner.seen.lock().unwrap().len(),
            1,
            "blocked until the deadline"
        );
    }

    #[test]
    fn an_azure_403_is_a_permission_problem_even_with_zero_tstus_left() {
        let r = resp(403, "{}", &[("x-ratelimit-remaining", "0")]);
        assert!(!is_rate_limited_for(ApiFamily::AzureDevops, &r));
        assert!(is_rate_limited_for(ApiFamily::Github, &r));
        assert!(is_rate_limited_for(
            ApiFamily::AzureDevops,
            &resp(429, "", &[])
        ));
    }

    #[test]
    fn bitbucket_near_limit_speaks_up_but_still_sends() {
        NOW.with(|n| n.set(5_000));
        let inner = Script::new(vec![
            resp(200, "{}", &[("X-RateLimit-NearLimit", "true")]),
            resp(200, "{}", &[]),
        ]);
        let t = ConditionalTransport::with_clock(inner.clone(), clock);
        let url = "https://api.bitbucket.org/2.0/user";
        block_on(t.send(HttpRequest::get(url))).unwrap();
        let note = t.rate_note().unwrap();
        assert!(
            note.contains("api.bitbucket.org") && note.contains("nearly"),
            "{note}"
        );
        // Still asked; the next calm answer clears the note.
        block_on(t.send(HttpRequest::get(url))).unwrap();
        assert!(t.rate_note().is_none());
    }

    #[test]
    fn trello_counts_the_token_bucket() {
        NOW.with(|n| n.set(6_000));
        let inner = Script::new(vec![
            resp(
                200,
                "[]",
                &[
                    ("x-rate-limit-api-token-remaining", "5"),
                    ("x-rate-limit-api-token-interval-ms", "10000"),
                ],
            ),
            resp(
                200,
                "[]",
                &[
                    ("x-rate-limit-api-token-remaining", "0"),
                    ("x-rate-limit-api-token-interval-ms", "10000"),
                ],
            ),
        ]);
        let t = ConditionalTransport::with_clock(inner.clone(), clock);
        let url = "https://api.trello.com/1/boards/x/cards";
        block_on(t.send(HttpRequest::get(url))).unwrap();
        let note = t.rate_note().unwrap();
        assert!(
            note.contains("api.trello.com") && note.contains("5 left"),
            "{note}"
        );
        block_on(t.send(HttpRequest::get(url))).unwrap();
        let err = block_on(t.send(HttpRequest::get(url))).unwrap_err();
        assert!(matches!(
            err,
            Error::RateLimited {
                retry_in_secs: 10,
                ..
            }
        ));
    }

    #[test]
    fn jira_429_blocks_until_the_retry_after_deadline_and_keeps_the_reason() {
        NOW.with(|n| n.set(7_000));
        let inner = Script::new(vec![
            resp(
                429,
                "",
                &[
                    ("Retry-After", "120"),
                    ("RateLimit-Reason", "jira-burst-based"),
                ],
            ),
            resp(200, "{}", &[]),
        ]);
        let t = ConditionalTransport::with_clock(inner.clone(), clock);
        let url = "https://acme.atlassian.net/rest/api/3/myself";
        let err = block_on(t.send(HttpRequest::get(url))).unwrap_err();
        assert!(matches!(
            err,
            Error::RateLimited {
                retry_in_secs: 120,
                ..
            }
        ));
        assert!(t.rate_note().unwrap().contains("jira-burst-based"));
        NOW.with(|n| n.set(7_060));
        assert!(block_on(t.send(HttpRequest::get(url))).is_err());
        assert_eq!(inner.seen.lock().unwrap().len(), 1);
        NOW.with(|n| n.set(7_121));
        assert!(block_on(t.send(HttpRequest::get(url))).is_ok());
        assert!(t.rate_note().is_none());
    }

    #[test]
    fn jira_ignores_generic_count_headers() {
        NOW.with(|n| n.set(7_000));
        let inner = Script::new(vec![resp(200, "{}", &[("ratelimit-remaining", "1")])]);
        let t = ConditionalTransport::with_clock(inner, clock);
        block_on(t.send(HttpRequest::get(
            "https://acme.atlassian.net/rest/api/3/myself",
        )))
        .unwrap();
        assert!(t.rate_note().is_none());
    }

    #[test]
    fn a_self_hosted_host_is_read_by_its_registered_family() {
        NOW.with(|n| n.set(8_000));
        let inner = Script::new(vec![resp(200, "{}", &[("X-RateLimit-Remaining", "2")])]);
        let t = ConditionalTransport::with_clock(inner, clock);
        t.register_host("tfs.corp.example", ApiFamily::AzureDevops);
        block_on(t.send(HttpRequest::get("https://tfs.corp.example/tfs/c/_apis/x"))).unwrap();
        assert!(t.rate_note().is_none());
    }
}
