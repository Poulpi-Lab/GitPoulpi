//! Conflict detection over an effective keymap. Pure: the
//! terminal facts are passed in, so tests are deterministic.
//!
//! Chords are compared by spelling class (`Ctrl+J` and `Ctrl+Enter` are
//! one chord on a terminal), never by raw text.

use super::chord::{commonly_taken_by, Key, KeyChord, Reach};
use super::{Action, Ctx, Keymap, ACTIONS};
use gitpoulpi_i18n::msg::keymap_conflict as msg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Level {
    /// The binding is refused (the config entry is ignored).
    Error,
    /// Allowed, shown in amber.
    Warning,
    /// Allowed, shown dim.
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Conflict {
    pub(crate) level: Level,
    pub(crate) ctx: Ctx,
    pub(crate) action: Action,
    /// The other party of a clash, when there is one.
    pub(crate) other: Option<Action>,
    pub(crate) chord: KeyChord,
    pub(crate) message: String,
}

/// What the running terminal can send; see `KeyChord::reach`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Platform {
    pub(crate) windows: bool,
    pub(crate) enhanced: bool,
}

impl Platform {
    pub(crate) fn current() -> Platform {
        Platform {
            windows: cfg!(windows),
            enhanced: crate::kitty_keyboard::is_active(),
        }
    }
}

/// The pane contexts and the Home tab are children of `global`: their
/// keys are looked up first, so a pane chord equal to a global one hides
/// it while that pane has focus.
pub(crate) const fn parent_of(ctx: Ctx) -> Option<Ctx> {
    if ctx.is_pane() || ctx.is_view() {
        Some(Ctx::Global)
    } else {
        None
    }
}

/// Defaults that deliberately hide a global key while their pane has
/// focus: Home's `f` stars a repository, `f` elsewhere searches the graph.
const SHADOWING_DEFAULTS: &[&str] = &[
    "home.favorite",
    // `q` closes a view instead of quitting; `r` flips the LFS card
    // instead of refreshing (both documented in the help overlay).
    "diff.close",
    "diff.lfs_raw",
    "file_insight.close",
];

fn deliberate_shadow(d: &super::action::ActionDef, chord: KeyChord) -> bool {
    d.shadows_parent
        || (SHADOWING_DEFAULTS.contains(&d.config_id())
            && d.defaults
                .iter()
                .filter_map(|t| KeyChord::parse(t).ok())
                .any(|c| c.class() == chord.class()))
}

fn def(action: Action) -> &'static super::action::ActionDef {
    ACTIONS
        .iter()
        .find(|d| d.action == action)
        .expect("every action is registered")
}

fn is_text_field_chord(c: KeyChord) -> bool {
    c.mods.ctrl
        && !c.mods.alt
        && !c.mods.shift
        && matches!(c.key, Key::Char('a' | 'u' | 'k' | 'c' | 'x'))
}

pub(crate) fn detect(km: &Keymap, platform: Platform) -> Vec<Conflict> {
    detect_with(km, platform, parent_of)
}

pub(crate) fn detect_with(
    km: &Keymap,
    platform: Platform,
    parent: fn(Ctx) -> Option<Ctx>,
) -> Vec<Conflict> {
    let mut out = Vec::new();
    let flat: Vec<(Action, Ctx, KeyChord)> = km
        .entries
        .iter()
        // An action this build lacks is neither offered nor in conflict.
        .filter(|e| def(e.action).built())
        .flat_map(|e| {
            e.chords
                .iter()
                .map(move |c| (e.action, e.ctx.scope(), c.class()))
        })
        .collect();

    for (i, (a, ctx_a, chord_a)) in flat.iter().enumerate() {
        let label_a = def(*a).display_label();
        // Rule 1: same context, same chord. Reported once, on the action
        // with the later id (the one a config file loses).
        for (b, ctx_b, chord_b) in &flat[i + 1..] {
            if a == b || chord_a != chord_b {
                continue;
            }
            if ctx_a == ctx_b {
                let (later, earlier) = if def(*a).config_id() > def(*b).config_id() {
                    (*a, *b)
                } else {
                    (*b, *a)
                };
                out.push(Conflict {
                    level: Level::Error,
                    ctx: *ctx_a,
                    action: later,
                    other: Some(earlier),
                    chord: *chord_a,
                    message: msg::already_bound(
                        chord_a,
                        def(earlier).display_label(),
                        ctx_a.name(),
                    ),
                });
            } else {
                // Rule 3: a child chord hiding a parent's.
                let (child, parent_action) = if parent(*ctx_a) == Some(*ctx_b) {
                    (*a, *b)
                } else if parent(*ctx_b) == Some(*ctx_a) {
                    (*b, *a)
                } else {
                    continue;
                };
                if deliberate_shadow(def(child), *chord_a) {
                    continue;
                }
                out.push(Conflict {
                    level: Level::Warning,
                    ctx: def(child).ctx,
                    action: child,
                    other: Some(parent_action),
                    chord: *chord_a,
                    message: msg::hides(
                        chord_a,
                        def(parent_action).display_label(),
                        def(child).ctx.name(),
                    ),
                });
            }
        }

        // Rule 2: a prefix is tested before everything else, so it needs a
        // modifier (a bare printable key would break typing and the panes).
        if matches!(a, Action::GlobalGitPrefix | Action::GlobalNavPrefix) {
            let printable = matches!(chord_a.key, Key::Char(_) | Key::Space)
                && !chord_a.mods.ctrl
                && !chord_a.mods.alt;
            if printable {
                out.push(Conflict {
                    level: Level::Error,
                    ctx: *ctx_a,
                    action: *a,
                    other: None,
                    chord: *chord_a,
                    message: msg::prefix_needs_modifier(label_a, chord_a),
                });
            }
        }
        // Rule 5: a text field keeps its editing chords.
        if *ctx_a == Ctx::Global && is_text_field_chord(*chord_a) {
            out.push(Conflict {
                level: Level::Warning,
                ctx: *ctx_a,
                action: *a,
                other: None,
                chord: *chord_a,
                message: msg::text_editing_chord(chord_a),
            });
        }
        // Rule 6: reach.
        match chord_a.reach(platform.windows, platform.enhanced) {
            Reach::Ok => {}
            Reach::NotDistinct(other) => out.push(Conflict {
                level: Level::Warning,
                ctx: *ctx_a,
                action: *a,
                other: None,
                chord: *chord_a,
                message: msg::sent_as(chord_a, other),
            }),
            Reach::Unsendable => out.push(Conflict {
                level: Level::Warning,
                ctx: *ctx_a,
                action: *a,
                other: None,
                chord: *chord_a,
                message: msg::not_sent(chord_a),
            }),
        }
        // Rule 7: commonly taken.
        if let Some(tool) = commonly_taken_by(*chord_a) {
            out.push(Conflict {
                level: Level::Info,
                ctx: *ctx_a,
                action: *a,
                other: None,
                chord: *chord_a,
                message: msg::commonly_taken(chord_a, tool.get()),
            });
        }
    }
    out.sort_by_key(|c| c.level);
    out
}

/// Why `chord` cannot be bound in `ctx`, when it is one of the keys that
/// stay fixed: Esc everywhere, and in `global` and the pane
/// contexts the motion and navigation keys and the digits (the vim
/// count prefix). The prefix tables take any key but Esc.
pub(crate) fn fixed_reason(ctx: Ctx, chord: KeyChord) -> Option<gitpoulpi_i18n::M> {
    let c = chord.class();
    if c.key == Key::Esc {
        return Some(msg::FIXED_ESC);
    }
    let plain = !c.mods.ctrl && !c.mods.alt;
    if matches!(ctx, Ctx::Editor | Ctx::Merge) && c.mods.ctrl && !c.mods.alt && !c.mods.shift {
        if let Key::Char('a' | 'u' | 'k' | 'c' | 'x') = c.key {
            return Some(msg::FIXED_EDITING_CHORD);
        }
    }
    if plain && (matches!(ctx, Ctx::Editor | Ctx::Merge) || ctx.is_popup()) {
        match c.key {
            Key::Enter
            | Key::Tab
            | Key::BackTab
            | Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::Home
            | Key::End
            | Key::Backspace
            | Key::Delete
            | Key::Space => return Some(msg::FIXED_NAVIGATION_EDITING),
            _ => {}
        }
    }
    if plain && ctx.is_typing() && c.key != Key::Char('/') {
        if let Key::Char(_) = c.key {
            return Some(msg::FIXED_TYPING_KEY);
        }
    }
    if !(ctx == Ctx::Global || ctx.is_pane() || ctx.is_view()) || !plain {
        return None;
    }
    match c.key {
        // Enter and Space stage in the diff, and Enter opens a blamed
        // row's commit: shortcuts there, not motion.
        Key::Enter if ctx.is_view() => None,
        Key::Enter
        | Key::Tab
        | Key::BackTab
        | Key::Up
        | Key::Down
        | Key::Left
        | Key::Right
        | Key::PageUp
        | Key::PageDown
        | Key::Home
        | Key::End
        | Key::Backspace => Some(msg::FIXED_NAVIGATION),
        Key::Char('h' | 'j' | 'k' | 'l') if !c.mods.shift => Some(msg::FIXED_HJKL),
        Key::Char('0'..='9') => Some(msg::FIXED_DIGITS),
        _ => None,
    }
}

/// What binding `new` to `action` would do, for the capture popup and
/// the tests. Nothing is saved here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Analysis {
    /// Why the binding cannot be saved at all.
    pub(crate) refusal: Option<String>,
    /// A same-context action that already has the chord: saving replaces
    /// (unbinds) it, after the user chose Replace.
    pub(crate) replace: Option<Action>,
    /// Warnings and hints, worst first.
    pub(crate) notes: Vec<(Level, String)>,
}

pub(crate) fn analyse(
    km: &Keymap,
    platform: Platform,
    action: Action,
    new: &[KeyChord],
) -> Analysis {
    let d = def(action);
    let mut out = Analysis::default();
    if !d.rebindable {
        out.refusal = Some(msg::THIS_KEY_FIXED.get().to_string());
        return out;
    }
    for chord in new {
        if let Some(reason) = fixed_reason(d.ctx.scope(), *chord) {
            out.refusal = Some(msg::chord_reason(chord, reason.get()));
            return out;
        }
        if d.ctx.ignores_modifiers() && chord.mods.alt {
            out.refusal = Some(msg::no_modifier_after_prefix(chord));
            return out;
        }
    }
    let hypothetical = km.with_chords(action, new.to_vec());
    for c in detect(&hypothetical, platform) {
        let involved = c.action == action || c.other == Some(action);
        if !involved || !new.iter().any(|n| n.class() == c.chord.class()) {
            continue;
        }
        if c.level == Level::Error {
            match (c.action == action, c.other) {
                (_, None) => out.refusal = Some(c.message.clone()),
                (mine, Some(other)) => {
                    let theirs = if mine { other } else { c.action };
                    if out.replace.is_none() {
                        out.replace = Some(theirs);
                    }
                    out.notes.push((Level::Error, c.message.clone()));
                }
            }
        } else {
            out.notes.push((c.level, c.message.clone()));
        }
    }
    out
}
