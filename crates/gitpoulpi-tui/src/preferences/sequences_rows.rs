//! The detail rows of the Sequences page: the selected sequence's, and the
//! selected step's on the steps face (`sequences_page` holds the state and
//! the writes).

use super::sequences_page::{
    option_label, pause_label, pause_modes, scope_label, SeqButton, SeqTarget, SCOPES,
};
use super::sequences_picker::step_label;
use super::*;
use gitpoulpi_config::sequences::{self as seqs, OptionKind, Scope, SequenceDef};
use gitpoulpi_i18n::msg::settings_sequences as t;

impl PreferencesState {
    pub(super) fn sequences_fields(&self) -> Vec<Field> {
        let Some(cur) = self.seq_cur() else {
            return vec![Field::plain(
                t::NO_SEQUENCES.get(),
                Control::Info(String::new()),
                Target::None,
            )];
        };
        if self.seq.in_steps {
            self.seq_step_fields(&cur)
        } else {
            self.seq_sequence_fields(&cur)
        }
    }

    fn seq_sequence_fields(&self, cur: &SequenceDef) -> Vec<Field> {
        let entries = self.seq_entries();
        let at = self.seq.index.min(entries.len().saturating_sub(1));
        let mut fields = vec![Field::heading(&t::editing(at + 1, entries.len()))];
        let seq = |target| Target::Sequence(target);
        fields.push(
            Field::plain(
                t::NAME.get(),
                Control::Text(cur.name.clone()),
                seq(SeqTarget::Name),
            )
            .with_help(t::NAME_HELP.get()),
        );
        fields.push(
            Field::plain(
                t::ID.get(),
                Control::Text(cur.id.clone()),
                seq(SeqTarget::Id),
            )
            .with_help(t::ID_HELP.get()),
        );
        fields.push(
            Field::plain(
                t::DESCRIPTION.get(),
                Control::Text(cur.description.clone().unwrap_or_default()),
                seq(SeqTarget::Description),
            )
            .with_help(t::DESCRIPTION_HELP.get()),
        );
        fields.push(
            Field::plain(
                t::SCOPE.get(),
                Control::Choice {
                    options: SCOPES.iter().map(|s| scope_label(*s).to_string()).collect(),
                    index: SCOPES.iter().position(|s| *s == cur.scope).unwrap_or(0),
                },
                seq(SeqTarget::Scope),
            )
            .with_help(t::SCOPE_HELP.get()),
        );
        // The shortcut: the button opens the capture popup of the Keyboard
        // shortcuts page.
        let chord = self.seq_chord(&cur.id);
        // A repository sequence has a shortcut once the repository's file is
        // trusted (only a trusted file is in the registry).
        let repo =
            cur.scope == Scope::Repo && crate::sequence_run::registry::slot_of(&cur.id).is_none();
        let mut buttons = vec![Btn {
            enabled: !repo,
            ..Btn::new(
                if chord.is_empty() {
                    t::SHORTCUT_NONE.get()
                } else {
                    &chord
                },
                PrefAction::Sequence(SeqButton::Shortcut),
            )
        }];
        if !chord.is_empty() {
            buttons.push(Btn::new(
                t::SHORTCUT_CLEAR.get(),
                PrefAction::Sequence(SeqButton::ClearShortcut),
            ));
        }
        fields.push(
            Field::buttons(t::SHORTCUT.get(), buttons).with_help(if repo {
                t::SHORTCUT_REPO_UNTRUSTED_HELP.get()
            } else {
                t::SHORTCUT_HELP.get()
            }),
        );
        let summary = cur
            .steps
            .iter()
            .map(|s| step_label(&s.run))
            .collect::<Vec<_>>()
            .join(" > ");
        fields.push(Field::plain(
            t::STEPS_ROW.get(),
            Control::Info(if summary.is_empty() {
                t::STEPS_NONE.get().to_string()
            } else {
                summary
            }),
            Target::None,
        ));
        let mut run = vec![
            Btn::new(
                &t::steps_edit(cur.steps.len()),
                PrefAction::Sequence(SeqButton::EditSteps),
            ),
            Btn::new(t::RUN_NOW.get(), PrefAction::Sequence(SeqButton::RunNow)),
        ];
        if self.seq_ai_offered() {
            run.push(Btn::new(
                t::EDIT_AI.get(),
                PrefAction::Sequence(SeqButton::EditWithAi),
            ));
        }
        fields.push(Field::buttons("", run).with_help(t::RUN_NOW_HELP.get()));
        fields.extend(self.seq_problem_rows(cur, None));
        fields
    }

    /// The red rows for a sequence's problems (`only_step`: just that step's).
    fn seq_problem_rows(&self, cur: &SequenceDef, only_step: Option<usize>) -> Vec<Field> {
        let problems: Vec<_> = seqs::validate(cur)
            .into_iter()
            .filter(|p| only_step.is_none_or(|i| p.step == Some(i)))
            .collect();
        if problems.is_empty() {
            return Vec::new();
        }
        let mut rows = vec![Field::heading(t::PROBLEMS.get())];
        for p in problems {
            let text = match p.step {
                Some(i) => t::step_problem(i + 1, p.reason),
                None => p.reason,
            };
            rows.push(
                Field::plain("", Control::Info(String::new()), Target::None).with_danger(&text),
            );
        }
        rows
    }

    fn seq_step_fields(&self, cur: &SequenceDef) -> Vec<Field> {
        let mut fields = vec![Field::button(
            t::CLOSE_STEPS.get(),
            PrefAction::Sequence(SeqButton::CloseSteps),
        )];
        let Some(step) = cur.steps.get(self.seq.step) else {
            fields.push(Field::heading(t::STEPS_NONE.get()));
            return fields;
        };
        let at = self.seq.step.min(cur.steps.len().saturating_sub(1));
        fields.push(Field::heading(&t::step_heading(
            at + 1,
            cur.steps.len(),
            step_label(&step.run),
        )));
        let Some(vocab) = seqs::step_kind(&step.run) else {
            fields.push(
                Field::plain("", Control::Info(String::new()), Target::None)
                    .with_danger(&t::unknown_step(&step.run)),
            );
            return fields;
        };
        fields.push(Field::plain(
            t::STEP_DOES.get(),
            Control::Info(vocab.meaning_en.to_string()),
            Target::None,
        ));
        let modes = pause_modes(Some(vocab));
        let mut pause = Field::plain(
            t::PAUSE.get(),
            Control::Choice {
                options: modes.iter().map(|m| pause_label(*m).to_string()).collect(),
                index: modes.iter().position(|m| *m == step.pause).unwrap_or(0),
            },
            Target::Sequence(SeqTarget::Pause),
        )
        .with_help(t::PAUSE_HELP.get());
        if vocab.pause_forced {
            pause = pause.with_warning(t::PAUSE_FORCED_HELP.get());
        } else if !vocab.pausable {
            pause = pause.with_warning(t::PAUSE_UNNEEDED_HELP.get());
        }
        fields.push(pause);
        for (i, spec) in vocab.options.iter().enumerate() {
            let value = step.options.get(spec.key);
            let target = Target::Sequence(SeqTarget::Option(i));
            let label = option_label(spec.key);
            let mut help = String::new();
            if spec.kind == OptionKind::Template {
                help.push_str(t::OPT_TEMPLATE_HELP.get());
            }
            if spec.required_when_never {
                if !help.is_empty() {
                    help.push(' ');
                }
                help.push_str(t::OPT_REQUIRED_HELP.get());
            }
            let text = value
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_string();
            let mut field = match spec.kind {
                OptionKind::Bool => Field::plain(
                    &label,
                    Control::Toggle(value.and_then(toml::Value::as_bool).unwrap_or(false)),
                    target,
                ),
                OptionKind::Choice(choices) => {
                    let mut options = vec![t::OPT_UNSET.get().to_string()];
                    options.extend(choices.iter().map(|c| c.to_string()));
                    let index = choices.iter().position(|c| *c == text).map_or(0, |p| p + 1);
                    Field::plain(&label, Control::Choice { options, index }, target)
                }
                OptionKind::Text | OptionKind::Template => {
                    match self.seq_option_choices(spec.key) {
                        Some(options) => Field::plain(
                            &label,
                            Control::Combo {
                                options,
                                value: text,
                                custom: true,
                            },
                            target,
                        ),
                        None => Field::plain(&label, Control::Text(text), target),
                    }
                }
            };
            if !help.is_empty() {
                field = field.with_help(&help);
            }
            fields.push(field);
        }
        fields.extend(self.seq_problem_rows(cur, Some(at)));
        fields
    }

    /// The values an option is picked from, where the config knows them.
    fn seq_option_choices(&self, key: &str) -> Option<Vec<(String, String)>> {
        let unset = (t::OPT_UNSET.get().to_string(), String::new());
        let mut out = vec![unset];
        match key {
            "remote" => out.extend(self.remote_names.iter().map(|n| (n.clone(), n.clone()))),
            "profile" => out.extend(self.config.ai.profiles.iter().map(|p| {
                let name = if p.name.trim().is_empty() {
                    &p.id
                } else {
                    &p.name
                };
                (name.clone(), p.id.clone())
            })),
            "label" => out.extend(
                self.open_in_list()
                    .into_iter()
                    .map(|e| (e.label.clone(), e.label)),
            ),
            _ => return None,
        }
        Some(out)
    }
}
