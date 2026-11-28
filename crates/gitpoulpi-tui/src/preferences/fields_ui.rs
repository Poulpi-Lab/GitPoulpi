//! The Preferences pages' field builders for the `ui.*` rows (a pool, see `pages.rs`) and Git & encoding.

use super::*;
use gitpoulpi_i18n::msg;

/// The encodings the Default encoding dropdowns offer (`encoding_rs` labels).
const ENCODINGS: &[&str] = &[
    "utf-8",
    "windows-1252",
    "iso-8859-1",
    "iso-8859-2",
    "iso-8859-15",
    "windows-1250",
    "windows-1251",
    "koi8-r",
    "shift_jis",
    "euc-jp",
    "gbk",
    "big5",
    "euc-kr",
    "utf-16le",
    "utf-16be",
];

/// [`ENCODINGS`], plus `current` when a hand-edited file holds something
/// else, so the dropdown never hides what is in force.
pub(super) fn encoding_options(current: &str) -> Vec<String> {
    let mut options: Vec<String> = ENCODINGS.iter().map(|e| (*e).to_string()).collect();
    if !current.is_empty() && !options.iter().any(|o| o.eq_ignore_ascii_case(current)) {
        options.push(current.to_string());
    }
    options
}
/// The Language combobox's values and labels (shared with every frontend).
pub(crate) use gitpoulpi_app::settings::{language_choices, language_options};

impl PreferencesState {
    pub(super) fn git_fields(&self) -> Vec<Field> {
        let prefs = &self.repo_state.preferences;
        let git = &self.config.git;
        let mut fields = vec![
            Field::global(
                msg::settings_git::DEFAULT_BACKEND.get(),
                "general.default_backend",
                Control::Choice {
                    options: vec!["git2".into(), "system".into()],
                    index: match self.config.general.default_backend {
                        DefaultBackend::Git2 => 0,
                        DefaultBackend::System => 1,
                    },
                },
            )
            .with_help(msg::settings_git::DEFAULT_BACKEND_HELP.get())
            .with_warning(msg::settings_git::DEFAULT_BACKEND_WARNING.get()),
            Field::global(
                msg::settings_git::PULL_MODE.get(),
                "git.pull_mode",
                Control::Choice {
                    options: PullButtonMode::ALL
                        .iter()
                        .map(|m| m.as_str().into())
                        .collect(),
                    index: PullButtonMode::ALL
                        .iter()
                        .position(|m| *m == git.pull_mode)
                        .unwrap_or(0),
                },
            )
            .with_help(&msg::settings_git::pull_mode_help(crate::keymap::paren(
                &crate::keymap::current(),
                crate::keymap::Action::GitPull,
            ))),
            Field::global(
                msg::settings_git::MERGE_FAST_FORWARD.get(),
                "git.merge_fast_forward",
                Control::Choice {
                    options: MergeFastForward::ALL
                        .iter()
                        .map(|m| m.as_str().into())
                        .collect(),
                    index: MergeFastForward::ALL
                        .iter()
                        .position(|m| *m == git.merge_fast_forward)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_git::MERGE_FAST_FORWARD_HELP.get()),
            Field::global(
                msg::settings_git::DEFAULT_ENCODING.get(),
                "git.default_encoding",
                Control::Choice {
                    options: encoding_options(&git.default_encoding),
                    index: encoding_options(&git.default_encoding)
                        .iter()
                        .position(|o| o.eq_ignore_ascii_case(&git.default_encoding))
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_git::DEFAULT_ENCODING_HELP.get()),
            Field::global(
                msg::settings_git::PUSH_BEHAVIOR.get(),
                "git.push_behavior",
                Control::Choice {
                    options: PushBehavior::ALL
                        .iter()
                        .map(|b| b.as_str().into())
                        .collect(),
                    index: PushBehavior::ALL
                        .iter()
                        .position(|b| *b == git.push_behavior)
                        .unwrap_or(0),
                },
            )
            .with_help(msg::settings_git::PUSH_BEHAVIOR_HELP.get()),
            Field::global(
                msg::settings_git::PUSH_GROUP_PREFLIGHT.get(),
                "git.push_group_preflight",
                Control::Toggle(git.push_group_preflight),
            )
            .with_help(msg::settings_git::PUSH_GROUP_PREFLIGHT_HELP.get()),
            Field::heading(&msg::settings_git::repo_heading(&self.repo_label)),
            Field::repo(
                msg::settings_git::DEFAULT_ENCODING.get(),
                RepoField::Encoding,
                Control::Choice {
                    options: std::iter::once(msg::settings_git::INHERITED.get().to_string())
                        .chain(encoding_options(prefs.encoding.as_deref().unwrap_or("")))
                        .collect(),
                    index: prefs
                        .encoding
                        .as_deref()
                        .and_then(|v| {
                            encoding_options(v)
                                .iter()
                                .position(|o| o.eq_ignore_ascii_case(v))
                        })
                        .map(|i| i + 1)
                        .unwrap_or(0),
                },
            )
            .live()
            .with_help(msg::settings_git::REPO_ENCODING_HELP.get()),
            git_config_field(
                "core.autocrlf",
                RepoField::AutoCrlf,
                &self.repo_config,
                msg::settings_git::CORE_AUTOCRLF_HELP.get(),
            ),
            git_config_field(
                "core.eol",
                RepoField::Eol,
                &self.repo_config,
                msg::settings_git::CORE_EOL_HELP.get(),
            ),
        ];
        // Extra push targets for new branches: a checkbox per remote.
        fields.push(Field::heading(
            msg::settings_git::DEFAULT_PUSH_TARGETS.get(),
        ));
        if self.remote_names.is_empty() {
            fields.push(Field::plain(
                msg::settings_git::DEFAULT_PUSH_TARGETS_NONE.get(),
                Control::Info(String::new()),
                Target::None,
            ));
        }
        for (i, remote) in self.remote_names.iter().enumerate() {
            fields.push(
                Field::repo(
                    remote,
                    RepoField::PushTarget(i),
                    Control::Toggle(prefs.default_push_targets.contains(remote)),
                )
                .live()
                .with_help(msg::settings_git::DEFAULT_PUSH_TARGETS_HELP.get()),
            );
        }
        // The LFS rows exist only in a build that has LFS.
        if crate::build_features::LFS {
            fields.extend([
                Field::repo(
                    msg::settings_git::GIT_LFS.get(),
                    RepoField::Lfs,
                    Control::Toggle(prefs.lfs_enabled.unwrap_or(git.lfs_enabled)),
                )
                .live()
                .with_help(msg::settings_git::GIT_LFS_HELP.get()),
                Field::global(
                    msg::settings_git::LFS_AUTOFIX_FILTERS.get(),
                    "git.lfs_autofix_filters",
                    Control::Toggle(git.lfs_autofix_filters),
                )
                .with_help(msg::settings_git::LFS_AUTOFIX_FILTERS_HELP.get()),
                Field::plain(
                    msg::settings_git::GIT_LFS_PROGRAM.get(),
                    Control::Info(
                        self.lfs_program.clone().unwrap_or_else(|| {
                            msg::settings_git::LFS_NOT_CHECKED.get().to_string()
                        }),
                    ),
                    Target::None,
                )
                .with_help(msg::settings_git::GIT_LFS_PROGRAM_HELP.get()),
                Field::button(
                    msg::settings_git::INITIALIZE_LFS_FOR_THIS_REPOSITORY.get(),
                    PrefAction::InitializeLfs,
                )
                .with_help(msg::settings_git::INITIALIZE_LFS_FOR_THIS_REPOSITORY_HELP.get()),
            ]);
        }
        fields
    }
}

/// The values `key` accepts (after "(inherited)", which clears it).
pub(super) fn git_config_values(key: &str) -> &'static [&'static str] {
    gitpoulpi_core::repo_config::REPO_CONFIG_KEYS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or(&[][..], |(_, v)| *v)
}

/// A Git & Encoding row for one of git's own repository-local settings:
/// the shown value is what git reports (`values`), not what a file remembers.
fn git_config_field(
    key: &'static str,
    field: RepoField,
    values: &std::collections::BTreeMap<String, Option<String>>,
    help: &str,
) -> Field {
    let accepted = git_config_values(key);
    let current = values.get(key).and_then(|v| v.as_deref());
    Field::repo(
        key,
        field,
        Control::Choice {
            options: std::iter::once(msg::settings_git::INHERITED.get().to_string())
                .chain(accepted.iter().map(|v| (*v).to_string()))
                .collect(),
            index: current
                .and_then(|v| accepted.iter().position(|a| *a == v))
                .map_or(0, |i| i + 1),
        },
    )
    .live()
    .with_help(help)
}
