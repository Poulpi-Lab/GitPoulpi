//! `gitpoulpi hosting` -- ssh key.

use gitpoulpi_hosting::ssh_key;
use serde_json::json;

use crate::cli::SshKeyAction;
use crate::repo_cmd::Emitted;

use super::*;

pub(super) async fn run_ssh_key(env: &HostingEnv, action: SshKeyAction) -> Result<Emitted, String> {
    match action {
        SshKeyAction::List { id } => {
            let integrations = env.integrations()?;
            let integration = pick(&integrations, id.as_deref())?;
            let keys = env
                .client(integration, &integrations)?
                .list_ssh_keys()
                .await
                .map_err(|e| e.to_string())?;
            let human = keys
                .iter()
                .map(|k| format!("{:<10} {:<24} {}", k.id, k.title, k.key))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(Emitted {
                exit_code: 0,
                json: serde_json::to_value(&keys)
                    .map_err(|e| format!("failed to encode output: {e}"))?,
                human,
            })
        }

        SshKeyAction::Delete { key_id, id } => {
            let mut integrations = env.integrations()?;
            let integration = pick(&integrations, id.as_deref())?.clone();
            env.client(&integration, &integrations)?
                .delete_ssh_key(&key_id)
                .await
                .map_err(|e| e.to_string())?;
            // A profile remembering this key as its own must stop doing so.
            let mut forgot = false;
            if let Some(ssh) = integrations
                .iter_mut()
                .find(|i| i.id == integration.id)
                .and_then(|i| i.ssh.as_mut())
                .filter(|s| s.remote_key_id.as_deref() == Some(key_id.as_str()))
            {
                ssh.remote_key_id = None;
                forgot = true;
            }
            if forgot {
                env.save_integrations(&integrations)?;
            }
            Ok(Emitted {
                exit_code: 0,
                json: json!({ "id": integration.id, "key_id": key_id, "deleted": true }),
                human: format!("deleted key {key_id} from `{}`", integration.id),
            })
        }

        SshKeyAction::Create {
            id,
            path,
            title,
            no_upload,
        } => {
            // `--no-upload` is resolved before the integration, so
            // generating a key locally works with nothing configured at
            // all.
            let integrations = if no_upload {
                Vec::new()
            } else {
                env.integrations()?
            };
            let integration = if no_upload {
                None
            } else {
                Some(pick(&integrations, id.as_deref())?)
            };

            let path = match path {
                Some(p) => std::path::PathBuf::from(p),
                None => ssh_key::default_key_path().map_err(|e| e.to_string())?,
            };
            let title = title.unwrap_or_else(ssh_key::default_key_title);
            let generated =
                ssh_key::generate_and_write(&path, &title).map_err(|e| e.to_string())?;

            let uploaded = match integration {
                Some(integration) => {
                    let key = env
                        .client(integration, &integrations)?
                        .upload_ssh_key(&title, &generated.public_key)
                        .await
                        .map_err(|e| {
                            // The pair is already on disk here; saying so
                            // turns "it failed" into "it half-worked", and
                            // the next attempt would otherwise hit the
                            // refuse-to-overwrite guard unexplained.
                            format!(
                                "{e}\n\nThe key pair was written to {} and {}; \
                                 upload it by hand or pass --path next time.",
                                generated.private_key_path.display(),
                                generated.public_key_path.display()
                            )
                        })?;
                    Some(key)
                }
                None => None,
            };

            let human = format!(
                "wrote {}\nfingerprint {}\n{}",
                generated.private_key_path.display(),
                generated.fingerprint,
                match &uploaded {
                    Some(key) => format!("uploaded as `{}`", key.title),
                    None => "not uploaded (--no-upload)".to_string(),
                }
            );
            Ok(Emitted {
                exit_code: 0,
                json: json!({
                    "private_key_path": generated.private_key_path,
                    "public_key_path": generated.public_key_path,
                    "public_key": generated.public_key,
                    "fingerprint": generated.fingerprint,
                    "uploaded": uploaded,
                }),
                human,
            })
        }
    }
}
