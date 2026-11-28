//! The Preferences pages' field builders. Every plain `config.toml` row is
//! built once in the pools ([`PreferencesState::row_pool`]: `general_rows`,
//! `ui_rows`, `editor_rows`) and the page tables in `pages.rs` pick them by
//! key, so where a setting lives is one table, not scattered builder code.

use super::*;
use gitpoulpi_i18n::msg;

impl PreferencesState {
    /// Profiles > General: the one setting that is not a profile's own.
    pub(super) fn profile_general_fields(&self) -> Vec<Field> {
        let mut options: Vec<String> =
            vec![msg::settings_profiles::NO_PROFILE_OPTION.get().to_string()];
        options.extend(
            self.config
                .profiles
                .iter()
                .map(|p| format!("{} <{}>", p.name, p.author_email)),
        );
        let active = self
            .config
            .profiles
            .iter()
            .position(|p| p.id == self.config.general.active_profile)
            .map(|i| i + 1)
            .unwrap_or(0);

        vec![Field::global(
            msg::settings_profiles::ACTIVE_PROFILE.get(),
            "general.active_profile",
            Control::Choice {
                options,
                index: active,
            },
        )
        .with_help(msg::settings_profiles::ACTIVE_PROFILE_HELP.get())]
    }

    /// Profiles > the list tab: the rows of the selected profile.
    pub(super) fn profile_item_fields(&self) -> Vec<Field> {
        let mut fields = Vec::new();
        // The list (left) adds and removes; the detail edits its selection.
        if self.config.profiles.is_empty() {
            fields.push(Field::heading(
                msg::settings_profiles::HEADING_NO_PROFILES_YET.get(),
            ));
            return fields;
        }

        let index = self.profile_index.min(self.config.profiles.len() - 1);
        let profile = &self.config.profiles[index];
        fields.push(Field::heading(&msg::settings_profiles::editing_profile(
            index + 1,
            self.config.profiles.len(),
        )));
        fields.push(
            Field::plain(
                msg::settings_profiles::PROFILE_ID.get(),
                Control::Text(profile.id.clone()),
                Target::ProfileField(index, TextSlot::Id),
            )
            .with_help(msg::settings_profiles::PROFILE_ID_HELP.get()),
        );
        fields.push(Field::plain(
            msg::settings_profiles::PROFILE_NAME.get(),
            Control::Text(profile.name.clone()),
            Target::ProfileField(index, TextSlot::Name),
        ));
        fields.push(Field::plain(
            msg::settings_profiles::AUTHOR_NAME.get(),
            Control::Text(profile.author_name.clone()),
            Target::ProfileField(index, TextSlot::AuthorName),
        ));
        fields.push(Field::plain(
            msg::settings_profiles::AUTHOR_EMAIL.get(),
            Control::Text(profile.author_email.clone()),
            Target::ProfileField(index, TextSlot::AuthorEmail),
        ));
        fields
    }

    pub(super) fn ssh_fields(&self) -> Vec<Field> {
        let ssh = &self.config.ssh;
        vec![
            Field::global(
                msg::settings_ssh::PRIVATE_KEY_PATH.get(),
                "ssh.private_key_path",
                Control::Text(ssh.private_key_path.clone().unwrap_or_default()),
            )
            .with_help(msg::settings_ssh::PRIVATE_KEY_PATH_HELP.get()),
            Field::global(
                msg::settings_ssh::PUBLIC_KEY_PATH.get(),
                "ssh.public_key_path",
                Control::Text(ssh.public_key_path.clone().unwrap_or_default()),
            )
            .with_help(msg::settings_ssh::PUBLIC_KEY_PATH_HELP.get()),
            Field::global(
                msg::settings_ssh::KEY_TYPE.get(),
                "ssh.key_type",
                Control::Choice {
                    options: vec!["ed25519".into(), "rsa".into(), "ecdsa".into()],
                    index: match ssh.key_type.as_str() {
                        "rsa" => 1,
                        "ecdsa" => 2,
                        _ => 0,
                    },
                },
            ),
            Field::heading(msg::settings_ssh::HEADING_HTTPS_CREDENTIALS.get()),
            Field::global(
                msg::settings_ssh::HTTPS_MODE.get(),
                "credentials.https_mode",
                Control::Choice {
                    options: gitpoulpi_config::HttpsMode::ALL
                        .iter()
                        .map(|m| m.as_str().into())
                        .collect(),
                    index: gitpoulpi_config::HttpsMode::ALL
                        .iter()
                        .position(|m| *m == self.config.credentials.https_mode)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_ssh::HTTPS_MODE_HELP.get()),
            Field::button(
                msg::settings_general::FORGET_ALL_USERNAMES_AND_PASSWORDS.get(),
                PrefAction::ForgetCredentials,
            )
            .with_help(msg::settings_general::FORGET_ALL_USERNAMES_AND_PASSWORDS_HELP.get()),
            Field::heading(msg::settings_ssh::HEADING_KEYS.get()),
            Field::button(
                msg::settings_ssh::GENERATE_KEY_PAIR.get(),
                PrefAction::GenerateSshKey,
            )
            .with_help(msg::settings_ssh::GENERATE_KEY_PAIR_HELP.get()),
        ]
    }
}
