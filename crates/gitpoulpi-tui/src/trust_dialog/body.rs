//! What the dialog says for each kind of change, and which answers it offers.

use gitpoulpi_hosting::trust::CertInfo;
use gitpoulpi_hosting::{TrustProblem, TrustSubject};
use gitpoulpi_i18n::msg::trust_dialog as msg;

use super::Choice;

/// How a body line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    Plain,
    Heading,
    Muted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Line {
    pub(crate) text: String,
    pub(crate) tone: Tone,
}

fn line(text: impl Into<String>, tone: Tone) -> Line {
    Line {
        text: text.into(),
        tone,
    }
}

/// The answers offered for `problem`, in the order they are numbered. An
/// integration can be disconnected; an AI key profile has no token here to
/// delete, and a check that only knows a host can only be cancelled.
pub(crate) fn choices(problem: &TrustProblem) -> Vec<Choice> {
    match problem.subject() {
        TrustSubject::Integration(_) => vec![Choice::Disconnect, Choice::Trust, Choice::Cancel],
        TrustSubject::Ai(_) => vec![Choice::Trust, Choice::Cancel],
        TrustSubject::Host(_) => vec![Choice::Cancel],
    }
}

/// The text of the "Trust ..." answer for this kind of change.
pub(crate) fn trust_label(problem: &TrustProblem) -> &'static str {
    match problem {
        TrustProblem::OriginChanged { .. } => msg::ANSWER_TRUST_ORIGIN.get(),
        TrustProblem::AccountChanged { .. } => msg::ANSWER_TRUST_ACCOUNT.get(),
        TrustProblem::UntrustedCertificate { .. } | TrustProblem::CertificateChanged { .. } => {
            msg::ANSWER_TRUST_CERT.get()
        }
        TrustProblem::PlainHttp { .. } => msg::ANSWER_ALLOW_HTTP.get(),
    }
}

/// The label of one answer.
pub(crate) fn label(problem: &TrustProblem, choice: Choice) -> String {
    match choice {
        Choice::Disconnect => msg::ANSWER_DISCONNECT.get().to_string(),
        Choice::Trust => trust_label(problem).to_string(),
        Choice::Cancel => msg::ANSWER_CANCEL.get().to_string(),
    }
}

/// `text` cut into lines of at most `width` columns.
fn wrapped(text: &str, width: usize, tone: Tone, out: &mut Vec<Line>) {
    for l in crate::wrap_words(text, width) {
        out.push(line(l, tone));
    }
}

/// One `label  value` pair, the value wrapped under itself.
fn field(label: &str, value: &str, label_w: usize, width: usize, out: &mut Vec<Line>) {
    let value = if value.is_empty() {
        msg::CERT_UNKNOWN.get()
    } else {
        value
    };
    let room = width.saturating_sub(label_w + 4).max(8);
    for (i, part) in crate::wrap_words(value, room).into_iter().enumerate() {
        let head = if i == 0 {
            format!("{label:<label_w$}")
        } else {
            " ".repeat(label_w)
        };
        out.push(line(format!("  {head}  {part}"), Tone::Plain));
    }
}

/// A certificate as a small table: its key fingerprint in groups of four,
/// subject, issuer and validity.
fn certificate(title: &str, cert: &CertInfo, width: usize, out: &mut Vec<Line>) {
    out.push(line(title, Tone::Heading));
    let labels = [
        msg::CERT_FINGERPRINT.get(),
        msg::CERT_SUBJECT.get(),
        msg::CERT_ISSUER.get(),
        msg::CERT_VALID_FROM.get(),
        msg::CERT_VALID_TO.get(),
    ];
    let label_w = labels
        .iter()
        .map(|l| crate::display_width(l))
        .max()
        .unwrap_or(0);
    let values = [
        cert.grouped(),
        cert.subject.clone(),
        cert.issuer.clone(),
        cert.not_before.clone(),
        cert.not_after.clone(),
    ];
    for (label, value) in labels.iter().zip(values.iter()) {
        field(label, value, label_w, width, out);
    }
}

/// The body for `problem`, wrapped to `width` columns.
pub(crate) fn lines(problem: &TrustProblem, width: usize) -> Vec<Line> {
    let name = problem.subject().name();
    let mut out = Vec::new();
    match problem {
        TrustProblem::OriginChanged { old, new, .. } => {
            wrapped(
                &msg::origin_changed(name, new, old),
                width,
                Tone::Plain,
                &mut out,
            );
        }
        TrustProblem::AccountChanged { old, new, .. } => {
            wrapped(&msg::account_changed(name), width, Tone::Plain, &mut out);
            out.push(line("", Tone::Plain));
            wrapped(
                &msg::account_was(&old.login, &old.id),
                width,
                Tone::Muted,
                &mut out,
            );
            wrapped(
                &msg::account_now(&new.login, &new.id),
                width,
                Tone::Plain,
                &mut out,
            );
        }
        TrustProblem::UntrustedCertificate { new, .. } => {
            wrapped(&msg::cert_untrusted(name), width, Tone::Plain, &mut out);
            out.push(line("", Tone::Plain));
            certificate(msg::CERT_NOW.get(), new, width, &mut out);
        }
        TrustProblem::CertificateChanged { old, new, .. } => {
            wrapped(&msg::cert_changed(name), width, Tone::Plain, &mut out);
            out.push(line("", Tone::Plain));
            certificate(msg::CERT_BEFORE.get(), old, width, &mut out);
            out.push(line("", Tone::Plain));
            certificate(msg::CERT_NOW.get(), new, width, &mut out);
        }
        TrustProblem::PlainHttp { origin, .. } => {
            wrapped(&msg::plain_http(name, origin), width, Tone::Plain, &mut out);
        }
    }
    out
}
