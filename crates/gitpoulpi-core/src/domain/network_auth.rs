//! How one repository's network git processes authenticate (ADR-0010), and
//! the report of a connection test.

use serde::{Deserialize, Serialize};

use crate::failure::ErrorDetail;

/// What a repository's network git processes (fetch, pull, push, submodules,
/// LFS, `ls-remote`) get **instead of** the process-wide credential mode and
/// SSH key. Held in memory by the backend, per repository, and never
/// persisted (see `GitBackend::set_network_auth`): the daemon rebuilds it from
/// the repository's profile before each transfer.
///
/// Everything here lands on the git command line (`-c` overrides) or in the
/// child's environment, so it holds no secret: a credential reaches git
/// through a helper, never through this struct.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkAuth {
    /// `None`: git's own helpers stay in force. `Some(list)`: the helper list
    /// is reset (`-c credential.helper=`, which comes after every config
    /// file, repository and included ones too, and clears URL-scoped helpers
    /// as well) and then each entry is added.
    pub credential_helpers: Option<Vec<String>>,
    /// `-c credential.username=`.
    pub credential_username: Option<String>,
    /// No prompt of any kind: `GIT_TERMINAL_PROMPT=0`, `-c core.askPass=`,
    /// and `GIT_ASKPASS` / `SSH_ASKPASS` removed from the child.
    pub no_prompt: bool,
    /// `-c http.extraHeader=`: an `Authorization` header from the repository's
    /// config (or one it includes) cannot interfere.
    pub reset_extra_header: bool,
    /// `GIT_SSH_COMMAND`. Wins over the environment's own `GIT_SSH_COMMAND`
    /// and over `core.sshCommand`, which is the point of an explicit choice
    /// for this repository.
    pub ssh_command: Option<String>,
    /// Extra environment for the child. Non-secret values only (a grant id,
    /// an endpoint name).
    pub env: Vec<(String, String)>,
}

/// What [`crate::GitBackend::remote_access_check`] should do.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RemoteAccessOptions {
    /// Test the push URL (`git remote get-url --push`) instead of the fetch
    /// one. The two differ with `pushurl` or `pushInsteadOf`.
    pub push: bool,
    /// Run ssh with `-v` so the report can name the key the server accepted.
    /// Only for a test, never for an ordinary operation.
    pub ssh_verbose: bool,
}

/// The transport of a remote URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteTransport {
    Https,
    Http,
    Ssh,
    Git,
    File,
    Other,
}

/// The answer of a connection test.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAccessReport {
    pub remote: String,
    /// The URL that was contacted, with any userinfo (`user:token@`) removed.
    pub url: String,
    pub transport: RemoteTransport,
    pub push: bool,
    pub ok: bool,
    /// Branches the remote advertised (`ls-remote --heads`); 0 on failure.
    pub ref_count: usize,
    pub duration_ms: u64,
    /// With `ssh_verbose`: the key the server accepted, as ssh names it (a
    /// path, or the agent key's comment).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_accepted_key: Option<String>,
    /// With `ssh_verbose`: the host ssh reported `Authenticated to`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_authenticated_to: Option<String>,
    /// Why it failed; [`ErrorDetail::kind`] classifies it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
}

/// `url` without its userinfo (`https://user:token@host/x` becomes
/// `https://host/x`). An scp-like `git@host:path` keeps its `git@`: that is
/// the account name, not a secret, and removing it would make the URL a
/// different address.
pub fn strip_userinfo(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return url.to_string();
    };
    let (scheme, rest) = url.split_at(scheme_end + 3);
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(authority_end);
    match authority.rfind('@') {
        Some(at) if scheme.starts_with("ssh") => {
            // `ssh://git@host/x`: the user name is not a secret, but anything
            // after a `:` in the userinfo is a password.
            let userinfo = &authority[..at];
            let user = userinfo.split(':').next().unwrap_or("");
            format!("{scheme}{user}@{}{tail}", &authority[at + 1..])
        }
        Some(at) => format!("{scheme}{}{tail}", &authority[at + 1..]),
        None => url.to_string(),
    }
}

/// The transport a remote URL names.
pub fn transport_of(url: &str) -> RemoteTransport {
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("https://") {
        RemoteTransport::Https
    } else if lower.starts_with("http://") {
        RemoteTransport::Http
    } else if lower.starts_with("ssh://") || lower.starts_with("git+ssh://") {
        RemoteTransport::Ssh
    } else if lower.starts_with("git://") {
        RemoteTransport::Git
    } else if lower.starts_with("file://") {
        RemoteTransport::File
    } else if lower.contains("::") {
        RemoteTransport::Other
    } else if !lower.contains("://") && url.contains(':') && !looks_like_drive_path(url) {
        // `git@host:path`
        RemoteTransport::Ssh
    } else {
        RemoteTransport::File
    }
}

fn looks_like_drive_path(url: &str) -> bool {
    let b = url.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn userinfo_is_stripped_but_an_ssh_account_name_stays() {
        assert_eq!(
            strip_userinfo("https://me:secret@github.com/o/r.git"),
            "https://github.com/o/r.git"
        );
        assert_eq!(
            strip_userinfo("https://tok@host:8443/o/r"),
            "https://host:8443/o/r"
        );
        assert_eq!(
            strip_userinfo("ssh://git:pw@host/o/r"),
            "ssh://git@host/o/r"
        );
        assert_eq!(strip_userinfo("git@host:o/r.git"), "git@host:o/r.git");
        assert_eq!(strip_userinfo("/tmp/x@y/r"), "/tmp/x@y/r");
        assert_eq!(strip_userinfo("https://host/a@b"), "https://host/a@b");
    }

    #[test]
    fn transports_are_read_from_the_url() {
        use RemoteTransport as T;
        assert_eq!(transport_of("https://h/r"), T::Https);
        assert_eq!(transport_of("ssh://git@h/r"), T::Ssh);
        assert_eq!(transport_of("git@h:o/r.git"), T::Ssh);
        assert_eq!(transport_of("git://h/r"), T::Git);
        assert_eq!(transport_of("/srv/r.git"), T::File);
        assert_eq!(transport_of("C:/srv/r.git"), T::File);
        assert_eq!(transport_of("ext::sh -c x"), T::Other);
    }
}
