//! Integration trust: what a token (or an AI key) may be sent to.
//!
//! A saved token is only as safe as the address it is sent to. The trust
//! store (`<private>/trust.toml`, [`store`]) remembers, per integration, the
//! normalized origin it was connected to, the account the token belongs to and
//! (for a self-hosted server whose certificate does not chain to a public
//! root, or when asked) the certificate key. [`TrustGate`] checks every
//! client against it *before* the client exists, and [`GuardedTransport`]
//! checks every request against the recorded origin. A change is a
//! [`TrustProblem`]: nothing is sent, and only a person (the CLI's
//! `integrations trust`, the TUI's dialog) can accept it.
//!
//! Read `docs/architecture/session-and-state.md` "Integration trust" for the
//! traps (one shared client with pooling off, signature delegation, why the
//! store is not in `config.toml`).

pub mod cert;
mod forget;
mod guard;
pub mod origin;
pub mod store;
#[cfg(feature = "network")]
pub mod tls;

#[cfg(test)]
mod fixtures;
#[cfg(feature = "hosting")]
mod verify;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use gitpoulpi_config::{ConfigPaths, Integration};

pub use self::forget::{forget, ForgetReport};
pub use self::guard::GuardedTransport;
pub use self::origin::{api_origins, integration_origin, is_public_saas_host, normalize};
pub use self::store::{
    AiRecord, CertInfo, IntegrationRecord, TlsMode, TlsRecord, TrustStore, TRUST_FILE,
};
#[cfg(feature = "hosting")]
pub use self::verify::VerifyReason;

use crate::{Error, Result};

/// Whom a trust problem is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustSubject {
    /// A configured integration, by id.
    Integration(String),
    /// An API-key AI profile, by id.
    Ai(String),
    /// Only the host is known (raised by the transport; the guard binds the
    /// real subject).
    Host(String),
}

impl TrustSubject {
    /// The id or host.
    pub fn name(&self) -> &str {
        match self {
            Self::Integration(s) | Self::Ai(s) | Self::Host(s) => s,
        }
    }
}

/// A login and the host's own id for the account.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Account {
    pub login: String,
    pub id: String,
}

/// Why a client was not built, or a request not sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustProblem {
    /// The address now differs from the recorded one.
    OriginChanged {
        subject: TrustSubject,
        old: String,
        new: String,
    },
    /// The token now belongs to another account.
    AccountChanged {
        subject: TrustSubject,
        old: Account,
        new: Account,
    },
    /// First use of a certificate that does not chain to a known root.
    UntrustedCertificate {
        subject: TrustSubject,
        new: CertInfo,
    },
    /// The certificate key is not the recorded one.
    CertificateChanged {
        subject: TrustSubject,
        old: CertInfo,
        new: CertInfo,
    },
    /// A non-loopback `http://` address that was not allowed.
    PlainHttp {
        subject: TrustSubject,
        origin: String,
    },
}

impl TrustProblem {
    pub fn subject(&self) -> &TrustSubject {
        match self {
            Self::OriginChanged { subject, .. }
            | Self::AccountChanged { subject, .. }
            | Self::UntrustedCertificate { subject, .. }
            | Self::CertificateChanged { subject, .. }
            | Self::PlainHttp { subject, .. } => subject,
        }
    }

    /// The same problem about `subject`.
    #[must_use]
    pub fn with_subject(mut self, new_subject: TrustSubject) -> Self {
        match &mut self {
            Self::OriginChanged { subject, .. }
            | Self::AccountChanged { subject, .. }
            | Self::UntrustedCertificate { subject, .. }
            | Self::CertificateChanged { subject, .. }
            | Self::PlainHttp { subject, .. } => *subject = new_subject,
        }
        self
    }

    /// The stable spelling of the kind of change (CLI JSON `change`).
    pub fn change(&self) -> &'static str {
        match self {
            Self::OriginChanged { .. } => "origin",
            Self::AccountChanged { .. } => "account",
            Self::UntrustedCertificate { .. } => "untrusted_certificate",
            Self::CertificateChanged { .. } => "certificate",
            Self::PlainHttp { .. } => "plain_http",
        }
    }

    /// What was trusted (empty when nothing was).
    pub fn old_text(&self) -> String {
        match self {
            Self::OriginChanged { old, .. } => old.clone(),
            Self::AccountChanged { old, .. } => format!("{} ({})", old.login, old.id),
            Self::CertificateChanged { old, .. } => old.spki_sha256.clone(),
            Self::UntrustedCertificate { .. } | Self::PlainHttp { .. } => String::new(),
        }
    }

    /// What is there now: the value to accept.
    pub fn new_text(&self) -> String {
        match self {
            Self::OriginChanged { new, .. } => new.clone(),
            Self::AccountChanged { new, .. } => new.id.clone(),
            Self::UntrustedCertificate { new, .. } | Self::CertificateChanged { new, .. } => {
                new.spki_sha256.clone()
            }
            Self::PlainHttp { origin, .. } => origin.clone(),
        }
    }

    /// The commands that resolve it, for a script or an agent to show a
    /// person.
    pub fn fix(&self) -> Vec<String> {
        let (cmd, name) = match self.subject() {
            TrustSubject::Ai(p) => ("gitpoulpi ai trust".to_string(), p.clone()),
            other => (
                "gitpoulpi hosting integrations trust".to_string(),
                other.name().to_string(),
            ),
        };
        let accept = match self {
            Self::OriginChanged { new, .. } => format!("--accept-origin {new}"),
            Self::AccountChanged { new, .. } => format!("--accept-account {}", new.id),
            Self::UntrustedCertificate { new, .. } | Self::CertificateChanged { new, .. } => {
                format!("--accept-certificate {}", new.spki_sha256)
            }
            Self::PlainHttp { .. } => "--allow-insecure-http on".to_string(),
        };
        vec![format!("{cmd} {name} {accept}")]
    }
}

impl std::fmt::Display for TrustProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let who = self.subject().name();
        match self {
            Self::OriginChanged { old, new, .. } => write!(
                f,
                "`{who}` now points to {new}, it was {old}: nothing was sent; \
                 review it with `gitpoulpi hosting integrations trust`"
            ),
            Self::AccountChanged { old, new, .. } => write!(
                f,
                "the token of `{who}` now belongs to {} (id {}), it was {} (id {})",
                new.login, new.id, old.login, old.id
            ),
            Self::UntrustedCertificate { new, .. } => write!(
                f,
                "`{who}` presents a certificate that is not signed by a known authority \
                 (key {}, {})",
                new.spki_sha256, new.subject
            ),
            Self::CertificateChanged { old, new, .. } => write!(
                f,
                "the certificate key of `{who}` changed: it was {}, it is {}",
                old.spki_sha256, new.spki_sha256
            ),
            Self::PlainHttp { origin, .. } => write!(
                f,
                "`{who}` uses {origin}, which is not encrypted: allow it explicitly to continue"
            ),
        }
    }
}

// --- the pending queue --------------------------------------------------------

static PENDING: Mutex<Vec<TrustProblem>> = Mutex::new(Vec::new());

fn pending() -> std::sync::MutexGuard<'static, Vec<TrustProblem>> {
    PENDING.lock().unwrap_or_else(|p| p.into_inner())
}

/// Queues `problem` for whoever asks a person (the TUI's dialog, the CLI's
/// JSON answer). One entry per subject and kind: a refusal repeated by ten
/// requests is one question.
pub fn report(problem: &TrustProblem) {
    let mut queue = pending();
    let same =
        |p: &TrustProblem| p.subject() == problem.subject() && p.change() == problem.change();
    queue.retain(|p| !same(p));
    queue.push(problem.clone());
}

/// Takes every queued problem, oldest first.
pub fn take_pending() -> Vec<TrustProblem> {
    std::mem::take(&mut *pending())
}

/// The queued problems, left in place.
pub fn peek_pending() -> Vec<TrustProblem> {
    pending().clone()
}

/// Drops the queued problems about `subject`.
pub fn clear_pending(subject: &TrustSubject) {
    pending().retain(|p| p.subject() != subject);
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

// --- the gate -----------------------------------------------------------------

/// A client's admission ticket: the record it was admitted under and the
/// origins its requests may reach.
#[derive(Debug, Clone)]
pub struct Admitted {
    pub subject: TrustSubject,
    pub record: IntegrationRecord,
    pub origins: Vec<String>,
}

/// Checks integrations and AI profiles against the trust store.
///
/// File-backed gates re-read `trust.toml` on every use, so a `trust` run in
/// another process (or the TUI's dialog) takes effect at once.
pub struct TrustGate {
    file: Option<PathBuf>,
    mem: Mutex<TrustStore>,
    /// Records of integrations that were never connected: held for this
    /// process only, written by [`TrustGate::record_after_check`].
    provisional: Mutex<HashMap<String, IntegrationRecord>>,
    provisional_ai: Mutex<HashMap<String, AiRecord>>,
    lenient: bool,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

impl TrustGate {
    fn new(file: Option<PathBuf>, lenient: bool) -> Self {
        Self {
            file,
            mem: Mutex::new(TrustStore::default()),
            provisional: Mutex::default(),
            provisional_ai: Mutex::default(),
            lenient,
        }
    }

    /// The gate over `<private>/trust.toml` of `paths`.
    pub fn load(paths: &ConfigPaths) -> Self {
        Self::new(Some(TrustStore::file_in(paths)), false)
    }

    /// [`Self::load`] over the real config directory; with none to be found
    /// the gate keeps its records in memory only.
    pub fn ambient() -> Self {
        match ConfigPaths::discover() {
            Some(paths) => Self::load(&paths),
            None => Self::new(None, false),
        }
    }

    /// A gate that remembers in memory and allows plain http: for tests of
    /// code that is not about trust. It still wraps every transport, so a
    /// request off the integration's origin fails there too.
    #[cfg(any(test, feature = "test-support"))]
    pub fn for_tests() -> Self {
        Self::new(None, true)
    }

    /// The same, over a store file in `dir`: for tests that look at the file.
    #[cfg(any(test, feature = "test-support"))]
    pub fn for_tests_in(paths: &ConfigPaths) -> Self {
        Self::new(Some(TrustStore::file_in(paths)), true)
    }

    fn store(&self) -> Result<TrustStore> {
        match &self.file {
            Some(path) => TrustStore::load(path).map_err(Error::TrustStore),
            None => Ok(lock(&self.mem).clone()),
        }
    }

    fn save(&self, store: &TrustStore) -> Result<()> {
        match &self.file {
            Some(path) => store.save(path).map_err(|e| {
                Error::TrustStore(format!("{} could not be written: {e}", path.display()))
            }),
            None => {
                *lock(&self.mem) = store.clone();
                Ok(())
            }
        }
    }

    /// The record `integration` would get from the config as it is now.
    fn fresh_record(&self, integration: &Integration, origin: String) -> IntegrationRecord {
        let user = integration.user.as_ref();
        IntegrationRecord {
            id: integration.id.clone(),
            origin,
            account_id: user.map(|u| u.id.clone()).unwrap_or_default(),
            account_login: user.map(|u| u.login.clone()).unwrap_or_default(),
            recorded_at: unix_now(),
            account_checked_at: 0,
            allow_plain_http: self.lenient,
            tls: None,
        }
    }

    fn refuse<T>(&self, problem: TrustProblem) -> Result<T> {
        report(&problem);
        Err(Error::Trust(problem))
    }

    /// The first two checks, before anything (a token included) is resolved:
    /// plain http, then origin. A record that does not exist yet is made from
    /// the config as it stands: written at once for an integration that was
    /// connected before (the migration: it keeps today's level of trust),
    /// held in memory for one that never was, until
    /// [`Self::record_after_check`] writes it.
    pub fn admit(&self, integration: &Integration) -> Result<Admitted> {
        let origin = integration_origin(integration)?;
        let subject = TrustSubject::Integration(integration.id.clone());
        let mut store = self.store()?;
        let record = match store.integration(&integration.id) {
            Some(r) => r.clone(),
            None => {
                let record = lock(&self.provisional)
                    .get(&integration.id)
                    .cloned()
                    .unwrap_or_else(|| self.fresh_record(integration, origin.clone()));
                if integration.user.is_some() {
                    store.put_integration(record.clone());
                    self.save(&store)?;
                } else {
                    lock(&self.provisional).insert(integration.id.clone(), record.clone());
                }
                record
            }
        };
        if origin::is_plain_http_remote(&origin) && !record.allow_plain_http {
            return self.refuse(TrustProblem::PlainHttp {
                subject,
                origin: origin.clone(),
            });
        }
        if record.origin != origin {
            return self.refuse(TrustProblem::OriginChanged {
                subject,
                old: record.origin.clone(),
                new: origin,
            });
        }
        let origins = api_origins(integration.kind, &record.origin);
        apply_tls_policy(&record.origin, record.tls.as_ref());
        Ok(Admitted {
            subject,
            record,
            origins,
        })
    }

    /// Writes the record of `integration` now if there is none: for a caller
    /// that holds a token for it (stored, from the environment or from the
    /// credential helper), which is the migration case.
    pub fn adopt(&self, integration: &Integration) -> Result<()> {
        if self.record_of(&integration.id).is_some() {
            return Ok(());
        }
        self.update_record(integration, |_| ())
    }

    /// [`Self::admit`], for a caller that only wants to know it passes
    /// before it resolves a token.
    pub fn preflight(&self, integration: &Integration) -> Result<()> {
        self.admit(integration).map(|_| ())
    }

    /// The record of `id`, if one was written.
    pub fn record_of(&self, id: &str) -> Option<IntegrationRecord> {
        self.store().ok()?.integration(id).cloned()
    }

    /// Changes the record of `integration` (made from the config first when
    /// there is none) and writes it. The one door every acceptance goes
    /// through.
    pub fn update_record<R>(
        &self,
        integration: &Integration,
        change: impl FnOnce(&mut IntegrationRecord) -> R,
    ) -> Result<R> {
        let origin = integration_origin(integration)?;
        let mut store = self.store()?;
        let mut record = match store.integration(&integration.id) {
            Some(r) => r.clone(),
            None => lock(&self.provisional)
                .remove(&integration.id)
                .unwrap_or_else(|| self.fresh_record(integration, origin)),
        };
        let out = change(&mut record);
        apply_tls_policy(&record.origin, record.tls.as_ref());
        store.put_integration(record);
        self.save(&store)?;
        lock(&self.provisional).remove(&integration.id);
        clear_pending(&TrustSubject::Integration(integration.id.clone()));
        Ok(out)
    }

    /// Records (or refreshes) the trust of `integration` after a check that
    /// succeeded: its current address, and the account the token belongs to.
    /// Does not widen anything by itself: [`Self::admit`] has already passed
    /// for this address.
    pub fn record_after_check(&self, integration: &Integration, account: &Account) -> Result<()> {
        let origin = integration_origin(integration)?;
        self.update_record(integration, |r| {
            if r.origin != origin {
                r.origin = origin;
            }
            r.account_id = account.id.clone();
            r.account_login = account.login.clone();
            r.account_checked_at = unix_now();
        })
    }

    /// Accepts the integration's current address as the trusted one. The
    /// account is forgotten with it: another server has other ids.
    pub fn trust_origin(&self, integration: &Integration) -> Result<()> {
        let origin = integration_origin(integration)?;
        self.update_record(integration, |r| {
            if r.origin != origin {
                r.origin = origin;
                r.account_id.clear();
                r.account_login.clear();
                r.account_checked_at = 0;
                r.tls = None;
            }
        })
    }

    /// Accepts `account` as the owner of the token.
    pub fn trust_account(&self, integration: &Integration, account: &Account) -> Result<()> {
        self.update_record(integration, |r| {
            r.account_id = account.id.clone();
            r.account_login = account.login.clone();
            r.account_checked_at = unix_now();
        })
    }

    /// Pins the certificate key of the integration's host.
    pub fn trust_certificate(
        &self,
        integration: &Integration,
        cert: &CertInfo,
        mode: TlsMode,
    ) -> Result<()> {
        self.update_record(integration, |r| {
            r.tls = Some(TlsRecord {
                mode,
                cert: cert.clone(),
            });
        })
    }

    /// Switches the pin between `tofu` and `strict` (it needs a pinned key).
    pub fn set_strict(&self, integration: &Integration, strict: bool) -> Result<bool> {
        self.update_record(integration, |r| match &mut r.tls {
            Some(tls) => {
                tls.mode = if strict {
                    TlsMode::Strict
                } else {
                    TlsMode::Tofu
                };
                true
            }
            None => false,
        })
    }

    pub fn set_allow_plain_http(&self, integration: &Integration, on: bool) -> Result<()> {
        self.update_record(integration, |r| r.allow_plain_http = on)
    }

    // --- AI profiles ----------------------------------------------------------

    fn fresh_ai(&self, profile: &str, origin: String) -> AiRecord {
        AiRecord {
            profile: profile.to_string(),
            origin,
            recorded_at: unix_now(),
            allow_plain_http: self.lenient,
            tls: None,
        }
    }

    /// The checks of [`Self::admit`] for an API-key AI profile bound to
    /// `base_url`. `has_key`: a key is stored, so a missing record is written
    /// now (migration); without one it is held in memory until the key is
    /// stored and [`Self::record_ai`] runs. Returns the origins a request may
    /// reach.
    pub fn admit_ai(&self, profile: &str, base_url: &str, has_key: bool) -> Result<Vec<String>> {
        let origin = normalize(base_url).ok_or_else(|| Error::BadUrl(base_url.to_string()))?;
        let subject = TrustSubject::Ai(profile.to_string());
        let mut store = self.store()?;
        let record = match store.ai_record(profile) {
            Some(r) => r.clone(),
            None => {
                let record = lock(&self.provisional_ai)
                    .get(profile)
                    .cloned()
                    .unwrap_or_else(|| self.fresh_ai(profile, origin.clone()));
                if has_key {
                    store.put_ai(record.clone());
                    self.save(&store)?;
                } else {
                    lock(&self.provisional_ai).insert(profile.to_string(), record.clone());
                }
                record
            }
        };
        if origin::is_plain_http_remote(&origin) && !record.allow_plain_http {
            return self.refuse(TrustProblem::PlainHttp { subject, origin });
        }
        if record.origin != origin {
            return self.refuse(TrustProblem::OriginChanged {
                subject,
                old: record.origin,
                new: origin,
            });
        }
        apply_tls_policy(&record.origin, record.tls.as_ref());
        Ok(origin::origin_of(&record.origin).into_iter().collect())
    }

    /// The AI record of `profile`, if one was written.
    pub fn ai_record_of(&self, profile: &str) -> Option<AiRecord> {
        self.store().ok()?.ai_record(profile).cloned()
    }

    /// Changes the AI record of `profile` (made from `base_url` first when
    /// there is none) and writes it.
    pub fn update_ai_record<R>(
        &self,
        profile: &str,
        base_url: &str,
        change: impl FnOnce(&mut AiRecord) -> R,
    ) -> Result<R> {
        let origin = normalize(base_url).ok_or_else(|| Error::BadUrl(base_url.to_string()))?;
        let mut store = self.store()?;
        let mut record = match store.ai_record(profile) {
            Some(r) => r.clone(),
            None => lock(&self.provisional_ai)
                .remove(profile)
                .unwrap_or_else(|| self.fresh_ai(profile, origin)),
        };
        let out = change(&mut record);
        apply_tls_policy(&record.origin, record.tls.as_ref());
        store.put_ai(record);
        self.save(&store)?;
        lock(&self.provisional_ai).remove(profile);
        clear_pending(&TrustSubject::Ai(profile.to_string()));
        Ok(out)
    }

    /// Binds the key of `profile` to the current origin of `base_url`: call
    /// it when a key is stored.
    pub fn record_ai(&self, profile: &str, base_url: &str) -> Result<()> {
        let origin = normalize(base_url).ok_or_else(|| Error::BadUrl(base_url.to_string()))?;
        self.update_ai_record(profile, base_url, |r| r.origin = origin)
    }

    /// Accepts the profile's current address as the trusted one.
    pub fn trust_ai_origin(&self, profile: &str, base_url: &str) -> Result<()> {
        self.record_ai(profile, base_url)
    }
}

/// Feeds the TLS verifier the pin of `origin`'s host.
#[cfg(feature = "network")]
fn apply_tls_policy(origin: &str, tls: Option<&TlsRecord>) {
    let Some(host) = origin::host_of(origin) else {
        return;
    };
    tls::set_policy(
        &host,
        tls.map(|t| match t.mode {
            TlsMode::Tofu => tls::TlsPolicy::Tofu {
                spki: t.cert.spki_sha256.clone(),
            },
            TlsMode::Strict => tls::TlsPolicy::Strict {
                spki: t.cert.spki_sha256.clone(),
            },
        }),
    );
}

#[cfg(not(feature = "network"))]
fn apply_tls_policy(_origin: &str, _tls: Option<&TlsRecord>) {}

#[cfg(test)]
mod tests;
