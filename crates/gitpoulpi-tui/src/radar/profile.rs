//! The Profile filter: which integration profiles (Settings > Integrations)
//! can contribute rows to the Radar, and how the combobox names them.

use std::path::Path;

/// The combobox value for "every profile".
pub(crate) const ALL: &str = "";

/// A profile the Profile combobox offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProfileChoice {
    pub(crate) id: String,
    pub(crate) label: String,
}

/// The enabled profiles of `integrations` that can list requests or issues,
/// named as Settings names them; two profiles with one name get their
/// provider appended (`work (GitHub)`, `work (GitLab)`).
pub(crate) fn choices_of(integrations: &[gitpoulpi_config::Integration]) -> Vec<ProfileChoice> {
    let usable: Vec<&gitpoulpi_config::Integration> = integrations
        .iter()
        .filter(|i| {
            let caps = i.kind.capabilities();
            i.enabled
                && caps.implemented
                && crate::build_features::kind_built(i.kind)
                && (caps.pull_requests || caps.issues)
        })
        .collect();
    usable
        .iter()
        .map(|i| {
            let name = i.display_name();
            let clash = usable
                .iter()
                .filter(|o| o.display_name().eq_ignore_ascii_case(name))
                .count()
                > 1;
            ProfileChoice {
                id: i.id.clone(),
                label: if clash {
                    format!("{name} ({})", crate::hosting_ui::provider_name(i.kind))
                } else {
                    name.to_string()
                },
            }
        })
        .collect()
}

/// [`choices_of`] over the profiles of `<config dir>/config.toml`.
pub(crate) fn choices_in(config_dir: &Path) -> Vec<ProfileChoice> {
    gitpoulpi_config::Config::load(&config_dir.join("config.toml"))
        .map(|c| choices_of(&c.integrations))
        .unwrap_or_default()
}

/// `saved` when it still names a profile of `choices`, else [`ALL`].
pub(crate) fn valid_or_all(choices: &[ProfileChoice], saved: Option<&str>) -> String {
    match saved {
        Some(id) if choices.iter().any(|c| c.id == id) => id.to_string(),
        _ => ALL.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::{Integration, IntegrationKind};

    fn profile(id: &str, kind: IntegrationKind, name: &str) -> Integration {
        let mut i = Integration::new(id, kind, "https://example.com");
        i.name = name.into();
        i
    }

    #[test]
    fn a_profile_is_named_as_settings_names_it() {
        let list = [
            profile("p1", IntegrationKind::Github, "personal"),
            profile("p2", IntegrationKind::GitlabSelfManaged, "work"),
        ];
        let labels: Vec<String> = choices_of(&list).into_iter().map(|c| c.label).collect();
        assert_eq!(labels, vec!["personal", "work"]);
    }

    #[test]
    fn two_profiles_with_one_name_get_their_provider() {
        let list = [
            profile("p1", IntegrationKind::Github, "work"),
            profile("p2", IntegrationKind::Gitlab, "Work"),
        ];
        let labels: Vec<String> = choices_of(&list).into_iter().map(|c| c.label).collect();
        assert_eq!(labels, vec!["work (GitHub)", "Work (GitLab)"]);
    }

    #[test]
    fn a_disabled_profile_is_not_offered_and_a_removed_one_falls_back_to_all() {
        let mut off = profile("p2", IntegrationKind::Github, "off");
        off.enabled = false;
        let list = [profile("p1", IntegrationKind::Github, "on"), off];
        let choices = choices_of(&list);
        assert_eq!(choices.len(), 1);
        assert_eq!(valid_or_all(&choices, Some("p1")), "p1");
        assert_eq!(valid_or_all(&choices, Some("p2")), ALL);
        assert_eq!(valid_or_all(&choices, None), ALL);
    }
}
