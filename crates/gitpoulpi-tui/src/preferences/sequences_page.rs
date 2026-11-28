//! The Sequences page: the user's named action sequences (`[[sequences]]`,
//! `.gitpoulpi/sequences.toml`) edited in Settings, every change applied and
//! written at once.
//!
//! One master-detail page (`list_page.rs`) with two faces over the same
//! generic list, so focus, keys and mouse are the ones every list page has:
//!
//! * the **sequences** face: the list of sequences (grouped by scope when
//!   there is more than one) and the selected sequence's rows (name, id,
//!   description, scope, shortcut, steps, Run now);
//! * the **steps** face (`SeqButton::EditSteps`, Esc or the cross goes
//!   back): the list shows the sequence's steps and the rows are the
//!   selected step's (pause, one row per option).
//!
//! A second `ListEditor` nested in the detail was tried on paper first: it
//! would need a second focus ring (list, detail, sub-list, sub-detail) and a
//! second set of list hit rects inside the generic detail walk. The drill-in
//! reuses the one generic list and costs nothing there, with a real
//! "Back" button and Esc as the way out.
//!
//! The shortcut is not a row of its own: the button opens the Keyboard
//! shortcuts page's capture popup (`keybindings_page`), so conflicts are
//! refused identically and the key is written to `[keybindings.sequence]` by
//! the same code. Renaming an id moves its key in the same write.
//!
//! Storage is `gitpoulpi_config::sequences`: `installation` and `machine`
//! in `config.toml`, `repo` in the repository's file. The registry (palette
//! rows, keymap slots) is refreshed after every write.

use super::list_editor::{AddKind, ListItem, ListOp, ListSpec, Mark};
use super::persist::unique_id;
use super::sequences_picker::{step_label, PickOutcome, StepPicker};
use super::*;
use crate::keymap::{Action, Keymap};
use crate::sequence_run::registry;
use gitpoulpi_config::sequences::{
    self as seqs, OptionKind, PauseMode, Scope, SequenceDef, StepKind, StepSpec,
    KEYBINDINGS_CONTEXT,
};
use gitpoulpi_config::{reset_keybinding_in_file, set_keybinding_in_file};
use gitpoulpi_i18n::msg::{settings_profiles as profiles, settings_sequences as t};

/// Rows of the Sequences page that are not plain config keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeqTarget {
    Name,
    Id,
    Description,
    Scope,
    Pause,
    /// The selected step's option at this index of its vocabulary entry.
    Option(usize),
}

/// The buttons of the Sequences page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeqButton {
    Shortcut,
    ClearShortcut,
    EditSteps,
    CloseSteps,
    RunNow,
    EditWithAi,
}

/// The page's own state.
#[derive(Default)]
pub(super) struct SequencesPage {
    /// The selected sequence.
    pub(super) index: usize,
    /// The selected step of the sequence, on the steps face.
    pub(super) step: usize,
    /// The steps face is in front.
    pub(super) in_steps: bool,
    pub(super) picker: Option<StepPicker>,
    /// "Create with AI" / "Edit with AI" (`sequences_ai.rs`).
    pub(super) ai: super::sequences_ai::SeqAiLink,
    /// The repository's file, read once per change (`fields` runs every frame).
    repo_cache: std::cell::RefCell<Option<Vec<SequenceDef>>>,
    /// The keymap generation `keys.km` was last rebuilt for.
    km_gen: u64,
    /// Tests pin the installation id (reading the real one seeds the config).
    pub(super) install: Option<String>,
}

pub(super) const SCOPES: [Scope; 3] = [Scope::Installation, Scope::Machine, Scope::Repo];

pub(super) fn scope_label(scope: Scope) -> &'static str {
    match scope {
        Scope::Installation => t::SCOPE_INSTALLATION.get(),
        Scope::Machine => t::SCOPE_MACHINE.get(),
        Scope::Repo => t::SCOPE_REPO.get(),
    }
}

fn group_label(scope: Scope) -> &'static str {
    match scope {
        Scope::Installation => t::GROUP_INSTALLATION.get(),
        Scope::Machine => t::GROUP_MACHINE.get(),
        Scope::Repo => t::GROUP_REPO.get(),
    }
}

pub(super) fn pause_label(mode: PauseMode) -> &'static str {
    match mode {
        PauseMode::Default => t::PAUSE_DEFAULT.get(),
        PauseMode::Always => t::PAUSE_ALWAYS.get(),
        PauseMode::Never => t::PAUSE_NEVER.get(),
    }
}

/// The pauses a step offers: `Never` is absent for a step that always asks.
pub(super) fn pause_modes(vocab: Option<&StepKind>) -> Vec<PauseMode> {
    let mut modes = vec![PauseMode::Default, PauseMode::Always];
    if !vocab.is_some_and(|k| k.pause_forced) {
        modes.push(PauseMode::Never);
    }
    modes
}

pub(super) fn option_label(key: &str) -> String {
    match key {
        "name" => t::OPT_NAME.get().into(),
        "message" => t::OPT_MESSAGE.get().into(),
        "branch" => t::OPT_BRANCH.get().into(),
        "open_tab" => t::OPT_OPEN_TAB.get().into(),
        "label" => t::OPT_LABEL.get().into(),
        "profile" => t::OPT_PROFILE.get().into(),
        "remote" => t::OPT_REMOTE.get().into(),
        other => other.to_string(),
    }
}

/// A preset as inserted: its name and description in the UI language.
fn localised(mut def: SequenceDef) -> SequenceDef {
    let texts = match def.id.as_str() {
        "backup" => (t::PRESET_BACKUP.get(), t::PRESET_BACKUP_DESC.get()),
        "compose_and_push" => (t::PRESET_COMPOSE.get(), t::PRESET_COMPOSE_DESC.get()),
        "worktree_agent" => (t::PRESET_WORKTREE.get(), t::PRESET_WORKTREE_DESC.get()),
        _ => return def,
    };
    def.name = texts.0.to_string();
    def.description = Some(texts.1.to_string());
    def
}

/// `[keybindings.sequence] <id>` as the file has it (a string or a list).
fn chords_in_file(path: &Path, id: &str) -> Vec<String> {
    // Through the loader: the bindings live in `tui.toml` once it exists.
    gitpoulpi_config::Config::load(path)
        .ok()
        .and_then(|c| {
            c.keybindings
                .get(KEYBINDINGS_CONTEXT, id)
                .map(<[String]>::to_vec)
        })
        .unwrap_or_default()
}

impl PreferencesState {
    // ---------------------------------------------------------------
    // What the page edits
    // ---------------------------------------------------------------

    pub(super) fn seq_install_id(&self) -> String {
        self.seq
            .install
            .clone()
            .unwrap_or_else(crate::keymap::install_id)
    }

    /// The repository folder when it is one (a repository sequence needs it).
    fn seq_repo(&self) -> Option<PathBuf> {
        self.repo_root
            .join(".git")
            .exists()
            .then(|| self.repo_root.clone())
    }

    fn seq_repo_defs(&self) -> Vec<SequenceDef> {
        let mut cache = self.seq.repo_cache.borrow_mut();
        if cache.is_none() {
            let defs = self
                .seq_repo()
                .and_then(|root| seqs::peek_repo_sequences(&root).ok())
                .unwrap_or_default();
            *cache = Some(defs);
        }
        cache.clone().unwrap_or_default()
    }

    /// Every sequence the page lists: this installation's, the shared ones,
    /// then the repository's, in the list's order.
    pub(super) fn seq_entries(&self) -> Vec<SequenceDef> {
        let mut out: Vec<SequenceDef> =
            seqs::visible_sequences(&self.config, &self.seq_install_id(), None)
                .into_iter()
                .map(|(d, _)| d)
                .collect();
        out.extend(self.seq_repo_defs());
        out
    }

    /// The selected item of the list in front (a sequence, or a step).
    pub(super) fn seq_selected(&self) -> usize {
        if self.seq.in_steps {
            let steps = self.seq_cur().map_or(0, |d| d.steps.len());
            self.seq.step.min(steps.saturating_sub(1))
        } else {
            self.seq
                .index
                .min(self.seq_entries().len().saturating_sub(1))
        }
    }

    /// The selected sequence.
    pub(super) fn seq_cur(&self) -> Option<SequenceDef> {
        let entries = self.seq_entries();
        let at = self.seq.index.min(entries.len().saturating_sub(1));
        entries.into_iter().nth(at)
    }

    pub(super) fn seq_select(&mut self, index: usize) {
        if self.seq.in_steps {
            self.seq.step = index;
        } else {
            self.seq.index = index;
            self.seq.step = 0;
        }
    }

    pub(super) fn seq_chord(&self, id: &str) -> String {
        registry::slot_of(id)
            .map(|slot| self.keys.km.shown(Action::Sequence(slot)))
            .unwrap_or_default()
    }

    /// Brings the registry and the keymap view in line with the config (the
    /// page may be reached after a sequence was added elsewhere).
    pub(super) fn seq_normalise(&mut self) {
        // A live window's registry is already built from the file (startup,
        // and every write below): Settings' copy of the config may be older.
        if !self.keys.live {
            registry::refresh(&self.config, &self.seq_install_id());
        }
        let generation = registry::generation();
        if self.seq.km_gen != generation {
            self.seq.km_gen = generation;
            self.keys.km = Keymap::from_config(&self.config.keybindings);
        }
        let len = self.seq_entries().len();
        self.seq.index = self.seq.index.min(len.saturating_sub(1));
        if len == 0 {
            self.seq.in_steps = false;
        }
    }

    // ---------------------------------------------------------------
    // The list
    // ---------------------------------------------------------------

    pub(super) fn seq_list_spec(&self) -> ListSpec {
        let Some(cur) = self.seq_cur() else {
            return self.seq_sequences_spec(&[]);
        };
        if self.seq.in_steps {
            let problems = seqs::validate(&cur);
            let items = cur
                .steps
                .iter()
                .enumerate()
                .map(|(i, step)| {
                    let mark = problems
                        .iter()
                        .find(|p| p.step == Some(i))
                        .map_or(Mark::None, |p| Mark::Problem(p.reason.clone()));
                    ListItem {
                        label: step_label(&step.run),
                        sub: (step.pause != PauseMode::Default)
                            .then(|| pause_label(step.pause).to_string()),
                        mark,
                        pristine: step.options.is_empty() && step.pause == PauseMode::Default,
                        ..ListItem::default()
                    }
                })
                .collect();
            return ListSpec {
                title: t::steps_title(if cur.name.trim().is_empty() {
                    &cur.id
                } else {
                    &cur.name
                }),
                items,
                duplicate: true,
                reorder: true,
                ..ListSpec::default()
            };
        }
        self.seq_sequences_spec(&self.seq_entries())
    }

    fn seq_sequences_spec(&self, entries: &[SequenceDef]) -> ListSpec {
        let mixed = entries.iter().any(|d| d.scope != entries[0].scope);
        let items = entries
            .iter()
            .map(|d| {
                let mark = seqs::validate(d)
                    .into_iter()
                    .next()
                    .map_or(Mark::None, |p| Mark::Problem(p.reason));
                let chord = self.seq_chord(&d.id);
                ListItem {
                    label: if d.name.trim().is_empty() {
                        d.id.clone()
                    } else {
                        d.name.clone()
                    },
                    sub: Some(chord).filter(|c| !c.is_empty()),
                    mark,
                    pristine: d.steps.is_empty() && d.name == t::NEW_NAME.get(),
                    group: mixed.then(|| group_label(d.scope).to_string()),
                    ..ListItem::default()
                }
            })
            .collect();
        let mut menu = vec![t::MENU_EMPTY.get().to_string()];
        menu.extend(
            seqs::presets()
                .into_iter()
                .map(|p| t::menu_preset(localised(p).name)),
        );
        if self.seq_ai_offered() {
            menu.push(t::MENU_AI.get().to_string());
        }
        ListSpec {
            title: t::LIST_TITLE.get().to_string(),
            items,
            add: AddKind::Menu(menu),
            duplicate: true,
            reorder: true,
            ..ListSpec::default()
        }
    }

    /// One list gesture.
    pub(super) fn seq_list_op(&mut self, op: ListOp) {
        if self.seq.in_steps {
            self.seq_steps_op(op);
            return;
        }
        let entries = self.seq_entries();
        let len = entries.len();
        match op {
            ListOp::Add(Some(pick)) => self.seq_add(pick),
            ListOp::Duplicate(i) if i < len => {
                let mut copy = entries[i].clone();
                copy.id = unique_id(&copy.id, |c| entries.iter().any(|d| d.id == c));
                copy.name = profiles::duplicate_name(&copy.name);
                let id = copy.id.clone();
                if self.seq_commit(None, copy) {
                    self.seq_select_id(&id);
                }
            }
            ListOp::Remove(i) if i < len => self.seq_remove(&entries[i]),
            ListOp::Move { from, to } if from < len && to < len => self.seq_move(from, to),
            _ => {}
        }
    }

    pub(super) fn seq_select_id(&mut self, id: &str) {
        if let Some(at) = self.seq_entries().iter().position(|d| d.id == id) {
            self.seq.index = at;
            self.seq.step = 0;
        }
    }

    fn seq_add(&mut self, pick: usize) {
        let presets = seqs::presets();
        let entries = self.seq_entries();
        let mut def = match pick {
            0 => SequenceDef {
                id: "sequence".into(),
                name: t::NEW_NAME.get().to_string(),
                ..SequenceDef::default()
            },
            n if n <= presets.len() => localised(presets[n - 1].clone()),
            _ => {
                self.seq_ai_open_create();
                return;
            }
        };
        def.id = unique_id(&def.id, |c| entries.iter().any(|d| d.id == c));
        let id = def.id.clone();
        if self.seq_commit(None, def) {
            self.seq_select_id(&id);
            self.focus_detail_target(|t| matches!(t, Target::Sequence(SeqTarget::Name)));
            self.focus = PrefFocus::List;
        }
    }

    fn seq_remove(&mut self, def: &SequenceDef) {
        let Some(path) = self.config_file() else {
            self.message = Some(t::NO_CONFIG_DIR.get().into());
            return;
        };
        let repo = self.seq_repo();
        let was_trusted = repo
            .as_deref()
            .map(crate::sequence_run::repo::trusted_before_edit);
        if let Err(err) = seqs::remove_sequence(def.scope, &def.id, &path, repo.as_deref()) {
            self.message = Some(t::write_failed(err));
            return;
        }
        let _ = reset_keybinding_in_file(&path, KEYBINDINGS_CONTEXT, &def.id);
        self.seq_reload();
        self.seq_retrust(was_trusted);
        let len = self.seq_entries().len();
        self.seq.index = self.seq.index.min(len.saturating_sub(1));
        self.message = Some(t::SAVED.get().into());
    }

    fn seq_move(&mut self, from: usize, to: usize) {
        let entries = self.seq_entries();
        // Moves stay inside a group (the list refuses to cross one already).
        if entries[from].scope != entries[to].scope {
            return;
        }
        let Some(path) = self.config_file() else {
            return;
        };
        let mut order: Vec<&SequenceDef> = entries.iter().collect();
        let moved = order.remove(from);
        order.insert(to, moved);
        let ids = |repo: bool| -> Vec<String> {
            order
                .iter()
                .filter(|d| (d.scope == Scope::Repo) == repo)
                .map(|d| d.id.clone())
                .collect()
        };
        let repo_dir = self.seq_repo();
        let result = seqs::reorder_sequences(Scope::Installation, &ids(false), &path, None)
            .and_then(|_| {
                seqs::reorder_sequences(Scope::Repo, &ids(true), &path, repo_dir.as_deref())
            });
        match result {
            Ok(_) => {
                self.seq.index = to;
                self.seq_reload();
            }
            Err(err) => self.message = Some(t::write_failed(err)),
        }
    }

    fn seq_steps_op(&mut self, op: ListOp) {
        let Some(cur) = self.seq_cur() else { return };
        let len = cur.steps.len();
        match op {
            ListOp::Add(None) => self.seq_open_picker(),
            ListOp::Duplicate(i) if i < len => {
                self.seq_edit(|d| {
                    let copy = d.steps[i].clone();
                    d.steps.insert(i + 1, copy);
                });
                self.seq.step = i + 1;
            }
            ListOp::Remove(i) if i < len => {
                self.seq_edit(|d| {
                    d.steps.remove(i);
                });
                self.seq.step = i.min(len.saturating_sub(2));
            }
            ListOp::Move { from, to } if from < len && to < len => {
                self.seq_edit(|d| {
                    let step = d.steps.remove(from);
                    d.steps.insert(to, step);
                });
                self.seq.step = to;
            }
            _ => {}
        }
    }

    pub(super) fn seq_open_picker(&mut self) {
        self.seq.picker = Some(StepPicker::new(self.config.ai.enabled));
    }

    // ---------------------------------------------------------------
    // Writing
    // ---------------------------------------------------------------

    /// The user wrote the repository's file themselves: it stays trusted when
    /// it was trusted (or empty) before the edit, never when other people's
    /// untrusted sequences were in it.
    fn seq_retrust(&mut self, was_trusted: Option<bool>) {
        if let (Some(root), Some(was)) = (self.seq_repo(), was_trusted) {
            crate::sequence_run::repo::retrust_after_edit(&root, was);
            self.seq.repo_cache.replace(None);
            self.seq_reload();
        }
    }

    /// Re-reads what the write changed and tells the registry (palette rows,
    /// keymap slots) and the live keymap.
    pub(super) fn seq_reload(&mut self) {
        self.seq.repo_cache.replace(None);
        if let Some(path) = self.config_file() {
            if let Ok(config) = Config::load(&path) {
                self.config.sequences = config.sequences;
                self.config.keybindings = config.keybindings;
            }
        }
        registry::refresh(&self.config, &self.seq_install_id());
        self.seq.km_gen = registry::generation();
        self.keys.km = Keymap::from_config(&self.config.keybindings);
        if self.keys.live {
            crate::keymap::install(Keymap::from_config(&self.config.keybindings));
            registry::note_config_loaded();
        }
    }

    /// Writes `new` where its scope says, replacing `old` (renamed or moved
    /// to the other storage), moves the shortcut with a renamed id, and
    /// reloads. `false`: refused or failed (the message says why).
    pub(super) fn seq_commit(&mut self, old: Option<&SequenceDef>, mut new: SequenceDef) -> bool {
        let Some(path) = self.config_file() else {
            self.message = Some(t::NO_CONFIG_DIR.get().into());
            return false;
        };
        let repo = self.seq_repo();
        if new.scope == Scope::Repo && repo.is_none() {
            self.message = Some(t::NO_REPO.get().into());
            return false;
        }
        if new.scope != Scope::Installation {
            new.owner = None;
        }
        let id_changed = old.is_some_and(|o| o.id != new.id);
        let repo_changed =
            old.is_some_and(|o| (o.scope == Scope::Repo) != (new.scope == Scope::Repo));
        let was_trusted = repo
            .as_deref()
            .map(crate::sequence_run::repo::trusted_before_edit);
        let chords = match old {
            Some(o) if id_changed => chords_in_file(&path, &o.id),
            _ => Vec::new(),
        };
        if let Some(o) = old.filter(|_| id_changed || repo_changed) {
            if let Err(err) = seqs::remove_sequence(o.scope, &o.id, &path, repo.as_deref()) {
                self.message = Some(t::write_failed(err));
                return false;
            }
        }
        if let Err(err) = seqs::save_sequence(&new, &path, &self.seq_install_id(), repo.as_deref())
        {
            self.message = Some(t::write_failed(err));
            return false;
        }
        if let Some(o) = old {
            // The key follows an id; a repository sequence has none yet.
            if id_changed || (repo_changed && new.scope == Scope::Repo) {
                let _ = reset_keybinding_in_file(&path, KEYBINDINGS_CONTEXT, &o.id);
                if id_changed && new.scope != Scope::Repo && !chords.is_empty() {
                    let _ = set_keybinding_in_file(&path, KEYBINDINGS_CONTEXT, &new.id, &chords);
                }
            }
        }
        self.seq_reload();
        self.seq_retrust(was_trusted);
        // The list is grouped by scope and a rename re-files the entry:
        // the selection follows the sequence.
        if let Some(at) = self.seq_entries().iter().position(|d| d.id == new.id) {
            self.seq.index = at;
        }
        self.message = Some(t::SAVED.get().into());
        true
    }

    /// Edits the selected sequence and writes it.
    pub(super) fn seq_edit(&mut self, edit: impl FnOnce(&mut SequenceDef)) -> bool {
        let Some(old) = self.seq_cur() else {
            return false;
        };
        let mut new = old.clone();
        edit(&mut new);
        if new == old {
            return true;
        }
        self.seq_commit(Some(&old), new)
    }

    /// Edits the selected step (with its vocabulary entry) and writes it.
    fn seq_edit_step(&mut self, edit: impl FnOnce(&mut StepSpec, Option<&'static StepKind>)) {
        let at = self.seq.step;
        self.seq_edit(|d| {
            if let Some(step) = d.steps.get_mut(at) {
                let vocab = seqs::step_kind(&step.run);
                edit(step, vocab);
            }
        });
    }

    // ---------------------------------------------------------------
    // Committing rows
    // ---------------------------------------------------------------

    pub(super) fn seq_commit_text(&mut self, target: SeqTarget, draft: &str) -> bool {
        let value = draft.trim().to_string();
        match target {
            SeqTarget::Name => {
                self.seq_edit(|d| d.name = value);
            }
            SeqTarget::Description => {
                self.seq_edit(|d| d.description = Some(value).filter(|v| !v.is_empty()));
            }
            SeqTarget::Id => {
                let Some(cur) = self.seq_cur() else {
                    return true;
                };
                if value == cur.id {
                    return true;
                }
                if !seqs::is_valid_id(&value) {
                    self.message = Some(t::id_invalid(value));
                    return false;
                }
                if self.seq_entries().iter().any(|d| d.id == value) {
                    self.message = Some(t::id_taken(value));
                    return false;
                }
                return self.seq_edit(|d| d.id = value);
            }
            SeqTarget::Option(i) => {
                self.seq_set_option(i, (!value.is_empty()).then(|| value.into()))
            }
            SeqTarget::Scope | SeqTarget::Pause => {}
        }
        true
    }

    pub(super) fn seq_commit_combo(&mut self, target: SeqTarget, value: &str) {
        self.seq_commit_text(target, value);
    }

    pub(super) fn seq_commit_bool(&mut self, target: SeqTarget, value: bool) {
        if let SeqTarget::Option(i) = target {
            self.seq_set_option(i, value.then_some(toml::Value::Boolean(true)));
        }
    }

    pub(super) fn seq_commit_choice(&mut self, target: SeqTarget, next: usize) {
        match target {
            SeqTarget::Scope => {
                let Some(scope) = SCOPES.get(next).copied() else {
                    return;
                };
                if scope == Scope::Repo && self.seq_repo().is_none() {
                    self.message = Some(t::NO_REPO.get().into());
                    return;
                }
                self.seq_edit(|d| d.scope = scope);
            }
            SeqTarget::Pause => {
                self.seq_edit_step(|step, vocab| {
                    if let Some(mode) = pause_modes(vocab).get(next) {
                        step.pause = *mode;
                    }
                });
            }
            SeqTarget::Option(i) => {
                let Some(cur) = self.seq_cur() else { return };
                let Some(spec) = cur
                    .steps
                    .get(self.seq.step)
                    .and_then(|s| seqs::step_kind(&s.run))
                    .and_then(|k| k.options.get(i))
                else {
                    return;
                };
                let value = match spec.kind {
                    OptionKind::Choice(choices) => next
                        .checked_sub(1)
                        .and_then(|n| choices.get(n))
                        .map(|c| toml::Value::String((*c).to_string())),
                    _ => None,
                };
                self.seq_set_option(i, value);
            }
            _ => {}
        }
    }

    /// Sets (or, with `None`, clears) the selected step's option `i`.
    fn seq_set_option(&mut self, i: usize, value: Option<toml::Value>) {
        self.seq_edit_step(|step, vocab| {
            let Some(spec) = vocab.and_then(|k| k.options.get(i)) else {
                return;
            };
            match value {
                Some(v) => {
                    step.options.insert(spec.key.to_string(), v);
                }
                None => {
                    step.options.remove(spec.key);
                }
            }
        });
    }

    // ---------------------------------------------------------------
    // Buttons
    // ---------------------------------------------------------------

    pub(super) fn seq_button(&mut self, button: SeqButton) {
        match button {
            SeqButton::EditSteps => {
                self.seq.in_steps = true;
                self.seq.step = 0;
                self.focus = PrefFocus::List;
                self.strip = false;
            }
            SeqButton::CloseSteps => self.seq_close_steps(),
            SeqButton::Shortcut => self.seq_open_shortcut(),
            SeqButton::ClearShortcut => {
                let Some(cur) = self.seq_cur() else { return };
                if let Some(slot) = registry::slot_of(&cur.id) {
                    if let Err(err) = self.keys_write(Action::Sequence(slot), &[]) {
                        self.message = Some(err);
                    }
                }
            }
            SeqButton::RunNow => {
                let Some(cur) = self.seq_cur() else { return };
                if let Some(problem) = seqs::validate(&cur).into_iter().next() {
                    self.message = Some(t::cannot_run(problem.reason));
                } else if let Some(slot) = registry::slot_of(&cur.id) {
                    let root = self.repo_root.clone();
                    crate::sequence_run::start(slot, &root, &mut self.message);
                }
            }
            SeqButton::EditWithAi => self.seq_ai_open_edit(),
        }
    }

    fn seq_close_steps(&mut self) {
        self.seq.in_steps = false;
        self.focus = PrefFocus::List;
        self.strip = false;
        self.edit = None;
        self.choice = None;
    }

    /// Esc on the steps face goes back to the sequences.
    pub(super) fn seq_steps_back_key(&mut self, key: &KeyEvent) -> bool {
        let on = self.current_section() == PrefSection::Sequences
            && self.seq.in_steps
            && self.focus != PrefFocus::Sections;
        if on && key.code == KeyCode::Esc {
            self.seq_close_steps();
            return true;
        }
        false
    }

    fn seq_open_shortcut(&mut self) {
        let Some(cur) = self.seq_cur() else { return };
        self.seq_normalise();
        let Some(slot) = registry::slot_of(&cur.id) else {
            return;
        };
        self.keys_open_capture_for(Action::Sequence(slot));
    }

    // ---------------------------------------------------------------
    // Overlays: the step picker and the shortcut's capture popup
    // ---------------------------------------------------------------

    pub(super) fn seq_overlay_open(&self) -> bool {
        self.seq.picker.is_some()
            || self.seq.ai.dialog.is_some()
            || (self.current_section() == PrefSection::Sequences && self.keys.popup.is_some())
    }

    pub(super) fn seq_overlay_key(&mut self, key: KeyEvent) -> bool {
        if self.seq.ai.dialog.is_some() {
            self.seq_ai_key(key);
        } else if let Some(picker) = self.seq.picker.as_mut() {
            match picker.on_key(&key) {
                PickOutcome::Stay => {}
                PickOutcome::Close => self.seq.picker = None,
                PickOutcome::Pick(id) => self.seq_add_step(id),
            }
        } else {
            self.keys_popup_key(&key);
        }
        true
    }

    pub(super) fn seq_overlay_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if self.seq.ai.dialog.is_some() {
            self.seq_ai_mouse(kind, col, row);
            return true;
        }
        if let Some(picker) = self.seq.picker.as_mut() {
            match picker.on_mouse(kind, col, row) {
                PickOutcome::Stay => {}
                PickOutcome::Close => self.seq.picker = None,
                PickOutcome::Pick(id) => self.seq_add_step(id),
            }
            return true;
        }
        self.keys_mouse(kind, col, row)
    }

    pub(super) fn seq_overlay_paste(&mut self, text: &str) {
        if self.seq.ai.dialog.is_some() {
            self.seq_ai_paste(text);
        } else if let Some(picker) = self.seq.picker.as_mut() {
            picker.type_text(text);
        } else {
            self.keys_paste(text);
        }
    }

    pub(super) fn render_sequence_overlays(&mut self, frame: &mut Frame, area: Rect) {
        if self.current_section() != PrefSection::Sequences {
            return;
        }
        let pointer = self.mouse_pos;
        if self.seq.ai.dialog.is_some() {
            self.seq_ai_render(frame.buffer_mut(), area, pointer);
        } else if let Some(picker) = self.seq.picker.as_mut() {
            picker.render(frame.buffer_mut(), area, pointer);
        } else if self.keys.popup.is_some() {
            self.keys_render_popup(frame.buffer_mut(), area);
        }
    }

    /// The picker chose `id`: append the step and select it.
    pub(super) fn seq_add_step(&mut self, id: &str) {
        self.seq.picker = None;
        let Some(cur) = self.seq_cur() else { return };
        let at = cur.steps.len();
        if self.seq_edit(|d| d.steps.push(StepSpec::new(id))) {
            self.seq.step = at;
            self.focus = PrefFocus::List;
        }
    }
}
