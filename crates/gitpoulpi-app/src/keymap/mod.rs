//! The keymap: one registry saying which key triggers which action in
//! which context (see the keybindings design). Pure and
//! unit-testable; the event loop reads the installed instance through
//! [`current`]. The defaults are the shipped bindings; `[keybindings]` in
//! `config.toml` overrides them ([`Keymap::from_config`]), and a bad
//! entry is reported as a [`KeymapIssue`] and ignored, never fatal.

// Registry metadata (labels, flags, `Ctx` helpers) is not all consumed.
#![allow(dead_code)]

pub mod action;
pub mod chord;
pub mod conflict;

pub use action::{Action, Ctx, ACTIONS, STATIC_ACTIONS};
pub use chord::KeyChord;
use chord::Mods;

use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct Entry {
    action: Action,
    ctx: Ctx,
    chords: Vec<KeyChord>,
    /// Set by `[keybindings]` rather than the shipped default.
    overridden: bool,
}

/// One config entry the keymap could not honour; the default stays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapIssue {
    /// `<ctx>.<action>` as written in the config.
    pub id: String,
    pub message: String,
}

#[derive(Clone)]
pub struct Keymap {
    entries: Vec<Entry>,
    issues: Vec<KeymapIssue>,
}

fn default_chords(def: &action::ActionDef) -> Vec<KeyChord> {
    def.defaults
        .iter()
        .filter_map(|text| {
            let parsed = KeyChord::parse(text);
            debug_assert!(
                parsed.is_ok(),
                "bad default chord {text:?} for {}", // i18n-english
                def.config_id()
            );
            parsed.ok()
        })
        .collect()
}

impl Keymap {
    /// The shipped bindings: every `ActionDef`'s `defaults`.
    pub fn defaults() -> Keymap {
        Keymap::defaults_of(&ACTIONS)
    }

    /// [`Keymap::defaults`] over a given action list. `STATIC_ACTIONS` alone
    /// is what the palette's own registry is built from (the full list is
    /// being built then).
    pub fn defaults_of(defs: &[action::ActionDef]) -> Keymap {
        let entries = defs
            .iter()
            .map(|def| Entry {
                action: def.action,
                ctx: def.ctx,
                chords: default_chords(def),
                overridden: false,
            })
            .collect();
        Keymap {
            entries,
            issues: Vec::new(),
        }
    }

    /// The defaults with `[keybindings]` applied. Every entry that cannot
    /// be honoured (unknown context or action, unparsable chord, a fixed
    /// or not-yet-live action, an error-level conflict) is skipped with a
    /// [`KeymapIssue`] and keeps its default.
    pub fn from_config(cfg: &gitpoulpi_config::KeybindingsConfig) -> Keymap {
        let mut km = Keymap::defaults();
        for (ctx, name, texts) in cfg.iter() {
            let id = format!("{ctx}.{name}");
            let issue = |message: String| KeymapIssue {
                id: id.clone(),
                message,
            };
            // `[keybindings.sequence]`: a slug resolves through the registry
            // of the sequences this installation sees.
            if ctx == gitpoulpi_config::sequences::KEYBINDINGS_CONTEXT
                && crate::sequences::slot_of(name).is_none()
            {
                km.issues
                    .push(issue(gitpoulpi_i18n::msg::keymap_issue::unknown_sequence(
                        name,
                    )));
                continue;
            }
            let Some(def) = ACTIONS.iter().find(|d| d.config_id() == id) else {
                let known = Ctx::ALL.iter().any(|c| c.name() == ctx);
                km.issues.push(issue(if known {
                    gitpoulpi_i18n::msg::keymap_issue::unknown_action(name, ctx)
                } else {
                    gitpoulpi_i18n::msg::keymap_issue::unknown_context(ctx)
                }));
                continue;
            };
            if !def.rebindable || !def.live() {
                km.issues.push(issue(if def.rebindable {
                    gitpoulpi_i18n::msg::keymap_issue::NOT_REBINDABLE
                        .get()
                        .to_string()
                } else {
                    gitpoulpi_i18n::msg::keymap_issue::KEY_FIXED
                        .get()
                        .to_string()
                }));
                continue;
            }
            let mut chords = Vec::new();
            let mut bad = None;
            for text in texts {
                match KeyChord::parse(text) {
                    Ok(c) if def.ctx.ignores_modifiers() && c.mods.alt => {
                        bad = Some(gitpoulpi_i18n::msg::keymap_issue::prefix_mods(text));
                    }
                    Ok(c) => match conflict::fixed_reason(def.ctx.scope(), c) {
                        Some(reason) => bad = Some(format!("'{text}': {}", reason.english())),
                        None => chords.push(c),
                    },
                    Err(e) => bad = Some(format!("'{text}': {e}")),
                }
            }
            if let Some(message) = bad {
                km.issues.push(issue(message));
                continue;
            }
            if let Some(entry) = km.entries.iter_mut().find(|e| e.action == def.action) {
                entry.chords = chords;
                entry.overridden = true;
            }
        }
        km.revert_error_conflicts();
        km
    }

    /// Puts back the default of every overridden action that takes part
    /// in an error-level conflict (the later id of a clash), until none
    /// is left.
    fn revert_error_conflicts(&mut self) {
        loop {
            let errors = conflict::detect(self, conflict::current_transport())
                .into_iter()
                .filter(|c| c.level == conflict::Level::Error);
            let culprit =
                errors
                    .map(|c| (c.action, c.other, c.message))
                    .find_map(|(a, other, message)| {
                        let overridden =
                            |x: Action| self.entries.iter().any(|e| e.action == x && e.overridden);
                        if overridden(a) {
                            Some((a, message))
                        } else {
                            other.filter(|o| overridden(*o)).map(|o| (o, message))
                        }
                    });
            let Some((action, message)) = culprit else {
                return;
            };
            let def = ACTIONS
                .iter()
                .find(|d| d.action == action)
                .expect("registered");
            if let Some(entry) = self.entries.iter_mut().find(|e| e.action == action) {
                entry.chords = default_chords(def);
                entry.overridden = false;
            }
            self.issues.push(KeymapIssue {
                id: def.config_id().to_string(),
                message,
            });
        }
    }

    /// Config entries that were ignored, for the startup message.
    pub fn issues(&self) -> &[KeymapIssue] {
        &self.issues
    }

    /// Conflicts, warnings and hints for the effective bindings.
    pub fn conflicts(&self) -> Vec<conflict::Conflict> {
        conflict::detect(self, conflict::current_transport())
    }

    /// Every plain character some `nav_prefix` binding uses: those are
    /// never handed out as navigation badges.
    pub fn nav_prefix_chars(&self) -> Vec<char> {
        self.entries
            .iter()
            .filter(|e| e.ctx == Ctx::NavPrefix)
            .flat_map(|e| e.chords.iter())
            .filter_map(|c| match c.key {
                chord::Key::Char(ch) => Some(ch),
                _ => None,
            })
            .collect()
    }

    /// A copy with `action`'s chords replaced (the Settings capture popup
    /// asks what the conflicts would be before saving anything).
    pub fn with_chords(&self, action: Action, chords: Vec<KeyChord>) -> Keymap {
        let mut km = self.clone();
        if let Some(entry) = km.entries.iter_mut().find(|e| e.action == action) {
            entry.chords = chords;
            entry.overridden = true;
        }
        km
    }

    /// The labels of the actions that already hold `chord` anywhere in the
    /// window (the global scope, where a sequence's shortcut lives).
    pub fn global_holders(&self, chord: KeyChord) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| {
                e.ctx.scope() == Ctx::Global && e.chords.iter().any(|c| c.class() == chord.class())
            })
            .filter_map(|e| ACTIONS.iter().find(|d| d.action == e.action))
            .map(|d| d.display_label().into_owned())
            .collect()
    }

    /// Whether `action`'s chords differ from its shipped defaults.
    pub fn is_modified(&self, action: Action) -> bool {
        let Some(def) = ACTIONS.iter().find(|d| d.action == action) else {
            return false;
        };
        let classes = |list: &[KeyChord]| list.iter().map(|c| c.class()).collect::<Vec<_>>();
        classes(self.chords(action)) != classes(&default_chords(def))
    }

    /// Every chord of `action`, as bound.
    pub fn chords_of(&self, action: Action) -> &[KeyChord] {
        self.chords(action)
    }

    fn chords(&self, action: Action) -> &[KeyChord] {
        self.entries
            .iter()
            .find(|e| e.action == action)
            .map(|e| e.chords.as_slice())
            .unwrap_or(&[])
    }

    /// Whether `chord` (already normalized) triggers `action`, any of its
    /// chords. Lenient about extra modifiers, like the hand-written guards
    /// this replaced.
    pub fn chord_is(&self, action: Action, chord: KeyChord) -> bool {
        self.chords(action).iter().any(|c| c.matches_chord(chord))
    }

    /// The action `chord` triggers in `ctx`, if any. An exact chord match
    /// wins over a lenient one, so a `Ctrl+Shift+N` binding beats a
    /// `Ctrl+N` one for the chord that spells both. A prefix context looks
    /// at the key alone: a Ctrl chord bound after the prefix (`Ctrl+G
    /// Ctrl+P`) matches exactly first, an unbound one falls back to the key
    /// with Shift only. Shift is kept because the bindings tell it apart
    /// (`Ctrl+G Shift+P` pushes, `Ctrl+G p` pulls; `Ctrl+N Shift+Left` and
    /// `Ctrl+N Left` are two actions); Ctrl and Alt are dropped, so holding
    /// Ctrl through the second key still works.
    pub fn action_for(&self, ctx: Ctx, chord: KeyChord) -> Option<Action> {
        if ctx.ignores_modifiers() {
            if chord.mods.ctrl {
                let exact = self.action_exact(ctx, KeyChord::new(Mods::CTRL, chord.key));
                if exact.is_some() {
                    return exact;
                }
            }
            let mods = Mods {
                shift: chord.mods.shift,
                ..Mods::NONE
            };
            return self.action_exact(ctx, KeyChord::new(mods, chord.key));
        }
        self.action_exact(ctx, chord).or_else(|| {
            self.entries
                .iter()
                .filter(|e| e.ctx == ctx)
                .find(|e| e.chords.iter().any(|c| c.matches_chord(chord)))
                .map(|e| e.action)
        })
    }

    /// The action in `ctx` with a chord of exactly `chord`'s spelling class.
    pub fn action_exact(&self, ctx: Ctx, chord: KeyChord) -> Option<Action> {
        let chord = chord.class();
        self.entries
            .iter()
            .filter(|e| e.ctx == ctx)
            .find(|e| e.chords.iter().any(|c| c.class() == chord))
            .map(|e| e.action)
    }

    fn prefix_chord(&self, action: Action) -> Option<&KeyChord> {
        // STATIC_ACTIONS: the palette's registry is built through this.
        match STATIC_ACTIONS
            .iter()
            .find(|d| d.action == action)
            .map(|d| d.ctx)
        {
            Some(Ctx::GitPrefix) => self.chords(Action::GlobalGitPrefix).first(),
            Some(Ctx::NavPrefix) => self.chords(Action::GlobalNavPrefix).first(),
            _ => None,
        }
    }

    /// Every chord of `action`, printed canonically (`Shift+P`);
    /// prefix-context actions carry the effective prefix (`"Ctrl+G s"`).
    pub fn labels(&self, action: Action) -> Vec<String> {
        let prefix = self.prefix_chord(action);
        self.chords(action)
            .iter()
            .map(|c| match prefix {
                Some(p) => format!("{p} {c}"),
                None => c.to_string(),
            })
            .collect()
    }

    /// The first chord's label, or `""` when unbound.
    pub fn label(&self, action: Action) -> String {
        self.labels(action).into_iter().next().unwrap_or_default()
    }

    /// [`Keymap::labels`] in the compact on-screen spelling (`Ctrl+G P`):
    /// what the palette, help, footers and messages show.
    pub fn shown_labels(&self, action: Action) -> Vec<String> {
        let prefix = self.prefix_chord(action);
        self.chords(action)
            .iter()
            .map(|c| match prefix {
                Some(p) => format!("{} {}", p.short(), c.short()),
                None => c.short(),
            })
            .collect()
    }

    /// The first on-screen label, `""` when unbound.
    pub fn shown(&self, action: Action) -> String {
        self.shown_labels(action)
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    /// Just the key, without the prefix (`"P"` for `Ctrl+G P`): what a
    /// prefix hint prints after the prefix itself. `""` when unbound.
    pub fn shown_key(&self, action: Action) -> String {
        self.chords(action)
            .first()
            .map(KeyChord::short)
            .unwrap_or_default()
    }
}

/// `" (Ctrl+G f)"` for the effective binding of `action`, or `""` when it
/// is unbound: a message never prints an empty `()`.
pub fn paren(km: &Keymap, action: Action) -> String {
    let label = km.shown(action);
    if label.is_empty() {
        String::new()
    } else {
        format!(" ({label})")
    }
}

/// `" [m]"` for the effective binding of `action`, `""` when it is
/// unbound: the compact accelerator hint of a toggle or checkbox.
pub fn bracket(km: &Keymap, action: Action) -> String {
    let label = km.shown(action);
    if label.is_empty() {
        String::new()
    } else {
        format!(" [{label}]")
    }
}

/// `" (r: what)"` for the effective binding of `action`, `""` when it is
/// unbound: a message never prints a dangling `(: what)`.
pub fn hint(km: &Keymap, action: Action, what: &str) -> String {
    let label = km.shown(action);
    if label.is_empty() {
        String::new()
    } else {
        format!(" ({label}: {what})")
    }
}

static INSTALLED: RwLock<Option<Arc<Keymap>>> = RwLock::new(None);

/// Publishes `keymap` as the effective one (the `install_*` pattern).
pub fn install(keymap: Keymap) {
    if let Ok(mut slot) = INSTALLED.write() {
        *slot = Some(Arc::new(keymap));
    }
}

/// Builds the keymap from `config` and installs it; returns the issues.
pub fn install_from_config(config: &gitpoulpi_config::Config) -> Vec<KeymapIssue> {
    install_from_config_with(config, &install_id(), Vec::new())
}

/// [`install_from_config`] for a given installation id and the active
/// repository's own (trusted) sequences. A frontend that has neither a real
/// installation identity (unit tests) nor repository sequences passes its own.
pub fn install_from_config_with(
    config: &gitpoulpi_config::Config,
    install_id: &str,
    repo_sequences: Vec<gitpoulpi_config::sequences::SequenceDef>,
) -> Vec<KeymapIssue> {
    // The sequences first: their keys resolve through the registry.
    let refused = crate::sequences::refresh_with(config, install_id, repo_sequences);
    crate::sequences::note_config_loaded();
    let km = Keymap::from_config(&config.keybindings);
    let mut issues = km.issues.clone();
    issues.extend(refused.slugs.iter().map(|slug| KeymapIssue {
        id: format!("sequence.{slug}"),
        message: gitpoulpi_i18n::msg::keymap_issue::sequence_limit(
            slug,
            crate::sequences::MAX_SEQUENCE_SLOTS,
        ),
    }));
    install(km);
    issues
}

/// The installation id `installation`-scope sequences are matched against.
pub fn install_id() -> String {
    gitpoulpi_config::installation::current()
        .identity
        .id
        .clone()
}

/// The status line for startup / live reload: `None` when nothing was ignored.
pub fn issues_message(issues: &[KeymapIssue]) -> Option<String> {
    match issues.len() {
        0 => None,
        1 => Some(gitpoulpi_i18n::msg::keymap_issue::ignored_one(
            &issues[0].id,
            &issues[0].message,
        )),
        n => Some(gitpoulpi_i18n::msg::keymap_issue::ignored_many(
            n,
            &issues[0].id,
            &issues[0].message,
        )),
    }
}

/// The effective keymap; the defaults until something is installed.
pub fn current() -> Arc<Keymap> {
    if let Some(km) = TEST_KEYMAP.with(|t| t.borrow().clone()) {
        return km;
    }
    if let Ok(slot) = INSTALLED.read() {
        if let Some(km) = slot.as_ref() {
            return km.clone();
        }
    }
    let km = Arc::new(Keymap::defaults());
    if let Ok(mut slot) = INSTALLED.write() {
        slot.get_or_insert_with(|| km.clone()).clone()
    } else {
        km
    }
}

thread_local! {
    static TEST_KEYMAP: std::cell::RefCell<Option<Arc<Keymap>>> =
        const { std::cell::RefCell::new(None) };
}

/// Runs `f` with `keymap` as the effective one for this thread only, so a
/// remap test never disturbs the tests running beside it.
pub fn with_test_keymap<R>(keymap: Keymap, f: impl FnOnce() -> R) -> R {
    TEST_KEYMAP.with(|t| *t.borrow_mut() = Some(Arc::new(keymap)));
    let out = f();
    TEST_KEYMAP.with(|t| *t.borrow_mut() = None);
    out
}

/// A keymap from `(context, action, chords)` config entries, for tests.
pub fn test_keymap(entries: &[(&str, &str, &[&str])]) -> Keymap {
    let km = Keymap::from_config(&gitpoulpi_config::KeybindingsConfig::from_entries(
        entries
            .iter()
            .map(|(c, a, v)| (*c, *a, v.iter().map(|s| s.to_string()).collect())),
    ));
    assert!(km.issues().is_empty(), "{:?}", km.issues());
    km
}
