//! `xtask build-custom`: a size-optimised build with only the chosen
//! features (see the custom-builds design section 9.1).
//!
//! The mapping from choices to cargo arguments is pure (`Choices::cargo_args`)
//! so it is unit-tested without ever running cargo. Feature names are the
//! fixed contract of plan section 1.1.

use gitpoulpi_logo_build::logo_build;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "\
Usage: cargo run -p xtask -- build-custom [options] [-- <cargo args>]

Languages and integrations:
  --langs fr,de|all|en     UI languages besides English (always built)
  --integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues|all|none
                           hosting providers; none = clone by URL only, no
                           issues, no PR/MR
Themes (GitPoulpi Dark, Light and Terminal are always built):
  --themes all|none|nord,catppuccin,...
                           theme families to compile in (default all);
                           --list-themes prints them
Logo (the pixel octopus of the About window and the Windows executable icon):
  --logo-colors all|violet,teal,...
                           colorways compiled in (default all, at least
                           one); --list-logo-colors prints them
  --logo-default <slug>    the About default and the fallback of a stored
                           `ui.logo_color` that is not compiled in (must be
                           one of the compiled ones; default lavender)
  --icon-color <slug>     color of the Windows .exe icon (any colorway;
                           default: the logo default)
Features (each defaults to on):
  --[no-]ai  --[no-]mcp  --[no-]lfs  --[no-]worktree  --[no-]submodules
  --[no-]avatar-fetch      off = initials badge only
  --[no-]image-view        off = no image preview (GIF/WebP/BMP/ICO decoders)
  --[no-]radar         off = no Radar (PRs/issues of every repo in one list)
  --[no-]mermaid           off = no Copy as Mermaid gitGraph, no `log --format mermaid`
  --[no-]board             off = no Kanban board view, no `hosting board`
  --[no-]review            off = no in-app code review view and its AI assist (on brings the
                           conversation view along: the Overview is that view)
  --[no-]conversation      off = no in-app issue / request conversation view (Enter opens the browser)
  --[no-]history-rewrite   off = no History workbench (advanced history rewriting:
                           authors, dates, messages, content)
  --[no-]global-search     off = no Global search (title-bar button, Ctrl+Alt+F, `search`)
  --[no-]workspaces        off = no workspaces (only the default tab list, no workspace commands)
  --[no-]rewrite-scripts   off = no embedded Lua: Script rules are a typed refusal
                           (needs history rewrite, else ignored with a warning)
                           (radar, board, review and conversation need a hosting integration)
Saved selections (KEY=value files, kept outside target/ so cargo clean keeps them):
  --save <name|file>       store the resolved choices as a preset
  --preset <name|file>     start from a preset; flags still override it
  --list-presets           print the saved presets and their folder
  --delete-preset <name>   remove a saved preset
Other:
  --last                   start from target/custom-build/last-choices.env
  --full                   no feature selection at all (the normal build)
  --profile custom|dist|release   (default custom)
  --dry-run                print the exact cargo command and stop
  --list-langs             print `code<TAB>native name` for each language
  --list-themes            print `family<TAB>label (n themes)` for each family
  --list-logo-colors      print `slug<TAB>label` for each logo colorway,
                           the table default is marked `*`
  --print-choices          print the resolved choices as KEY=value and stop
  - <cargo args>         passed to cargo build (for example --target ...)";

/// Hosting providers the script can offer.
pub const INTEGRATIONS: [&str; 12] = [
    "azure-devops",
    "bitbucket",
    "gerrit",
    "gitea",
    "github",
    "gitlab",
    "jira",
    "linear",
    "local-issues",
    "redmine",
    "trello",
    "youtrack",
];

/// Everything the user picks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choices {
    /// Lower-case language codes besides English, sorted, deduplicated.
    pub langs: Vec<String>,
    pub integrations: Vec<String>,
    /// Theme families compiled in besides the fixed ones; `None` = all.
    pub themes: Option<Vec<String>>,
    /// Logo colorways compiled in (table order); `None` = all.
    pub logo_colors: Option<Vec<String>>,
    /// The About default; `None` = the table's default (or the first compiled).
    pub logo_default: Option<String>,
    /// Color of the Windows executable icon; `None` = the logo default.
    pub icon_color: Option<String>,
    pub ai: bool,
    pub mcp: bool,
    pub lfs: bool,
    pub worktree: bool,
    pub submodules: bool,
    pub avatar_fetch: bool,
    pub image_view: bool,
    pub radar: bool,
    pub mermaid: bool,
    pub board: bool,
    pub review: bool,
    pub conversation: bool,
    pub history_rewrite: bool,
    pub global_search: bool,
    pub workspaces: bool,
    /// Embedded Lua for rewrite scripts; only meaningful with `history_rewrite`.
    pub rewrite_scripts: bool,
}

impl Choices {
    pub fn all(available_langs: &[String]) -> Self {
        Choices {
            langs: available_langs.to_vec(),
            integrations: INTEGRATIONS.iter().map(|s| s.to_string()).collect(),
            themes: None,
            logo_colors: None,
            logo_default: None,
            icon_color: None,
            ai: true,
            mcp: true,
            lfs: true,
            worktree: true,
            submodules: true,
            avatar_fetch: true,
            image_view: true,
            radar: true,
            mermaid: true,
            board: true,
            review: true,
            conversation: true,
            history_rewrite: true,
            global_search: true,
            workspaces: true,
            rewrite_scripts: true,
        }
    }

    /// Everything off, English only.
    pub fn none() -> Self {
        Choices {
            langs: vec![],
            integrations: vec![],
            themes: Some(vec![]),
            logo_colors: None,
            logo_default: None,
            icon_color: None,
            ai: false,
            mcp: false,
            lfs: false,
            worktree: false,
            submodules: false,
            avatar_fetch: false,
            image_view: false,
            radar: false,
            mermaid: false,
            board: false,
            review: false,
            conversation: false,
            history_rewrite: false,
            global_search: false,
            workspaces: false,
            rewrite_scripts: false,
        }
    }

    /// The `--features` values (`crate/feature`), one cargo invocation over
    /// all selected packages so features unify once.
    pub fn features(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut add = |krate: &str, feat: &str| out.push(format!("gitpoulpi-{krate}/{feat}"));
        let mut bins = vec!["cli", "tui"];
        if self.mcp {
            bins.push("mcp");
        }
        for bin in bins {
            for lang in &self.langs {
                add(bin, &format!("lang-{lang}"));
            }
            for kind in &self.integrations {
                add(bin, kind);
            }
            for (on, feat) in [
                (self.ai, "ai"),
                (self.lfs, "lfs"),
                (self.worktree, "worktree"),
                (self.submodules, "submodule"),
                (self.avatar_fetch, "avatar-fetch"),
                (self.image_view, "image-view"),
                (self.radar, "radar"),
                (self.mermaid, "mermaid"),
                (self.board, "board"),
                (self.review, "review"),
                // The review's Overview is the conversation view: Review brings it along.
                (self.conversation || self.review, "conversation"),
                (self.history_rewrite, "history-rewrite"),
                (self.global_search, "global-search"),
                (self.workspaces, "workspaces"),
                // Embedded Lua needs the rewrite engine it plugs into.
                (
                    self.history_rewrite && self.rewrite_scripts,
                    "rewrite-scripts",
                ),
            ] {
                if on {
                    add(bin, feat);
                }
            }
            // The `mcp` feature lives on the crates that gate the subcommand.
            if self.mcp && bin != "mcp" {
                add(bin, "mcp");
            }
        }
        out
    }

    /// Environment variables the cargo run needs (`GITPOULPI_THEMES`, read by
    /// `gitpoulpi-config`'s build script). Empty when every theme is built.
    pub fn env(&self) -> Vec<(&'static str, String)> {
        let mut out = match &self.themes {
            None => vec![],
            Some(list) if list.is_empty() => vec![("GITPOULPI_THEMES", "none".into())],
            Some(list) => vec![("GITPOULPI_THEMES", list.join(","))],
        };
        if let Some(list) = &self.logo_colors {
            out.push((logo_build::COLORS_ENV, list.join(",")));
        }
        if let Some(d) = &self.logo_default {
            out.push((logo_build::DEFAULT_ENV, d.clone()));
        }
        if let Some(i) = &self.icon_color {
            out.push((logo_build::ICON_ENV, i.clone()));
        }
        out
    }

    /// Checks the logo choices against each other (the build script does the
    /// same, but fails much later): the default must be compiled in.
    pub fn validate_logo(&self) -> Result<(), String> {
        let table = logo_table();
        let default = logo_build::resolve_default(
            &table,
            &self.logo_colors,
            self.logo_default.as_deref().unwrap_or(""),
        )?;
        logo_build::resolve_icon(&table, self.icon_color.as_deref().unwrap_or(""), &default)?;
        Ok(())
    }

    /// `(colors, default, icon)` as the build will resolve them.
    pub fn resolved_logo(&self) -> (Vec<String>, String, String) {
        let table = logo_table();
        let colors = self
            .logo_colors
            .clone()
            .unwrap_or_else(|| logo_build::slugs(&table));
        let default = logo_build::resolve_default(
            &table,
            &self.logo_colors,
            self.logo_default.as_deref().unwrap_or(""),
        )
        .unwrap_or_default();
        let icon =
            logo_build::resolve_icon(&table, self.icon_color.as_deref().unwrap_or(""), &default)
                .unwrap_or_default();
        (colors, default, icon)
    }

    pub fn packages(&self) -> Vec<&'static str> {
        let mut p = vec!["gitpoulpi-cli", "gitpoulpi-tui", "gitpoulpi-updater"];
        if self.mcp {
            p.push("gitpoulpi-mcp");
        }
        p
    }

    /// Arguments after `cargo`.
    pub fn cargo_args(&self, profile: &str, full: bool, extra: &[String]) -> Vec<String> {
        let mut a: Vec<String> = ["build", "--profile", profile, "--locked"]
            .map(String::from)
            .to_vec();
        for p in self.packages() {
            a.push("-p".into());
            a.push(p.into());
        }
        if !full {
            a.push("--no-default-features".into());
            let f = self.features();
            if !f.is_empty() {
                a.push("--features".into());
                a.push(f.join(","));
            }
        }
        a.extend(extra.iter().cloned());
        a
    }

    /// `last-choices.env` content: plain `KEY=value`, readable by sh.
    pub fn to_env(&self) -> String {
        let join = |v: &[String], empty: &str| {
            if v.is_empty() {
                empty.to_string()
            } else {
                v.join(",")
            }
        };
        let b = |x: bool| if x { 1 } else { 0 };
        let mut s = String::new();
        let _ = writeln!(s, "LANGS={}", join(&self.langs, "en"));
        let _ = writeln!(s, "INTEGRATIONS={}", join(&self.integrations, "none"));
        let _ = writeln!(
            s,
            "THEMES={}",
            match &self.themes {
                None => "all".to_string(),
                Some(l) => join(l, "none"),
            }
        );
        let _ = writeln!(
            s,
            "LOGO_COLORS={}",
            self.logo_colors
                .as_ref()
                .map_or("all".to_string(), |l| l.join(","))
        );
        let _ = writeln!(
            s,
            "LOGO_DEFAULT={}",
            self.logo_default.as_deref().unwrap_or("auto")
        );
        let _ = writeln!(
            s,
            "ICON_COLOR={}",
            self.icon_color.as_deref().unwrap_or("auto")
        );
        let _ = writeln!(s, "AI={}", b(self.ai));
        let _ = writeln!(s, "MCP={}", b(self.mcp));
        let _ = writeln!(s, "LFS={}", b(self.lfs));
        let _ = writeln!(s, "WORKTREE={}", b(self.worktree));
        let _ = writeln!(s, "SUBMODULES={}", b(self.submodules));
        let _ = writeln!(s, "AVATAR_FETCH={}", b(self.avatar_fetch));
        let _ = writeln!(s, "IMAGE_VIEW={}", b(self.image_view));
        let _ = writeln!(s, "RADAR={}", b(self.radar));
        let _ = writeln!(s, "MERMAID={}", b(self.mermaid));
        let _ = writeln!(s, "BOARD={}", b(self.board));
        let _ = writeln!(s, "REVIEW={}", b(self.review));
        let _ = writeln!(s, "CONVERSATION={}", b(self.conversation));
        let _ = writeln!(s, "HISTORY_REWRITE={}", b(self.history_rewrite));
        let _ = writeln!(s, "GLOBAL_SEARCH={}", b(self.global_search));
        let _ = writeln!(s, "WORKSPACES={}", b(self.workspaces));
        let _ = writeln!(s, "REWRITE_SCRIPTS={}", b(self.rewrite_scripts));
        // Every integration this GitPoulpi knows: a later version tells a new
        // one (added with its default) from one the maintainer switched off.
        let _ = writeln!(s, "KNOWN_INTEGRATIONS={}", INTEGRATIONS.join(","));
        s
    }

    /// Strict reading, notes dropped (`--last`'s own file, tests).
    pub fn from_env(text: &str, available: &[String], families: &[String]) -> Result<Self, String> {
        Self::from_env_report(text, available, families).map(|(c, _)| c)
    }

    /// Lenient reading of a saved selection. A key this build does not know,
    /// or a language / integration / theme / color it no longer has, is
    /// dropped with a note; a key the file lacks, and an integration added
    /// since it was written, take their default, also with a note.
    pub fn from_env_report(
        text: &str,
        available: &[String],
        families: &[String],
    ) -> Result<(Self, Vec<String>), String> {
        let mut c = Choices::all(available);
        let mut notes = Vec::new();
        let mut seen: Vec<&str> = Vec::new();
        let mut known_integrations: Option<Vec<String>> = None;
        let logo_slugs = logo_build::slugs(&logo_table());
        let integrations: Vec<String> = INTEGRATIONS.iter().map(|s| s.to_string()).collect();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                notes.push(format!("ignored the line '{line}' (not KEY=value)"));
                continue;
            };
            let key = k.trim();
            let flag = v.trim() == "1";
            let Some(canon) = ENV_KEYS
                .iter()
                .find(|(n, alias)| *n == key || *alias == key)
            else {
                if key == "KNOWN_INTEGRATIONS" {
                    known_integrations = Some(split_csv(v));
                } else {
                    notes.push(format!(
                        "unknown key {key} ignored (from another GitPoulpi version?)"
                    ));
                }
                continue;
            };
            seen.push(canon.0);
            match canon.0 {
                "LANGS" => {
                    let v = lenient(v, available, &["all", "en", "none"], "language", &mut notes);
                    c.langs = parse_langs(&v, available)?
                }
                "INTEGRATIONS" => {
                    let v = lenient(
                        v,
                        &integrations,
                        &["all", "none"],
                        "integration",
                        &mut notes,
                    );
                    c.integrations = parse_integrations(&v)?
                }
                "THEMES" => {
                    let v = lenient(v, families, &["all", "none"], "theme family", &mut notes);
                    c.themes = parse_themes(&v, families)?
                }
                "LOGO_COLORS" => {
                    let mut v = lenient(v, &logo_slugs, &["all", "auto"], "logo color", &mut notes);
                    if v.is_empty() {
                        v = "all".into();
                    }
                    c.logo_colors = parse_logo_colors(&v)?
                }
                "LOGO_DEFAULT" | "ICON_COLOR" => {
                    let what = if canon.0 == "ICON_COLOR" {
                        "icon color"
                    } else {
                        "default logo color"
                    };
                    let slot = match parse_logo_slug(v, what) {
                        Ok(x) => x,
                        Err(e) => {
                            notes.push(format!("{e}; using auto"));
                            None
                        }
                    };
                    if canon.0 == "ICON_COLOR" {
                        c.icon_color = slot;
                    } else {
                        c.logo_default = slot;
                    }
                }
                "AI" => c.ai = flag,
                "MCP" => c.mcp = flag,
                "LFS" => c.lfs = flag,
                "WORKTREE" => c.worktree = flag,
                "SUBMODULES" => c.submodules = flag,
                "AVATAR_FETCH" => c.avatar_fetch = flag,
                "IMAGE_VIEW" => c.image_view = flag,
                "RADAR" => c.radar = flag,
                "MERMAID" => c.mermaid = flag,
                "BOARD" => c.board = flag,
                "REVIEW" => c.review = flag,
                "CONVERSATION" => c.conversation = flag,
                "HISTORY_REWRITE" => c.history_rewrite = flag,
                "GLOBAL_SEARCH" => c.global_search = flag,
                "WORKSPACES" => c.workspaces = flag,
                "REWRITE_SCRIPTS" => c.rewrite_scripts = flag,
                _ => {}
            }
        }
        let defaults = Choices::all(available).to_env();
        for (name, _) in ENV_KEYS {
            if !seen.contains(&name) {
                let default = defaults
                    .lines()
                    .find_map(|l| l.strip_prefix(&format!("{name}=")))
                    .unwrap_or_default();
                notes.push(format!(
                    "{name} is not in this selection: added with its default ({default})"
                ));
            }
        }
        if let Some(known) = known_integrations {
            for i in INTEGRATIONS {
                if !known.iter().any(|k| k == i) {
                    if !c.integrations.iter().any(|x| x == i) {
                        c.integrations.push(i.to_string());
                    }
                    notes.push(format!(
                        "integration {i} was added after this selection was saved: included (its default is on)"
                    ));
                }
            }
            c.integrations.sort();
        }
        Ok((c, notes))
    }

    /// What `--version` should eventually show; also `BUILD-FEATURES.txt`.
    pub fn summary(&self) -> String {
        let mut langs = vec!["en".to_string()];
        langs.extend(self.langs.iter().cloned());
        let mut feats: Vec<&str> = self.integrations.iter().map(String::as_str).collect();
        for (on, n) in [
            (self.ai, "ai"),
            (self.mcp, "mcp"),
            (self.lfs, "lfs"),
            (self.worktree, "worktree"),
            (self.submodules, "submodule"),
            (self.avatar_fetch, "avatar-fetch"),
            (self.image_view, "image-view"),
            (self.radar, "radar"),
            (self.mermaid, "mermaid"),
            (self.board, "board"),
            (self.review, "review"),
            (self.conversation || self.review, "conversation"),
            (self.history_rewrite, "history-rewrite"),
            (self.global_search, "global-search"),
            (self.workspaces, "workspaces"),
        ] {
            if on {
                feats.push(n);
            }
        }
        let themes = match &self.themes {
            None => "all".to_string(),
            Some(l) => ["dark", "light", "terminal"]
                .into_iter()
                .chain(l.iter().map(String::as_str))
                .collect::<Vec<_>>()
                .join(" "),
        };
        let (colors, default, icon) = self.resolved_logo();
        format!(
            "languages: {}\nthemes: {themes}\nlogo: {} (default {default}, icon {icon})\nfeatures: {}\n",
            langs.join(" "),
            colors.join(" "),
            if feats.is_empty() {
                "(none)".into()
            } else {
                feats.join(" ")
            }
        )
    }
}

/// Every selection-file key with its accepted spelling variant.
const ENV_KEYS: [(&str, &str); 22] = [
    ("LANGS", "LANGS"),
    ("INTEGRATIONS", "INTEGRATIONS"),
    ("THEMES", "THEMES"),
    ("LOGO_COLORS", "LOGO_COLOURS"),
    ("LOGO_DEFAULT", "LOGO_DEFAULT"),
    ("ICON_COLOR", "ICON_COLOUR"),
    ("AI", "AI"),
    ("MCP", "MCP"),
    ("LFS", "LFS"),
    ("WORKTREE", "WORKTREE"),
    ("SUBMODULES", "SUBMODULES"),
    ("AVATAR_FETCH", "AVATAR_FETCH"),
    ("IMAGE_VIEW", "IMAGE_VIEW"),
    ("RADAR", "RADAR"),
    ("MERMAID", "MERMAID"),
    ("BOARD", "BOARD"),
    ("REVIEW", "REVIEW"),
    ("CONVERSATION", "CONVERSATION"),
    ("HISTORY_REWRITE", "HISTORY_REWRITE"),
    ("GLOBAL_SEARCH", "GLOBAL_SEARCH"),
    ("WORKSPACES", "WORKSPACES"),
    ("REWRITE_SCRIPTS", "REWRITE_SCRIPTS"),
];

fn split_csv(v: &str) -> Vec<String> {
    v.split(',')
        .map(|x| x.trim().to_ascii_lowercase())
        .filter(|x| !x.is_empty())
        .collect()
}

/// Drops the tokens of a comma list that are neither valid nor a keyword,
/// noting each one.
fn lenient(
    value: &str,
    valid: &[String],
    keywords: &[&str],
    what: &str,
    notes: &mut Vec<String>,
) -> String {
    let mut keep = Vec::new();
    for tok in split_csv(value) {
        let norm = tok.replace('_', "-");
        if keywords.contains(&tok.as_str()) || valid.contains(&norm) {
            keep.push(tok);
        } else {
            notes.push(format!(
                "{what} '{tok}' is not available in this GitPoulpi: ignored"
            ));
        }
    }
    keep.join(",")
}

/// Comma list -> validated, lower-case, sorted codes. `en` is implicit and
/// dropped; `all` means every available language.
pub fn parse_langs(value: &str, available: &[String]) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    for raw in value.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let code = raw.to_ascii_lowercase().replace('_', "-");
        if code == "all" {
            out.extend(available.iter().cloned());
        } else if code == "en" || code == "none" {
        } else if available.contains(&code) {
            out.push(code);
        } else {
            return Err(format!(
                "unknown language '{raw}'. Available: en (always), {}",
                available.join(", ")
            ));
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn logo_table() -> serde_json::Value {
    serde_json::from_str(logo_build::TABLE_JSON).expect("colourways.json parses")
}

/// `all` (or `auto`) -> `None`, else the validated slugs in table order.
pub fn parse_logo_colors(value: &str) -> Result<Option<Vec<String>>, String> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }
    logo_build::parse_colors(v, &logo_build::slugs(&logo_table()))
}

/// `auto` / empty -> `None`, else a validated colorway slug.
pub fn parse_logo_slug(value: &str, what: &str) -> Result<Option<String>, String> {
    let v = value.trim().to_ascii_lowercase();
    if v.is_empty() || v == "auto" {
        return Ok(None);
    }
    if logo_build::slugs(&logo_table()).contains(&v) {
        Ok(Some(v))
    } else {
        Err(format!(
            "unknown {what} '{value}'. Available: {}",
            logo_build::slugs(&logo_table()).join(", ")
        ))
    }
}

/// `slug<TAB>label` of every logo colorway (what `--list-logo-colors` prints).
pub fn logo_colorways() -> Vec<(String, String)> {
    let t = logo_table();
    let default = t["default"].as_str().unwrap_or_default().to_string();
    t["ways"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|w| {
            let slug = w["slug"].as_str().unwrap_or_default().to_string();
            let label = format!(
                "{slug} ({}){}",
                w["body"].as_str().unwrap_or_default(),
                if slug == default { " *" } else { "" }
            );
            (slug, label)
        })
        .collect()
}

/// `all` (the default) is `None`; `none` is the empty list; otherwise a
/// validated, sorted list of theme families.
pub fn parse_themes(value: &str, families: &[String]) -> Result<Option<Vec<String>>, String> {
    let mut out: Vec<String> = Vec::new();
    for raw in value.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let f = raw.to_ascii_lowercase();
        if f == "all" {
            return Ok(None);
        } else if f == "none" {
        } else if families.contains(&f) {
            out.push(f);
        } else {
            return Err(format!(
                "unknown theme family '{raw}'. Available: all, none, {}",
                families.join(", ")
            ));
        }
    }
    out.sort();
    out.dedup();
    Ok(Some(out))
}

/// `(family, labels summary)` of every selectable theme family, sorted:
/// every built-in preset except the always-built ones, grouped by family.
pub fn theme_families() -> Vec<(String, String)> {
    let mut map: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for spec in gitpoulpi_config::theme::presets() {
        if gitpoulpi_config::theme::ALWAYS_THEMES.contains(&spec.slug.as_str()) {
            continue;
        }
        map.entry(gitpoulpi_config::theme::family_of(&spec.slug).to_string())
            .or_default()
            .push(spec.name);
    }
    map.into_iter()
        .map(|(f, names)| {
            let label = if names.len() == 1 {
                names[0].clone()
            } else {
                format!("{} ({} themes)", common_words(&names), names.len())
            };
            (f, label)
        })
        .collect()
}

/// The words every name starts with ("Gruvbox Dark", "Gruvbox Light" ->
/// "Gruvbox"), at least the first one.
fn common_words(names: &[String]) -> String {
    let first: Vec<&str> = names[0].split(' ').collect();
    let mut n = first.len();
    for name in &names[1..] {
        let w: Vec<&str> = name.split(' ').collect();
        n = n.min(first.iter().zip(&w).take_while(|(a, b)| a == b).count());
    }
    first[..n.max(1)].join(" ")
}

pub fn parse_integrations(value: &str) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::new();
    for raw in value.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let v = raw.to_ascii_lowercase();
        if v == "all" {
            out.extend(INTEGRATIONS.iter().map(|s| s.to_string()));
        } else if v == "none" {
        } else if INTEGRATIONS.contains(&v.as_str()) {
            out.push(v);
        } else {
            return Err(format!(
                "unknown integration '{raw}'. Available: github, gitlab, azure-devops, bitbucket, jira, trello, gitea, linear, youtrack, redmine, gerrit, local-issues, none"
            ));
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repo root")
        .to_path_buf()
}

/// `(lower-case code, native name)` for every `locales/*.toml` except
/// English, sorted by code. The code is the file stem.
pub fn available_languages(root: &Path) -> Result<Vec<(String, String)>, String> {
    let dir = root.join("crates/gitpoulpi-i18n/locales");
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let code = stem.to_ascii_lowercase();
        if code == "en" || code.is_empty() {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        out.push((code.clone(), language_name(&text).unwrap_or(code)));
    }
    out.sort();
    Ok(out)
}

fn language_name(toml: &str) -> Option<String> {
    for line in toml.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("language_name") {
            let rest = rest.trim_start().strip_prefix('=')?.trim();
            return Some(rest.trim_matches('"').to_string());
        }
    }
    None
}

/// Folder of the saved selections: `custom-builds/` in GitPoulpi's config
/// directory (`ConfigPaths`: XDG `~/.config/gitpoulpi`, `%APPDATA%\GitPoulpi\config`,
/// `~/Library/Application Support/GitPoulpi`; `GITPOULPI_CONFIG_DIR` moves it).
/// Outside `target/`, so `cargo clean` keeps it.
pub fn presets_dir() -> Option<PathBuf> {
    gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_dir().join("custom-builds"))
}

/// A preset name: letters, digits, `-`, `_`, `.`; no leading dot.
fn valid_preset_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Does `spec` name a file rather than a saved preset? (`dir/x.env`,
/// `.\x.env`, `~`-less on purpose: the shell expands that.)
fn is_preset_path(spec: &str) -> bool {
    spec.contains('/') || spec.contains('\\') || spec.ends_with(".env")
}

/// The file a `--preset` / `--save` value points at.
pub fn preset_file(spec: &str, dir: Option<&Path>) -> Result<PathBuf, String> {
    if is_preset_path(spec) {
        return Ok(PathBuf::from(spec));
    }
    if !valid_preset_name(spec) {
        return Err(format!(
            "bad preset name '{spec}' (letters, digits, - _ . only; or give a file path)"
        ));
    }
    let dir = dir.ok_or("no config directory on this machine: give a file path instead")?;
    Ok(dir.join(format!("{spec}.env")))
}

pub fn save_preset(spec: &str, dir: Option<&Path>, choices: &Choices) -> Result<PathBuf, String> {
    let file = preset_file(spec, dir)?;
    if let Some(parent) = file.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    std::fs::write(&file, choices.to_env())
        .map_err(|e| format!("writing {}: {e}", file.display()))?;
    Ok(file)
}

/// Names of the saved presets, sorted.
pub fn list_presets(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            (p.extension().and_then(|x| x.to_str()) == Some("env"))
                .then(|| p.file_stem().and_then(|x| x.to_str()).map(String::from))
                .flatten()
        })
        .collect();
    out.sort();
    out
}

pub fn delete_preset(name: &str, dir: Option<&Path>) -> Result<PathBuf, String> {
    if is_preset_path(name) {
        return Err("--delete-preset takes a saved name, not a file path".into());
    }
    let file = preset_file(name, dir)?;
    std::fs::remove_file(&file)
        .map_err(|_| format!("no saved preset '{name}' ({})", file.display()))?;
    Ok(file)
}

#[derive(Debug)]
struct Options {
    choices: Choices,
    profile: String,
    full: bool,
    dry_run: bool,
    list_langs: bool,
    print_choices: bool,
    /// `--save <name|file>`, written by `run` once the choices are valid.
    save: Option<String>,
    /// Warnings and "added with its default" notes, for stderr.
    notes: Vec<String>,
    extra: Vec<String>,
}

fn parse_args(
    args: &[String],
    root: &Path,
    available: &[String],
    families: &[String],
    presets: Option<&Path>,
) -> Result<Options, String> {
    let env_path = root.join("target/custom-build/last-choices.env");
    let mut choices = Choices::all(available);
    let mut notes = Vec::new();
    // The base is the last `--last` / `--preset` given; every other flag then
    // overrides it, wherever it stands on the line.
    let mut base: Option<(String, PathBuf)> = None;
    let mut scan = args.iter();
    while let Some(a) = scan.next() {
        match a.as_str() {
            "--" => break,
            "--last" => base = Some(("--last".into(), env_path.clone())),
            "--preset" => {
                let spec = scan.next().ok_or("--preset needs a value")?;
                base = Some((format!("--preset {spec}"), preset_file(spec, presets)?));
            }
            _ => {}
        }
    }
    if let Some((what, file)) = base {
        let text = std::fs::read_to_string(&file)
            .map_err(|_| format!("{what}: no saved choices at {}", file.display()))?;
        let (c, n) = Choices::from_env_report(&text, available, families)?;
        choices = c;
        notes.extend(n.into_iter().map(|n| format!("{what}: {n}")));
    }
    let mut explicit_scripts = false;
    let mut o = Options {
        choices,
        profile: "custom".into(),
        full: false,
        dry_run: false,
        list_langs: false,
        print_choices: false,
        save: None,
        notes,
        extra: vec![],
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        match a.as_str() {
            "--" => {
                o.extra = it.by_ref().cloned().collect();
            }
            "--last" => {}
            "--preset" | "--delete-preset" => {
                // Handled up front (`--preset`) or in `run` (`--delete-preset`).
                value(a)?;
            }
            "--list-presets" => {}
            "--save" => o.save = Some(value("--save")?),
            "--full" => o.full = true,
            "--dry-run" => o.dry_run = true,
            "--list-langs" => o.list_langs = true,
            "--list-themes" => {}
            "--themes" => o.choices.themes = parse_themes(&value("--themes")?, families)?,
            "--list-logo-colors" => {}
            "--logo-colors" | "--logo-colours" => {
                o.choices.logo_colors = parse_logo_colors(&value("--logo-colors")?)?
            }
            "--logo-default" => {
                o.choices.logo_default =
                    parse_logo_slug(&value("--logo-default")?, "default logo color")?
            }
            "--icon-color" | "--icon-colour" => {
                o.choices.icon_color = parse_logo_slug(&value("--icon-color")?, "icon color")?
            }
            "--print-choices" => o.print_choices = true,
            "--langs" => o.choices.langs = parse_langs(&value("--langs")?, available)?,
            "--integrations" => {
                o.choices.integrations = parse_integrations(&value("--integrations")?)?
            }
            "--profile" => {
                let p = value("--profile")?;
                if !["custom", "dist", "release"].contains(&p.as_str()) {
                    return Err(format!("unknown profile '{p}' (custom, dist, release)"));
                }
                o.profile = p;
            }
            "--ai" | "--no-ai" => o.choices.ai = a == "--ai",
            "--mcp" | "--no-mcp" => o.choices.mcp = a == "--mcp",
            "--lfs" | "--no-lfs" => o.choices.lfs = a == "--lfs",
            "--worktree" | "--no-worktree" => o.choices.worktree = a == "--worktree",
            "--submodules" | "--no-submodules" => o.choices.submodules = a == "--submodules",
            "--avatar-fetch" | "--no-avatar-fetch" => {
                o.choices.avatar_fetch = a == "--avatar-fetch"
            }
            "--image-view" | "--no-image-view" => o.choices.image_view = a == "--image-view",
            "--radar" | "--no-radar" => o.choices.radar = a == "--radar",
            "--mermaid" | "--no-mermaid" => o.choices.mermaid = a == "--mermaid",
            "--board" | "--no-board" => o.choices.board = a == "--board",
            "--review" | "--no-review" => o.choices.review = a == "--review",
            "--conversation" | "--no-conversation" => {
                o.choices.conversation = a == "--conversation"
            }
            "--global-search" | "--no-global-search" => {
                o.choices.global_search = a == "--global-search"
            }
            "--workspaces" | "--no-workspaces" => o.choices.workspaces = a == "--workspaces",
            "--history-rewrite" | "--no-history-rewrite" => {
                o.choices.history_rewrite = a == "--history-rewrite"
            }
            "--rewrite-scripts" | "--no-rewrite-scripts" => {
                o.choices.rewrite_scripts = a == "--rewrite-scripts";
                explicit_scripts = true;
            }
            "-h" | "--help" => return Err(USAGE.to_string()),
            other => return Err(format!("unknown option '{other}'.\n\n{USAGE}")),
        }
    }
    o.choices.validate_logo()?;
    if explicit_scripts && !o.choices.history_rewrite {
        o.notes.push(
            "warning: --[no-]rewrite-scripts has no effect while history rewrite is off".into(),
        );
    }
    Ok(o)
}

/// Directory cargo writes the binaries to for this profile / `--target`.
fn out_dir(root: &Path, profile: &str, extra: &[String]) -> PathBuf {
    let mut target = None;
    let mut it = extra.iter();
    while let Some(a) = it.next() {
        if a == "--target" {
            target = it.next().cloned();
        } else if let Some(t) = a.strip_prefix("--target=") {
            target = Some(t.to_string());
        }
    }
    let dir = if profile == "dev" { "debug" } else { profile };
    let mut p = root.join("target");
    if let Some(t) = target {
        p.push(t);
    }
    p.join(dir)
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let root = repo_root();
    let langs = available_languages(&root)?;
    if args.iter().any(|a| a == "--list-langs") {
        for (code, name) in &langs {
            println!("{code}\t{name}");
        }
        return Ok(0);
    }
    let codes: Vec<String> = langs.iter().map(|(c, _)| c.clone()).collect();
    let theme_list = theme_families();
    if args.iter().any(|a| a == "--list-themes") {
        for (family, label) in &theme_list {
            println!("{family}\t{label}");
        }
        return Ok(0);
    }
    if args.iter().any(|a| a == "--list-logo-colors") {
        for (slug, label) in logo_colorways() {
            println!("{slug}\t{label}");
        }
        return Ok(0);
    }
    let families: Vec<String> = theme_list.iter().map(|(f, _)| f.clone()).collect();
    let presets = presets_dir();
    if args.iter().any(|a| a == "--list-presets") {
        let dir = presets.ok_or("no config directory on this machine")?;
        println!("# {}", dir.display());
        for name in list_presets(&dir) {
            println!("{name}");
        }
        return Ok(0);
    }
    if let Some(i) = args.iter().position(|a| a == "--delete-preset") {
        let name = args.get(i + 1).ok_or("--delete-preset needs a value")?;
        let file = delete_preset(name, presets.as_deref())?;
        println!("deleted {}", file.display());
        return Ok(0);
    }
    let o = parse_args(args, &root, &codes, &families, presets.as_deref())?;
    for n in &o.notes {
        eprintln!(
            "{}",
            if n.contains("warning:") {
                n.clone()
            } else {
                format!("note: {n}")
            }
        );
    }
    if let Some(spec) = &o.save {
        let file = save_preset(spec, presets.as_deref(), &o.choices)?;
        eprintln!("saved the selection to {}", file.display());
    }
    if o.print_choices {
        print!("{}", o.choices.to_env());
        return Ok(0);
    }
    let cargo_args = o.choices.cargo_args(&o.profile, o.full, &o.extra);
    println!("{}", o.choices.summary().trim_end());
    let env = if o.full { vec![] } else { o.choices.env() };
    let env_text: String = env.iter().map(|(k, v)| format!("{k}={v} ")).collect();
    println!("{env_text}RUSTC_WRAPPER= cargo {}", cargo_args.join(" "));
    if o.dry_run {
        return Ok(0);
    }

    let env_dir = root.join("target/custom-build");
    std::fs::create_dir_all(&env_dir).map_err(|e| e.to_string())?;
    std::fs::write(env_dir.join("last-choices.env"), o.choices.to_env())
        .map_err(|e| e.to_string())?;

    // sccache cannot cache LTO'd final links; same as build-opti.
    let status = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(&root)
        .env("RUSTC_WRAPPER", "")
        .envs(env)
        .status()
        .map_err(|e| format!("running cargo: {e}"))?;
    if !status.success() {
        return Ok(status.code().unwrap_or(1));
    }

    let out = out_dir(&root, &o.profile, &o.extra);
    let _ = std::fs::write(out.join("BUILD-FEATURES.txt"), o.choices.summary());
    println!("\nCustom binaries ({}):", out.display());
    let exe = std::env::consts::EXE_SUFFIX;
    for bin in [
        "gitpoulpi",
        "gitpoulpi-tui",
        "gitpoulpi-mcp",
        "gitpoulpi-launch",
        "gitpoulpi-updater",
    ] {
        let file = out.join(format!("{bin}{exe}"));
        let Ok(meta) = std::fs::metadata(&file) else {
            continue;
        };
        let mb = |n: u64| n as f64 / 1_048_576.0;
        let mut line = format!("  {bin:<15} {:6.1} MB", mb(meta.len()));
        if let Ok(d) = std::fs::metadata(root.join("target/dist").join(format!("{bin}{exe}"))) {
            let delta = mb(meta.len()) - mb(d.len());
            let _ = write!(line, "  ({delta:+.1} MB vs target/dist)");
        }
        println!("{line}");
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn avail() -> Vec<String> {
        ["de", "fr", "pt-br"].map(String::from).to_vec()
    }

    fn fams() -> Vec<String> {
        ["catppuccin", "nord", "nightfox"]
            .map(String::from)
            .to_vec()
    }

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn mapping_for_a_typical_choice() {
        let mut c = Choices::none();
        c.langs = s(&["fr"]);
        c.integrations = s(&["github"]);
        c.mcp = true;
        let a = c.cargo_args("custom", false, &[]);
        assert_eq!(
            a.join(" "),
            "build --profile custom --locked -p gitpoulpi-cli -p gitpoulpi-tui -p gitpoulpi-updater -p gitpoulpi-mcp \
             --no-default-features --features \
             gitpoulpi-cli/lang-fr,gitpoulpi-cli/github,gitpoulpi-cli/mcp,\
             gitpoulpi-tui/lang-fr,gitpoulpi-tui/github,gitpoulpi-tui/mcp,\
             gitpoulpi-mcp/lang-fr,gitpoulpi-mcp/github"
        );
    }

    #[test]
    fn the_new_providers_map_to_their_own_cargo_features() {
        let mut c = Choices::none();
        c.integrations = parse_integrations(
            "jira,trello,azure-devops,bitbucket,gitea,linear,youtrack,redmine,gerrit,local-issues",
        )
        .unwrap();
        let f = c.features();
        for p in [
            "azure-devops",
            "bitbucket",
            "jira",
            "trello",
            "gitea",
            "linear",
            "youtrack",
            "redmine",
            "gerrit",
            "local-issues",
        ] {
            for bin in ["cli", "tui"] {
                assert!(f.contains(&format!("gitpoulpi-{bin}/{p}")), "{bin}/{p}");
            }
        }
        assert_eq!(parse_integrations("all").unwrap().len(), INTEGRATIONS.len());
        assert!(parse_integrations("forgejo").is_err());
    }

    #[test]
    fn all_off_has_no_features_flag_and_no_mcp_package() {
        let a = Choices::none().cargo_args("custom", false, &[]);
        assert!(a.contains(&"--no-default-features".to_string()));
        assert!(!a.contains(&"--features".to_string()));
        assert!(!a.contains(&"gitpoulpi-mcp".to_string()));
    }

    #[test]
    fn mcp_off_drops_package_and_feature() {
        let mut c = Choices::all(&avail());
        c.mcp = false;
        let a = c.cargo_args("custom", false, &[]);
        assert!(!a.contains(&"gitpoulpi-mcp".to_string()));
        assert!(!c.features().iter().any(|f| f.ends_with("/mcp")));
        assert!(!c.features().iter().any(|f| f.starts_with("gitpoulpi-mcp/")));
        assert!(c.features().contains(&"gitpoulpi-cli/ai".to_string()));
    }

    #[test]
    fn no_integrations_never_names_hosting_features() {
        let mut c = Choices::all(&avail());
        c.integrations = vec![];
        let f = c.features();
        assert!(!f.iter().any(
            |x| INTEGRATIONS.iter().any(|i| x.ends_with(&format!("/{i}")))
                || x.ends_with("/hosting")
        ));
    }

    #[test]
    fn english_only_has_no_lang_features() {
        let mut c = Choices::all(&avail());
        c.langs = parse_langs("en", &avail()).unwrap();
        assert!(!c.features().iter().any(|f| f.contains("/lang-")));
    }

    #[test]
    fn full_skips_feature_selection_and_extra_args_follow() {
        let a = Choices::all(&avail()).cargo_args("dist", true, &s(&["--target", "x"]));
        assert!(!a.contains(&"--no-default-features".to_string()));
        assert_eq!(&a[a.len() - 2..], ["--target", "x"]);
    }

    #[test]
    fn avatar_fetch_goes_to_the_tui() {
        let f = Choices::all(&avail()).features();
        assert!(f.contains(&"gitpoulpi-tui/avatar-fetch".to_string()));
        let mut c = Choices::all(&avail());
        c.avatar_fetch = false;
        assert!(!c.features().iter().any(|f| f.contains("avatar")));
    }

    #[test]
    fn conversation_goes_to_every_binary_and_has_its_own_flag() {
        let f = Choices::all(&avail()).features();
        for bin in ["cli", "tui", "mcp"] {
            assert!(
                f.contains(&format!("gitpoulpi-{bin}/conversation")),
                "{bin}"
            );
        }
        let off = parse(&["--no-conversation", "--no-review"]).unwrap();
        assert!(!off
            .choices
            .features()
            .iter()
            .any(|x| x.contains("conversation")));
        assert!(parse(&[])
            .unwrap()
            .choices
            .summary()
            .contains("conversation"));
    }

    #[test]
    fn carved_out_features_go_to_every_binary_and_have_their_own_flags() {
        let all = Choices::all(&avail());
        for feat in [
            "radar",
            "mermaid",
            "board",
            "review",
            "history-rewrite",
            "global-search",
            "workspaces",
        ] {
            let f = all.features();
            for bin in ["cli", "tui", "mcp"] {
                assert!(
                    f.contains(&format!("gitpoulpi-{bin}/{feat}")),
                    "{bin} {feat}"
                );
            }
            let on = parse(&[]).unwrap();
            assert!(on.choices.features().iter().any(|x| x.ends_with(feat)));
            let off = parse(&[&format!("--no-{feat}")]).unwrap();
            assert!(
                !off.choices
                    .features()
                    .iter()
                    .any(|x| x.ends_with(&format!("/{feat}"))),
                "{feat}"
            );
            assert!(!off.choices.summary().contains(&format!(" {feat}")));
            let text = off.choices.to_env();
            let key = feat.to_ascii_uppercase().replace('-', "_");
            assert!(text.contains(&format!("{key}=0\n")), "{text}");
            assert_eq!(
                Choices::from_env(&text, &avail(), &fams()).unwrap(),
                off.choices
            );
        }
        assert!(!Choices::none()
            .features()
            .iter()
            .any(|f| f.ends_with("/radar")));
    }

    #[test]
    fn image_view_goes_to_the_tui() {
        let f = Choices::all(&avail()).features();
        assert!(f.contains(&"gitpoulpi-tui/image-view".to_string()));
        let mut c = Choices::all(&avail());
        c.image_view = false;
        assert!(!c.features().iter().any(|f| f.contains("image-view")));
    }

    #[test]
    fn env_round_trip() {
        let mut c = Choices::all(&avail());
        c.langs = s(&["de", "pt-br"]);
        c.integrations = vec![];
        c.ai = false;
        c.lfs = false;
        let text = c.to_env();
        assert!(text.contains("LANGS=de,pt-br\n"));
        assert!(text.contains("INTEGRATIONS=none\n"));
        assert_eq!(Choices::from_env(&text, &avail(), &fams()).unwrap(), c);
        let none = Choices::none();
        assert_eq!(
            Choices::from_env(&none.to_env(), &avail(), &fams()).unwrap(),
            none
        );
    }

    #[test]
    fn themes_default_to_all_and_set_the_build_script_env() {
        assert_eq!(Choices::all(&avail()).env(), vec![]);
        let mut c = Choices::all(&avail());
        c.themes = parse_themes("nord, Catppuccin", &fams()).unwrap();
        assert_eq!(
            c.env(),
            vec![("GITPOULPI_THEMES", "catppuccin,nord".to_string())]
        );
        assert!(c
            .summary()
            .contains("themes: dark light terminal catppuccin nord\n"));
        // Themes are an env var, never cargo features.
        assert!(!c.features().iter().any(|f| f.contains("theme")));
        assert_eq!(
            Choices::none().env(),
            vec![("GITPOULPI_THEMES", "none".to_string())]
        );
        assert!(Choices::all(&avail()).summary().contains("themes: all\n"));
    }

    fn parse(a: &[&str]) -> Result<Options, String> {
        let args: Vec<String> = a.iter().map(|x| x.to_string()).collect();
        parse_args(&args, Path::new("/nonexistent"), &avail(), &fams(), None)
    }

    #[test]
    fn logo_flags_set_the_build_script_env_in_table_order() {
        let o = parse(&[
            "--logo-colors",
            "teal,violet",
            "--logo-default",
            "teal",
            "--icon-color",
            "sun",
        ])
        .unwrap();
        assert_eq!(o.choices.logo_colors, Some(s(&["violet", "teal"])));
        let env = o.choices.env();
        assert!(env.contains(&("GITPOULPI_LOGO_COLORS", "violet,teal".to_string())));
        assert!(env.contains(&("GITPOULPI_LOGO_DEFAULT", "teal".to_string())));
        assert!(env.contains(&("GITPOULPI_ICON_COLOR", "sun".to_string())));
        let sum = o.choices.summary();
        assert!(
            sum.contains("logo: violet teal (default teal, icon sun)\n"),
            "{sum}"
        );
        // Nothing chosen: no env at all, lavender everywhere.
        let none = Choices::all(&avail());
        assert!(none.env().is_empty());
        assert!(none.summary().contains("default lavender, icon lavender)"));
    }

    #[test]
    fn logo_choices_are_validated_together() {
        assert!(parse(&["--logo-colors", "nope"])
            .unwrap_err()
            .contains("unknown logo color 'nope'"));
        assert!(parse(&["--logo-colors", "none"]).is_err());
        let e = parse(&["--logo-colors", "violet,teal", "--logo-default", "sun"]).unwrap_err();
        assert!(e.contains("not among the compiled"), "{e}");
        assert!(parse(&["--icon-color", "zzz"])
            .unwrap_err()
            .contains("unknown icon color"));
        // The icon color need not be compiled in.
        assert!(parse(&["--logo-colors", "violet", "--icon-color", "sun"]).is_ok());
        // Default left out, lavender not compiled: the first compiled one.
        let o = parse(&["--logo-colors", "violet,teal"]).unwrap();
        assert_eq!(o.choices.resolved_logo().1, "violet");
    }

    #[test]
    fn logo_choices_round_trip_through_last_choices() {
        let mut c = Choices::all(&avail());
        c.logo_colors = Some(s(&["violet", "teal"]));
        c.logo_default = Some("teal".into());
        c.icon_color = Some("sun".into());
        let text = c.to_env();
        assert!(text.contains("LOGO_COLORS=violet,teal\n"));
        assert!(text.contains("LOGO_DEFAULT=teal\n"));
        assert_eq!(Choices::from_env(&text, &avail(), &fams()).unwrap(), c);
        let all = Choices::all(&avail());
        assert!(all
            .to_env()
            .contains("LOGO_COLORS=all\nLOGO_DEFAULT=auto\nICON_COLOR=auto\n"));
        assert_eq!(
            Choices::from_env(&all.to_env(), &avail(), &fams()).unwrap(),
            all
        );
    }

    #[test]
    fn theme_selection_parsing_and_env_round_trip() {
        assert_eq!(parse_themes("all", &fams()).unwrap(), None);
        assert_eq!(parse_themes("none", &fams()).unwrap(), Some(vec![]));
        let e = parse_themes("nord,zzz", &fams()).unwrap_err();
        assert!(e.contains("'zzz'") && e.contains("catppuccin, nord"));
        let mut c = Choices::all(&avail());
        c.themes = Some(s(&["nord"]));
        let text = c.to_env();
        assert!(text.contains("THEMES=nord\n"));
        assert_eq!(Choices::from_env(&text, &avail(), &fams()).unwrap(), c);
        assert!(Choices::all(&avail()).to_env().contains("THEMES=all\n"));
    }

    #[test]
    fn the_real_theme_families_exclude_the_fixed_themes() {
        let fams = theme_families();
        let names: Vec<&str> = fams.iter().map(|(f, _)| f.as_str()).collect();
        for fixed in ["dark", "light", "terminal"] {
            assert!(!names.contains(&fixed));
        }
        for f in [
            "catppuccin",
            "nightfox",
            "zenbones",
            "rose-pine",
            "tokyo-night",
        ] {
            assert!(names.contains(&f), "{f} in {names:?}");
        }
        let cat = fams.iter().find(|(f, _)| f == "catppuccin").unwrap();
        assert_eq!(cat.1, "Catppuccin (4 themes)");
    }

    #[test]
    fn language_validation() {
        assert_eq!(
            parse_langs("FR, pt_BR,en,fr", &avail()).unwrap(),
            ["fr", "pt-br"]
        );
        assert_eq!(parse_langs("all", &avail()).unwrap().len(), 3);
        let e = parse_langs("fr,xx", &avail()).unwrap_err();
        assert!(e.contains("'xx'") && e.contains("de, fr, pt-br"));
        assert!(parse_integrations("gitlab-ce").is_err());
        assert!(parse_integrations("none").unwrap().is_empty());
    }

    #[test]
    fn real_locales_are_listed_with_native_names() {
        let langs = available_languages(&repo_root()).unwrap();
        assert!(langs.iter().all(|(c, _)| c != "en"));
        let fr = langs.iter().find(|(c, _)| c == "fr").expect("fr locale");
        assert_eq!(fr.1, "Français");
        assert!(langs.iter().any(|(c, _)| c == "pt-br"));
    }

    #[test]
    fn target_changes_the_output_dir() {
        let root = Path::new("/r");
        assert_eq!(out_dir(root, "custom", &[]), Path::new("/r/target/custom"));
        assert_eq!(
            out_dir(root, "custom", &s(&["--target", "a-b"])),
            Path::new("/r/target/a-b/custom")
        );
    }

    fn parse_in(a: &[&str], dir: &Path) -> Result<Options, String> {
        let args: Vec<String> = a.iter().map(|x| x.to_string()).collect();
        parse_args(
            &args,
            Path::new("/nonexistent"),
            &avail(),
            &fams(),
            Some(dir),
        )
    }

    #[test]
    fn rewrite_scripts_flag_maps_to_the_feature_and_the_env_file() {
        let on = parse(&[]).unwrap();
        assert!(on.choices.rewrite_scripts);
        assert!(on
            .choices
            .features()
            .contains(&"gitpoulpi-cli/rewrite-scripts".to_string()));
        let off = parse(&["--no-rewrite-scripts"]).unwrap();
        let f = off.choices.features();
        assert!(f.contains(&"gitpoulpi-cli/history-rewrite".to_string()));
        assert!(!f.iter().any(|x| x.ends_with("rewrite-scripts")));
        assert!(off.notes.is_empty());
        assert!(off.choices.to_env().contains("REWRITE_SCRIPTS=0\n"));
        assert!(on.choices.to_env().contains("REWRITE_SCRIPTS=1\n"));
        // The cargo line carries no Lua feature either.
        let args = off.choices.cargo_args("custom", false, &[]).join(" ");
        assert!(!args.contains("rewrite-scripts"), "{args}");
        // Turning the rewrite off drops Lua with it, and the flag warns.
        let hr = parse(&["--no-history-rewrite"]).unwrap();
        assert!(!hr.choices.features().iter().any(|x| x.contains("rewrite")));
        assert!(hr.notes.is_empty());
        let both = parse(&["--no-history-rewrite", "--rewrite-scripts"]).unwrap();
        assert!(both.notes.iter().any(|n| n.contains("no effect")));
        assert!(!both
            .choices
            .features()
            .iter()
            .any(|x| x.contains("rewrite")));
    }

    #[test]
    fn a_preset_round_trips_through_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = Choices::all(&avail());
        c.langs = s(&["fr"]);
        c.integrations = s(&["github"]);
        c.themes = Some(s(&["nord"]));
        c.logo_colors = Some(s(&["violet", "teal"]));
        c.logo_default = Some("teal".into());
        c.ai = false;
        c.rewrite_scripts = false;
        let file = save_preset("mine", Some(dir.path()), &c).unwrap();
        assert_eq!(file, dir.path().join("mine.env"));
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("LANGS=fr\n") && text.contains("REWRITE_SCRIPTS=0\n"));
        let (back, notes) = Choices::from_env_report(&text, &avail(), &fams()).unwrap();
        assert_eq!(back, c);
        assert!(notes.is_empty(), "{notes:?}");
        let o = parse_in(&["--preset", "mine"], dir.path()).unwrap();
        assert_eq!(o.choices, c);
        assert_eq!(list_presets(dir.path()), ["mine"]);
        delete_preset("mine", Some(dir.path())).unwrap();
        assert!(list_presets(dir.path()).is_empty());
        assert!(delete_preset("mine", Some(dir.path())).is_err());
    }

    #[test]
    fn flags_override_the_preset_wherever_they_stand() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = Choices::all(&avail());
        c.ai = false;
        c.mcp = false;
        save_preset("p", Some(dir.path()), &c).unwrap();
        for args in [
            ["--preset", "p", "--ai", "--langs", "de"],
            ["--ai", "--langs", "de", "--preset", "p"],
        ] {
            let o = parse_in(&args, dir.path()).unwrap();
            assert!(o.choices.ai, "{args:?}");
            assert!(!o.choices.mcp, "{args:?}");
            assert_eq!(o.choices.langs, ["de"]);
        }
        let o = parse_in(&["--preset", "p", "--save", "q"], dir.path()).unwrap();
        assert_eq!(o.save.as_deref(), Some("q"));
        assert!(parse_in(&["--preset", "absent"], dir.path())
            .unwrap_err()
            .contains("absent"));
        assert!(parse_in(&["--preset"], dir.path()).is_err());
    }

    #[test]
    fn a_preset_can_be_a_file_path() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("dot").join("small.env");
        let mut c = Choices::none();
        c.langs = s(&["de"]);
        assert_eq!(save_preset(file.to_str().unwrap(), None, &c).unwrap(), file);
        let o = parse_in(&["--preset", file.to_str().unwrap()], Path::new("/unused")).unwrap();
        assert_eq!(o.choices, c);
        // A bare `x.env` is a path too, relative to the current directory.
        assert_eq!(preset_file("x.env", None).unwrap(), Path::new("x.env"));
        assert!(preset_file(".hidden", Some(dir.path())).is_err());
        assert!(preset_file("", Some(dir.path())).is_err());
        assert!(preset_file("name", None).is_err());
        assert!(delete_preset("a/b.env", Some(dir.path())).is_err());
    }

    #[test]
    fn unknown_keys_and_values_warn_and_missing_keys_take_the_default() {
        let text = "LANGS=fr,xx\nINTEGRATIONS=github,forgejo2\nTHEMES=nord,gone\n\
                    LOGO_COLORS=nope\nLOGO_DEFAULT=zzz\nFUTURE_THING=1\nAI=0\n# comment\n";
        let (c, notes) = Choices::from_env_report(text, &avail(), &fams()).unwrap();
        assert_eq!(c.langs, ["fr"]);
        assert_eq!(c.integrations, ["github"]);
        assert_eq!(c.themes, Some(s(&["nord"])));
        assert_eq!(c.logo_colors, None);
        assert_eq!(c.logo_default, None);
        assert!(!c.ai);
        let all = notes.join("\n");
        assert!(all.contains("unknown key FUTURE_THING ignored"), "{all}");
        assert!(all.contains("language 'xx'"), "{all}");
        assert!(all.contains("integration 'forgejo2'"), "{all}");
        assert!(all.contains("theme family 'gone'"), "{all}");
        assert!(all.contains("logo color 'nope'"), "{all}");
        // Missing keys: default value, reported with it.
        assert!(c.rewrite_scripts && c.history_rewrite && c.mcp);
        assert!(
            all.contains("REWRITE_SCRIPTS is not in this selection: added with its default (1)"),
            "{all}"
        );
        assert!(!all.contains("AI is not in this selection"), "{all}");
    }

    #[test]
    fn an_integration_added_since_the_preset_was_saved_is_included_and_reported() {
        let text = "INTEGRATIONS=github\nKNOWN_INTEGRATIONS=github,gitlab,jira\n";
        let (c, notes) = Choices::from_env_report(text, &avail(), &fams()).unwrap();
        // gitlab and jira were known and left off; every later one is added.
        assert!(!c.integrations.contains(&"gitlab".to_string()));
        assert!(c.integrations.contains(&"github".to_string()));
        assert!(c.integrations.contains(&"gerrit".to_string()));
        assert!(notes
            .iter()
            .any(|n| n.contains("integration gerrit was added")));
        assert!(!notes
            .iter()
            .any(|n| n.contains("integration jira was added")));
        // Without the known-list (an old file) nothing is guessed.
        let (c, _) = Choices::from_env_report("INTEGRATIONS=github\n", &avail(), &fams()).unwrap();
        assert_eq!(c.integrations, ["github"]);
    }

    #[test]
    fn the_preset_flags_do_not_reach_cargo() {
        let o = parse(&["--no-rewrite-scripts", "--save", "x", "--dry-run"]).unwrap();
        let a = o.choices.cargo_args(&o.profile, o.full, &o.extra).join(" ");
        assert!(!a.contains("save") && !a.contains("preset"));
    }
}
