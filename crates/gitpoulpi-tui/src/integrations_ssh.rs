//! The IO half of the Integrations page's per-profile SSH key section
//! (ADR-0008): generate a key in process, register it with the provider
//! (after checking the provider does not already hold it), attach an
//! existing key, and remove or delete one.
//!
//! Everything here is a plain function over `(profile, token store,
//! transport)` so it runs on a worker thread and is testable in a
//! tempdir with a `RecordedTransport`. Nothing in this module touches
//! the user's real `~/.ssh` unless the caller passes it.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gitpoulpi_config::{Integration, IntegrationSsh, SshMode};
use gitpoulpi_hosting::ssh_key::{
    fingerprint_of, generate_and_write, profile_key_path, same_public_key,
};
use gitpoulpi_hosting::{client_for, HttpTransport};
use gitpoulpi_i18n::msg::integrations_ssh as msg;
use gitpoulpi_secrets::TokenStore;

type Tokens = Arc<dyn TokenStore + Send + Sync>;

/// What a finished background key job did to the profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SshOutcome {
    /// The profile now uses this key. `note` says what happened besides.
    Attached { ssh: IntegrationSsh, note: String },
    /// The profile no longer has a key.
    Detached { note: String },
}

/// What to generate.
pub(crate) struct GenerateRequest {
    pub(crate) integration: Integration,
    pub(crate) ssh_dir: PathBuf,
    pub(crate) title: String,
    pub(crate) upload: bool,
}

fn block_on<F: std::future::Future>(f: F) -> Result<F::Output, String> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(msg::request_start_failed)
        .map(|rt| rt.block_on(f))
}

fn transport_or_default(
    transport: Option<Arc<dyn HttpTransport>>,
) -> Result<Arc<dyn HttpTransport>, String> {
    match transport {
        Some(t) => Ok(t),
        None => Ok(Arc::new(
            gitpoulpi_hosting::ReqwestTransport::new().map_err(|e| e.to_string())?,
        )),
    }
}

fn token_for(tokens: &Tokens, integration: &Integration) -> Result<String, String> {
    tokens
        .load(&integration.token_ref)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| msg::NO_TOKEN_CONNECT_FIRST.get().to_string())
}

/// Generates the profile's key pair in `ssh_dir` and, when asked, registers
/// the public half. `list_ssh_keys` runs first: a provider refuses a
/// duplicate key with an unhelpful error, and a key already there is
/// simply adopted. An upload that fails does not lose the key - it stays
/// on disk, attached, with a note saying the upload did not happen.
pub(crate) fn generate(
    tokens: &Tokens,
    transport: Option<Arc<dyn HttpTransport>>,
    req: GenerateRequest,
) -> Result<SshOutcome, String> {
    let path = profile_key_path(&req.ssh_dir, &req.integration.id);
    let generated = generate_and_write(&path, &req.title).map_err(|e| e.to_string())?;
    let mut ssh = IntegrationSsh {
        mode: SshMode::Generated,
        private_key_path: Some(generated.private_key_path.display().to_string()),
        public_key_path: Some(generated.public_key_path.display().to_string()),
        title: Some(req.title.clone()),
        remote_key_id: None,
    };
    let shown = generated.private_key_path.display();
    let mut note = msg::generated(&shown);
    if req.upload {
        let uploaded = (|| -> Result<(String, bool), String> {
            let secret = token_for(tokens, &req.integration)?;
            let client = client_for(
                req.integration.clone(),
                secret,
                transport_or_default(transport)?,
                &gitpoulpi_hosting::TrustGate::ambient(),
            )
            .map_err(|e| e.to_string())?;
            block_on(async {
                let existing = client.list_ssh_keys().await.map_err(|e| e.to_string())?;
                if let Some(k) = existing
                    .iter()
                    .find(|k| same_public_key(&k.key, &generated.public_key))
                {
                    return Ok((k.id.clone(), true));
                }
                let key = client
                    .upload_ssh_key(&req.title, &generated.public_key)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok((key.id, false))
            })?
        })();
        match uploaded {
            Ok((id, already)) => {
                ssh.remote_key_id = Some(id);
                note = if already {
                    msg::generated_adopted(&shown)
                } else {
                    msg::generated_uploaded(&shown)
                };
            }
            Err(err) => note = msg::generated_not_added(&shown, err),
        }
    }
    Ok(SshOutcome::Attached { ssh, note })
}

/// Detaches the profile's key: deletes it on the provider when it was
/// registered there, and with `delete_files` unlinks the local pair too.
/// A provider failure aborts before anything local is touched, so the
/// user is never left with a key file whose provider copy is still live
/// and forgotten.
pub(crate) fn remove(
    tokens: &Tokens,
    transport: Option<Arc<dyn HttpTransport>>,
    integration: &Integration,
    delete_files: bool,
) -> Result<SshOutcome, String> {
    let ssh = integration
        .ssh
        .clone()
        .ok_or_else(|| msg::NO_KEY.get().to_string())?;
    let mut deleted_remote = false;
    if let Some(id) = ssh.remote_key_id.as_deref() {
        let secret = token_for(tokens, integration)?;
        let client = client_for(
            integration.clone(),
            secret,
            transport_or_default(transport)?,
            &gitpoulpi_hosting::TrustGate::ambient(),
        )
        .map_err(|e| e.to_string())?;
        block_on(client.delete_ssh_key(id))?.map_err(msg::delete_remote_failed)?;
        deleted_remote = true;
    }
    if delete_files {
        for p in [&ssh.private_key_path, &ssh.public_key_path]
            .into_iter()
            .flatten()
        {
            match std::fs::remove_file(p) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(msg::delete_file_failed(p, e)),
            }
        }
    }
    let note = match (deleted_remote, delete_files) {
        (false, false) => msg::REMOVED.get().to_string(),
        (true, false) => msg::REMOVED_REMOTE.get().to_string(),
        (false, true) => msg::REMOVED_FILES.get().to_string(),
        (true, true) => msg::REMOVED_REMOTE_FILES.get().to_string(),
    };
    Ok(SshOutcome::Detached { note })
}

/// Attaches an existing private key. The public half is `<path>.pub` when
/// it exists next to it.
pub(crate) fn attach_custom(input: &str, home: Option<&Path>) -> Result<IntegrationSsh, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err(msg::CHOOSE_KEY_PATH.get().into());
    }
    let path = match (input.strip_prefix("~/"), home) {
        (Some(rest), Some(h)) => h.join(rest),
        _ => PathBuf::from(input),
    };
    if !path.is_file() {
        return Err(msg::not_a_file(path.display()));
    }
    let mut public = path.clone().into_os_string();
    public.push(".pub");
    let public = PathBuf::from(public);
    Ok(IntegrationSsh {
        mode: SshMode::Custom,
        private_key_path: Some(path.display().to_string()),
        public_key_path: public.is_file().then(|| public.display().to_string()),
        title: None,
        remote_key_id: None,
    })
}

/// `SHA256:...` of the key's public file, read once by the page and cached.
pub(crate) fn fingerprint_of_file(public_path: &str) -> Option<String> {
    std::fs::read_to_string(public_path)
        .ok()
        .and_then(|line| fingerprint_of(&line))
}

#[cfg(test)]
#[cfg(all(feature = "github", feature = "gitlab"))]
mod tests {
    use super::*;
    use gitpoulpi_config::IntegrationKind;
    use gitpoulpi_hosting::{Method, RecordedTransport};
    use gitpoulpi_secrets::InMemoryTokens;

    fn profile() -> (Integration, Tokens) {
        let i = Integration::new(
            "gitlab-self-managed",
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.example.com",
        );
        let tokens: Tokens = Arc::new(InMemoryTokens::default());
        tokens.store(&i.token_ref, "glpat-x").unwrap();
        (i, tokens)
    }

    fn req(i: &Integration, dir: &Path, upload: bool) -> GenerateRequest {
        GenerateRequest {
            integration: i.clone(),
            ssh_dir: dir.to_path_buf(),
            title: "GitPoulpi test".into(),
            upload,
        }
    }

    #[test]
    fn generate_writes_a_per_profile_pair_and_uploads_after_listing() {
        let dir = tempfile::tempdir().unwrap();
        let (i, tokens) = profile();
        let t = Arc::new(RecordedTransport::new().on("/user/keys?", 200, "[]").on(
            "/user/keys",
            201,
            r#"{"id":42,"title":"GitPoulpi test","key":"ssh-ed25519 AAAA"}"#,
        ));
        let out = generate(&tokens, Some(t.clone()), req(&i, dir.path(), true)).unwrap();
        let SshOutcome::Attached { ssh, note } = out else {
            panic!("not attached")
        };
        assert_eq!(ssh.mode, SshMode::Generated);
        assert_eq!(ssh.remote_key_id.as_deref(), Some("42"));
        let private = ssh.private_key_path.unwrap();
        assert!(
            private.ends_with("gitpoulpi_gitlab-self-managed_ed25519"),
            "{private}"
        );
        assert!(Path::new(&private).is_file());
        assert!(Path::new(&ssh.public_key_path.unwrap()).is_file());
        assert!(note.contains("added it to the provider"), "{note}");
        let seen = t.requests();
        assert_eq!(seen[0].method, Method::Get, "list before upload");
        assert!(seen[0].url.contains("/user/keys"));
        assert!(seen.iter().any(|r| r.method == Method::Post));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&private).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn a_key_the_provider_already_has_is_adopted_not_uploaded_twice() {
        let dir = tempfile::tempdir().unwrap();
        let (i, tokens) = profile();
        // The list answer cannot contain the key we are about to generate,
        // so the dedupe match itself is pinned on the helper.
        assert!(same_public_key(
            "ssh-ed25519 AAA me@x",
            "ssh-ed25519 AAA gitpoulpi"
        ));
        let t = Arc::new(RecordedTransport::new().on("/user/keys?", 200, "[]").on(
            "/user/keys",
            201,
            r#"{"id":5,"title":"t","key":"ssh-ed25519 AAAA"}"#,
        ));
        let out = generate(&tokens, Some(t.clone()), req(&i, dir.path(), true)).unwrap();
        assert!(matches!(out, SshOutcome::Attached { .. }));
        let posts = t
            .requests()
            .iter()
            .filter(|r| r.method == Method::Post)
            .count();
        assert_eq!(posts, 1);
    }

    #[test]
    fn a_failed_upload_keeps_the_key_and_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let (i, tokens) = profile();
        let t = Arc::new(RecordedTransport::new().on("/user/keys?", 200, "[]").on(
            "/user/keys",
            403,
            r#"{"message":"no"}"#,
        ));
        let out = generate(&tokens, Some(t), req(&i, dir.path(), true)).unwrap();
        let SshOutcome::Attached { ssh, note } = out else {
            panic!()
        };
        assert_eq!(ssh.remote_key_id, None);
        assert!(note.contains("not added"), "{note}");
        assert!(Path::new(&ssh.private_key_path.unwrap()).is_file());
    }

    #[test]
    fn generate_without_upload_makes_no_request_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let (i, tokens) = profile();
        let t = Arc::new(RecordedTransport::new());
        generate(&tokens, Some(t.clone()), req(&i, dir.path(), false)).unwrap();
        assert!(t.requests().is_empty());
        let again = generate(&tokens, Some(t), req(&i, dir.path(), false)).unwrap_err();
        assert!(again.contains("already exists"), "{again}");
    }

    #[test]
    fn remove_deletes_on_the_provider_and_only_delete_unlinks_files() {
        let dir = tempfile::tempdir().unwrap();
        let (mut i, tokens) = profile();
        let t = Arc::new(RecordedTransport::new().on("/user/keys/42", 204, ""));
        let SshOutcome::Attached { ssh, .. } =
            generate(&tokens, Some(t.clone()), req(&i, dir.path(), false)).unwrap()
        else {
            panic!()
        };
        let private = ssh.private_key_path.clone().unwrap();
        i.ssh = Some(IntegrationSsh {
            remote_key_id: Some("42".into()),
            ..ssh
        });

        let out = remove(&tokens, Some(t.clone()), &i, false).unwrap();
        assert!(matches!(out, SshOutcome::Detached { .. }));
        assert!(t.requests().iter().any(|r| r.method == Method::Delete));
        assert!(Path::new(&private).is_file(), "Remove keeps the files");

        let out = remove(&tokens, Some(t), &i, true).unwrap();
        let SshOutcome::Detached { note } = out else {
            panic!()
        };
        assert!(note.contains("files deleted"));
        assert!(!Path::new(&private).exists());
    }

    #[test]
    fn a_provider_failure_leaves_the_local_files_alone() {
        let dir = tempfile::tempdir().unwrap();
        let (mut i, tokens) = profile();
        let SshOutcome::Attached { ssh, .. } =
            generate(&tokens, None, req(&i, dir.path(), false)).unwrap()
        else {
            panic!()
        };
        let private = ssh.private_key_path.clone().unwrap();
        i.ssh = Some(IntegrationSsh {
            remote_key_id: Some("9".into()),
            ..ssh
        });
        let t = Arc::new(RecordedTransport::new().on("/user/keys/9", 500, "boom"));
        assert!(remove(&tokens, Some(t), &i, true).is_err());
        assert!(Path::new(&private).is_file());
    }

    #[test]
    fn a_custom_key_needs_an_existing_file_and_finds_its_pub() {
        let dir = tempfile::tempdir().unwrap();
        let k = dir.path().join("id_work");
        assert!(attach_custom("", None).is_err());
        assert!(attach_custom(&k.display().to_string(), None).is_err());
        std::fs::write(&k, "x").unwrap();
        let s = attach_custom(&k.display().to_string(), None).unwrap();
        assert_eq!((s.mode, s.public_key_path), (SshMode::Custom, None));
        std::fs::write(dir.path().join("id_work.pub"), "ssh-ed25519 AAA").unwrap();
        assert!(attach_custom(&k.display().to_string(), None)
            .unwrap()
            .public_key_path
            .is_some());
        let s = attach_custom("~/id_work", Some(dir.path())).unwrap();
        assert!(s.private_key_path.unwrap().ends_with("id_work"));
    }
}
