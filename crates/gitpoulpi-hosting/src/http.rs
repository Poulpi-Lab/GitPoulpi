//! The HTTP seam.
//!
//! Every request the GitHub and GitLab clients make goes through
//! [`HttpTransport`]; nothing in this crate constructs a socket itself,
//! so the whole crate is testable without a network. The clients' tests
//! drive [`RecordedTransport`]; only `ReqwestTransport` talks to a host.

use std::collections::BTreeMap;

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    /// Already-serialized JSON, or `None`.
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: Method::Get,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn delete(url: impl Into<String>) -> Self {
        Self {
            method: Method::Delete,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn post_json(url: impl Into<String>, body: &serde_json::Value) -> Self {
        Self {
            method: Method::Post,
            url: url.into(),
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: Some(body.to_string()),
        }
    }

    pub fn put_json(url: impl Into<String>, body: &serde_json::Value) -> Self {
        Self {
            method: Method::Put,
            ..Self::post_json(url, body)
        }
    }

    pub fn patch_json(url: impl Into<String>, body: &serde_json::Value) -> Self {
        Self {
            method: Method::Patch,
            ..Self::post_json(url, body)
        }
    }

    /// A DELETE that carries a JSON body (GitHub's reviewer removal).
    pub fn delete_json(url: impl Into<String>, body: &serde_json::Value) -> Self {
        Self {
            method: Method::Delete,
            ..Self::post_json(url, body)
        }
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Turns a non-2xx into a typed error carrying whatever the host said.
    /// 401 and 403 are separated because they are the two a user can fix,
    /// and a bare "401" sends people looking for a network problem that
    /// was a stale token.
    pub fn error_for_status(&self, what: &str) -> Result<()> {
        if self.is_success() {
            return Ok(());
        }
        if crate::conditional::is_rate_limited(self) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let wait = crate::conditional::rate_state(self, now)
                .map(|(_, reset)| reset.saturating_sub(now))
                .unwrap_or(60);
            return Err(Error::RateLimited {
                host: what.to_string(),
                retry_in_secs: wait.max(1),
            });
        }
        let detail = host_message(&self.body).unwrap_or_else(|| self.body.trim().to_string());
        let detail = truncate(&detail, 300);
        Err(match self.status {
            401 => Error::Unauthorized {
                what: what.to_string(),
                detail,
            },
            403 => Error::Forbidden {
                what: what.to_string(),
                detail,
            },
            status => Error::Api {
                what: what.to_string(),
                status,
                detail,
            },
        })
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_str(&self.body).map_err(|e| Error::Decode(e.to_string()))
    }
}

/// GitHub answers `{"message": "Bad credentials"}`, GitLab answers
/// `{"message": "401 Unauthorized"}` or `{"error": "..."}`. Reading the
/// one the host actually sent beats printing the raw body, which for a
/// misconfigured base URL is a page of HTML.
fn host_message(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    for key in ["message", "error", "error_description"] {
        match value.get(key) {
            Some(serde_json::Value::String(text)) => {
                // GitHub's 422 keeps the useful part in `errors`.
                let extra: Vec<String> = value
                    .get("errors")
                    .and_then(|e| e.as_array())
                    .map(|rows| {
                        rows.iter()
                            .map(|row| match row {
                                serde_json::Value::String(s) => s.clone(),
                                other => other
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .map(str::to_string)
                                    .unwrap_or_else(|| {
                                        let field = other
                                            .get("field")
                                            .and_then(|f| f.as_str())
                                            .unwrap_or("?");
                                        let code = other
                                            .get("code")
                                            .and_then(|c| c.as_str())
                                            .unwrap_or("invalid");
                                        format!("{field}: {code}")
                                    }),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                return Some(if extra.is_empty() {
                    text.clone()
                } else {
                    format!("{text} ({})", extra.join("; "))
                });
            }
            // GitLab's validation errors: `{"message": {"title": ["can't be blank"]}}`.
            Some(obj @ serde_json::Value::Object(_)) => return Some(obj.to_string()),
            _ => {}
        }
    }
    None
}

fn truncate(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max).collect();
    format!("{cut}...")
}

#[async_trait::async_trait]
pub trait HttpTransport: Send + Sync {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse>;

    /// Fetches a binary body (a release archive). `progress(received, total)`
    /// runs after every chunk; returning `false` cancels the download. The
    /// default refuses: only transports that can stream bytes implement it.
    async fn download(&self, request: HttpRequest, progress: &Progress<'_>) -> Result<Vec<u8>> {
        let _ = (request, progress);
        Err(Error::Http("this transport cannot download files".into()))
    }
}

/// A download's progress callback: bytes received so far and the total when
/// the host said it; `false` stops the download.
pub type Progress<'a> = dyn Fn(u64, Option<u64>) -> bool + Send + Sync + 'a;

/// A transport that answers from a table instead of from the network.
///
/// Public rather than `#[cfg(test)]` so the daemon, CLI and TUI can reuse
/// it. It records what it was asked, so a test can assert on the
/// *request* (the auth header and pagination parameters).
#[derive(Default)]
pub struct RecordedTransport {
    responses: std::sync::Mutex<Vec<(String, HttpResponse)>>,
    files: std::sync::Mutex<Vec<(String, Vec<u8>)>>,
    seen: std::sync::Mutex<Vec<HttpRequest>>,
}

impl RecordedTransport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Answers any request whose URL *contains* `url_fragment` with
    /// `status`/`body`, so a test need not spell out every query parameter
    /// it does not care about; `requests()` covers the ones it does.
    pub fn on(self, url_fragment: &str, status: u16, body: &str) -> Self {
        self.responses.lock().unwrap().push((
            url_fragment.to_string(),
            HttpResponse {
                status,
                headers: BTreeMap::new(),
                body: body.to_string(),
            },
        ));
        self
    }

    /// Like `on`, plus response headers - for the `Link:` header both
    /// hosts paginate with.
    pub fn on_with_headers(
        self,
        url_fragment: &str,
        status: u16,
        body: &str,
        headers: &[(&str, &str)],
    ) -> Self {
        self.responses.lock().unwrap().push((
            url_fragment.to_string(),
            HttpResponse {
                status,
                headers: headers
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                body: body.to_string(),
            },
        ));
        self
    }

    /// Answers a [`HttpTransport::download`] whose URL contains `url_fragment`
    /// with `bytes`, in chunks of 64 KiB so the progress callback runs.
    pub fn on_file(self, url_fragment: &str, bytes: Vec<u8>) -> Self {
        self.files
            .lock()
            .unwrap()
            .push((url_fragment.to_string(), bytes));
        self
    }

    pub fn requests(&self) -> Vec<HttpRequest> {
        self.seen.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl HttpTransport for RecordedTransport {
    async fn download(&self, request: HttpRequest, progress: &Progress<'_>) -> Result<Vec<u8>> {
        self.seen.lock().unwrap().push(request.clone());
        let bytes = {
            let files = self.files.lock().unwrap();
            files
                .iter()
                .find(|(fragment, _)| request.url.contains(fragment.as_str()))
                .map(|(_, bytes)| bytes.clone())
        };
        let Some(bytes) = bytes else {
            return Err(Error::Http(format!(
                "RecordedTransport has no file for {}",
                request.url
            )));
        };
        let total = bytes.len() as u64;
        let mut received = 0u64;
        for chunk in bytes.chunks(64 * 1024) {
            received += chunk.len() as u64;
            if !progress(received, Some(total)) {
                return Err(Error::Http("download canceled".into()));
            }
        }
        Ok(bytes)
    }

    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        self.seen.lock().unwrap().push(request.clone());
        let responses = self.responses.lock().unwrap();
        // First match wins, and the table is consulted in insertion
        // order, so a test that registers two pages of the same endpoint
        // orders them the way the host would answer them.
        let position = responses
            .iter()
            .position(|(fragment, _)| request.url.contains(fragment.as_str()));
        match position {
            Some(i) => Ok(responses[i].1.clone()),
            None => Err(Error::Http(format!(
                "RecordedTransport has no response for {} {}",
                request.method.as_str(),
                request.url
            ))),
        }
    }
}

/// Env var that makes the real transports refuse non-loopback hosts. Set by
/// the PTY driver (`xtask`) for the TUI and the daemon it starts; always on
/// in this crate's own unit tests.
pub const BLOCK_EXTERNAL_ENV: &str = "GITPOULPI_BLOCK_EXTERNAL_HOSTS";
/// The master switch of live provider tests (`gitpoulpi_testkit::live`).
pub const LIVE_TESTS_ENV: &str = "GITPOULPI_LIVE_TESTS";

/// `Some(reason)` when a test build must not send `url`: the host is not
/// loopback, blocking is on (`cfg(test)` or [`BLOCK_EXTERNAL_ENV`]) and
/// `GITPOULPI_LIVE_TESTS=1` is not set. A leaked real call fails loudly
/// instead of connecting. Production builds (neither set) never refuse.
pub fn external_host_refusal(url: &str) -> Option<String> {
    let on = |name: &str| matches!(std::env::var(name).as_deref(), Ok("1") | Ok("true"));
    refusal(
        url,
        cfg!(test) || on(BLOCK_EXTERNAL_ENV),
        on(LIVE_TESTS_ENV),
    )
}

fn refusal(url: &str, blocking: bool, live: bool) -> Option<String> {
    if !blocking || live {
        return None;
    }
    let host = url_host(url)?;
    if is_loopback_host(&host) {
        return None;
    }
    Some(format!(
        "refusing to connect to {host} from a test: tests only reach loopback; \
         set {LIVE_TESTS_ENV}=1 for a live run"
    ))
}

fn url_host(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let authority = rest.split(['/', '?', '#']).next()?;
    let hostport = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = if let Some(v6) = hostport.strip_prefix('[') {
        v6.split(']').next()?
    } else {
        hostport.split(':').next()?
    };
    Some(host.to_ascii_lowercase())
}

fn is_loopback_host(host: &str) -> bool {
    host == "localhost" || host.ends_with(".localhost") || host == "::1" || host.starts_with("127.")
}

/// The real transport. Built once per client, not per request:
/// `reqwest::Client` owns the connection pool.
#[cfg(feature = "network")]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

#[cfg(feature = "network")]
impl ReqwestTransport {
    pub fn new() -> Result<Self> {
        Self::with_timeout(std::time::Duration::from_secs(30))
    }

    /// Like [`ReqwestTransport::new`] with a caller-chosen request
    /// timeout - a language-model answer routinely outlasts the 30 s
    /// that is right for a hosting API.
    pub fn with_timeout(timeout: std::time::Duration) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!("gitpoulpi/", env!("CARGO_PKG_VERSION")))
            // A hung host must not hang the daemon's connection with it.
            .timeout(timeout)
            // Callers run each task on a runtime of its own and share this
            // transport: a kept-alive connection is a task of the runtime that
            // opened it, frozen once that one is parked, and the request that
            // picks it up hangs until the timeout. No connection outlives its
            // request.
            .pool_max_idle_per_host(0)
            .build()
            .map_err(|e| Error::Http(e.to_string()))?;
        Ok(Self { client })
    }
}

#[cfg(feature = "network")]
#[async_trait::async_trait]
impl HttpTransport for ReqwestTransport {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        let method = match request.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Delete => reqwest::Method::DELETE,
        };
        if let Some(why) = external_host_refusal(&request.url) {
            return Err(Error::Http(why));
        }
        let mut builder = self.client.request(method, &request.url);
        for (name, value) in &request.headers {
            builder = builder.header(name, value);
        }
        if let Some(body) = request.body {
            builder = builder.body(body);
        }
        let response = builder
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(k, v)| Some((k.as_str().to_string(), v.to_str().ok()?.to_string())))
            .collect();
        let body = response
            .text()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    /// Streams the body chunk by chunk. Redirects are followed only to the
    /// hosts [`crate::updates::download_host_allowed`] names (a release asset
    /// redirects to GitHub's CDN), and the first URL must be one of them too.
    async fn download(&self, request: HttpRequest, progress: &Progress<'_>) -> Result<Vec<u8>> {
        if !crate::updates::download_host_allowed(&request.url) {
            return Err(Error::Http(format!(
                "refusing to download from {}: not a GitHub release host",
                request.url
            )));
        }
        if let Some(why) = external_host_refusal(&request.url) {
            return Err(Error::Http(why));
        }
        let policy = reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 5 {
                attempt.error("too many redirects")
            } else if crate::updates::download_host_allowed(attempt.url().as_str()) {
                attempt.follow()
            } else {
                let host = attempt.url().host_str().unwrap_or("?").to_string();
                attempt.error(format!(
                    "redirect to {host} refused: not a GitHub release host"
                ))
            }
        });
        let client = reqwest::Client::builder()
            .user_agent(concat!("gitpoulpi/", env!("CARGO_PKG_VERSION")))
            .redirect(policy)
            // A stalled connection ends; a slow but moving one may take long.
            .read_timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(15))
            .pool_max_idle_per_host(0)
            .build()
            .map_err(|e| Error::Http(e.to_string()))?;
        let mut builder = client.get(&request.url);
        for (name, value) in &request.headers {
            builder = builder.header(name, value);
        }
        let mut response = builder
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Http(format!(
                "the download answered HTTP {}",
                status.as_u16()
            )));
        }
        let total = response.content_length();
        let mut out = Vec::with_capacity(total.unwrap_or(0).min(256 << 20) as usize);
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| Error::Http(e.to_string()))?
        {
            out.extend_from_slice(&chunk);
            if !progress(out.len() as u64, total) {
                return Err(Error::Http("download canceled".into()));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hosts_own_error_message_survives_into_the_error() {
        let response = HttpResponse {
            status: 401,
            headers: BTreeMap::new(),
            body: r#"{"message":"Bad credentials"}"#.into(),
        };
        let err = response.error_for_status("listing projects").unwrap_err();
        let text = err.to_string();
        assert!(text.contains("Bad credentials"), "{text}");
        assert!(text.contains("listing projects"), "{text}");
        assert!(matches!(err, Error::Unauthorized { .. }));
    }

    /// A base URL pointing at something that is not an API answers with
    /// an HTML page. Printing all of it into a TUI status line is the
    /// failure mode this guards.
    #[test]
    fn a_non_json_error_body_is_truncated_rather_than_dumped() {
        let response = HttpResponse {
            status: 500,
            headers: BTreeMap::new(),
            body: "x".repeat(5000),
        };
        let text = response
            .error_for_status("listing projects")
            .unwrap_err()
            .to_string();
        assert!(text.len() < 400, "error was {} chars", text.len());
        assert!(text.ends_with("..."));
    }

    #[test]
    fn a_success_is_not_an_error() {
        let response = HttpResponse {
            status: 204,
            headers: BTreeMap::new(),
            body: String::new(),
        };
        assert!(response.error_for_status("anything").is_ok());
    }

    /// The transport is shared by tasks that each run on a runtime of their
    /// own, some of which stay alive without being driven any more. A pooled
    /// connection is a task of the runtime that opened it, so on such a
    /// runtime it is frozen: the next request that picks it up hangs until
    /// the timeout (seen against Gitea in the review view). No connection
    /// may outlive its request.
    #[cfg(feature = "network")]
    #[test]
    fn a_request_never_picks_up_a_connection_frozen_on_another_runtime() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/x", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                std::thread::spawn(move || {
                    let mut stream = stream;
                    let mut seen = Vec::new();
                    let mut buf = [0u8; 1024];
                    // keep-alive: answer every request on the connection
                    while let Ok(n) = stream.read(&mut buf) {
                        if n == 0 {
                            return;
                        }
                        seen.extend_from_slice(&buf[..n]);
                        while let Some(end) = seen.windows(4).position(|w| w == b"\r\n\r\n") {
                            seen.drain(..end + 4);
                            let _ =
                                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
                        }
                    }
                });
            }
        });
        let transport = ReqwestTransport::with_timeout(std::time::Duration::from_secs(3)).unwrap();
        let mut parked = Vec::new();
        for _ in 0..3 {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let request = HttpRequest {
                method: Method::Get,
                url: url.clone(),
                headers: Vec::new(),
                body: None,
            };
            let response = rt.block_on(transport.send(request)).unwrap();
            assert_eq!(response.body, "ok");
            parked.push(rt);
        }
    }

    #[test]
    fn a_blocked_build_refuses_external_hosts_but_not_loopback() {
        for url in [
            "http://127.0.0.1:3000/api",
            "http://localhost:8081/",
            "http://[::1]:80/x",
            "http://user:pw@127.0.0.1/",
        ] {
            assert_eq!(refusal(url, true, false), None, "{url}");
        }
        for url in [
            "https://api.github.com/user",
            "https://gitlab.com/api/v4",
            "http://user@example.org:80/x",
        ] {
            assert!(refusal(url, true, false).is_some(), "{url}");
            assert_eq!(refusal(url, true, true), None, "live switch lifts it");
            assert_eq!(refusal(url, false, false), None, "production never refuses");
        }
    }
}
