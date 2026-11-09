//! The lines of the connection test's results panel: per remote, the
//! fetch and the push check as ok or failed, the identity that answered
//! (the SSH key the server accepted, or the account whose token was
//! accepted), the duration, and for a failure the friendly reading.

use gitpoulpi_core::domain::{RemoteAccessReport, RemoteIdentity};
use gitpoulpi_i18n::msg::repo_auth as t;

use super::panes::Kind;

/// One check's line and, for a failure, the reading under it.
fn check_lines(op: &str, report: Option<&RemoteAccessReport>) -> Vec<(Kind, String)> {
    let Some(r) = report else {
        return Vec::new();
    };
    let mut extra = Vec::new();
    if r.ok {
        extra.push(t::result_branches(r.ref_count as u64));
    }
    extra.push(t::result_ms(r.duration_ms));
    if r.ok {
        if let Some(key) = &r.ssh_accepted_key {
            extra.push(t::result_accepted_key(key));
        } else if let Some(who) = &r.accepted_identity {
            extra.push(t::result_accepted_account(who));
        } else {
            extra.push(t::RESULT_SYSTEM.get().to_string());
        }
    }
    let extra = extra.join(", ");
    if r.ok {
        return vec![(Kind::Good, t::result_ok(op, extra))];
    }
    let mut out = vec![(Kind::Bad, t::result_failed(op, extra))];
    if let Some(detail) = &r.error {
        let kind = detail.kind();
        match crate::activity_log::friendly::wording(kind, &detail.stderr) {
            Some((summary, hint)) => {
                out.push((Kind::Normal, t::result_reason(summary)));
                out.push((Kind::Muted, t::result_reason(hint)));
            }
            None => {
                if let Some(line) = detail.stderr.lines().rev().find(|l| !l.trim().is_empty()) {
                    out.push((Kind::Normal, t::result_reason(line.trim())));
                }
            }
        }
    }
    out
}

/// The panel's lines for both checks.
pub(super) fn lines(
    identities: &[RemoteIdentity],
    fetch: &[RemoteAccessReport],
    push: &[RemoteAccessReport],
) -> Vec<(Kind, String)> {
    let mut out = vec![
        (Kind::Muted, t::RESULT_SAVED_NOTE.get().to_string()),
        (Kind::Normal, String::new()),
    ];
    let mut names: Vec<&str> = Vec::new();
    for r in fetch.iter().chain(push) {
        if !names.contains(&r.remote.as_str()) {
            names.push(&r.remote);
        }
    }
    if names.is_empty() {
        out.push((Kind::Normal, t::RESULT_NONE.get().to_string()));
        return out;
    }
    let fetch_op = t::RESULTS_FETCH.get();
    let push_op = t::RESULTS_PUSH.get();
    for name in names {
        let first = fetch
            .iter()
            .chain(push)
            .find(|r| r.remote == name)
            .map(|r| format!("{:?}", r.transport).to_ascii_lowercase())
            .unwrap_or_default();
        let who = identities
            .iter()
            .find(|i| i.remote == name)
            .map_or("?", |i| i.identity.as_str());
        out.push((Kind::Heading, t::results_remote(name, first, who)));
        out.extend(check_lines(
            fetch_op,
            fetch.iter().find(|r| r.remote == name),
        ));
        out.extend(check_lines(push_op, push.iter().find(|r| r.remote == name)));
        out.push((Kind::Normal, String::new()));
    }
    out
}

/// The panel when the test itself could not run.
pub(super) fn error_lines(why: &str) -> Vec<(Kind, String)> {
    vec![(Kind::Bad, t::result_error(why))]
}
