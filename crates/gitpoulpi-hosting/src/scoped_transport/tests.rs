use super::*;
use crate::http::Method;
use std::sync::Mutex;

/// Hands out `tokens` in order (the last one repeats); records the asks.
pub(crate) struct Scopes {
    pub(crate) tokens: Mutex<Vec<String>>,
    pub(crate) repo: &'static str,
    pub(crate) asks: Mutex<Vec<bool>>,
    pub(crate) blocked: bool,
}

#[async_trait::async_trait]
impl RepoScopes for Scopes {
    fn assignment_blocked(&self, _: &str, project: &str) -> bool {
        self.blocked && project.eq_ignore_ascii_case(self.repo)
    }
    async fn token(&self, _: &str, project: &str, refresh: bool) -> Option<Result<String>> {
        if !project.eq_ignore_ascii_case(self.repo) {
            return None;
        }
        self.asks.lock().unwrap().push(refresh);
        let mut tokens = self.tokens.lock().unwrap();
        Some(Ok(if tokens.len() > 1 {
            tokens.remove(0)
        } else {
            tokens[0].clone()
        }))
    }
    fn acts_as_bot(&self, _: &str, _: &str) -> bool {
        true
    }
}

/// Answers 401 to the token `ghs_old`, 200 to any other; records the
/// Authorization header of each request.
#[derive(Default)]
pub(crate) struct Host {
    pub(crate) seen: Mutex<Vec<String>>,
}

#[async_trait::async_trait]
impl HttpTransport for Host {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        let auth = request
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let status = if auth.contains("ghs_old") { 401 } else { 200 };
        self.seen.lock().unwrap().push(auth);
        Ok(HttpResponse {
            status,
            headers: Default::default(),
            body: "{}".into(),
        })
    }
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(f)
}

fn get(url: &str) -> HttpRequest {
    HttpRequest {
        method: Method::Get,
        url: url.into(),
        headers: vec![("Authorization".into(), "Bearer personal".into())],
        body: None,
    }
}

fn setup(tokens: &[&str]) -> (Arc<Host>, Arc<Scopes>, ScopedTransport) {
    let host = Arc::new(Host::default());
    let scopes = Arc::new(Scopes {
        tokens: Mutex::new(tokens.iter().map(|t| t.to_string()).collect()),
        repo: "acme/app",
        asks: Mutex::default(),
        blocked: false,
    });
    let transport = ScopedTransport::new(host.clone(), "gh".into()).with_scopes(scopes.clone());
    (host, scopes, transport)
}

#[test]
fn the_repository_calls_carry_the_commands_token_and_others_the_personal_one() {
    let (host, _scopes, transport) = setup(&["ghs_new"]);
    block_on(transport.send(get("https://api.example.com/repos/acme/app/issues"))).unwrap();
    block_on(transport.send(get("https://api.example.com/repos/other/lib/issues"))).unwrap();
    block_on(transport.send(get("https://api.example.com/user"))).unwrap();
    assert_eq!(
        *host.seen.lock().unwrap(),
        vec!["Bearer ghs_new", "Bearer personal", "Bearer personal"]
    );
}

#[test]
fn a_401_asks_the_command_again_once_and_retries() {
    let (host, scopes, transport) = setup(&["ghs_old", "ghs_new"]);
    let response =
        block_on(transport.send(get("https://api.example.com/repos/acme/app/issues"))).unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(
        *host.seen.lock().unwrap(),
        vec!["Bearer ghs_old", "Bearer ghs_new"]
    );
    assert_eq!(*scopes.asks.lock().unwrap(), vec![false, true]);
}

#[test]
fn a_second_401_is_answered_as_is_not_retried_again() {
    let (host, _scopes, transport) = setup(&["ghs_old"]);
    let response =
        block_on(transport.send(get("https://api.example.com/repos/acme/app/issues"))).unwrap();
    assert_eq!(response.status, 401);
    assert_eq!(host.seen.lock().unwrap().len(), 2);
}
