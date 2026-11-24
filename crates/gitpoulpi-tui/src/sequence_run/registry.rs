//! The process-wide registry of sequence slugs and their keymap slots.
//!
//! The keymap's action list is built once and never changes, so a sequence
//! cannot be an action of its own. Instead a fixed pool of
//! [`MAX_SEQUENCE_SLOTS`] actions (`Action::Sequence(slot)`) exists, and this
//! registry hands slots to slugs: append-only, a slot is never reused within
//! a process (a removed sequence keeps its slot, hidden, so ids and bindings
//! stay stable across a config reload). Past the cap a new slug is refused
//! and reported; the cap is per process, not per config.
//!
//! The registry reads no config by itself: [`refresh`] is handed one that is
//! already loaded (the action list initialises from constants only, never
//! from the config: re-entrance trap).

use gitpoulpi_config::sequences::{validate, validate_all, SequenceDef};
use gitpoulpi_config::Config;
use std::sync::Mutex;

/// How many distinct sequence slugs one process can bind keys to.
pub(crate) const MAX_SEQUENCE_SLOTS: usize = 128;

struct Slot {
    slug: &'static str,
    /// `sequence.<slug>`, leaked once with the slot.
    id: &'static str,
    name: String,
    visible: bool,
    /// The first reason `validate` gives, when the sequence is not runnable.
    problem: Option<String>,
}

struct Registry {
    slots: Vec<Slot>,
    generation: u64,
}

impl Registry {
    const fn new() -> Self {
        Registry {
            slots: Vec::new(),
            generation: 0,
        }
    }
}

/// Generations are unique process-wide (a cache keyed on one never mixes
/// two registries up).
static NEXT_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[cfg(not(test))]
static REGISTRY: Mutex<Registry> = Mutex::new(Registry::new());

// Unit tests run in parallel threads, each with a registry of its own.
#[cfg(test)]
thread_local! {
    static REGISTRY: std::cell::RefCell<Registry> = const { std::cell::RefCell::new(Registry::new()) };
}

fn with<R>(f: impl FnOnce(&mut Registry) -> R) -> R {
    #[cfg(not(test))]
    {
        f(&mut REGISTRY.lock().unwrap_or_else(|e| e.into_inner()))
    }
    #[cfg(test)]
    {
        REGISTRY.with(|r| f(&mut r.borrow_mut()))
    }
}

/// What the rest of the UI may know about one visible sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Info {
    pub(crate) slot: u16,
    pub(crate) slug: &'static str,
    pub(crate) name: String,
    /// Why it cannot run (English, from `validate`), if it cannot.
    pub(crate) problem: Option<String>,
}

fn info_of(index: usize, s: &Slot) -> Info {
    Info {
        slot: index as u16,
        slug: s.slug,
        name: s.name.clone(),
        problem: s.problem.clone(),
    }
}

/// The sequences a refresh could not take (the pool is full).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Refused {
    pub(crate) slugs: Vec<String>,
}

/// Brings the registry in line with the sequences visible to `install_id`
/// (repository sequences are not offered until their trust prompt exists).
pub(crate) fn refresh(config: &Config, install_id: &str) -> Refused {
    let defs: Vec<SequenceDef> =
        gitpoulpi_config::sequences::visible_sequences(config, install_id, None)
            .into_iter()
            .map(|(d, _)| d)
            .collect();
    refresh_defs(&defs)
}

/// [`refresh`] over an explicit list.
pub(crate) fn refresh_defs(defs: &[SequenceDef]) -> Refused {
    let duplicates: Vec<(usize, String)> = validate_all(defs)
        .into_iter()
        .map(|(i, p)| (i, p.reason))
        .collect();
    with(|reg| apply(reg, defs, &duplicates, MAX_SEQUENCE_SLOTS))
}

/// [`refresh_defs`] on a given registry with a given pool size.
fn apply(
    reg: &mut Registry,
    defs: &[SequenceDef],
    duplicates: &[(usize, String)],
    cap: usize,
) -> Refused {
    let before = fingerprint(&reg.slots);
    let mut refused = Refused::default();
    for s in &mut reg.slots {
        s.visible = false;
    }
    let mut seen: Vec<&str> = Vec::new();
    for (i, def) in defs.iter().enumerate() {
        if def.id.is_empty() || seen.contains(&def.id.as_str()) {
            continue;
        }
        seen.push(&def.id);
        let problem = validate(def)
            .into_iter()
            .next()
            .map(|p| p.reason)
            .or_else(|| {
                duplicates
                    .iter()
                    .find(|(n, _)| *n == i)
                    .map(|(_, r)| r.clone())
            });
        let index = match reg.slots.iter().position(|s| s.slug == def.id) {
            Some(index) => index,
            None if reg.slots.len() >= cap => {
                refused.slugs.push(def.id.clone());
                continue;
            }
            None => {
                let slug: &'static str = Box::leak(def.id.clone().into_boxed_str());
                let id: &'static str = Box::leak(format!("sequence.{slug}").into_boxed_str());
                reg.slots.push(Slot {
                    slug,
                    id,
                    name: String::new(),
                    visible: false,
                    problem: None,
                });
                reg.slots.len() - 1
            }
        };
        let slot = &mut reg.slots[index];
        slot.visible = true;
        slot.name = if def.name.trim().is_empty() {
            def.id.clone()
        } else {
            def.name.clone()
        };
        slot.problem = problem;
    }
    if fingerprint(&reg.slots) != before {
        reg.generation = NEXT_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    refused
}

type Print = Vec<(&'static str, bool, String, Option<String>)>;

fn fingerprint(slots: &[Slot]) -> Print {
    slots
        .iter()
        .map(|s| (s.slug, s.visible, s.name.clone(), s.problem.clone()))
        .collect()
}

/// Bumped whenever a refresh changed what is visible: palette caches key on it.
pub(crate) fn generation() -> u64 {
    with(|reg| reg.generation)
}

/// The slot of a currently visible sequence.
pub(crate) fn slot_of(slug: &str) -> Option<u16> {
    with(|reg| {
        reg.slots
            .iter()
            .position(|s| s.visible && s.slug == slug)
            .map(|i| i as u16)
    })
}

/// The sequence in `slot`, when it is visible.
pub(crate) fn info(slot: u16) -> Option<Info> {
    with(|reg| {
        let s = reg.slots.get(slot as usize)?;
        s.visible.then(|| info_of(slot as usize, s))
    })
}

/// Every visible sequence, in slot order.
pub(crate) fn visible() -> Vec<Info> {
    with(|reg| {
        reg.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.visible)
            .map(|(i, s)| info_of(i, s))
            .collect()
    })
}

/// `sequence.<slug>` of a visible slot, else `None` (the pool's placeholder
/// id is the keymap's business).
pub(crate) fn config_id(slot: u16) -> Option<&'static str> {
    with(|reg| {
        reg.slots
            .get(slot as usize)
            .filter(|s| s.visible)
            .map(|s| s.id)
    })
}

#[cfg_attr(test, allow(dead_code))]
type Stamp = (Option<std::time::SystemTime>, u64);

/// The `config.toml` the registry (and the keymap) were last built from.
#[cfg_attr(test, allow(dead_code))]
static LOADED: Mutex<Option<Stamp>> = Mutex::new(None);

#[cfg_attr(test, allow(dead_code))]
fn config_stamp() -> Option<Stamp> {
    let path = gitpoulpi_config::ConfigPaths::discover()?.config_file();
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok(), meta.len()))
}

/// Remembers which `config.toml` the keymap was just built from.
pub(crate) fn note_config_loaded() {
    #[cfg(not(test))]
    {
        *LOADED.lock().unwrap_or_else(|e| e.into_inner()) = config_stamp();
    }
}

/// Re-reads `config.toml` when it changed since the last load (the keymap
/// and the registry together: a new sequence and its binding apply live).
/// Unit tests never read the machine's config: they refresh by hand.
pub(crate) fn reload_if_changed() {
    #[cfg(not(test))]
    {
        let now = config_stamp();
        {
            let mut seen = LOADED.lock().unwrap_or_else(|e| e.into_inner());
            if *seen == now {
                return;
            }
            *seen = now;
        }
        if let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() {
            let own = crate::launcher_sync::own_config(settings.config());
            crate::keymap::install_from_config(&own);
        }
    }
}

#[cfg(test)]
pub(crate) fn test_def(id: &str) -> SequenceDef {
    let mut d = SequenceDef {
        id: id.into(),
        name: format!("Name of {id}"),
        ..SequenceDef::default()
    };
    d.steps
        .push(gitpoulpi_config::sequences::StepSpec::new("stage_all"));
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_stable_across_refreshes_and_never_reused() {
        refresh_defs(&[test_def("reg-a"), test_def("reg-b")]);
        let a = slot_of("reg-a").unwrap();
        let b = slot_of("reg-b").unwrap();
        assert_ne!(a, b);
        refresh_defs(&[test_def("reg-b"), test_def("reg-c")]);
        assert_eq!(slot_of("reg-a"), None, "removed: hidden");
        assert_eq!(slot_of("reg-b"), Some(b), "kept its slot");
        let c = slot_of("reg-c").unwrap();
        assert_ne!(c, a, "a removed slug's slot is not handed out again");
        refresh_defs(&[test_def("reg-a")]);
        assert_eq!(slot_of("reg-a"), Some(a), "comes back to its slot");
        assert_eq!(config_id(a), Some("sequence.reg-a"));
    }

    #[test]
    fn past_the_pool_a_new_slug_is_refused_and_a_known_one_is_not() {
        let mut reg = Registry::new();
        let defs: Vec<_> = (0..5).map(|n| test_def(&format!("cap-{n}"))).collect();
        let refused = apply(&mut reg, &defs[..3], &[], 3);
        assert!(refused.slugs.is_empty());
        let refused = apply(&mut reg, &defs, &[], 3);
        assert_eq!(refused.slugs, ["cap-3", "cap-4"]);
        assert_eq!(reg.slots.len(), 3);
        assert!(reg.slots.iter().all(|s| s.visible), "the known ones stay");
        assert_eq!(MAX_SEQUENCE_SLOTS, 128);
    }

    #[test]
    fn invalid_sequences_carry_their_reason_and_generation_moves() {
        let mut bad = test_def("reg-bad");
        bad.steps.clear();
        refresh_defs(&[bad.clone()]);
        let slot = slot_of("reg-bad").unwrap();
        assert!(info(slot).unwrap().problem.is_some());
        let before = generation();
        refresh_defs(&[bad]);
        assert_eq!(generation(), before, "no change, no bump");
        refresh_defs(&[test_def("reg-bad")]);
        assert_ne!(generation(), before);
        assert_eq!(info(slot).unwrap().problem, None);
    }
}
