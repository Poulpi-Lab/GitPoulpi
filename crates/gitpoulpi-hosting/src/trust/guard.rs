//! [`GuardedTransport`]: the per-request half of the gate.
//!
//! [`TrustGate::admit`](super::TrustGate::admit) decides whether a client may
//! exist; this wraps its transport so that a bug (a provider building a URL on
//! a second host, a value read from a server response used as a URL) cannot
//! send the token anywhere else either. A request whose origin is not one of
//! the integration's is refused **before** it is sent.

use std::sync::Arc;

use crate::http::{HttpRequest, HttpResponse, HttpTransport, Progress};
use crate::{Error, Result};

use super::store::TlsRecord;
use super::{origin, report, TrustProblem, TrustSubject};

pub struct GuardedTransport {
    inner: Arc<dyn HttpTransport>,
    subject: TrustSubject,
    /// The recorded origin, for the message when a request strays.
    recorded: String,
    /// Origins (`scheme://host[:port]`) a request may reach.
    allowed: Vec<String>,
    /// The pinned certificate, to say what changed.
    tls: Option<TlsRecord>,
}

impl GuardedTransport {
    pub fn new(
        inner: Arc<dyn HttpTransport>,
        subject: TrustSubject,
        recorded: String,
        allowed: Vec<String>,
        tls: Option<TlsRecord>,
    ) -> Self {
        Self {
            inner,
            subject,
            recorded,
            allowed,
            tls,
        }
    }

    fn check(&self, url: &str) -> Result<()> {
        match origin::origin_of(url) {
            Some(o) if self.allowed.iter().any(|a| *a == o) => Ok(()),
            other => {
                let problem = TrustProblem::OriginChanged {
                    subject: self.subject.clone(),
                    old: self.recorded.clone(),
                    new: other.unwrap_or_else(|| url.to_string()),
                };
                report(&problem);
                Err(Error::Trust(problem))
            }
        }
    }

    /// Binds the subject of a certificate problem the transport raised from
    /// the host alone, fills in what was pinned, and queues it.
    fn bind(&self, error: Error) -> Error {
        match error {
            Error::Trust(problem) if matches!(problem.subject(), TrustSubject::Host(_)) => {
                let problem = match problem {
                    TrustProblem::CertificateChanged { new, old, subject } => {
                        let old = match &self.tls {
                            Some(t) if t.cert.spki_sha256 == old.spki_sha256 => t.cert.clone(),
                            _ => old,
                        };
                        TrustProblem::CertificateChanged { subject, old, new }
                    }
                    other => other,
                }
                .with_subject(self.subject.clone());
                report(&problem);
                Error::Trust(problem)
            }
            other => other,
        }
    }
}

#[async_trait::async_trait]
impl HttpTransport for GuardedTransport {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        self.check(&request.url)?;
        self.inner.send(request).await.map_err(|e| self.bind(e))
    }

    async fn download(&self, request: HttpRequest, progress: &Progress<'_>) -> Result<Vec<u8>> {
        self.check(&request.url)?;
        self.inner
            .download(request, progress)
            .await
            .map_err(|e| self.bind(e))
    }
}
