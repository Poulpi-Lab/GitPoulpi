//! `GitBackend::remote_access_check`: `git ls-remote --heads` against a
//! remote's URL, through the same command builder as every transfer, so the
//! repository's `NetworkAuth` (or the global credential mode and key) applies
//! exactly as it will to a fetch or a push.

use std::time::Instant;

use gitpoulpi_core::domain::{
    strip_userinfo, transport_of, RemoteAccessOptions, RemoteAccessReport, RemoteTransport, RepoRef,
};
use gitpoulpi_core::{Error, ErrorDetail, Result};

use crate::process::{run, run_full_async_with};

/// See the module docs.
pub async fn check(
    repo: &RepoRef,
    remote: &str,
    opts: &RemoteAccessOptions,
) -> Result<RemoteAccessReport> {
    gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    // `get-url` applies `insteadOf` / `pushInsteadOf`: it is the address git
    // would really contact. An unknown remote fails here, as an error.
    let mut get_url = vec!["remote", "get-url"];
    if opts.push {
        get_url.push("--push");
    }
    get_url.push(remote);
    let url = run(&repo.root, &get_url)?.trim().to_string();
    if url.is_empty() {
        return Err(Error::Backend(format!("remote `{remote}` has no URL")));
    }
    let transport = transport_of(&url);
    let shown = strip_userinfo(&url);

    let started = Instant::now();
    let output = run_full_async_with(
        &repo.root,
        &["ls-remote", "--heads", "--", &url],
        opts.ssh_verbose && transport == RemoteTransport::Ssh,
    )
    .await?;
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

    let (accepted, authenticated) = if opts.ssh_verbose {
        parse_ssh_verbose(&output.stderr)
    } else {
        (None, None)
    };
    let ref_count = if output.success {
        output
            .stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count()
    } else {
        0
    };
    // The detail names the address without its userinfo, like the report.
    let error = (!output.success).then(|| {
        ErrorDetail::from_git(
            &["ls-remote", "--heads", "--", &shown],
            output.exit_code,
            &output.stderr,
        )
        .with_operation("RemoteAccessCheck")
    });
    Ok(RemoteAccessReport {
        remote: remote.to_string(),
        url: shown,
        transport,
        push: opts.push,
        ok: output.success,
        ref_count,
        duration_ms,
        ssh_accepted_key: accepted,
        ssh_authenticated_to: authenticated,
        error,
    })
}

/// The key the server accepted and the host that authenticated, from the
/// stderr of `ssh -v`:
///
/// - `debug1: Server accepts key: <key> <type> SHA256:... <how>` - `<key>` is
///   the identity file's path, or the comment of an agent's key. Older ssh
///   prints `Server accepts key: pkalg <alg> blen <n>`: then the algorithm.
///   ssh tries every offered key in turn, so the **last** such line is the
///   one the server took.
/// - `Authenticated to <host> ([<ip>]:<port>) using "publickey".`
fn parse_ssh_verbose(stderr: &str) -> (Option<String>, Option<String>) {
    let mut accepted = None;
    let mut authenticated = None;
    for line in stderr.lines() {
        if let Some(rest) = line.split_once("Server accepts key: ").map(|(_, r)| r) {
            // `<key> <TYPE> <FINGERPRINT> <how>`: the key is a path that may
            // hold spaces, so it is everything before the type that precedes
            // the fingerprint.
            let fingerprint = [" SHA256:", " MD5:"]
                .iter()
                .filter_map(|marker| rest.find(marker))
                .min();
            let mut words = rest.split_whitespace();
            accepted = match (fingerprint, words.next()) {
                (_, Some("pkalg")) => words.next().map(|alg| format!("pkalg {alg}")),
                (Some(at), _) => rest[..at]
                    .rsplit_once(' ')
                    .map(|(key, _type)| key.trim().to_string())
                    .filter(|key| !key.is_empty())
                    .or(accepted),
                (None, Some(key)) => Some(key.to_string()),
                (None, None) => accepted,
            };
        } else if let Some(rest) = line.split_once("Authenticated to ").map(|(_, r)| r) {
            authenticated = rest.split_whitespace().next().map(str::to_string);
        }
    }
    (accepted, authenticated)
}

#[cfg(test)]
mod tests {
    use super::parse_ssh_verbose;

    #[test]
    fn the_last_accepted_key_and_the_host_are_read_from_ssh_verbose() {
        let stderr = "debug1: Offering public key: /k/a ED25519 SHA256:x\n\
                      debug1: Server accepts key: /k/a ED25519 SHA256:x explicit\n\
                      debug1: Server accepts key: work-agent-key ED25519 SHA256:y agent\n\
                      Authenticated to github.com ([140.82.121.4]:22) using \"publickey\".\n";
        assert_eq!(
            parse_ssh_verbose(stderr),
            (
                Some("work-agent-key".to_string()),
                Some("github.com".to_string())
            )
        );
        assert_eq!(
            parse_ssh_verbose("debug1: Server accepts key: pkalg ssh-rsa blen 279\n").0,
            Some("pkalg ssh-rsa".to_string())
        );
        assert_eq!(
            parse_ssh_verbose(
                "debug1: Server accepts key: /home/me/my keys/id work ED25519 SHA256:z explicit\n"
            )
            .0,
            Some("/home/me/my keys/id work".to_string())
        );
        assert_eq!(parse_ssh_verbose("fatal: nothing\n"), (None, None));
    }
}
