//! Which GitPoulpi installation is this, and what are its own settings.
//!
//! A machine can hold several installations at once (a Chocolatey package, a
//! dev checkout, a cargo build, an unpacked archive). Each gets a stable
//! identity here, plus its own entry in `config.toml`'s `[installation]`
//! table (the stored shape is in `schema/installation.rs`), so one
//! installation's colour, names, aliases and environment never leak into
//! another's. Everything in this module is pure: the file system, the OS
//! flavour, `$CARGO_HOME` and the environment variables arrive through
//! [`IdentityEnv`], so tests never look at the real machine. Paths are
//! handled as text with an explicit Windows/other flavour, so a test can
//! check Windows spellings on Linux.
//!
//! **Primary.** Nobody is primary by default. The primary owns the legacy
//! names and the legacy `ui.*` keys (old builds only know those); every other
//! installation keeps its look in its entry. See [`effective`].

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::release::{InstallChannel, Marker, MARKER_FILE};
use crate::{
    insert_dotted, remove_dotted, secure_fs, AiConfig, Config, Error, InstallationConfig,
    InstallationEntry, Kind, Result, UiBefore,
};

/// The plain launcher name, owned by the primary installation only.
pub const PRIMARY_NAME: &str = "GitPoulpi";
/// Pins the identity (tests, scripts): id `p-<sanitised value>`, kind `pinned`.
pub const INSTALL_ID_ENV: &str = "GITPOULPI_INSTALL_ID";
/// Overrides the environment (daemons, sessions, undo journal) of any installation.
pub const ENV_OVERRIDE_ENV: &str = "GITPOULPI_ENV";
/// The colour a dev checkout wears (it must not look like a release).
pub const DEV_LOGO_COLOR: &str = "sun";
/// Colours given to a new installation whose preferred colour is taken, in order.
pub const SEED_COLORS: [&str; 5] = ["teal", "orange", "magenta", "green", "ocean"];

// ---------------------------------------------------------------- identity

/// The reads [`identify`] needs of the file system. Text paths, so a fake
/// can speak Windows on any host.
pub trait InstallFs {
    /// The absolute, link-free spelling of `path`; `None` when it cannot be resolved.
    fn canonicalize(&self, path: &str) -> Option<String>;
    fn is_file(&self, path: &str) -> bool;
    fn is_dir(&self, path: &str) -> bool;
    fn read_to_string(&self, path: &str) -> Option<String>;
}

/// The real file system.
pub struct RealFs;

impl InstallFs for RealFs {
    fn canonicalize(&self, path: &str) -> Option<String> {
        std::fs::canonicalize(path)
            .ok()
            .map(|p| p.to_string_lossy().into_owned())
    }
    fn is_file(&self, path: &str) -> bool {
        Path::new(path).is_file()
    }
    fn is_dir(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }
    fn read_to_string(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
}

/// Everything [`identify`] may depend on besides the exe path.
pub struct IdentityEnv<'a> {
    /// Windows path rules: `\` separators, case-insensitive.
    pub windows: bool,
    /// The target triple a release marker must name to count as ours.
    pub host_target: String,
    /// `$CARGO_HOME`, or `~/.cargo`.
    pub cargo_home: Option<String>,
    /// The value of [`INSTALL_ID_ENV`].
    pub install_id: Option<String>,
    /// The value of [`ENV_OVERRIDE_ENV`].
    pub env_override: Option<String>,
    pub fs: &'a dyn InstallFs,
}

impl<'a> IdentityEnv<'a> {
    /// The running machine. Production only: tests build the struct by hand.
    pub fn real(fs: &'a RealFs) -> Self {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        Self {
            windows: cfg!(windows),
            host_target: crate::release::host_target(),
            cargo_home: var("CARGO_HOME").or_else(|| {
                crate::home_dir().map(|h| h.join(".cargo").to_string_lossy().into_owned())
            }),
            install_id: var(INSTALL_ID_ENV),
            env_override: var(ENV_OVERRIDE_ENV),
            fs,
        }
    }
}

/// Who an exe is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// 16 hex digits (FNV-1a-64 of the normalised root), or `p-<name>` when pinned.
    pub id: String,
    pub kind: Kind,
    /// The installation folder, cleaned but with its case kept (display, and
    /// the entry's `path`).
    pub root: String,
    /// The marker's channel, for an `install`; `None` for a legacy marker.
    pub channel: Option<InstallChannel>,
}

impl Identity {
    /// `i<8hex>`: the id in file names, window classes and GTK ids.
    pub fn short(&self) -> String {
        short_id(&self.id)
    }
}

/// Hand-written so the id never changes with a toolchain (`DefaultHasher` does).
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// `i<8hex>` for an id. A hashed id already is hex; a pinned one is hashed first.
pub fn short_id(id: &str) -> String {
    let hex = if id.len() == 16 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        id.to_string()
    } else {
        format!("{:016x}", fnv1a64(id.as_bytes()))
    };
    format!("i{}", &hex[..8])
}

/// A name that is safe as a folder name and an env value: lower-case ASCII
/// letters, digits, `-` and `_`; anything else becomes one `-`. Dots go too,
/// so a value can never be `..`.
pub fn sanitize(text: &str) -> String {
    let mut out = String::new();
    for c in text.trim().chars() {
        let c = if c.is_ascii_alphanumeric() || c == '_' {
            c.to_ascii_lowercase()
        } else {
            '-'
        };
        if c == '-' && out.ends_with('-') {
            continue;
        }
        out.push(c);
    }
    out.trim_matches('-').chars().take(48).collect()
}

fn sep(windows: bool) -> char {
    if windows {
        '\\'
    } else {
        '/'
    }
}

/// Drops the `\\?\` prefix canonicalisation adds on Windows (`\\?\UNC\srv\x`
/// becomes `\\srv\x`), unifies separators on Windows and removes trailing ones.
fn tidy(path: &str, windows: bool) -> String {
    let path = if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else {
        path.strip_prefix(r"\\?\").unwrap_or(path).to_string()
    };
    let mut path = if windows {
        path.replace('/', "\\")
    } else {
        path
    };
    let s = sep(windows);
    while path.len() > 1 && path.ends_with(s) {
        // `C:\` keeps its separator: `C:` alone means "the current folder of C".
        if windows && path.len() == 3 && path.as_bytes()[1] == b':' {
            break;
        }
        path.pop();
    }
    path
}

/// The spelling two paths are compared and hashed by.
fn norm(path: &str, windows: bool) -> String {
    let t = tidy(path, windows);
    if windows {
        t.to_lowercase()
    } else {
        t
    }
}

fn parent(path: &str, windows: bool) -> Option<String> {
    let i = path.rfind(sep(windows))?;
    Some(if i == 0 {
        sep(windows).to_string()
    } else {
        path[..i].to_string()
    })
}

fn file_name(path: &str, windows: bool) -> &str {
    path.rsplit(sep(windows)).next().unwrap_or(path)
}

fn join(dir: &str, name: &str, windows: bool) -> String {
    if dir.ends_with(sep(windows)) {
        format!("{dir}{name}")
    } else {
        format!("{dir}{}{name}", sep(windows))
    }
}

fn same_text(a: &str, b: &str, windows: bool) -> bool {
    if windows {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}

/// `<root>` of `<root>/target/<profile>/<exe>` when it is a GitPoulpi
/// checkout. The same test as [`crate::dev_mode::checkout_root`], over the
/// injected file system.
fn checkout_root(exe: &str, env: &IdentityEnv) -> Option<String> {
    let w = env.windows;
    let profile = parent(exe, w)?;
    let target = parent(&profile, w)?;
    if !same_text(file_name(&target, w), "target", w) {
        return None;
    }
    let root = parent(&target, w)?;
    let crates = join(&root, "crates", w);
    (env.fs.is_file(&join(&root, "Cargo.toml", w))
        && env.fs.is_dir(&join(&crates, "gitpoulpi-tui", w)))
    .then_some(root)
}

/// Scoop keeps each version in `apps\<app>\<ver>` beside a `current`
/// junction; the version folder maps to `current` so an update keeps the
/// identity. By path pattern only (plus the `current` sibling).
fn scoop_current(dir: &str, env: &IdentityEnv) -> Option<String> {
    let w = env.windows;
    let app = parent(dir, w)?;
    let apps = parent(&app, w)?;
    if !same_text(file_name(&apps, w), "apps", w) {
        return None;
    }
    let current = join(&app, "current", w);
    env.fs.is_dir(&current).then_some(current)
}

/// Homebrew keeps each version in `Cellar/<formula>/<ver>/bin`; the formula
/// folder is the installation.
fn cellar_formula(dir: &str, env: &IdentityEnv) -> Option<String> {
    let w = env.windows;
    if file_name(dir, w) != "bin" {
        return None;
    }
    let version = parent(dir, w)?;
    let formula = parent(&version, w)?;
    let cellar = parent(&formula, w)?;
    (file_name(&cellar, w) == "Cellar").then_some(formula)
}

fn hashed(kind: Kind, root: String, channel: Option<InstallChannel>, w: bool) -> Identity {
    Identity {
        id: format!("{:016x}", fnv1a64(norm(&root, w).as_bytes())),
        kind,
        root,
        channel,
    }
}

/// Who `exe` is. First match wins:
///
/// 1. [`IdentityEnv::install_id`] set: `p-<sanitised>`, kind `pinned`.
/// 2. The exe is `<checkout>/target/<profile>/<file>`: kind `dev`, root = the
///    checkout. Checked before the marker, so a marker under `target/` cannot
///    split a checkout into one installation per profile.
/// 3. A `.gitpoulpi-dist` for the host target next to the exe: kind `install`.
///    Scoop and Homebrew version folders map to the stable folder.
/// 4. The exe's folder is `$CARGO_HOME/bin`: kind `cargo`.
/// 5. Otherwise `portable`, root = the exe's folder.
///
/// The id hashes the root only, never the kind or the channel: a legacy
/// marker gains a `channel=` at its first update, which must not change it.
pub fn identify(exe: &Path, env: &IdentityEnv) -> Identity {
    let w = env.windows;
    let given = exe.to_string_lossy();
    let canonical = env
        .fs
        .canonicalize(&given)
        .unwrap_or_else(|| given.into_owned());
    let exe = tidy(&canonical, w);
    let dir = parent(&exe, w).unwrap_or_default();

    if let Some(name) = env
        .install_id
        .as_deref()
        .map(sanitize)
        .filter(|n| !n.is_empty())
    {
        return Identity {
            id: format!("p-{name}"),
            kind: Kind::Pinned,
            root: dir,
            channel: None,
        };
    }
    if let Some(root) = checkout_root(&exe, env) {
        return hashed(Kind::Dev, root, None, w);
    }
    let marker = env
        .fs
        .read_to_string(&join(&dir, MARKER_FILE, w))
        .and_then(|text| Marker::parse(&text))
        .filter(|m| m.target == env.host_target);
    if let Some(marker) = marker {
        let root = if w {
            scoop_current(&dir, env)
        } else {
            cellar_formula(&dir, env)
        }
        .unwrap_or_else(|| dir.clone());
        return hashed(Kind::Install, root, marker.channel, w);
    }
    let cargo_bin = env.cargo_home.as_deref().map(|h| join(h, "bin", w));
    if cargo_bin.is_some_and(|bin| norm(&bin, w) == norm(&dir, w)) {
        return hashed(Kind::Cargo, dir, None, w);
    }
    hashed(Kind::Portable, dir, None, w)
}

/// The environment (daemons, sessions, undo journal) an installation joins.
/// `GITPOULPI_ENV` beats the entry's `env`, which beats the installation's own
/// id, so by default every installation is its own world, and a dev
/// checkout's dist and release builds (one installation) share theirs.
pub fn environment(config: &Config, identity: &Identity, env: &IdentityEnv) -> String {
    let usable = |t: &str| Some(sanitize(t)).filter(|s| !s.is_empty());
    env.env_override
        .as_deref()
        .and_then(usable)
        .or_else(|| {
            config
                .installation
                .settings
                .get(&identity.id)
                .and_then(|e| usable(&e.env))
        })
        .unwrap_or_else(|| sanitize(&identity.id))
}

// ----------------------------------------------------------------- names

/// The English word for an installation inside its launcher name. English on
/// purpose (`// i18n-english`): it is written into OS launcher names that other
/// installations and old builds must read the same way.
pub fn label(kind: Kind, channel: Option<InstallChannel>) -> &'static str {
    match kind {
        Kind::Dev => "dev",
        Kind::Cargo => "cargo",
        Kind::Pinned => "pinned",
        Kind::Portable => "portable",
        // A legacy marker has no channel and is an archive in practice. An
        // archive release put in place by the install script is installed,
        // not a movable portable copy.
        Kind::Install => match channel {
            None | Some(InstallChannel::Archive) => "release",
            Some(InstallChannel::InnoUser) => "per-user",
            Some(InstallChannel::InnoMachine) => "all users",
            Some(InstallChannel::Choco | InstallChannel::ChocoInternal) => "Chocolatey",
            Some(InstallChannel::Winget) => "winget",
            Some(InstallChannel::Scoop) => "Scoop",
            Some(InstallChannel::Msi) => "MSI",
            Some(InstallChannel::Homebrew) => "Homebrew",
            Some(InstallChannel::Apt) => "apt",
            Some(InstallChannel::Aur) => "AUR",
            Some(InstallChannel::Docker) => "Docker",
        },
    }
}

/// `GitPoulpi (<label>)`. Even a lone installation gets the suffix: a frozen
/// name never changes when a second installation appears, and the plain name
/// stays free for the primary and for old builds.
pub fn default_name(kind: Kind, channel: Option<InstallChannel>) -> String {
    format!("{PRIMARY_NAME} ({})", label(kind, channel))
}

/// `base`, or `base 2`, `base 3`... when a name in `taken` already equals it
/// (compared trimmed and case-insensitively, like the file systems that hold
/// the launchers).
pub fn unique_name(base: &str, taken: &[String]) -> String {
    let key = |s: &str| s.trim().to_lowercase();
    let taken: BTreeSet<String> = taken.iter().map(|t| key(t)).collect();
    if !taken.contains(&key(base)) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base} {n}"))
        .find(|c| !taken.contains(&key(c)))
        .expect("an unbounded range always yields")
}

/// Why a launcher name is refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NameError {
    #[error("the name is empty")]
    Empty,
    #[error("the name is longer than 64 characters")]
    TooLong,
    #[error("the name contains {0:?}, which a launcher file name cannot hold")]
    BadChar(char),
    #[error("the name ends with a dot or a space")]
    TrailingDotOrSpace,
    #[error("{0} is a reserved device name on Windows")]
    Reserved(String),
}

/// A launcher name becomes a file name on every OS, so it must be a valid one
/// on all of them: 1-64 characters, none of `\/:*?"<>|`, no control
/// characters, no trailing dot or space, not a Windows device name.
pub fn validate_name(name: &str) -> std::result::Result<(), NameError> {
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    if name.chars().count() > 64 {
        return Err(NameError::TooLong);
    }
    if let Some(c) = name
        .chars()
        .find(|c| c.is_control() || "\\/:*?\"<>|".contains(*c))
    {
        return Err(NameError::BadChar(c));
    }
    if name.ends_with('.') || name.ends_with(' ') {
        return Err(NameError::TrailingDotOrSpace);
    }
    // `CON.txt` is as reserved as `CON`.
    let stem = name.split('.').next().unwrap_or(name).trim_end();
    let upper = stem.to_ascii_uppercase();
    let numbered = |p: &str| {
        upper
            .strip_prefix(p)
            .is_some_and(|n| n.len() == 1 && matches!(n.as_bytes()[0], b'1'..=b'9'))
    };
    if ["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str()) || numbered("COM") || numbered("LPT")
    {
        return Err(NameError::Reserved(stem.to_string()));
    }
    Ok(())
}

/// `<major>.<minor>` of a version (`1.2.3-rc1` gives `1.2`).
pub fn major_minor(version: &str) -> String {
    let mut parts = version.split('.').map(|p| {
        let digits: String = p.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() {
            "0".to_string()
        } else {
            digits
        }
    });
    let major = parts.next().unwrap_or_else(|| "0".to_string());
    format!(
        "{major}.{}",
        parts.next().unwrap_or_else(|| "0".to_string())
    )
}

/// The name a launcher shows: `base` as is, or, when `with_version`, with
/// `<major>.<minor>` after a leading "GitPoulpi" (`GitPoulpi 1.2 (Scoop)`),
/// else appended. Only the base is ever stored, so an update causes no config
/// churn.
pub fn launcher_name(base: &str, with_version: bool, version: &str) -> String {
    if !with_version {
        return base.to_string();
    }
    let v = major_minor(version);
    match base.strip_prefix(PRIMARY_NAME) {
        Some(rest) if !rest.chars().next().is_some_and(char::is_alphanumeric) => {
            format!("{PRIMARY_NAME} {v}{rest}")
        }
        _ => format!("{base} {v}"),
    }
}

// -------------------------------------------------------- effective values

/// The `ui.*` keys that are purely about colour and therefore belong to one
/// installation each (the primary keeps them in `ui.*`, see
/// [`InstallationEntry::look`]).
pub const LOOK_KEYS: [&str; 5] = [
    "theme",
    "graph_color_theme",
    "graph_colorization",
    "graph_lane_background",
    "graph_row_tint_strength",
];

/// Whether a dotted config key (`ui.theme`) is one of [`LOOK_KEYS`].
pub fn is_look_key(key: &str) -> bool {
    key.strip_prefix("ui.")
        .is_some_and(|k| LOOK_KEYS.contains(&k))
}

/// The value `ui` holds for a look key, as the text stored in the file.
pub fn look_value(ui: &crate::UiConfig, key: &str) -> String {
    match key {
        "theme" => ui.theme_slug().to_string(),
        "graph_color_theme" => ui.graph_color_slug().to_string(),
        _ => serde_json::to_value(ui)
            .ok()
            .and_then(|v| v.get(key).and_then(|s| s.as_str().map(str::to_string)))
            .unwrap_or_default(),
    }
}

/// `config` with the look keys of `eff` written into `ui`: what this
/// installation shows, whatever another installation stored.
pub fn overlay_look(config: &Config, eff: &Effective) -> Config {
    let mut value = serde_json::to_value(&config.ui).unwrap_or_default();
    if let Some(map) = value.as_object_mut() {
        for (k, v) in &eff.look {
            map.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
    }
    let mut out = config.clone();
    if let Ok(ui) = serde_json::from_value(value) {
        out.ui = ui;
    }
    out
}

/// What an installation actually uses, after the fallbacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effective {
    pub primary: bool,
    /// The base launcher name (see [`launcher_name`] for the version).
    pub name: String,
    pub name_with_version: bool,
    pub logo_color: String,
    /// The colour keys in force, by [`LOOK_KEYS`] name.
    pub look: BTreeMap<String, String>,
    pub command_alias: String,
    pub command_alias_add: String,
    pub folder_menu_submenu: String,
    /// The configured environment: the entry's, else the id. The one in force
    /// also honours `GITPOULPI_ENV`, see [`environment`].
    pub env: String,
    pub mcp_bind: String,
}

fn effective_from(
    config: &Config,
    id: &str,
    kind: Kind,
    channel: Option<InstallChannel>,
) -> Effective {
    let entry = config.installation.settings.get(id);
    let blank = InstallationEntry::default();
    let e = entry.unwrap_or(&blank);
    let primary = config.installation.primary.as_deref() == Some(id);
    // The entry wins; the primary falls back to the shared `ui.*` keys that
    // old builds read; everyone else to the kind's default.
    let pick = |own: &str, legacy: &str, default: &str| {
        if !own.is_empty() {
            own.to_string()
        } else if primary && !legacy.is_empty() {
            legacy.to_string()
        } else {
            default.to_string()
        }
    };
    // `ui.*` may hold what a 1.0 build wrote for another installation: a name
    // another entry holds stays that installation's, the primary's too.
    let ui_alias = |name: &str| {
        if claimed_by_another(config, id, name) {
            String::new()
        } else {
            name.to_string()
        }
    };
    let kind_color = if kind == Kind::Dev {
        DEV_LOGO_COLOR
    } else {
        ""
    };
    Effective {
        primary,
        // `GitPoulpi` belongs to the primary; the entry's name waits for the
        // day it stops being primary.
        name: if primary {
            PRIMARY_NAME.to_string()
        } else if e.name.is_empty() {
            default_name(kind, channel)
        } else {
            e.name.clone()
        },
        name_with_version: e.name_with_version,
        logo_color: pick(&e.logo_color, &config.ui.logo_color, kind_color),
        look: LOOK_KEYS
            .iter()
            .map(|k| {
                let own = e.look.get(*k).map(String::as_str).unwrap_or("");
                let legacy = if primary {
                    look_value(&config.ui, k)
                } else {
                    String::new()
                };
                let default = look_value(&crate::UiConfig::default(), k);
                (k.to_string(), pick(own, &legacy, &default))
            })
            .collect(),
        command_alias: pick(&e.command_alias, &ui_alias(&config.ui.command_alias), ""),
        command_alias_add: pick(
            &e.command_alias_add,
            &ui_alias(&config.ui.command_alias_add),
            "",
        ),
        folder_menu_submenu: e.folder_menu_submenu.clone(),
        env: if e.env.trim().is_empty() {
            id.to_string()
        } else {
            e.env.trim().to_string()
        },
        mcp_bind: pick(&e.mcp_bind, "", &config.mcp.bind),
    }
}

/// The values `identity` uses: its entry's field if set, else `ui.*` when it is
/// the primary, else the kind's default (dev wears `sun`).
pub fn effective(config: &Config, identity: &Identity) -> Effective {
    let channel = identity.channel.or_else(|| {
        config
            .installation
            .settings
            .get(&identity.id)
            .and_then(|e| InstallChannel::parse(&e.channel))
    });
    effective_from(config, &identity.id, identity.kind, channel)
}

/// [`effective`] for an installation known only by its entry (the "other
/// installations" of Settings); `None` without an entry.
pub fn effective_of(config: &Config, id: &str) -> Option<Effective> {
    let e = config.installation.settings.get(id)?;
    Some(effective_from(
        config,
        id,
        e.kind,
        InstallChannel::parse(&e.channel),
    ))
}

/// Which installation each alias name belongs to: every entry's own
/// `command_alias(_add)`, then the primary's `ui.*` fallback for the names no
/// entry claims (`ui.*` may hold what a 1.0 build wrote for another
/// installation, whose entry then wins). Names are lower-cased.
pub fn alias_claims(config: &Config) -> Vec<(String, String)> {
    let mut claims: Vec<(String, String)> = Vec::new();
    let mut claim = |name: &str, id: &str| {
        let name = name.trim().to_ascii_lowercase();
        if !name.is_empty() && !claims.iter().any(|(n, _)| *n == name) {
            claims.push((name, id.to_string()));
        }
    };
    for (id, e) in &config.installation.settings {
        claim(&e.command_alias, id);
        claim(&e.command_alias_add, id);
    }
    if let Some(primary) = config.installation.primary.as_deref() {
        claim(&config.ui.command_alias, primary);
        claim(&config.ui.command_alias_add, primary);
    }
    claims
}

/// Does an installation other than `id` have `name` in its own entry?
fn claimed_by_another(config: &Config, id: &str, name: &str) -> bool {
    !name.is_empty()
        && config.installation.settings.iter().any(|(other, e)| {
            other != id
                && [&e.command_alias, &e.command_alias_add]
                    .iter()
                    .any(|n| n.eq_ignore_ascii_case(name))
        })
}

// ----------------------------------------------------------------- seeding

/// The file inspection behind alias migration: where does the alias file
/// named `alias` point? (Production reads the shim or symlink; tests inject.)
pub trait AliasEvidence {
    /// The exe the alias runs, `None` when there is no such alias.
    fn target_of(&self, alias: &str) -> Option<String>;
}

fn new_entry(identity: &Identity, name: String) -> InstallationEntry {
    InstallationEntry {
        path: identity.root.clone(),
        kind: identity.kind,
        channel: identity
            .channel
            .map(|c| c.as_str().to_string())
            .unwrap_or_default(),
        name,
        ..InstallationEntry::default()
    }
}

/// The entry to write for `identity` at its first run, or to refresh when its
/// folder or channel changed; `None` when the stored entry is already right.
/// `is_live` says whether another entry's installation still exists (its path
/// is there): a gone installation neither blocks a name nor wears a colour.
///
/// **First run** freezes:
/// - the name: the default, numbered when a live installation has it;
/// - the colour: dev wears `sun`; anything else keeps today's `ui.logo_color`
///   unless a live installation wears it, then the first free of
///   [`SEED_COLORS`] (so the first non-dev installation keeps its colour);
/// - the aliases: each `ui.command_alias(_add)` name whose file points at
///   *this* installation (`aliases`, resolved with [`identify`]).
///
/// `ui.*` is never moved or deleted: old builds still read it.
pub fn seed(
    config: &Config,
    identity: &Identity,
    env: &IdentityEnv,
    aliases: &dyn AliasEvidence,
    is_live: &dyn Fn(&InstallationEntry) -> bool,
) -> Option<InstallationEntry> {
    let w = env.windows;
    if let Some(existing) = config.installation.settings.get(&identity.id) {
        let mut updated = existing.clone();
        if norm(&existing.path, w) != norm(&identity.root, w) {
            updated.path = identity.root.clone();
        }
        // A legacy marker learns its channel at its first update.
        if let Some(c) = identity.channel {
            updated.channel = c.as_str().to_string();
        }
        updated.kind = identity.kind;
        // An entry from before the colours were per installation follows
        // today's `ui.*` once, so nothing changes on screen; from then on it
        // keeps its own. The primary keeps following `ui.*`.
        if config.installation.primary.as_deref() != Some(&identity.id) {
            for k in LOOK_KEYS {
                updated
                    .look
                    .entry(k.to_string())
                    .or_insert_with(|| look_value(&config.ui, k));
            }
        }
        return (updated != *existing).then_some(updated);
    }

    let others: Vec<(&String, &InstallationEntry)> = config
        .installation
        .settings
        .iter()
        .filter(|(id, e)| **id != identity.id && is_live(e))
        .collect();
    let taken_names: Vec<String> = others
        .iter()
        .filter_map(|(id, _)| effective_of(config, id))
        .map(|e| e.name)
        .collect();
    let worn: Vec<String> = others
        .iter()
        .filter_map(|(id, _)| effective_of(config, id))
        .map(|e| e.logo_color)
        .collect();

    let mut entry = new_entry(
        identity,
        unique_name(&default_name(identity.kind, identity.channel), &taken_names),
    );
    entry.logo_color = if identity.kind == Kind::Dev {
        DEV_LOGO_COLOR.to_string()
    } else if !worn.contains(&config.ui.logo_color) {
        config.ui.logo_color.clone()
    } else {
        SEED_COLORS
            .iter()
            .find(|c| !worn.iter().any(|w| w == **c))
            .unwrap_or(&SEED_COLORS[0])
            .to_string()
    };
    if config.installation.primary.as_deref() != Some(&identity.id) {
        for k in LOOK_KEYS {
            entry.look.insert(k.to_string(), look_value(&config.ui, k));
        }
    }
    let mine = |alias: &str| {
        !alias.is_empty()
            && aliases
                .target_of(alias)
                .is_some_and(|t| identify(Path::new(&t), env).id == identity.id)
    };
    if mine(&config.ui.command_alias) {
        entry.command_alias = config.ui.command_alias.clone();
    }
    if mine(&config.ui.command_alias_add) {
        entry.command_alias_add = config.ui.command_alias_add.clone();
    }
    Some(entry)
}

// ----------------------------------------------------------------- primary

/// The legacy `ui.*` values the primary owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLook {
    pub logo_color: String,
    /// The look keys' new `ui.*` values.
    pub look: BTreeMap<String, String>,
    pub command_alias: String,
    pub command_alias_add: String,
}

/// The config side of changing the primary: one write, computed here, applied
/// by [`become_primary_in_file`] / [`resign_primary_in_file`]. The desktop side
/// (renaming the legacy-named launchers) follows `demoted` and `promoted`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryHandover {
    /// The `[installation]` table after the change.
    pub installation: InstallationConfig,
    /// The new `ui.*` values; `None` when `ui.*` stays as it is.
    pub ui: Option<UiLook>,
    /// The `ui.*` keys as the file had them before the first primary: set
    /// back (and the keys it did not have removed) when the last one resigns.
    pub ui_restore: Option<UiBefore>,
    /// The installation that stops being primary, if any.
    pub demoted: Option<String>,
    /// The installation that becomes primary, if any.
    pub promoted: Option<String>,
}

fn other_names(config: &Config, except: &str) -> Vec<String> {
    config
        .installation
        .settings
        .keys()
        .filter(|id| id.as_str() != except)
        .filter_map(|id| effective_of(config, id))
        .map(|e| e.name)
        .collect()
}

/// Makes `old` keep what it wears: its effective colour and aliases go into
/// its own entry (the `ui.*` keys are about to change owner), and its own
/// name is frozen, since the plain name leaves with the primary role.
fn freeze_own_values(config: &Config, inst: &mut InstallationConfig, id: &str) {
    let eff = effective_of(config, id)
        .unwrap_or_else(|| effective_from(config, id, Kind::default(), None));
    let entry = inst.settings.entry(id.to_string()).or_default();
    entry.logo_color = eff.logo_color;
    entry.look = eff.look;
    entry.command_alias = eff.command_alias;
    entry.command_alias_add = eff.command_alias_add;
    if entry.name.is_empty() {
        let channel = InstallChannel::parse(&entry.channel);
        entry.name = unique_name(&default_name(entry.kind, channel), &other_names(config, id));
    }
}

/// The handover for `new` becoming the primary; `None` when it already is.
///
/// - The old primary's effective values are copied into its own entry first,
///   so it keeps its colour and aliases.
/// - `new`'s effective values are written to `ui.*` and cleared from its
///   entry: an entry field outranks `ui.*`, so leaving them would freeze the
///   primary's look against later `ui.*` writes.
/// - `new`'s entry name is kept (the plain name applies while it is primary).
pub fn plan_primary(config: &Config, new: &Identity) -> Option<PrimaryHandover> {
    if config.installation.primary.as_deref() == Some(new.id.as_str()) {
        return None;
    }
    let mut inst = config.installation.clone();
    let wanted = effective(config, new);
    let demoted = config.installation.primary.clone();
    // With nobody primary, `ui.*` is today's shared look and the first primary
    // inherits it (aliases and colour it did not set itself) instead of wiping
    // it; behind an old primary those values were just copied to that one.
    let own = config.installation.settings.get(&new.id);
    let inherit = |own: Option<&str>, wanted: String, legacy: &str| match own {
        Some(v) if !v.is_empty() => wanted,
        _ if demoted.is_none() => legacy.to_string(),
        _ => wanted,
    };
    // `ui.*` aliases another installation's entry holds (a 1.0 dev build wrote
    // them) are that installation's: the new primary does not take them.
    // (`effective` already leaves them out; the inherited `ui.*` text is raw.)
    let inherit_alias = |own: Option<&str>, wanted: String, legacy: &str| {
        let name = inherit(own, wanted, legacy);
        if claimed_by_another(config, &new.id, &name) {
            String::new()
        } else {
            name
        }
    };
    if let Some(old) = &demoted {
        freeze_own_values(config, &mut inst, old);
    }
    let channel = new.channel;
    let name = unique_name(
        &default_name(new.kind, channel),
        &other_names(config, &new.id),
    );
    let entry = inst
        .settings
        .entry(new.id.clone())
        .or_insert_with(|| new_entry(new, name));
    entry.logo_color.clear();
    entry.look.clear();
    entry.command_alias.clear();
    entry.command_alias_add.clear();
    inst.primary = Some(new.id.clone());
    // Behind nobody, `ui.*` is about to become a primary's: remember it (a
    // handover between primaries keeps what was remembered).
    if demoted.is_none() && inst.ui_before.is_none() {
        inst.ui_before = Some(UiBefore {
            logo_color: Some(config.ui.logo_color.clone()),
            look: LOOK_KEYS
                .iter()
                .map(|k| (k.to_string(), look_value(&config.ui, k)))
                .collect(),
            command_alias: Some(config.ui.command_alias.clone()),
            command_alias_add: Some(config.ui.command_alias_add.clone()),
        });
    }
    Some(PrimaryHandover {
        ui_restore: None,
        installation: inst,
        ui: Some(UiLook {
            logo_color: inherit(
                own.map(|e| e.logo_color.as_str()),
                wanted.logo_color,
                &config.ui.logo_color,
            ),
            look: LOOK_KEYS
                .iter()
                .map(|k| {
                    let own = own.and_then(|e| e.look.get(*k)).map(String::as_str);
                    let now = look_value(&config.ui, k);
                    let want = wanted.look.get(*k).cloned().unwrap_or_default();
                    (k.to_string(), inherit(own, want, &now))
                })
                .collect(),
            command_alias: inherit_alias(
                own.map(|e| e.command_alias.as_str()),
                wanted.command_alias,
                &config.ui.command_alias,
            ),
            command_alias_add: inherit_alias(
                own.map(|e| e.command_alias_add.as_str()),
                wanted.command_alias_add,
                &config.ui.command_alias_add,
            ),
        }),
        demoted,
        promoted: Some(new.id.clone()),
    })
}

/// The handover for `id` stopping being the primary; `None` when it is not.
/// Its values move from `ui.*` into its entry, and `ui.*` goes back to what
/// the file held before the first primary (so older builds do not wear a
/// colour or aliases nobody owns).
pub fn plan_resign(config: &Config, id: &str) -> Option<PrimaryHandover> {
    if config.installation.primary.as_deref() != Some(id) {
        return None;
    }
    let mut inst = config.installation.clone();
    freeze_own_values(config, &mut inst, id);
    inst.primary = None;
    let ui_restore = inst.ui_before.take();
    Some(PrimaryHandover {
        installation: inst,
        ui: None,
        ui_restore,
        demoted: Some(id.to_string()),
        promoted: None,
    })
}

// -------------------------------------------------------------- the file

/// One `ui.<key>` change of a handover: set it, or remove it (`None`).
type UiAction = (&'static str, Option<String>);

fn read_raw(path: &Path) -> Result<String> {
    Ok(if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    })
}

fn parse_document(raw: &str) -> Result<toml::Value> {
    let mut document = if raw.trim().is_empty() {
        toml::Value::Table(toml::map::Map::new())
    } else {
        toml::from_str(raw)?
    };
    AiConfig::migrate_document(&mut document);
    Ok(document)
}

fn put_installation(document: &mut toml::Value, installation: &InstallationConfig) -> Result<()> {
    if installation.is_empty() {
        remove_dotted(document, "installation");
        Ok(())
    } else {
        insert_dotted(
            document,
            "installation",
            toml::Value::try_from(installation)?,
        )
    }
}

/// Validates the edited document as a whole `Config` (like `set_in_file`)
/// before anything touches the disk.
fn validate(document: &toml::Value) -> Result<()> {
    let _: Config = document
        .clone()
        .try_into()
        .map_err(|err: toml::de::Error| Error::BadValue {
            key: "installation".to_string(),
            value: "<table>".to_string(),
            expected: err.to_string(),
        })?;
    Ok(())
}

fn write_text(path: &Path, raw: &str, text: &str) -> Result<()> {
    if text == raw {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        secure_fs::create_private_dir_all(parent)?;
    }
    secure_fs::write_private(path, text.as_bytes())?;
    Ok(())
}

/// The edited `[installation]` table (and the `ui.*` look of a handover)
/// written into `raw` as text, so every byte outside them survives. A file
/// spelling those tables in a way the scanner does not handle (dotted or
/// inline) is rewritten whole from `document` instead.
fn splice(
    raw: &str,
    document: &toml::Value,
    installation: &InstallationConfig,
    ui: &[UiAction],
) -> Result<String> {
    let table = if installation.is_empty() {
        String::new()
    } else {
        let mut root = toml::map::Map::new();
        root.insert(
            "installation".to_string(),
            toml::Value::try_from(installation)?,
        );
        toml::to_string(&toml::Value::Table(root))?
    };
    let mut text = text_edit::replace_table(raw, "installation", &table);
    for (key, value) in ui {
        text = text.and_then(|t| match value {
            Some(v) => {
                let quoted = toml::Value::String(v.clone()).to_string();
                text_edit::set_key(&t, "ui", key, &quoted)
            }
            None => text_edit::remove_key(&t, "ui", key),
        });
    }
    match text {
        Some(t) => Ok(t),
        None => Ok(toml::to_string_pretty(document)?),
    }
}

/// Reads the `[installation]` table of the document at `path`, lets `edit`
/// change it and writes it back. Read and write are one call, so two
/// installations updating their own entries do not overwrite each other's
/// unless they race inside it. An empty table is removed rather than written
/// empty. Everything outside the `[installation]` tables (comments, key
/// order, blank lines) keeps its bytes.
pub fn set_installation_in_file(
    path: &Path,
    edit: impl FnOnce(&mut InstallationConfig),
) -> Result<()> {
    let raw = read_raw(path)?;
    let mut document = parse_document(&raw)?;
    let mut installation: InstallationConfig = match document.get("installation") {
        Some(v) => v.clone().try_into()?,
        None => InstallationConfig::default(),
    };
    edit(&mut installation);
    put_installation(&mut document, &installation)?;
    validate(&document)?;
    let text = splice(&raw, &document, &installation, &[])?;
    write_text(path, &raw, &text)
}

/// Stores the colour key `key` (`theme`, `graph_color_theme`, ...) `identity`
/// picked. The primary writes the shared `ui.<key>` (older builds read it) and
/// drops a stale entry value, which would outrank it; every other installation
/// writes only its own entry, so one installation's pick never recolours
/// another. Returns the stored text.
pub fn set_look_in_file(
    path: &Path,
    identity: &Identity,
    key: &str,
    value: &str,
) -> Result<String> {
    if !LOOK_KEYS.contains(&key) {
        return Err(Error::UnknownKey(format!("ui.{key}")));
    }
    let config = Config::load(path).unwrap_or_default();
    let id = identity.id.clone();
    if config.installation.primary.as_deref() == Some(id.as_str()) {
        let stored = crate::set_in_file(path, &format!("ui.{key}"), value)?;
        if config
            .installation
            .settings
            .get(&id)
            .is_some_and(|e| e.look.contains_key(key))
        {
            set_installation_in_file(path, |i| {
                if let Some(e) = i.settings.get_mut(&id) {
                    e.look.remove(key);
                }
            })?;
        }
        return Ok(stored.as_str().unwrap_or(value).to_string());
    }
    let ident = identity.clone();
    let (k, v) = (key.to_string(), value.to_string());
    set_installation_in_file(path, |i| {
        let entry = i.settings.entry(id).or_insert_with(|| InstallationEntry {
            path: ident.root.clone(),
            kind: ident.kind,
            ..Default::default()
        });
        // The other colours keep what this installation showed (today's
        // `ui.*`), which an entry not seeded on disk (a pinned id) lacks.
        for key in LOOK_KEYS {
            entry
                .look
                .entry(key.to_string())
                .or_insert_with(|| look_value(&config.ui, key));
        }
        entry.look.insert(k, v);
    })?;
    Ok(value.to_string())
}

fn apply_handover(
    path: &Path,
    plan: impl FnOnce(&Config) -> Option<PrimaryHandover>,
) -> Result<Option<PrimaryHandover>> {
    let raw = read_raw(path)?;
    let mut document = parse_document(&raw)?;
    let config = Config::from_toml_str(&toml::to_string(&document)?)?.0;
    let Some(mut handover) = plan(&config) else {
        return Ok(None);
    };
    let present = |key: &str| document.get("ui").and_then(|u| u.get(key)).is_some();
    // A look remembered just now keeps only the keys the file really had.
    if config.installation.ui_before.is_none() {
        if let Some(before) = handover.installation.ui_before.as_mut() {
            for (key, slot) in [
                ("logo_color", &mut before.logo_color),
                ("command_alias", &mut before.command_alias),
                ("command_alias_add", &mut before.command_alias_add),
            ] {
                if !present(key) {
                    *slot = None;
                }
            }
            before.look.retain(|key, _| present(key));
        }
    }
    // Only what changes is touched, so a key keeps its line and comment.
    let mut actions: Vec<UiAction> = Vec::new();
    let current = [
        ("logo_color", &config.ui.logo_color),
        ("command_alias", &config.ui.command_alias),
        ("command_alias_add", &config.ui.command_alias_add),
    ];
    if let Some(ui) = &handover.ui {
        let wanted = [&ui.logo_color, &ui.command_alias, &ui.command_alias_add];
        for ((key, now), want) in current.iter().zip(wanted) {
            if now != &want {
                actions.push((key, Some(want.clone())));
            }
        }
        for key in LOOK_KEYS {
            if let Some(want) = ui.look.get(key) {
                if *want != look_value(&config.ui, key) {
                    actions.push((key, Some(want.clone())));
                }
            }
        }
    }
    if let Some(before) = &handover.ui_restore {
        let old = [
            &before.logo_color,
            &before.command_alias,
            &before.command_alias_add,
        ];
        for ((key, now), old) in current.iter().zip(old) {
            match old {
                Some(v) if v != *now || !present(key) => actions.push((key, Some(v.clone()))),
                None if present(key) => actions.push((key, None)),
                _ => {}
            }
        }
        for key in LOOK_KEYS {
            match before.look.get(key) {
                Some(v) if *v != look_value(&config.ui, key) || !present(key) => {
                    actions.push((key, Some(v.clone())));
                }
                None if present(key) => actions.push((key, None)),
                _ => {}
            }
        }
    }
    put_installation(&mut document, &handover.installation)?;
    for (key, value) in &actions {
        let dotted = format!("ui.{key}");
        match value {
            Some(v) => insert_dotted(&mut document, &dotted, toml::Value::String(v.clone()))?,
            None => remove_dotted(&mut document, &dotted),
        }
    }
    validate(&document)?;
    let text = splice(&raw, &document, &handover.installation, &actions)?;
    write_text(path, &raw, &text)?;
    Ok(Some(handover))
}

/// The config write of "Become the primary": [`plan_primary`] applied in one
/// write to the file at `path`. Returns what was done (`None`: already primary).
pub fn become_primary_in_file(path: &Path, new: &Identity) -> Result<Option<PrimaryHandover>> {
    apply_handover(path, |config| plan_primary(config, new))
}

/// [`plan_resign`] applied in one write.
pub fn resign_primary_in_file(path: &Path, id: &str) -> Result<Option<PrimaryHandover>> {
    apply_handover(path, |config| plan_resign(config, id))
}

// ----------------------------------------------------------------- current

/// The running installation as every frontend needs it: who it is, its
/// seeded config and the context the desktop writers take.
#[derive(Debug, Clone)]
pub struct Current {
    pub identity: Identity,
    /// The config after seeding this installation's entry.
    pub config: Config,
    pub effective: Effective,
    /// The environment in force (`GITPOULPI_ENV` included).
    pub environment: String,
    pub ctx: crate::desktop::InstallCtx,
    /// The `config.toml` the entry lives in, when the config folder is known.
    pub config_file: Option<std::path::PathBuf>,
}

/// Where each alias file points, read off the real bin directory.
struct DiskAliases(crate::desktop::alias::Env);

impl AliasEvidence for DiskAliases {
    fn target_of(&self, alias: &str) -> Option<String> {
        crate::desktop::alias::target_of(&self.0, alias).map(|p| p.to_string_lossy().into_owned())
    }
}

/// True for `<...>/target/<profile>/deps`, where cargo puts test binaries. A
/// test binary is not an installation: `identify` reads it as a `portable` one
/// (it is not `target/<profile>/<file>`), and seeding it would add a bogus
/// entry to the developer's real `config.toml` whenever a test reaches
/// [`current`] without `GITPOULPI_CONFIG_DIR`.
fn is_cargo_test_root(root: &str) -> bool {
    let parts: Vec<&str> = root.split(['/', '\\']).filter(|p| !p.is_empty()).collect();
    let n = parts.len();
    n >= 3 && parts[n - 1] == "deps" && parts[n - 3] == "target"
}

/// Writes a freshly seeded `entry` into the file; true when the caller may
/// use it. A pinned id (`GITPOULPI_INSTALL_ID`) is an explicit override, not a
/// discovered installation: its entry stays in memory only (still true) until
/// the user changes a per-installation setting, which writes it.
fn persist_seed(file: &Path, identity: &Identity, entry: &InstallationEntry) -> bool {
    if identity.kind == Kind::Pinned || is_cargo_test_root(&identity.root) {
        return true;
    }
    let id = identity.id.clone();
    let stored = entry.clone();
    set_installation_in_file(file, |i| {
        i.settings.insert(id, stored);
    })
    .is_ok()
}

/// Identifies the running exe, seeds its entry in `config.toml` the first time
/// (or refreshes its path and channel) and returns the result, worked out once
/// per process. Frontends (`alias`, `desktop`, `folder-menu`, `uninstall`)
/// act through it. Tests of the pure rules use [`identify`] and [`seed`]
/// directly; this reads the real machine.
pub fn current() -> &'static Current {
    static CURRENT: std::sync::OnceLock<Current> = std::sync::OnceLock::new();
    CURRENT.get_or_init(|| {
        let fs = RealFs;
        let env = IdentityEnv::real(&fs);
        let exe = std::env::current_exe().unwrap_or_default();
        let identity = identify(&exe, &env);
        let config_file = crate::ConfigPaths::discover().map(|p| p.config_file());
        let mut config = config_file
            .as_deref()
            .and_then(|f| Config::load(f).ok())
            .unwrap_or_default();
        let live = |e: &InstallationEntry| !e.path.is_empty() && Path::new(&e.path).exists();
        if let Some(home) = crate::home_dir() {
            let lookup = |k: &str| std::env::var(k).ok();
            let aliases = DiskAliases(crate::desktop::alias::Env::discover(
                crate::open_in::Os::current(),
                &lookup,
                &home,
                &exe,
            ));
            if let (Some(entry), Some(file)) = (
                seed(&config, &identity, &env, &aliases, &live),
                config_file.as_deref(),
            ) {
                if persist_seed(file, &identity, &entry) {
                    config
                        .installation
                        .settings
                        .insert(identity.id.clone(), entry);
                }
            }
        }
        let effective = effective(&config, &identity);
        let environment = environment(&config, &identity, &env);
        let ctx = crate::desktop::InstallCtx::from_config(&config, &identity, &|path| {
            !path.is_empty() && Path::new(path).exists()
        });
        Current {
            identity,
            config,
            effective,
            environment,
            ctx,
            config_file,
        }
    })
}

/// The folders on the user `PATH` that `path add` and `path remove` of `cur`
/// leave alone: other live installations' folders, and the shared alias `bin`
/// folder while another installation still has an alias in it.
pub fn path_keep(cur: &Current, aliases: &crate::desktop::alias::Env) -> Vec<std::path::PathBuf> {
    let mut keep = other_live_paths(&cur.config, &cur.identity.id);
    if crate::desktop::alias::others_have_aliases(aliases) {
        keep.push(aliases.bin_dir.clone());
    }
    keep
}

/// The entries' paths of the other live installations.
pub fn other_live_paths(config: &Config, id: &str) -> Vec<std::path::PathBuf> {
    config
        .installation
        .settings
        .iter()
        .filter(|(other, e)| other.as_str() != id && !e.path.is_empty())
        .filter(|(_, e)| Path::new(&e.path).exists())
        .map(|(_, e)| std::path::PathBuf::from(&e.path))
        .collect()
}

#[cfg(test)]
mod tests;
mod text_edit;
