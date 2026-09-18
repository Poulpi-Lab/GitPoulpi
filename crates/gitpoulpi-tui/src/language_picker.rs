//! `App: Change Language...` (`Ctrl+N g`): the palette's second-level list of
//! the languages the Settings > General > Language combo offers (`Auto`, then
//! every language built into this binary, endonyms as labels). Typing narrows
//! it, Enter applies the highlighted language live and writes `ui.language`,
//! Esc restores the previous language. Moving the highlight previews the
//! language live: the whole app behind the popup repaints in it.
//!
//! The same popup and model also serve `App: Change Graph Color Theme...`
//! ([`PickerKind::GraphColorTheme`]): the lane palettes the Settings > Log &
//! graph combo offers (`Follow theme`, then every palette), Enter writes
//! `ui.graph_color_theme`, and moving the highlight previews the palette on the
//! Log's graph (Esc puts the previous one back). The mode, the keys and the painting are shared, so
//! a new picker of this shape needs no new `InputMode`.
//!
//! [`PickerKind::RuleFile`] is the third use: "which `.gitignore` /
//! `.gitattributes` does this rule go into?" (the root file, the file of each
//! folder down to the path's, the per-clone `.git/info/*` file). Enter runs
//! the job (`rule_file_action`) instead of writing a setting, and there is no
//! live preview to restore.

use gitpoulpi_config::{Config, ConfigPaths, UiLanguage};
use gitpoulpi_i18n::msg;

/// Which setting the picker edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PickerKind {
    Language,
    GraphColorTheme,
    /// Which rule file an ignore / treat-as rule is written to.
    RuleFile,
}

impl PickerKind {
    /// The `config.toml` key Enter writes.
    fn key(self) -> &'static str {
        match self {
            PickerKind::Language => "ui.language",
            PickerKind::GraphColorTheme => "ui.graph_color_theme",
            PickerKind::RuleFile => "",
        }
    }
}

/// What a [`PickerKind::RuleFile`] picker will write once a file is chosen.
/// Takes a list of paths so a multi-selection can reuse it; the candidate
/// files are those of the first path.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuleFileJob {
    pub(crate) kind: gitpoulpi_core::domain::RuleKind,
    /// Repo-relative, `/` separated.
    pub(crate) paths: Vec<String>,
    pub(crate) scope: gitpoulpi_core::domain::RuleScope,
    /// `Attributes` only: what to treat the paths as.
    pub(crate) treat: Option<gitpoulpi_core::domain::Treat>,
}

/// The one-line note after a candidate file's name: where it sits, and
/// whether confirming creates it.
fn rule_file_note(file: &str, new: bool) -> Option<String> {
    use msg::status_pane as m;
    let local = file.starts_with(".git/");
    let root = !local && !file.contains('/');
    let note = match (local, root, new) {
        (true, _, false) => m::RF_NOTE_LOCAL.get(),
        (true, _, true) => m::RF_NOTE_LOCAL_NEW.get(),
        (_, true, false) => m::RF_NOTE_ROOT.get(),
        (_, true, true) => m::RF_NOTE_ROOT_NEW.get(),
        (_, _, true) => m::RF_NOTE_NEW.get(),
        _ => return None,
    };
    Some(note.to_string())
}

/// The rows of a rule-file picker: root first (the default), each folder's
/// file down to the path's own, the per-clone file last. A missing file is
/// marked "new" (confirming creates it); the line each one would receive is
/// shown after its name.
pub(crate) fn rule_file_choices(
    job: &RuleFileJob,
    repo_root: &std::path::Path,
) -> Vec<LanguageChoice> {
    use gitpoulpi_core::domain::{candidate_files, rule_pattern};
    let first = job.paths.first().map(String::as_str).unwrap_or_default();
    candidate_files(job.kind, first)
        .into_iter()
        .filter_map(|file| {
            let pattern = rule_pattern(job.kind, &file, first, job.scope)?;
            let exists = if file.starts_with(".git/") {
                // Only a plain `.git` folder is looked into; a linked
                // worktree's `.git` is a file and git decides there.
                let git = repo_root.join(".git");
                git.is_dir()
                    .then(|| git.join(file.trim_start_matches(".git/")).is_file())
            } else {
                Some(repo_root.join(&file).is_file())
            };
            let note = rule_file_note(&file, exists == Some(false));
            let label = match note {
                Some(note) => msg::status_pane::rf_row(&file, note, &pattern),
                None => msg::status_pane::rf_row_plain(&file, &pattern),
            };
            Some(LanguageChoice {
                label,
                value: file,
                chips: Vec::new(),
            })
        })
        .collect()
}

/// One row of the list.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LanguageChoice {
    pub(crate) label: String,
    /// The stored value (`ui.language` / `ui.graph_color_theme`).
    pub(crate) value: String,
    /// Lane colors shown as chips (graph color themes only).
    pub(crate) chips: Vec<ratatui::style::Color>,
}

/// The language list's content: exactly the Settings combo's, same order and labels.
pub(crate) fn choices(current: gitpoulpi_config::UiLanguage) -> Vec<LanguageChoice> {
    let values = crate::preferences::language_choices(current);
    let labels = crate::preferences::language_options(current);
    values
        .into_iter()
        .zip(labels)
        .map(|(value, label)| LanguageChoice {
            label,
            value: value.as_str().to_string(),
            chips: Vec::new(),
        })
        .collect()
}

/// The lane palette list: `Follow theme`, then every built-in and user-theme
/// palette (the Settings combo's options), each with its first colors.
pub(crate) fn graph_color_choices(
    user: &[gitpoulpi_config::theme::ThemeSpec],
) -> Vec<LanguageChoice> {
    let mut specs = gitpoulpi_config::theme::presets();
    specs.extend(user.iter().cloned());
    let mut out = vec![LanguageChoice {
        label: msg::settings_ui::FOLLOW_THEME.get().to_string(),
        value: gitpoulpi_config::GRAPH_COLOR_FOLLOW_THEME.to_string(),
        chips: Vec::new(),
    }];
    out.extend(
        gitpoulpi_config::theme::lane_palettes_with(&specs)
            .into_iter()
            .map(|p| LanguageChoice {
                label: p.label,
                value: p.slug,
                chips: p
                    .colors
                    .into_iter()
                    .take(4)
                    .map(crate::theme::color_of)
                    .collect(),
            }),
    );
    out
}

/// The open picker: what `InputMode::LanguagePicker` carries.
#[derive(Debug)]
pub(crate) struct LanguagePicker {
    pub(crate) draft: String,
    /// Index into the *filtered* list.
    pub(crate) selected: usize,
    pub(crate) kind: PickerKind,
    /// The stored value when the picker opened (ticked).
    pub(crate) current: String,
    /// The value installed right now, preview included.
    shown: String,
    /// The `[ui]` table when the picker opened (a language preview swaps
    /// only `language` in it).
    base_ui: gitpoulpi_config::UiConfig,
    all: Vec<LanguageChoice>,
    config_path: Option<std::path::PathBuf>,
    /// `RuleFile` only: what Enter writes.
    pub(crate) job: Option<RuleFileJob>,
}

impl LanguagePicker {
    /// The "which file?" picker of an ignore / treat-as rule. Opens on the
    /// repository's root file, the default.
    pub(crate) fn open_rule_file(job: RuleFileJob, repo_root: &std::path::Path) -> Self {
        let all = rule_file_choices(&job, repo_root);
        let current = all.first().map(|c| c.value.clone()).unwrap_or_default();
        Self {
            draft: String::new(),
            selected: 0,
            kind: PickerKind::RuleFile,
            shown: current.clone(),
            base_ui: Default::default(),
            current,
            all,
            config_path: None,
            job: Some(job),
        }
    }

    /// Enter on a rule-file picker: the daemon action that writes the rule
    /// into the highlighted file, and the status line saying what and where.
    /// `None`: nothing matches the filter, or the pattern cannot be named
    /// from that file.
    pub(crate) fn rule_file_action(&self) -> Option<(crate::daemon_link::Action, String)> {
        use crate::daemon_link::Action;
        use gitpoulpi_core::domain::{rule_pattern, RuleKind, Treat};
        let job = self.job.as_ref()?;
        let file = self.highlighted()?.value.clone();
        let first = job.paths.first()?;
        let pattern = rule_pattern(job.kind, &file, first, job.scope)?;
        let default = job.kind.file_name();
        let arg = (file != default).then(|| file.clone());
        Some(match job.kind {
            RuleKind::Ignore => (
                Action::AddToGitignore {
                    pattern: pattern.clone(),
                    file: arg,
                },
                msg::status_list::ignoring_in(&pattern, &file),
            ),
            RuleKind::Attributes => {
                let treat = job.treat?;
                let text = match treat {
                    Treat::Binary => msg::status_list::treating_binary(first, &pattern, &file),
                    Treat::Text => msg::status_list::treating_text(first, &pattern, &file),
                };
                (
                    Action::TreatAs {
                        paths: job.paths.clone(),
                        scope: job.scope,
                        treat,
                        file: arg,
                    },
                    text,
                )
            }
        })
    }

    /// A selection's ignore rule writes one line per file (or extension): the
    /// actions for the paths after the first, with the line already written
    /// and repeats left out, and the file they go into. Empty for a single
    /// path and for the attribute rules (one `TreatAs` holds every path).
    pub(crate) fn rule_file_extra_actions(&self) -> (Vec<crate::daemon_link::Action>, String) {
        use crate::daemon_link::Action;
        use gitpoulpi_core::domain::{rule_pattern, RuleKind};
        let (Some(job), Some(file)) = (
            self.job.as_ref(),
            self.highlighted().map(|c| c.value.clone()),
        ) else {
            return (Vec::new(), String::new());
        };
        if job.kind != RuleKind::Ignore {
            return (Vec::new(), file);
        }
        let default = job.kind.file_name();
        let arg = (file != default).then(|| file.clone());
        let mut seen: Vec<String> = job
            .paths
            .first()
            .and_then(|p| rule_pattern(job.kind, &file, p, job.scope))
            .into_iter()
            .collect();
        let mut out = Vec::new();
        for path in job.paths.iter().skip(1) {
            let Some(pattern) = rule_pattern(job.kind, &file, path, job.scope) else {
                continue;
            };
            if seen.contains(&pattern) {
                continue;
            }
            seen.push(pattern.clone());
            out.push(Action::AddToGitignore {
                pattern,
                file: arg.clone(),
            });
        }
        (out, file)
    }

    pub(crate) fn open() -> Self {
        let config_path = ConfigPaths::discover().map(|p| p.config_file());
        let ui = config_path
            .as_deref()
            .and_then(|p| Config::load(p).ok())
            .map(|c| c.ui)
            .unwrap_or_default();
        let mut picker = Self::with_parts(ui.language, config_path);
        picker.base_ui = ui;
        picker
    }

    /// `App: Change Graph Color Theme...`, against the real config.
    pub(crate) fn open_graph_color_theme() -> Self {
        let paths = ConfigPaths::discover();
        let config_path = paths.as_ref().map(|p| p.config_file());
        let current = config_path
            .as_deref()
            .and_then(|p| Config::load(p).ok())
            .map(|c| c.ui.graph_color_slug().to_string())
            .unwrap_or_else(|| gitpoulpi_config::GRAPH_COLOR_FOLLOW_THEME.to_string());
        let user = paths
            .map(|p| gitpoulpi_config::theme::load_user_themes(&p.themes_dir()).themes)
            .unwrap_or_default();
        Self::graph_color_with_parts(current, &user, config_path)
    }

    pub(crate) fn graph_color_with_parts(
        current: String,
        user: &[gitpoulpi_config::theme::ThemeSpec],
        config_path: Option<std::path::PathBuf>,
    ) -> Self {
        let all = graph_color_choices(user);
        let selected = all.iter().position(|c| c.value == current).unwrap_or(0);
        Self {
            draft: String::new(),
            selected,
            kind: PickerKind::GraphColorTheme,
            shown: current.clone(),
            base_ui: Default::default(),
            current,
            all,
            config_path,
            job: None,
        }
    }

    pub(crate) fn with_parts(
        current: gitpoulpi_config::UiLanguage,
        config_path: Option<std::path::PathBuf>,
    ) -> Self {
        let all = choices(current);
        let current = current.as_str().to_string();
        let selected = all.iter().position(|c| c.value == current).unwrap_or(0);
        Self {
            draft: String::new(),
            selected,
            kind: PickerKind::Language,
            shown: current.clone(),
            base_ui: gitpoulpi_config::UiConfig {
                language: UiLanguage::ALL
                    .iter()
                    .copied()
                    .find(|l| l.as_str() == current)
                    .unwrap_or_default(),
                ..Default::default()
            },
            current,
            all,
            config_path,
            job: None,
        }
    }

    pub(crate) fn filtered(&self) -> Vec<&LanguageChoice> {
        let query = self.draft.trim().to_lowercase();
        let mut scored: Vec<(i32, usize, &LanguageChoice)> = self
            .all
            .iter()
            .enumerate()
            .filter_map(|(i, c)| crate::palette::score(&c.label, &query).map(|s| (s, i, c)))
            .collect();
        scored.sort_by_key(|(s, i, _)| (-*s, *i));
        scored.into_iter().map(|(_, _, c)| c).collect()
    }

    pub(crate) fn highlighted(&self) -> Option<&LanguageChoice> {
        self.filtered().get(self.selected).copied()
    }

    /// Puts `value` on screen: the language as the app's language, a graph
    /// palette on `lane`. A no-op while it is the one shown.
    fn show(&mut self, value: &str, lane: &mut crate::graph::LaneTheme) {
        if value == self.shown {
            return;
        }
        match self.kind {
            PickerKind::Language => {
                let mut ui = self.base_ui.clone();
                ui.language = UiLanguage::ALL
                    .iter()
                    .copied()
                    .find(|l| l.as_str() == value)
                    .unwrap_or_default();
                crate::live_settings::install_language(&ui);
            }
            PickerKind::GraphColorTheme => *lane = crate::graph::LaneTheme::from_slug(value),
            PickerKind::RuleFile => {}
        }
        self.shown = value.to_string();
    }

    /// The live preview: installs the highlighted row.
    pub(crate) fn sync_preview(&mut self, lane: &mut crate::graph::LaneTheme) {
        if let Some(value) = self.highlighted().map(|c| c.value.clone()) {
            self.show(&value, lane);
        }
    }

    /// Esc, a click outside: the previous value comes back.
    pub(crate) fn restore(&mut self, lane: &mut crate::graph::LaneTheme) {
        let previous = self.current.clone();
        self.show(&previous, lane);
    }

    /// Enter: writes the key and applies it live; the status line message
    /// says what happened. A graph color theme is applied by the caller
    /// (`applied_lane_theme`), which owns the window's `LaneTheme`.
    pub(crate) fn commit(&mut self) -> String {
        let Some(choice) = self.highlighted().cloned() else {
            return match self.kind {
                PickerKind::Language => msg::input::NO_MATCHING_LANGUAGE.get().to_string(),
                PickerKind::GraphColorTheme => msg::overlays::NO_MATCHING_GRAPH_COLOR_THEME
                    .get()
                    .to_string(),
                PickerKind::RuleFile => msg::overlays::NO_MATCHING_RULE_FILE.get().to_string(),
            };
        };
        let Some(path) = self.config_path.clone() else {
            self.restore(&mut crate::graph::LaneTheme::from_slug(""));
            return msg::settings_screen::NO_CONFIG_DIR.get().to_string();
        };
        let key = self.kind.key();
        match gitpoulpi_config::set_in_file(&path, key, &choice.value) {
            Ok(_) => {
                if self.kind == PickerKind::Language {
                    let ui = Config::load(&path).map(|c| c.ui).unwrap_or_default();
                    crate::live_settings::install_language(&ui);
                }
                self.current = choice.value.clone();
                self.shown = choice.value.clone();
                format!("{key} = {}", choice.value)
            }
            Err(err) => {
                self.restore(&mut crate::graph::LaneTheme::from_slug(""));
                format!("{key}: {err}")
            }
        }
    }
}

/// `App: Toggle Graph Row Tints`: flips `ui.graph_lane_background` between
/// `off` and `row_tint` in the file and in the running window. Returns the
/// status line message.
pub(crate) fn toggle_row_tints(live: &mut crate::graph::LaneBackground) -> String {
    use gitpoulpi_config::GraphLaneBackground as B;
    let Some(path) = ConfigPaths::discover().map(|p| p.config_file()) else {
        return msg::settings_screen::NO_CONFIG_DIR.get().to_string();
    };
    let now = Config::load(&path)
        .map(|c| c.ui.graph_lane_background)
        .unwrap_or_default();
    let next = if now == B::Off { B::RowTint } else { B::Off };
    match gitpoulpi_config::set_in_file(&path, "ui.graph_lane_background", next.as_str()) {
        Ok(_) => {
            *live = next.into();
            format!("ui.graph_lane_background = {}", next.as_str())
        }
        Err(err) => format!("ui.graph_lane_background: {err}"),
    }
}

/// Paints the picker (see [`crate::picker_view`]).
pub(crate) fn render(
    frame: &mut ratatui::Frame,
    picker: &LanguagePicker,
    field_editor: &crate::text_edit::FieldEditor<crate::modes::MainField>,
) -> crate::chrome::PalettePopup {
    use crate::picker_view::{render, PickerRow, PickerView};
    let found = picker.filtered();
    let rows: Vec<PickerRow> = found
        .iter()
        .map(|c| PickerRow {
            label: &c.label,
            ticked: c.value == picker.current,
            chips: &c.chips,
        })
        .collect();
    let (title, empty) = match picker.kind {
        PickerKind::Language => (
            msg::overlays::change_language_title(found.len(), picker.all.len()),
            msg::overlays::NO_MATCHING_LANGUAGE.get(),
        ),
        PickerKind::GraphColorTheme => (
            msg::overlays::change_graph_color_theme_title(found.len(), picker.all.len()),
            msg::overlays::NO_MATCHING_GRAPH_COLOR_THEME.get(),
        ),
        PickerKind::RuleFile => {
            use gitpoulpi_core::domain::Treat;
            let (shown, total) = (found.len(), picker.all.len());
            let title = match picker.job.as_ref().and_then(|j| j.treat) {
                None => msg::overlays::rule_file_title_ignore(shown, total),
                Some(Treat::Binary) => msg::overlays::rule_file_title_binary(shown, total),
                Some(Treat::Text) => msg::overlays::rule_file_title_text(shown, total),
            };
            (title, msg::overlays::NO_MATCHING_RULE_FILE.get())
        }
    };
    let view = PickerView {
        field: crate::modes::MainField::LanguagePicker,
        title,
        draft: &picker.draft,
        selected: picker.selected,
        empty,
        hint: match picker.kind {
            PickerKind::RuleFile => msg::overlays::RULE_FILE_PICKER_HINT.get(),
            _ => msg::overlays::LANGUAGE_PICKER_HINT.get(),
        },
        segments: None,
    };
    render(frame, &view, &rows, field_editor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::UiLanguage;

    #[test]
    fn the_list_is_the_settings_combos_with_auto_first() {
        let list = choices(UiLanguage::Auto);
        let combo = crate::preferences::language_options(UiLanguage::Auto);
        assert_eq!(
            list.iter().map(|c| c.label.clone()).collect::<Vec<_>>(),
            combo
        );
        assert_eq!(list[0].value, "auto");
        assert!(list.iter().any(|c| c.value == "en"));
    }

    #[test]
    fn it_opens_on_the_current_language_and_narrows_by_typing() {
        let mut picker = LanguagePicker::with_parts(UiLanguage::En, None);
        assert_eq!(picker.highlighted().unwrap().value, "en");
        picker.draft = "english".into();
        picker.selected = 0;
        let rows = picker.filtered();
        assert_eq!(rows.first().unwrap().value, "en");
    }

    fn ignore_job(path: &str, scope: gitpoulpi_core::domain::RuleScope) -> RuleFileJob {
        RuleFileJob {
            kind: gitpoulpi_core::domain::RuleKind::Ignore,
            paths: vec![path.into()],
            scope,
            treat: None,
        }
    }

    #[test]
    fn the_rule_file_picker_lists_root_first_marks_new_files_and_shows_each_line() {
        use gitpoulpi_core::domain::RuleScope;
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git/info")).unwrap();
        std::fs::write(dir.path().join(".git/info/exclude"), "# x\n").unwrap();
        std::fs::create_dir_all(dir.path().join("src/a")).unwrap();
        std::fs::write(dir.path().join(".gitignore"), "target\n").unwrap();
        let _guard = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::En);
        let picker =
            LanguagePicker::open_rule_file(ignore_job("src/a/b.txt", RuleScope::File), dir.path());
        let rows: Vec<(String, String)> = picker
            .filtered()
            .iter()
            .map(|c| (c.value.clone(), c.label.clone()))
            .collect();
        assert_eq!(
            rows,
            [
                (
                    ".gitignore".to_string(),
                    ".gitignore (repository root)   /src/a/b.txt".to_string()
                ),
                (
                    "src/.gitignore".to_string(),
                    "src/.gitignore (new)   /a/b.txt".to_string()
                ),
                (
                    "src/a/.gitignore".to_string(),
                    "src/a/.gitignore (new)   /b.txt".to_string()
                ),
                (
                    ".git/info/exclude".to_string(),
                    ".git/info/exclude (this clone only, not committed)   /src/a/b.txt".to_string()
                ),
            ]
        );
        // Opens on the root file, the default.
        assert_eq!(picker.highlighted().unwrap().value, ".gitignore");
        assert_eq!(picker.current, ".gitignore");
    }

    #[test]
    fn enter_on_a_rule_file_row_writes_that_files_relative_line() {
        use crate::daemon_link::Action;
        use gitpoulpi_core::domain::RuleScope;
        let dir = tempfile::tempdir().unwrap();
        let _guard = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::En);
        let mut picker =
            LanguagePicker::open_rule_file(ignore_job("src/a/b.txt", RuleScope::File), dir.path());
        let (action, message) = picker.rule_file_action().unwrap();
        assert!(matches!(
            &action,
            Action::AddToGitignore { pattern, file: None } if pattern == "/src/a/b.txt"
        ));
        assert_eq!(message, "ignoring /src/a/b.txt in .gitignore...");
        picker.selected = 1;
        let (action, _) = picker.rule_file_action().unwrap();
        assert!(matches!(
            &action,
            Action::AddToGitignore { pattern, file: Some(f) }
                if pattern == "/a/b.txt" && f == "src/.gitignore"
        ));
        // Typing narrows to the per-clone file.
        picker.draft = "exclude".into();
        picker.selected = 0;
        let (action, _) = picker.rule_file_action().unwrap();
        assert!(matches!(
            &action,
            Action::AddToGitignore { file: Some(f), .. } if f == ".git/info/exclude"
        ));
        picker.draft = "zzz".into();
        assert!(picker.rule_file_action().is_none());
    }

    #[test]
    fn a_treat_job_becomes_a_treat_action_with_the_files_extension_rule() {
        use crate::daemon_link::Action;
        use gitpoulpi_core::domain::{RuleKind, RuleScope, Treat};
        let dir = tempfile::tempdir().unwrap();
        let _guard = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::En);
        let job = RuleFileJob {
            kind: RuleKind::Attributes,
            paths: vec!["data/x.dat".into()],
            scope: RuleScope::Extension,
            treat: Some(Treat::Binary),
        };
        let mut picker = LanguagePicker::open_rule_file(job, dir.path());
        picker.selected = 1;
        let (action, message) = picker.rule_file_action().unwrap();
        assert!(matches!(
            &action,
            Action::TreatAs { paths, scope: RuleScope::Extension, treat: Treat::Binary, file: Some(f) }
                if paths == &["data/x.dat".to_string()] && f == "data/.gitattributes"
        ));
        assert_eq!(
            message,
            "treating data/x.dat as binary: writing *.dat to data/.gitattributes..."
        );
    }

    #[test]
    fn commit_writes_ui_language_and_without_a_config_dir_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let _guard = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::En);
        let mut picker = LanguagePicker::with_parts(UiLanguage::Auto, Some(path.clone()));
        picker.draft = "english".into();
        picker.selected = 0;
        assert_eq!(picker.commit(), "ui.language = en");
        assert!(std::fs::read_to_string(&path).unwrap().contains("\"en\""));
        let mut none = LanguagePicker::with_parts(UiLanguage::Auto, None);
        none.draft = "english".into();
        assert_ne!(none.commit(), "ui.language = en");
    }
}
