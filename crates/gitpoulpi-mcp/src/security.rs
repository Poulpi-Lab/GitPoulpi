//! What guards the HTTP endpoint: the bearer-token layer, the TLS setup and
//! the pure rule for `--allow-remote`.
//!
//! * The bearer layer ([`BearerGate`]) runs before `StreamableHttpService`
//!   and answers 401 with `WWW-Authenticate: Bearer` unless the request
//!   carries the token (constant-time comparison). The token file is read on
//!   every request, so `gitpoulpi mcp token --rotate` takes effect without a
//!   restart; an unreadable file refuses everything.
//! * TLS is terminated here (rustls). Only a finished handshake reaches the
//!   HTTP layer and no plain listener exists beside it, so "remote traffic
//!   is encrypted" is established per connection, not assumed.
//! * A tunnel or reverse proxy cannot be detected from inside (it connects
//!   like any client and its headers can be forged), so
//!   `[mcp] behind_proxy` is an explicit acknowledgement.

use std::convert::Infallible;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use bytes::Bytes;
use gitpoulpi_config::mcp_secrets;
use gitpoulpi_config::{secure_fs, ConfigPaths, McpConfig, McpTls};
use http::{header, HeaderMap, Request, Response, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use subtle::ConstantTimeEq;

/// The body type `StreamableHttpService` answers with.
pub type BoxResponse = Response<BoxBody<Bytes, Infallible>>;

/// Where the expected token comes from.
#[derive(Debug, Clone)]
pub enum TokenSource {
    /// A fixed value (tests, embedders).
    Fixed(String),
    /// The token file, read per request.
    File(PathBuf),
}

impl TokenSource {
    /// The token in force now; `None` when there is none to check against.
    pub fn current(&self) -> Option<String> {
        match self {
            TokenSource::Fixed(token) => Some(token.clone()),
            TokenSource::File(path) => {
                let text = std::fs::read_to_string(path).ok()?;
                let text = text.trim();
                (!text.is_empty()).then(|| text.to_string())
            }
        }
    }
}

/// What the endpoint enforces.
#[derive(Clone, Default)]
pub struct Security {
    /// `None`: no bearer check (`http_auth = false`).
    pub token: Option<TokenSource>,
    /// `None`: plain HTTP.
    pub tls: Option<Arc<rustls::ServerConfig>>,
    /// SHA-256 fingerprint of the served certificate, for the start notice.
    pub fingerprint: Option<String>,
}

impl Security {
    /// No authentication, no TLS: the loopback test setup.
    pub fn open() -> Self {
        Self::default()
    }
}

/// Whether `Authorization` carries `Bearer <expected>`.
pub fn bearer_matches(headers: &HeaderMap, expected: &str) -> bool {
    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Some((scheme, presented)) = value.split_once(' ') else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("bearer") {
        return false;
    }
    presented
        .trim()
        .as_bytes()
        .ct_eq(expected.as_bytes())
        .into()
}

fn unauthorized() -> BoxResponse {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, "Bearer")
        .body(
            Full::new(Bytes::from_static(
                b"missing or wrong bearer token (see `gitpoulpi mcp token`)",
            ))
            .boxed(),
        )
        .expect("valid response")
}

/// A tower service that checks the bearer token, then calls `inner`.
#[derive(Clone)]
pub struct BearerGate<S> {
    inner: S,
    token: Option<TokenSource>,
}

impl<S> BearerGate<S> {
    pub fn new(inner: S, token: Option<TokenSource>) -> Self {
        Self { inner, token }
    }
}

impl<S, B> tower_service::Service<Request<B>> for BearerGate<S>
where
    S: tower_service::Service<Request<B>, Response = BoxResponse, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = BoxResponse;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<BoxResponse, Infallible>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        if let Some(source) = &self.token {
            let allowed = source
                .current()
                .is_some_and(|expected| bearer_matches(request.headers(), &expected));
            if !allowed {
                return Box::pin(std::future::ready(Ok(unauthorized())));
            }
        }
        // The usual tower clone dance: the ready service stays here, the
        // clone (ready too, the services are stateless) serves the request.
        let clone = self.inner.clone();
        let mut ready = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move { ready.call(request).await })
    }
}

/// The pure rule for a non-loopback listen address: authentication on with a
/// token in place, and encryption (TLS here, or a proxy the user vouches
/// for). The refusal names everything missing.
pub fn remote_decision(
    http_auth: bool,
    token_exists: bool,
    tls_on: bool,
    behind_proxy: bool,
) -> Result<(), String> {
    let mut missing: Vec<&str> = Vec::new();
    if !http_auth {
        missing.push("authentication is off (set mcp.http_auth = true)");
    } else if !token_exists {
        missing.push("there is no token yet (run `gitpoulpi mcp token`)");
    }
    if !tls_on && !behind_proxy {
        missing.push(
            "the traffic would be unencrypted (set mcp.tls = \"self_signed\" or \"files\", or \
             mcp.behind_proxy = true if a tunnel or reverse proxy encrypts it)",
        );
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "refusing --allow-remote: {}. The safest setup keeps the loopback address and \
             reaches it through `ssh -L`.",
            missing.join("; and ")
        ))
    }
}

/// Everything `serve_http` decides before it listens: the `--allow-remote`
/// rule, the token (created on first use), the TLS configuration (a
/// self-signed pair generated on first use). `Err` is the refusal text.
pub fn prepare(
    cfg: &McpConfig,
    paths: Option<&ConfigPaths>,
    addr: SocketAddr,
    allow_remote: bool,
) -> Result<Security, String> {
    let paths_or_err = || {
        paths.ok_or_else(|| "no config folder is available to keep the MCP secrets in".to_string())
    };
    let mut security = Security::default();
    let mut token_exists = false;
    if cfg.http_auth {
        let paths = paths_or_err()?;
        mcp_secrets::ensure_token(paths).map_err(|e| format!("cannot keep the MCP token: {e}"))?;
        security.token = Some(TokenSource::File(mcp_secrets::token_file(paths)));
        token_exists = true;
    }
    if cfg.tls.is_on() {
        let (config, fingerprint) = match cfg.tls {
            McpTls::SelfSigned => {
                let paths = paths_or_err()?;
                let (cert, key) = ensure_self_signed(paths, addr.ip())?;
                load_server_config(&cert, &key)?
            }
            _ => {
                let (cert, key) = (cfg.tls_cert.trim(), cfg.tls_key.trim());
                if cert.is_empty() || key.is_empty() {
                    return Err(
                        "mcp.tls = \"files\" needs mcp.tls_cert and mcp.tls_key (PEM paths)".into(),
                    );
                }
                load_server_config(Path::new(cert), Path::new(key))?
            }
        };
        security.tls = Some(config);
        security.fingerprint = fingerprint;
    }
    if !addr.ip().is_loopback() {
        if !allow_remote {
            return Err(format!(
                "refusing to listen on non-loopback address {addr}: pass --allow-remote, which \
                 needs authentication (mcp.http_auth) and TLS (mcp.tls) or mcp.behind_proxy"
            ));
        }
        remote_decision(
            cfg.http_auth,
            token_exists,
            cfg.tls.is_on(),
            cfg.behind_proxy,
        )?;
    }
    Ok(security)
}

/// The names a self-signed certificate answers to.
pub fn self_signed_names(bind_ip: IpAddr) -> Vec<String> {
    let mut names: Vec<String> = ["localhost", "127.0.0.1", "::1"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let host = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .ok()
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty());
    if let Some(host) = host {
        names.push(host);
    }
    if !bind_ip.is_unspecified() && !bind_ip.is_loopback() {
        names.push(bind_ip.to_string());
    }
    names.sort();
    names.dedup();
    names
}

/// The self-signed pair under `<private>/mcp-tls/`, generated when missing
/// or when the names it must answer to changed (a different bind address).
pub fn ensure_self_signed(
    paths: &ConfigPaths,
    bind_ip: IpAddr,
) -> Result<(PathBuf, PathBuf), String> {
    let names = self_signed_names(bind_ip);
    let dir = mcp_secrets::tls_dir(paths);
    let (cert, key) = (
        mcp_secrets::self_signed_cert(paths),
        mcp_secrets::self_signed_key(paths),
    );
    let names_file = dir.join("names.txt");
    let wanted = names.join("\n");
    let current = std::fs::read_to_string(&names_file).ok();
    if cert.is_file() && key.is_file() && current.as_deref() == Some(wanted.as_str()) {
        return Ok((cert, key));
    }
    let io = |e: std::io::Error| format!("cannot write the self-signed pair: {e}");
    secure_fs::create_private_dir_all(&dir).map_err(io)?;
    let (cert_pem, key_pem) = generate_self_signed(&names)?;
    secure_fs::write_private(&key, key_pem.as_bytes()).map_err(io)?;
    secure_fs::write_private(&cert, cert_pem.as_bytes()).map_err(io)?;
    secure_fs::write_private(&names_file, wanted.as_bytes()).map_err(io)?;
    Ok((cert, key))
}

/// A fresh self-signed certificate and key (PEM) for `names`.
pub fn generate_self_signed(names: &[String]) -> Result<(String, String), String> {
    let mut params = rcgen::CertificateParams::new(names.to_vec())
        .map_err(|e| format!("cannot build the certificate: {e}"))?;
    params.not_before = rcgen::date_time_ymd(2025, 1, 1);
    params.not_after = rcgen::date_time_ymd(2125, 1, 1);
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, "GitPoulpi MCP");
    let key = rcgen::KeyPair::generate().map_err(|e| format!("cannot generate a key: {e}"))?;
    let cert = params
        .self_signed(&key)
        .map_err(|e| format!("cannot sign the certificate: {e}"))?;
    Ok((cert.pem(), key.serialize_pem()))
}

/// The rustls server configuration for a certificate chain and key (PEM
/// files) and the certificate's fingerprint.
pub fn load_server_config(
    cert: &Path,
    key: &Path,
) -> Result<(Arc<rustls::ServerConfig>, Option<String>), String> {
    let chain: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(cert)
        .map_err(|e| format!("cannot read the certificate {}: {e}", cert.display()))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("bad certificate {}: {e}", cert.display()))?;
    if chain.is_empty() {
        return Err(format!("no certificate in {}", cert.display()));
    }
    let key_der = PrivateKeyDer::from_pem_file(key)
        .map_err(|e| format!("cannot read the key {}: {e}", key.display()))?;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("TLS setup failed: {e}"))?
        .with_no_client_auth()
        .with_single_cert(chain, key_der)
        .map_err(|e| format!("the certificate and key do not fit together: {e}"))?;
    Ok((Arc::new(config), mcp_secrets::fingerprint_of_file(cert)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_service::Service;

    /// Answers 200 to everything.
    #[derive(Clone)]
    struct Ok200;
    impl Service<Request<()>> for Ok200 {
        type Response = BoxResponse;
        type Error = Infallible;
        type Future = std::future::Ready<Result<BoxResponse, Infallible>>;
        fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, _: Request<()>) -> Self::Future {
            std::future::ready(Ok(Response::new(Full::new(Bytes::new()).boxed())))
        }
    }

    async fn status(gate: &mut BearerGate<Ok200>, authorization: Option<&str>) -> StatusCode {
        let mut request = Request::builder().uri("/mcp");
        if let Some(value) = authorization {
            request = request.header(header::AUTHORIZATION, value);
        }
        let response = gate.call(request.body(()).unwrap()).await.unwrap();
        if response.status() == StatusCode::UNAUTHORIZED {
            assert_eq!(response.headers()[header::WWW_AUTHENTICATE], "Bearer");
        }
        response.status()
    }

    #[tokio::test]
    async fn the_layer_refuses_a_missing_or_wrong_token_and_passes_the_right_one() {
        let mut gate = BearerGate::new(Ok200, Some(TokenSource::Fixed("s3cret".into())));
        assert_eq!(status(&mut gate, None).await, StatusCode::UNAUTHORIZED);
        assert_eq!(
            status(&mut gate, Some("Bearer wrong")).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(&mut gate, Some("Basic s3cret")).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(&mut gate, Some("s3cret")).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            status(&mut gate, Some("Bearer s3cret")).await,
            StatusCode::OK
        );
        assert_eq!(
            status(&mut gate, Some("bearer s3cret")).await,
            StatusCode::OK
        );
    }

    #[tokio::test]
    async fn without_a_token_source_the_layer_lets_everything_through() {
        let mut gate = BearerGate::new(Ok200, None);
        assert_eq!(status(&mut gate, None).await, StatusCode::OK);
    }

    #[tokio::test]
    async fn an_unreadable_token_file_refuses_everything_and_a_rotation_applies_at_once() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("token");
        let mut gate = BearerGate::new(Ok200, Some(TokenSource::File(file.clone())));
        assert_eq!(
            status(&mut gate, Some("Bearer one")).await,
            StatusCode::UNAUTHORIZED
        );
        std::fs::write(&file, "one\n").unwrap();
        assert_eq!(status(&mut gate, Some("Bearer one")).await, StatusCode::OK);
        std::fs::write(&file, "two\n").unwrap();
        assert_eq!(
            status(&mut gate, Some("Bearer one")).await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(status(&mut gate, Some("Bearer two")).await, StatusCode::OK);
    }

    /// Every combination of the four facts: only a token plus encryption
    /// (TLS or a vouched proxy) passes, and the refusal names each gap.
    #[test]
    fn allow_remote_needs_auth_a_token_and_encryption() {
        for http_auth in [false, true] {
            for token in [false, true] {
                for tls in [false, true] {
                    for proxy in [false, true] {
                        let result = remote_decision(http_auth, token, tls, proxy);
                        let ok = http_auth && token && (tls || proxy);
                        assert_eq!(result.is_ok(), ok, "{http_auth} {token} {tls} {proxy}");
                        if let Err(text) = result {
                            assert_eq!(text.contains("mcp.http_auth"), !http_auth);
                            assert_eq!(text.contains("no token"), http_auth && !token);
                            assert_eq!(text.contains("unencrypted"), !tls && !proxy);
                        }
                    }
                }
            }
        }
    }

    fn addr(text: &str) -> SocketAddr {
        text.parse().unwrap()
    }

    #[test]
    fn prepare_applies_the_rule_only_to_a_non_loopback_address() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        let cfg = McpConfig::default();
        // Loopback, defaults: a token is created, nothing else needed.
        let security = prepare(&cfg, Some(&paths), addr("127.0.0.1:1"), false).unwrap();
        assert!(security.token.is_some() && security.tls.is_none());
        assert!(mcp_secrets::read_token(&paths).is_some());
        // Remote without the flag, then with it but unencrypted.
        let err = prepare(&cfg, Some(&paths), addr("0.0.0.0:1"), false)
            .err()
            .unwrap();
        assert!(err.contains("--allow-remote"), "{err}");
        let err = prepare(&cfg, Some(&paths), addr("0.0.0.0:1"), true)
            .err()
            .unwrap();
        assert!(err.contains("unencrypted"), "{err}");
        // Vouching for a proxy lifts only the encryption gap.
        let proxied = McpConfig {
            behind_proxy: true,
            ..McpConfig::default()
        };
        assert!(prepare(&proxied, Some(&paths), addr("0.0.0.0:1"), true).is_ok());
        let no_auth = McpConfig {
            http_auth: false,
            behind_proxy: true,
            ..McpConfig::default()
        };
        let err = prepare(&no_auth, Some(&paths), addr("0.0.0.0:1"), true)
            .err()
            .unwrap();
        assert!(err.contains("mcp.http_auth"), "{err}");
        // Loopback without auth is allowed (the binary warns).
        assert!(prepare(&no_auth, Some(&paths), addr("127.0.0.1:1"), false)
            .unwrap()
            .token
            .is_none());
        // Self-signed TLS satisfies the encryption gap.
        let tls = McpConfig {
            tls: McpTls::SelfSigned,
            ..McpConfig::default()
        };
        let secured = prepare(&tls, Some(&paths), addr("0.0.0.0:1"), true).unwrap();
        assert!(secured.tls.is_some() && secured.fingerprint.is_some());
    }

    #[test]
    fn files_mode_needs_both_paths() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        let cfg = McpConfig {
            tls: McpTls::Files,
            ..McpConfig::default()
        };
        let err = prepare(&cfg, Some(&paths), addr("127.0.0.1:1"), false)
            .err()
            .unwrap();
        assert!(err.contains("tls_cert"), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn the_self_signed_pair_is_written_user_only_and_regenerated_for_new_names() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        let (cert, key) = ensure_self_signed(&paths, ip).unwrap();
        for file in [&cert, &key] {
            let mode = std::fs::metadata(file).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "{}", file.display());
        }
        let first = mcp_secrets::fingerprint_of_file(&cert).unwrap();
        // Same names: kept. A new bind address: regenerated.
        ensure_self_signed(&paths, ip).unwrap();
        assert_eq!(mcp_secrets::fingerprint_of_file(&cert).unwrap(), first);
        ensure_self_signed(&paths, "192.0.2.7".parse().unwrap()).unwrap();
        assert_ne!(mcp_secrets::fingerprint_of_file(&cert).unwrap(), first);
    }

    /// A TLS client over an in-memory pipe reaches the server through the
    /// acceptor; a plain-text client fails the handshake.
    #[tokio::test]
    async fn the_acceptor_serves_a_tls_client_and_refuses_plain_text() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        let (cert, key) = ensure_self_signed(&paths, "127.0.0.1".parse().unwrap()).unwrap();
        let (server_config, _) = load_server_config(&cert, &key).unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(server_config);

        // A client that trusts exactly that certificate.
        let mut roots = rustls::RootCertStore::empty();
        for der in CertificateDer::pem_file_iter(&cert).unwrap() {
            roots.add(der.unwrap()).unwrap();
        }
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let client_config = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let server = tokio::spawn({
            let acceptor = acceptor.clone();
            async move {
                let mut stream = acceptor.accept(server_io).await.unwrap();
                let mut buf = [0u8; 4];
                stream.read_exact(&mut buf).await.unwrap();
                assert_eq!(&buf, b"ping");
                stream.write_all(b"pong").await.unwrap();
                stream.shutdown().await.unwrap();
            }
        });
        let name = rustls_pki_types::ServerName::try_from("localhost").unwrap();
        let mut stream = connector.connect(name, client_io).await.unwrap();
        stream.write_all(b"ping").await.unwrap();
        let mut reply = Vec::new();
        let _ = stream.read_to_end(&mut reply).await;
        assert_eq!(reply, b"pong");
        server.await.unwrap();

        let (mut plain_client, plain_server) = tokio::io::duplex(64 * 1024);
        let refused = tokio::spawn(async move { acceptor.accept(plain_server).await.is_err() });
        plain_client
            .write_all(b"GET /mcp HTTP/1.1\r\nHost: x\r\n\r\n")
            .await
            .unwrap();
        drop(plain_client);
        assert!(refused.await.unwrap(), "plain text must fail the handshake");
    }
}
