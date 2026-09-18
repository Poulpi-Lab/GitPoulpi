//! The current theme: one `Acquire` load per [`th`] call, an interned
//! leaked `&'static Theme` behind it (plan 2.3), and a generation counter
//! for caches that bake colors in.

use std::collections::HashMap;
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use gitpoulpi_config::theme::{preset, ThemeSpec, DEFAULT_SLUG};

use super::roles::Theme;

static CURRENT: AtomicPtr<Theme> = AtomicPtr::new(std::ptr::null_mut());
static GENERATION: AtomicU64 = AtomicU64::new(1);
static SYSTEM_IS_LIGHT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What `system` resolves to: Light when start-up detection found a light
/// terminal background, Dark otherwise (not detected, or dark).
pub(crate) fn set_system_is_light(light: bool) {
    SYSTEM_IS_LIGHT.store(light, Ordering::Relaxed);
}

fn dark() -> &'static Theme {
    static DARK: OnceLock<&'static Theme> = OnceLock::new();
    DARK.get_or_init(|| intern(Theme::dark()))
}

/// Interned themes by slug: switching back and forth leaks nothing new
/// (an identical theme reuses its entry; an edited user file adds one).
fn intern(theme: Theme) -> &'static Theme {
    static POOL: Mutex<Option<HashMap<&'static str, Vec<&'static Theme>>>> = Mutex::new(None);
    let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
    let list = pool
        .get_or_insert_with(HashMap::new)
        .entry(theme.slug)
        .or_default();
    if let Some(found) = list.iter().find(|t| ***t == theme) {
        return found;
    }
    let leaked: &'static Theme = Box::leak(Box::new(theme));
    list.push(leaked);
    leaked
}

#[cfg(test)]
thread_local! {
    static OVERRIDE: std::cell::Cell<Option<(&'static Theme, u64)>> =
        const { std::cell::Cell::new(None) };
}

/// The current theme.
pub(crate) fn th() -> &'static Theme {
    #[cfg(test)]
    if let Some((t, _)) = OVERRIDE.with(std::cell::Cell::get) {
        return t;
    }
    let p = CURRENT.load(Ordering::Acquire);
    if p.is_null() {
        dark()
    } else {
        // SAFETY: only ever set from an interned, leaked `&'static Theme`.
        unsafe { &*p }
    }
}

/// Bumped by every [`install_theme`]; caches that bake colors compare it.
pub(crate) fn theme_generation() -> u64 {
    #[cfg(test)]
    if let Some((_, g)) = OVERRIDE.with(std::cell::Cell::get) {
        return g;
    }
    GENERATION.load(Ordering::Acquire)
}

/// Resolves `slug` (a user theme first, then a preset; `system` and unknown
/// names fall back to Dark) to its interned theme.
pub(super) fn resolve_theme(slug: &str, user_themes: &[ThemeSpec]) -> &'static Theme {
    let slug = if slug == gitpoulpi_config::THEME_SYSTEM && SYSTEM_IS_LIGHT.load(Ordering::Relaxed)
    {
        "light"
    } else {
        slug
    };
    let spec = user_themes
        .iter()
        .find(|t| t.slug == slug)
        .cloned()
        .or_else(|| preset(slug))
        .or_else(|| preset(DEFAULT_SLUG))
        .expect("the default preset exists");
    intern(Theme::from(&spec))
}

/// Makes the theme for `slug` current (see [`resolve_theme`]).
pub(crate) fn install_theme(slug: &str, user_themes: &[ThemeSpec]) {
    // A test that pinned its thread's theme must not move the global one
    // under the tests running beside it.
    #[cfg(test)]
    if OVERRIDE.with(std::cell::Cell::get).is_some() {
        return;
    }
    let theme = resolve_theme(slug, user_themes);
    let ptr = theme as *const Theme as *mut Theme;
    if CURRENT.swap(ptr, Ordering::AcqRel) != ptr {
        GENERATION.fetch_add(1, Ordering::AcqRel);
    }
}

/// Makes the user themes under `<config>/themes` (if any) available and
/// installs `slug`. Warnings are returned for the caller to surface.
pub(crate) fn install_theme_from_config(slug: &str) -> Vec<String> {
    let set = gitpoulpi_config::ConfigPaths::discover()
        .map(|p| gitpoulpi_config::theme::load_user_themes(&p.themes_dir()))
        .unwrap_or_default();
    install_theme(slug, &set.themes);
    set.warnings
}

/// Test-only: makes `theme` current for this thread until the guard drops,
/// so parallel tests never race on the global. Has its own generation.
#[cfg(test)]
pub(crate) struct ThemeGuard(Option<(&'static Theme, u64)>);

#[cfg(test)]
pub(crate) fn override_theme(theme: &Theme) -> ThemeGuard {
    static NEXT: AtomicU64 = AtomicU64::new(1 << 40);
    let leaked: &'static Theme = Box::leak(Box::new(*theme));
    let gen = NEXT.fetch_add(1, Ordering::Relaxed);
    ThemeGuard(OVERRIDE.with(|o| o.replace(Some((leaked, gen)))))
}

#[cfg(test)]
impl Drop for ThemeGuard {
    fn drop(&mut self) {
        OVERRIDE.with(|o| o.set(self.0));
    }
}
