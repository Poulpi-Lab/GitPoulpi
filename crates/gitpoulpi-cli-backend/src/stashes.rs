use gitpoulpi_core::domain::{decode_commit_text, Oid, RepoRef, Stash, StashId};
use gitpoulpi_core::Result;

use crate::process::run_bytes;

const FIELD_SEP: char = '\u{1}';

pub fn stashes(repo: &RepoRef) -> Result<Vec<Stash>> {
    // `%H` and `%P` ride along in the same call: the stash's own commit
    // and its parents, the first of which is the commit it was made on.
    // The Log draws each stash beside that commit, and one more `git`
    // spawn per stash would cost ~1.5 s each on Windows.
    let format = format!("%gd{FIELD_SEP}%H{FIELD_SEP}%P{FIELD_SEP}%at{FIELD_SEP}%gs");
    let raw = run_bytes(
        &repo.root,
        &["stash", "list", &format!("--format={format}")],
    )?;

    let stashes = raw
        .split(|b| *b == b'\n')
        // Per line: the message may be in the fallback encoding (it embeds
        // the HEAD commit's subject), and UTF-8 lines must stay as they are.
        .map(|line| decode_commit_text(line, None).into_owned())
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            // The message last, so a stray separator in it cannot shift
            // the fields before it.
            let mut fields = line.splitn(5, FIELD_SEP);
            let gd = fields.next()?;
            let commit = Oid::from_hex(fields.next()?.trim());
            let base = fields
                .next()?
                .split_whitespace()
                .next()
                .and_then(Oid::from_hex);
            let timestamp = fields.next()?.trim().parse().ok()?;
            let message = fields.next()?.to_string();

            let id = gd
                .trim_start_matches("stash@{")
                .trim_end_matches('}')
                .parse()
                .ok()?;

            let branch_at_creation = branch_from_message(&message);

            Some(Stash {
                id: StashId(id),
                message,
                branch_at_creation,
                timestamp,
                commit,
                base,
            })
        })
        .collect();

    Ok(stashes)
}

/// `git stash` messages default to "WIP on <branch>: ..." or "On
/// <branch>: ..." (custom `stash push -m` messages won't match, and
/// that's fine - branch_at_creation is best-effort).
fn branch_from_message(message: &str) -> String {
    for prefix in ["WIP on ", "On "] {
        if let Some(rest) = message.strip_prefix(prefix) {
            if let Some((branch, _)) = rest.split_once(':') {
                return branch.to_string();
            }
        }
    }
    String::new()
}
