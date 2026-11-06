//! Doing what the person answered.
//!
//! Trusting is a few small writes to `trust.toml` through the same
//! `TrustGate` doors the CLI's `integrations trust` uses, so the two can never
//! disagree. Disconnecting deletes the saved token, which may be a credential
//! helper subprocess: it runs on a worker thread (the render thread never
//! waits), and the Settings page re-reads the profile list once it is done
//! ([`take_changed`]).

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use gitpoulpi_config::{set_list_in_file, Config, ConfigPaths, Integration};
use gitpoulpi_hosting::trust::{self, CertInfo, TlsMode, TlsRecord};
use gitpoulpi_hosting::{Account, TrustGate, TrustProblem, TrustSubject};
use gitpoulpi_i18n::msg::trust_dialog as msg;
use gitpoulpi_secrets::{CredentialHelperTokens, TokenStore};

use super::{Choice, Outcome};
use crate::preferences::PreferencesState;

/// Set when a background answer rewrote the profile list.
static CHANGED: AtomicBool = AtomicBool::new(false);

/// Whether the profile list changed under an open Settings page since the
/// last call.
pub(crate) fn take_changed() -> bool {
    CHANGED.swap(false, Ordering::SeqCst)
}

/// Performs `outcome` against the real config directory; `message` gets the
/// status line.
pub(crate) fn apply(
    outcome: Outcome,
    repo_root: &Path,
    preferences: &mut Option<PreferencesState>,
    message: &mut Option<String>,
) {
    let Some(paths) = ConfigPaths::discover() else {
        return;
    };
    apply_in(&paths, outcome, repo_root, preferences, message);
}

/// [`apply`] over an explicit config directory (the tests' own).
pub(super) fn apply_in(
    paths: &ConfigPaths,
    outcome: Outcome,
    repo_root: &Path,
    preferences: &mut Option<PreferencesState>,
    message: &mut Option<String>,
) {
    let gate = TrustGate::load(paths);
    match outcome.problem.subject().clone() {
        TrustSubject::Integration(id) => {
            let config = Config::load(&paths.config_file()).unwrap_or_default();
            let Some(integration) = config.integrations.iter().find(|i| i.id == id).cloned() else {
                *message = Some(msg::no_integration(&id));
                return;
            };
            match outcome.choice {
                Choice::Disconnect => {
                    *message = Some(msg::disconnecting(&id));
                    disconnect(paths.clone(), integration);
                }
                Choice::Trust => {
                    let (said, accepted) = trust_integration(&gate, &outcome.problem, &integration);
                    *message = Some(said);
                    let origin_change =
                        matches!(outcome.problem, TrustProblem::OriginChanged { .. });
                    if origin_change && accepted {
                        let state =
                            preferences.get_or_insert_with(|| PreferencesState::open(repo_root));
                        state.show_retoken(&id);
                    } else if let Some(state) = preferences.as_mut() {
                        state.reload_integrations();
                    }
                }
                Choice::Cancel => {}
            }
        }
        TrustSubject::Ai(profile) => {
            if outcome.choice == Choice::Trust {
                *message = Some(trust_ai(&gate, &outcome.problem, &profile));
            }
        }
        TrustSubject::Host(_) => {}
    }
}

fn disconnect(paths: ConfigPaths, integration: Integration) {
    std::thread::spawn(move || {
        let tokens = CredentialHelperTokens::new();
        if let Err(err) = tokens.delete(&integration.token_ref) {
            crate::activity_log::report_problem(
                crate::activity_log::ProblemSource::Hosting,
                &err.to_string(),
            );
            return;
        }
        let file = paths.config_file();
        if let Ok(config) = Config::load(&file) {
            let mut list = config.integrations;
            if let Some(entry) = list.iter_mut().find(|i| i.id == integration.id) {
                entry.user = None;
            }
            let _ = set_list_in_file(&file, "integrations", &list);
        }
        trust::forget(&paths, &integration);
        CHANGED.store(true, Ordering::SeqCst);
        crate::redraw::poke();
    });
}

/// The current address of `integration` must still be the one the dialog
/// showed: a change since then is a new question. Returns the status line and
/// whether something was accepted.
fn trust_integration(
    gate: &TrustGate,
    problem: &TrustProblem,
    integration: &Integration,
) -> (String, bool) {
    let id = integration.id.as_str();
    let done = match problem {
        TrustProblem::OriginChanged { new, .. } => {
            let current = trust::integration_origin(integration).ok();
            if current.as_deref() != Some(new.as_str()) {
                return (msg::changed_again(id), false);
            }
            gate.trust_origin(integration)
                .map(|()| msg::trusted_origin(id))
        }
        TrustProblem::AccountChanged { new, .. } => gate
            .trust_account(
                integration,
                &Account {
                    login: new.login.clone(),
                    id: new.id.clone(),
                },
            )
            .map(|()| msg::trusted_account(id)),
        TrustProblem::UntrustedCertificate { new, .. }
        | TrustProblem::CertificateChanged { new, .. } => {
            let mode = gate
                .record_of(id)
                .and_then(|r| r.tls.map(|t| t.mode))
                .unwrap_or(TlsMode::Tofu);
            gate.trust_certificate(integration, new, mode)
                .map(|()| msg::trusted_cert(id))
        }
        TrustProblem::PlainHttp { .. } => gate
            .set_allow_plain_http(integration, true)
            .map(|()| msg::allowed_http(id)),
    };
    match done {
        Ok(said) => (said, true),
        Err(e) => (e.to_string(), false),
    }
}

/// The same answers for an API-key AI profile (bound to the origin of its
/// base URL).
fn trust_ai(gate: &TrustGate, problem: &TrustProblem, profile: &str) -> String {
    let origin = match gate.ai_record_of(profile) {
        Some(r) => r.origin,
        None => problem.new_text(),
    };
    let done = match problem {
        TrustProblem::OriginChanged { new, .. } => gate
            .trust_ai_origin(profile, new)
            .map(|()| msg::trusted_origin(profile)),
        TrustProblem::PlainHttp { origin: url, .. } => gate
            .update_ai_record(profile, url, |r| r.allow_plain_http = true)
            .map(|()| msg::allowed_http(profile)),
        TrustProblem::UntrustedCertificate { new, .. }
        | TrustProblem::CertificateChanged { new, .. } => gate
            .update_ai_record(profile, &origin, |r| r.tls = Some(pin(new, r.tls.as_ref())))
            .map(|()| msg::trusted_cert(profile)),
        TrustProblem::AccountChanged { .. } => Ok(String::new()),
    };
    done.unwrap_or_else(|e| e.to_string())
}

fn pin(cert: &CertInfo, old: Option<&TlsRecord>) -> TlsRecord {
    TlsRecord {
        mode: old.map_or(TlsMode::Tofu, |t| t.mode),
        cert: cert.clone(),
    }
}
