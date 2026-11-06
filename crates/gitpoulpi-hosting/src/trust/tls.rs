//! The certificate verifier of every hosting and AI connection.
//!
//! [`GuardedVerifier`] wraps rustls' own `WebPkiServerVerifier` (webpki roots
//! plus the system roots: the same set as before) and adds one thing: an
//! optional key pin per host.
//!
//! - no pin for the host: webpki decides, exactly as it always did;
//! - [`TlsPolicy::Tofu`]: webpki decides, **except** that a chain failing only
//!   on issuer, self-signature or name is accepted when the leaf's key is the
//!   recorded one (a private CA, a self-signed server);
//! - [`TlsPolicy::Strict`]: the leaf's key must be the recorded one *and* the
//!   chain must validate or fail only in that way.
//!
//! The handshake signatures (`verify_tls12_signature`, `verify_tls13_signature`
//! and the supported schemes) are **always** webpki's: that signature is what
//! proves the server holds the key the pin names. Skipping it would turn a
//! pin into "anyone who has seen the certificate".

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::WebPkiServerVerifier;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{
    CertificateError, DigitallySignedStruct, Error as TlsError, RootCertStore, SignatureScheme,
};

use super::store::CertInfo;
use super::{TrustProblem, TrustSubject};

/// What a host's certificate is held to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsPolicy {
    Tofu { spki: String },
    Strict { spki: String },
}

impl TlsPolicy {
    pub fn spki(&self) -> &str {
        match self {
            Self::Tofu { spki } | Self::Strict { spki } => spki,
        }
    }
}

/// Why the verifier refused a host's certificate (read back by the
/// transport to build a [`TrustProblem`]).
#[derive(Debug, Clone)]
pub struct Rejection {
    pub new: CertInfo,
    /// The pinned key, when the host had a policy.
    pub pinned: Option<String>,
}

fn lock<T>(m: &'static Mutex<T>) -> std::sync::MutexGuard<'static, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

static POLICIES: OnceLock<Mutex<HashMap<String, TlsPolicy>>> = OnceLock::new();
static OBSERVED: OnceLock<Mutex<HashMap<String, CertInfo>>> = OnceLock::new();
static REJECTED: OnceLock<Mutex<HashMap<String, Rejection>>> = OnceLock::new();

fn policies() -> &'static Mutex<HashMap<String, TlsPolicy>> {
    POLICIES.get_or_init(Default::default)
}
fn observed_map() -> &'static Mutex<HashMap<String, CertInfo>> {
    OBSERVED.get_or_init(Default::default)
}
fn rejected_map() -> &'static Mutex<HashMap<String, Rejection>> {
    REJECTED.get_or_init(Default::default)
}

/// Sets (or, with `None`, clears) the policy of `host`. `TrustGate` calls it
/// whenever it builds a client from a record.
pub fn set_policy(host: &str, policy: Option<TlsPolicy>) {
    let host = host.to_ascii_lowercase();
    let mut map = lock(policies());
    match policy {
        Some(p) => map.insert(host, p),
        None => map.remove(&host),
    };
}

pub fn policy_of(host: &str) -> Option<TlsPolicy> {
    lock(policies()).get(&host.to_ascii_lowercase()).cloned()
}

/// The certificate the last handshake with `host` presented.
pub fn observed(host: &str) -> Option<CertInfo> {
    lock(observed_map())
        .get(&host.to_ascii_lowercase())
        .cloned()
}

/// Takes (and forgets) the verifier's refusal of `host`, if it just refused.
pub fn take_rejection(host: &str) -> Option<Rejection> {
    lock(rejected_map()).remove(&host.to_ascii_lowercase())
}

/// The problem a refused handshake with `host` is, subject still unbound.
pub fn problem_for(host: &str) -> Option<TrustProblem> {
    let rejection = take_rejection(host)?;
    let subject = TrustSubject::Host(host.to_ascii_lowercase());
    Some(match rejection.pinned {
        Some(spki) => TrustProblem::CertificateChanged {
            subject,
            old: CertInfo {
                spki_sha256: spki,
                ..CertInfo::default()
            },
            new: rejection.new,
        },
        None => TrustProblem::UntrustedCertificate {
            subject,
            new: rejection.new,
        },
    })
}

/// Whether a webpki failure is one a pinned key may stand in for: the chain
/// does not lead to a known root (or is signed by a key that is not one), or
/// the name is not the one on the certificate. Expiry, revocation, a wrong
/// purpose and the like are never excused by a pin.
pub fn pin_can_excuse(error: &TlsError) -> bool {
    matches!(
        error,
        TlsError::InvalidCertificate(
            CertificateError::UnknownIssuer
                | CertificateError::BadSignature
                | CertificateError::NotValidForName
                | CertificateError::NotValidForNameContext { .. }
        )
    )
}

/// The decision, separated from rustls so it can be tested as a table.
#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Accept,
    /// Refuse with the webpki error, or (`pin_mismatch`) because the key is
    /// not the pinned one.
    Reject {
        pin_mismatch: bool,
    },
}

pub fn decide(
    webpki_ok: bool,
    excusable: bool,
    policy: Option<&TlsPolicy>,
    spki: &str,
) -> Decision {
    match policy {
        None => {
            if webpki_ok {
                Decision::Accept
            } else {
                Decision::Reject {
                    pin_mismatch: false,
                }
            }
        }
        Some(TlsPolicy::Tofu { spki: pinned }) => {
            if webpki_ok || (excusable && pinned == spki) {
                Decision::Accept
            } else {
                Decision::Reject {
                    pin_mismatch: excusable && pinned != spki,
                }
            }
        }
        Some(TlsPolicy::Strict { spki: pinned }) => {
            if pinned != spki {
                Decision::Reject { pin_mismatch: true }
            } else if webpki_ok || excusable {
                Decision::Accept
            } else {
                Decision::Reject {
                    pin_mismatch: false,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GuardedVerifier {
    inner: Arc<WebPkiServerVerifier>,
}

impl GuardedVerifier {
    /// A verifier over `roots`.
    pub fn new(roots: RootCertStore) -> Result<Self, String> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let inner = WebPkiServerVerifier::builder_with_provider(Arc::new(roots), provider)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self { inner })
    }

    /// The process-wide verifier over the webpki roots plus the system's.
    pub fn system() -> Result<Arc<Self>, String> {
        static SHARED: OnceLock<Result<Arc<GuardedVerifier>, String>> = OnceLock::new();
        SHARED
            .get_or_init(|| {
                let mut roots = RootCertStore::empty();
                roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                for cert in rustls_native_certs::load_native_certs().certs {
                    let _ = roots.add(cert);
                }
                GuardedVerifier::new(roots).map(Arc::new)
            })
            .clone()
    }
}

fn host_of(name: &ServerName<'_>) -> String {
    name.to_str().to_ascii_lowercase()
}

impl ServerCertVerifier for GuardedVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        let host = host_of(server_name);
        let info = CertInfo::from_der(end_entity.as_ref()).unwrap_or_default();
        lock(observed_map()).insert(host.clone(), info.clone());
        let webpki = self.inner.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        );
        let policy = policy_of(&host);
        let decision = decide(
            webpki.is_ok(),
            webpki.as_ref().err().is_some_and(pin_can_excuse),
            policy.as_ref(),
            &info.spki_sha256,
        );
        match decision {
            Decision::Accept => {
                lock(rejected_map()).remove(&host);
                Ok(webpki.unwrap_or_else(|_| ServerCertVerified::assertion()))
            }
            Decision::Reject { .. } => {
                lock(rejected_map()).insert(
                    host,
                    Rejection {
                        new: info,
                        pinned: policy.map(|p| p.spki().to_string()),
                    },
                );
                // The pin refused a chain webpki accepted: say so with the
                // closest standard error.
                Err(webpki.err().unwrap_or(TlsError::InvalidCertificate(
                    CertificateError::ApplicationVerificationFailure,
                )))
            }
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

/// The `rustls` client configuration every real transport uses.
pub fn client_config() -> Result<rustls::ClientConfig, String> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = GuardedVerifier::system()?;
    Ok(rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| e.to_string())?
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth())
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::*;

    fn der(hex: &str) -> CertificateDer<'static> {
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        CertificateDer::from(bytes)
    }

    /// 2027-01-01: after every fixture's `notBefore`, long before its end.
    fn now() -> UnixTime {
        UnixTime::since_unix_epoch(std::time::Duration::from_secs(1_798_761_600))
    }

    fn verifier() -> GuardedVerifier {
        let mut roots = RootCertStore::empty();
        roots.add(der(CA)).unwrap();
        GuardedVerifier::new(roots).unwrap()
    }

    fn check(v: &GuardedVerifier, leaf: &str, host: &'static str) -> Result<(), TlsError> {
        v.verify_server_cert(
            &der(leaf),
            &[],
            &ServerName::try_from(host).unwrap(),
            &[],
            now(),
        )
        .map(|_| ())
    }

    #[test]
    fn cert_fields_are_read_and_the_key_fingerprint_matches_openssl() {
        let info = CertInfo::from_der(der(SELF).as_ref()).unwrap();
        assert_eq!(info.spki_sha256, SELF_SPKI);
        assert_eq!(info.subject, "CN=self.test, O=Fixture Self");
        assert_eq!(info.issuer, info.subject);
        assert!(
            info.not_before.starts_with("2026-11-"),
            "{}",
            info.not_before
        );
        assert!(info.not_after.starts_with("2126-"), "{}", info.not_after);
        assert!(CertInfo::from_der(&[1, 2, 3]).is_none());
    }

    #[test]
    fn a_chain_to_a_known_root_passes_with_no_policy() {
        let v = verifier();
        assert!(check(&v, GOOD, "good.test").is_ok());
        // The name is still checked: webpki says no.
        assert!(check(&v, GOOD, "other.test").is_err());
    }

    #[test]
    fn self_signed_is_refused_until_its_key_is_pinned_and_then_only_for_that_key() {
        let v = verifier();
        let spki = CertInfo::from_der(der(SELF).as_ref()).unwrap().spki_sha256;
        // First use: refused, and the refusal carries what was presented.
        assert!(check(&v, SELF, "self.test").is_err());
        let problem = problem_for("self.test").expect("a refusal was recorded");
        match problem {
            TrustProblem::UntrustedCertificate { new, .. } => assert_eq!(new.spki_sha256, spki),
            other => panic!("{other:?}"),
        }
        // Pinned (TOFU): accepted.
        set_policy("self.test", Some(TlsPolicy::Tofu { spki: spki.clone() }));
        assert!(check(&v, SELF, "self.test").is_ok());
        // A different key under the same name: refused, reported as a change.
        set_policy(
            "self.test",
            Some(TlsPolicy::Tofu {
                spki: "00".repeat(32),
            }),
        );
        assert!(check(&v, SELF, "self.test").is_err());
        assert!(matches!(
            problem_for("self.test"),
            Some(TrustProblem::CertificateChanged { .. })
        ));
        set_policy("self.test", None);
    }

    #[test]
    fn strict_requires_the_pinned_key_even_when_the_chain_validates() {
        let v = verifier();
        let good = CertInfo::from_der(der(GOOD).as_ref()).unwrap().spki_sha256;
        set_policy("good.test", Some(TlsPolicy::Strict { spki: good }));
        assert!(check(&v, GOOD, "good.test").is_ok());
        // The CA re-issued the name on a new key: valid chain, wrong pin.
        assert!(check(&v, ROTATED, "good.test").is_err());
        assert!(matches!(
            problem_for("good.test"),
            Some(TrustProblem::CertificateChanged { .. })
        ));
        // Under TOFU the same rotation is fine: the chain validates.
        let old = CertInfo::from_der(der(GOOD).as_ref()).unwrap().spki_sha256;
        set_policy("good.test", Some(TlsPolicy::Tofu { spki: old }));
        assert!(check(&v, ROTATED, "good.test").is_ok());
        set_policy("good.test", None);
    }

    #[test]
    fn the_decision_table() {
        let tofu = TlsPolicy::Tofu { spki: "aa".into() };
        let strict = TlsPolicy::Strict { spki: "aa".into() };
        use Decision::*;
        assert_eq!(decide(true, false, None, "aa"), Accept);
        assert_eq!(
            decide(false, true, None, "aa"),
            Reject {
                pin_mismatch: false
            }
        );
        assert_eq!(decide(false, true, Some(&tofu), "aa"), Accept);
        assert_eq!(
            decide(false, true, Some(&tofu), "bb"),
            Reject { pin_mismatch: true }
        );
        // Expiry and the like are never excused by a pin.
        assert_eq!(
            decide(false, false, Some(&tofu), "aa"),
            Reject {
                pin_mismatch: false
            }
        );
        assert_eq!(
            decide(true, false, Some(&strict), "bb"),
            Reject { pin_mismatch: true }
        );
        assert_eq!(decide(false, true, Some(&strict), "aa"), Accept);
        assert_eq!(
            decide(false, false, Some(&strict), "aa"),
            Reject {
                pin_mismatch: false
            }
        );
    }

    /// Runs a TLS handshake between a client using `verifier` and a server
    /// presenting `cert_hex` but signing with `key_hex`, entirely in memory.
    fn handshake(
        verifier: GuardedVerifier,
        host: &'static str,
        cert_hex: &str,
        key_hex: &str,
    ) -> Result<(), String> {
        use rustls::pki_types::PrivatePkcs8KeyDer;
        use rustls::sign::CertifiedKey;
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let key_bytes: Vec<u8> = (0..key_hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&key_hex[i..i + 2], 16).unwrap())
            .collect();
        let signing = provider
            .key_provider
            .load_private_key(PrivatePkcs8KeyDer::from(key_bytes).into())
            .unwrap();
        // `CertifiedKey::new` does not check that the key belongs to the
        // certificate: a server that holds the wrong key is exactly the case.
        let certified = Arc::new(CertifiedKey::new(vec![der(cert_hex)], signing));
        #[derive(Debug)]
        struct One(Arc<CertifiedKey>);
        impl rustls::server::ResolvesServerCert for One {
            fn resolve(&self, _: rustls::server::ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
                Some(self.0.clone())
            }
        }
        let server_cfg = rustls::ServerConfig::builder_with_provider(provider.clone())
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(One(certified)));
        let client_cfg = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(verifier))
            .with_no_client_auth();
        let mut client = rustls::ClientConnection::new(
            Arc::new(client_cfg),
            ServerName::try_from(host).unwrap(),
        )
        .unwrap();
        let mut server = rustls::ServerConnection::new(Arc::new(server_cfg)).unwrap();
        for _ in 0..20 {
            let mut buf = Vec::new();
            while client.wants_write() {
                client.write_tls(&mut buf).unwrap();
            }
            if !buf.is_empty() {
                server.read_tls(&mut buf.as_slice()).unwrap();
                server
                    .process_new_packets()
                    .map_err(|e| format!("server: {e}"))?;
            }
            let mut buf = Vec::new();
            while server.wants_write() {
                server.write_tls(&mut buf).unwrap();
            }
            if !buf.is_empty() {
                client.read_tls(&mut buf.as_slice()).unwrap();
                client
                    .process_new_packets()
                    .map_err(|e| format!("client: {e}"))?;
            }
            if !client.is_handshaking() && !server.is_handshaking() {
                return Ok(());
            }
        }
        Err("handshake did not finish".into())
    }

    #[test]
    fn the_handshake_signature_is_still_checked_so_a_pin_is_not_a_bearer_token() {
        // The pin matches the certificate presented, and the certificate
        // passes `verify_server_cert` -- but the server cannot sign with the
        // key it names. Only the delegated signature check catches that.
        let spki = CertInfo::from_der(der(SELF).as_ref()).unwrap().spki_sha256;
        set_policy("sig.test", Some(TlsPolicy::Tofu { spki }));
        // Same certificate, key that is not its own: fails.
        let forged = handshake(verifier(), "sig.test", SELF, GOOD_KEY);
        // The owner of the key: succeeds.
        let honest = handshake(verifier(), "sig.test", SELF, SELF_KEY);
        set_policy("sig.test", None);
        let err = forged.expect_err("a certificate replayed without its key must fail");
        assert!(err.contains("client"), "{err}");
        honest.expect("the real key completes the handshake");
    }
}
