//! The "Edit in" image editors (`[ui] image_editors`, `[ui] image_editor_default`):
//! a list of labeled commands the image view offers for the picture on show.
//! A sibling of [`crate::open_in`], which it reuses for the entry type, the OS
//! enum and the argument splitter.
//!
//! Pure functions only: nothing is spawned and nothing reads the process
//! environment or the file system (the caller passes `env`, `on_path` and
//! `exists` closures), so every rule is table-testable with a fake machine.
//!
//! - **Expansion** ([`expand`]): the command is split into arguments first,
//!   then `${file}` (absolute path), `${dir}`, `${name}`, `${repo}` and
//!   `${env:NAME}` are substituted inside each argument, so a path with spaces
//!   stays one argument. No implicit shell. An unknown `${x}` is an error.
//! - **Presets** ([`presets`]): the catalog "Add preset" offers. `verified`
//!   means the flags were checked against the installed program's own help;
//!   the rest follow published documentation. The UI shows no flag either
//!   way.
//! - **Detection** ([`detect`]): which presets look installed, from `PATH`
//!   and known install locations. Nothing is spawned.
//! - **Effective list** ([`effective_list`]): while `ui.image_editors` is
//!   absent, the detected presets in catalog order, then the OS default app.

use crate::open_in::{OpenInEntry, Os};

/// What the variables stand for.
#[derive(Debug, Clone, Copy, Default)]
pub struct ImageVars<'a> {
    /// The image's absolute path (the real file or a read-only temp copy).
    pub file: &'a str,
    /// The folder holding it.
    pub dir: &'a str,
    /// Its file name.
    pub name: &'a str,
    /// The repository root, empty for a temp copy.
    pub repo: &'a str,
}

fn substitute(
    arg: &str,
    vars: &ImageVars,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let mut out = String::with_capacity(arg.len());
    let mut rest = arg;
    while let Some(i) = rest.find('$') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let Some(after) = rest.strip_prefix("${") else {
            out.push('$');
            rest = &rest[1..];
            continue;
        };
        let Some(end) = after.find('}') else {
            return Err(format!("unterminated ${{ in \"{arg}\""));
        };
        let name = &after[..end];
        match name {
            "file" => out.push_str(vars.file),
            "dir" => out.push_str(vars.dir),
            "name" => out.push_str(vars.name),
            "repo" => out.push_str(vars.repo),
            _ => match name.strip_prefix("env:") {
                Some(var) if !var.is_empty() => out.push_str(&env(var).unwrap_or_default()),
                _ => return Err(format!("unknown variable ${{{name}}}")),
            },
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Splits `command`, then substitutes the variables inside each argument.
/// An empty command or an unknown variable is an error.
pub fn expand(
    command: &str,
    vars: &ImageVars,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<String>, String> {
    let args = crate::launch::split_argv(command)?;
    if args.is_empty() {
        return Err("the command is empty".into());
    }
    args.iter().map(|a| substitute(a, vars, env)).collect()
}

/// Checks a command without running it: it splits, is not empty and names only
/// known variables. What Settings shows next to the Command field.
pub fn validate(command: &str) -> Result<(), String> {
    expand(command, &ImageVars::default(), &|_| None).map(|_| ())
}

// ------------------------------------------------------------------- presets

/// One built-in editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImagePreset {
    pub id: &'static str,
    pub label: &'static str,
    pub command: &'static str,
    /// `"linux"`, `"macos"`, `"windows"`.
    pub platforms: &'static [&'static str],
    /// The program looked up on `PATH`; empty when only `probes` decide.
    pub bin: &'static str,
    /// Known install locations (`${env:X}` is expanded), a file or a folder.
    /// With neither `bin` nor `probes` the preset is always available.
    pub probes: &'static [&'static str],
    /// A native program that makes this (Flatpak) preset redundant.
    pub unless_bin: &'static str,
    pub verified: bool,
}

impl ImagePreset {
    pub fn entry(&self) -> OpenInEntry {
        OpenInEntry::new(self.label, self.command)
    }

    pub fn available_on(&self, os: Os) -> bool {
        self.platforms.contains(&os_name(os))
    }
}

fn os_name(os: Os) -> &'static str {
    match os {
        Os::Windows => "windows",
        Os::Macos => "macos",
        Os::Linux => "linux",
    }
}

const LINUX: &[&str] = &["linux"];
const MACOS: &[&str] = &["macos"];
const WINDOWS: &[&str] = &["windows"];

const fn linux(
    id: &'static str,
    label: &'static str,
    command: &'static str,
    bin: &'static str,
) -> ImagePreset {
    ImagePreset {
        id,
        label,
        command,
        platforms: LINUX,
        bin,
        probes: &[],
        unless_bin: "",
        verified: false,
    }
}

const fn flatpak(
    id: &'static str,
    label: &'static str,
    command: &'static str,
    probes: &'static [&'static str],
    unless_bin: &'static str,
) -> ImagePreset {
    ImagePreset {
        id,
        label,
        command,
        platforms: LINUX,
        bin: "",
        probes,
        unless_bin,
        verified: false,
    }
}

const fn windows(
    id: &'static str,
    label: &'static str,
    command: &'static str,
    probes: &'static [&'static str],
) -> ImagePreset {
    ImagePreset {
        id,
        label,
        command,
        platforms: WINDOWS,
        bin: "",
        probes,
        unless_bin: "",
        verified: false,
    }
}

const fn mac(
    id: &'static str,
    label: &'static str,
    command: &'static str,
    probes: &'static [&'static str],
) -> ImagePreset {
    ImagePreset {
        id,
        label,
        command,
        platforms: MACOS,
        bin: "",
        probes,
        unless_bin: "",
        verified: false,
    }
}

const PRESETS: &[ImagePreset] = &[
    // Linux, on PATH.
    linux("gimp", "GIMP", "gimp ${file}", "gimp"),
    linux("krita", "Krita", "krita ${file}", "krita"),
    linux("inkscape", "Inkscape", "inkscape ${file}", "inkscape"),
    linux("pinta", "Pinta", "pinta ${file}", "pinta"),
    linux(
        "pixelorama",
        "Pixelorama",
        "pixelorama ${file}",
        "pixelorama",
    ),
    linux("aseprite", "Aseprite", "aseprite ${file}", "aseprite"),
    linux(
        "kolourpaint",
        "KolourPaint",
        "kolourpaint ${file}",
        "kolourpaint",
    ),
    linux("mtpaint", "mtPaint", "mtpaint ${file}", "mtpaint"),
    linux("darktable", "darktable", "darktable ${file}", "darktable"),
    // Linux, Flatpak: checked on disk, never by running flatpak.
    flatpak(
        "gimp-flatpak",
        "GIMP (Flatpak)",
        "flatpak run org.gimp.GIMP ${file}",
        &[
            "/var/lib/flatpak/app/org.gimp.GIMP",
            "${env:HOME}/.local/share/flatpak/app/org.gimp.GIMP",
        ],
        "gimp",
    ),
    flatpak(
        "krita-flatpak",
        "Krita (Flatpak)",
        "flatpak run org.kde.krita ${file}",
        &[
            "/var/lib/flatpak/app/org.kde.krita",
            "${env:HOME}/.local/share/flatpak/app/org.kde.krita",
        ],
        "krita",
    ),
    flatpak(
        "inkscape-flatpak",
        "Inkscape (Flatpak)",
        "flatpak run org.inkscape.Inkscape ${file}",
        &[
            "/var/lib/flatpak/app/org.inkscape.Inkscape",
            "${env:HOME}/.local/share/flatpak/app/org.inkscape.Inkscape",
        ],
        "inkscape",
    ),
    // Windows, known install paths (newest first). Affinity Photo comes from
    // the Microsoft Store with no stable executable path: write a custom entry.
    windows(
        "paintnet",
        "Paint.NET",
        "\"${env:ProgramFiles}\\paint.net\\paintdotnet.exe\" ${file}",
        &["${env:ProgramFiles}\\paint.net\\paintdotnet.exe"],
    ),
    windows(
        "gimp-3-win",
        "GIMP 3",
        "\"${env:ProgramFiles}\\GIMP 3\\bin\\gimp-3.0.exe\" ${file}",
        &["${env:ProgramFiles}\\GIMP 3\\bin\\gimp-3.0.exe"],
    ),
    windows(
        "gimp-2-win",
        "GIMP 2",
        "\"${env:ProgramFiles}\\GIMP 2\\bin\\gimp-2.10.exe\" ${file}",
        &["${env:ProgramFiles}\\GIMP 2\\bin\\gimp-2.10.exe"],
    ),
    windows(
        "krita-win",
        "Krita",
        "\"${env:ProgramFiles}\\Krita (x64)\\bin\\krita.exe\" ${file}",
        &["${env:ProgramFiles}\\Krita (x64)\\bin\\krita.exe"],
    ),
    windows(
        "inkscape-win",
        "Inkscape",
        "\"${env:ProgramFiles}\\Inkscape\\bin\\inkscape.exe\" ${file}",
        &["${env:ProgramFiles}\\Inkscape\\bin\\inkscape.exe"],
    ),
    windows(
        "photoshop-2026-win",
        "Photoshop 2026",
        "\"${env:ProgramFiles}\\Adobe\\Adobe Photoshop 2026\\Photoshop.exe\" ${file}",
        &["${env:ProgramFiles}\\Adobe\\Adobe Photoshop 2026\\Photoshop.exe"],
    ),
    windows(
        "photoshop-2025-win",
        "Photoshop 2025",
        "\"${env:ProgramFiles}\\Adobe\\Adobe Photoshop 2025\\Photoshop.exe\" ${file}",
        &["${env:ProgramFiles}\\Adobe\\Adobe Photoshop 2025\\Photoshop.exe"],
    ),
    windows(
        "photoshop-2024-win",
        "Photoshop 2024",
        "\"${env:ProgramFiles}\\Adobe\\Adobe Photoshop 2024\\Photoshop.exe\" ${file}",
        &["${env:ProgramFiles}\\Adobe\\Adobe Photoshop 2024\\Photoshop.exe"],
    ),
    windows(
        "aseprite-win",
        "Aseprite",
        "\"${env:ProgramFiles(x86)}\\Steam\\steamapps\\common\\Aseprite\\Aseprite.exe\" ${file}",
        &["${env:ProgramFiles(x86)}\\Steam\\steamapps\\common\\Aseprite\\Aseprite.exe"],
    ),
    // macOS, `/Applications/*.app` (or `~/Applications`).
    mac(
        "pixelmator-mac",
        "Pixelmator Pro",
        "open -a \"Pixelmator Pro\" ${file}",
        &[
            "/Applications/Pixelmator Pro.app",
            "${env:HOME}/Applications/Pixelmator Pro.app",
        ],
    ),
    mac(
        "gimp-mac",
        "GIMP",
        "open -a GIMP ${file}",
        &[
            "/Applications/GIMP.app",
            "${env:HOME}/Applications/GIMP.app",
        ],
    ),
    mac(
        "krita-mac",
        "Krita",
        "open -a Krita ${file}",
        &[
            "/Applications/krita.app",
            "/Applications/Krita.app",
            "${env:HOME}/Applications/Krita.app",
        ],
    ),
    mac(
        "inkscape-mac",
        "Inkscape",
        "open -a Inkscape ${file}",
        &[
            "/Applications/Inkscape.app",
            "${env:HOME}/Applications/Inkscape.app",
        ],
    ),
    mac(
        "affinity-mac",
        "Affinity Photo 2",
        "open -a \"Affinity Photo 2\" ${file}",
        &["/Applications/Affinity Photo 2.app"],
    ),
    mac(
        "photoshop-2026-mac",
        "Photoshop 2026",
        "open -a \"Adobe Photoshop 2026\" ${file}",
        &["/Applications/Adobe Photoshop 2026/Adobe Photoshop 2026.app"],
    ),
    mac(
        "photoshop-2025-mac",
        "Photoshop 2025",
        "open -a \"Adobe Photoshop 2025\" ${file}",
        &["/Applications/Adobe Photoshop 2025/Adobe Photoshop 2025.app"],
    ),
    mac(
        "photoshop-2024-mac",
        "Photoshop 2024",
        "open -a \"Adobe Photoshop 2024\" ${file}",
        &["/Applications/Adobe Photoshop 2024/Adobe Photoshop 2024.app"],
    ),
    mac(
        "aseprite-mac",
        "Aseprite",
        "open -a Aseprite ${file}",
        &[
            "/Applications/Aseprite.app",
            "${env:HOME}/Applications/Aseprite.app",
        ],
    ),
    // Always there.
    ImagePreset {
        id: "mspaint",
        label: "Paint",
        command: "mspaint.exe ${file}",
        platforms: WINDOWS,
        bin: "",
        probes: &[],
        unless_bin: "",
        verified: false,
    },
    ImagePreset {
        id: "preview",
        label: "Preview",
        command: "open -a Preview ${file}",
        platforms: MACOS,
        bin: "",
        probes: &[],
        unless_bin: "",
        verified: false,
    },
    ImagePreset {
        id: "default-windows",
        label: "Default app",
        command: "cmd /C start \"\" ${file}",
        platforms: WINDOWS,
        bin: "",
        probes: &[],
        unless_bin: "",
        verified: false,
    },
    ImagePreset {
        id: "default-macos",
        label: "Default app",
        command: "open ${file}",
        platforms: MACOS,
        bin: "",
        probes: &[],
        unless_bin: "",
        verified: false,
    },
    ImagePreset {
        id: "default-linux",
        label: "Default app",
        command: "xdg-open ${file}",
        platforms: LINUX,
        bin: "",
        probes: &[],
        unless_bin: "",
        verified: false,
    },
];

/// The whole catalog, in the order the computed list uses.
pub fn presets() -> &'static [ImagePreset] {
    PRESETS
}

pub fn preset(id: &str) -> Option<&'static ImagePreset> {
    PRESETS.iter().find(|p| p.id == id)
}

/// The presets this OS offers, in catalog order.
pub fn presets_for(os: Os) -> Vec<&'static ImagePreset> {
    PRESETS.iter().filter(|p| p.available_on(os)).collect()
}

/// For "Add preset": what was detected first, then the rest, each in catalog
/// order.
pub fn picker_order(os: Os, detected: &Detected) -> Vec<&'static ImagePreset> {
    let all = presets_for(os);
    let (found, other): (Vec<_>, Vec<_>) = all.into_iter().partition(|p| detected.has(p.id));
    found.into_iter().chain(other).collect()
}

// ----------------------------------------------------------------- detection

/// What [`detect`] found: the ids of the presets that look installed, in
/// catalog order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Detected {
    pub ids: Vec<&'static str>,
}

impl Detected {
    pub fn has(&self, id: &str) -> bool {
        self.ids.contains(&id)
    }
}

/// Which presets of `os` look installed. `on_path` answers "is this program
/// on `PATH`", `exists` "is there a file or folder here", `env` reads an
/// environment variable; none of them spawns anything.
pub fn detect(
    env: &dyn Fn(&str) -> Option<String>,
    os: Os,
    on_path: &dyn Fn(&str) -> bool,
    exists: &dyn Fn(&str) -> bool,
) -> Detected {
    let mut found = Detected::default();
    for p in presets_for(os) {
        if !p.unless_bin.is_empty() && on_path(p.unless_bin) {
            continue;
        }
        let always = p.bin.is_empty() && p.probes.is_empty();
        let hit = always
            || (!p.bin.is_empty() && on_path(p.bin))
            || p.probes.iter().any(|probe| {
                // A probe naming an unset variable is a path that does not
                // exist, not the root of the disk.
                expand_probe(probe, env).is_some_and(|path| exists(&path))
            });
        if hit {
            found.ids.push(p.id);
        }
    }
    found
}

fn expand_probe(probe: &str, env: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    let missing = std::cell::Cell::new(false);
    let out = substitute(probe, &ImageVars::default(), &|k| {
        let v = env(k).filter(|v| !v.is_empty());
        if v.is_none() {
            missing.set(true);
        }
        v
    })
    .ok()?;
    (!missing.get()).then_some(out)
}

// ------------------------------------------------------------ effective list

/// The list that applies while `ui.image_editors` is absent: the detected
/// presets in catalog order, which ends with the OS default app.
pub fn effective_list(detected: &Detected, os: Os) -> Vec<OpenInEntry> {
    presets_for(os)
        .into_iter()
        .filter(|p| detected.has(p.id))
        .map(ImagePreset::entry_of)
        .collect()
}

impl ImagePreset {
    fn entry_of(p: &ImagePreset) -> OpenInEntry {
        p.entry()
    }
}

/// The index of the default entry: the one labeled `default_label`, else the
/// first (an empty or stale label falls back quietly).
pub fn default_index(entries: &[OpenInEntry], default_label: &str) -> usize {
    crate::open_in::default_index(entries, default_label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| map.get(k).cloned()
    }

    fn machine(os: Os, env: &[(&str, &str)], path: &[&str], files: &[&str]) -> Detected {
        let path: HashSet<String> = path.iter().map(|s| s.to_string()).collect();
        let files: HashSet<String> = files.iter().map(|s| s.to_string()).collect();
        detect(&env_of(env), os, &|p| path.contains(p), &|f| {
            files.contains(f)
        })
    }

    fn vars() -> ImageVars<'static> {
        ImageVars {
            file: "/tmp/my pics/a b.png",
            dir: "/tmp/my pics",
            name: "a b.png",
            repo: "/home/me/repo",
        }
    }

    #[test]
    fn a_path_with_spaces_stays_one_argument() {
        let argv = expand("gimp ${file}", &vars(), &|_| None).unwrap();
        assert_eq!(argv, vec!["gimp", "/tmp/my pics/a b.png"]);
        let argv = expand(
            "tool --dir '${dir}' --name=${name} -r ${repo}",
            &vars(),
            &|_| None,
        )
        .unwrap();
        assert_eq!(
            argv,
            vec![
                "tool",
                "--dir",
                "/tmp/my pics",
                "--name=a b.png",
                "-r",
                "/home/me/repo"
            ]
        );
    }

    #[test]
    fn env_variables_expand_and_an_unset_one_is_empty() {
        let env = env_of(&[("HOME", "/home/me")]);
        let argv = expand("t ${env:HOME}/x ${env:NOPE}", &vars(), &env).unwrap();
        assert_eq!(argv, vec!["t", "/home/me/x", ""]);
    }

    #[test]
    fn an_unknown_variable_or_an_empty_command_is_an_error() {
        let err = expand("t ${cwd}", &vars(), &|_| None).unwrap_err();
        assert!(err.contains("${cwd}"), "{err}");
        assert!(expand("   ", &vars(), &|_| None).is_err());
        assert!(expand("t ${file", &vars(), &|_| None).is_err());
        assert!(expand("t 'unterminated", &vars(), &|_| None).is_err());
        assert!(validate("gimp ${file}").is_ok());
        assert!(validate("gimp ${nope}").is_err());
    }

    #[test]
    fn a_dollar_that_does_not_open_a_variable_is_literal() {
        let argv = expand("sh -c 'echo $HOME ${file}'", &vars(), &|_| None).unwrap();
        assert_eq!(argv[2], "echo $HOME /tmp/my pics/a b.png");
    }

    #[test]
    fn every_preset_parses_and_expands_with_a_file_variable() {
        let env = env_of(&[
            ("ProgramFiles", "C:\\Program Files"),
            ("ProgramFiles(x86)", "C:\\Program Files (x86)"),
            ("HOME", "/home/me"),
        ]);
        let mut ids = HashSet::new();
        for p in presets() {
            assert!(ids.insert(p.id), "duplicate id {}", p.id);
            assert!(!p.platforms.is_empty(), "{}", p.id);
            validate(p.command).unwrap_or_else(|e| panic!("{}: {e}", p.id));
            assert!(p.command.contains("${file}"), "{} must take the file", p.id);
            let argv = expand(p.command, &vars(), &env).unwrap();
            assert!(!argv[0].is_empty(), "{}", p.id);
            assert!(
                argv.iter().any(|a| a == "/tmp/my pics/a b.png"),
                "{}: the file is one argument: {argv:?}",
                p.id
            );
            for probe in p.probes {
                let path = expand_probe(probe, &env).unwrap_or_else(|| panic!("{}", p.id));
                assert!(!path.contains("${"), "{}: {path}", p.id);
            }
        }
    }

    #[test]
    fn linux_detects_path_programs_and_flatpak_installs() {
        let d = machine(
            Os::Linux,
            &[("HOME", "/home/me")],
            &["krita", "inkscape"],
            &["/home/me/.local/share/flatpak/app/org.gimp.GIMP"],
        );
        assert_eq!(
            d.ids,
            vec!["krita", "inkscape", "gimp-flatpak", "default-linux"]
        );
        let entries = effective_list(&d, Os::Linux);
        assert_eq!(entries.last().unwrap().command, "xdg-open ${file}");
        assert_eq!(entries[2].command, "flatpak run org.gimp.GIMP ${file}");
    }

    #[test]
    fn a_native_install_hides_its_flatpak_twin() {
        let d = machine(
            Os::Linux,
            &[("HOME", "/h")],
            &["gimp"],
            &["/var/lib/flatpak/app/org.gimp.GIMP"],
        );
        assert!(d.has("gimp") && !d.has("gimp-flatpak"));
    }

    #[test]
    fn a_bare_linux_machine_has_only_the_default_app() {
        let d = machine(Os::Linux, &[], &[], &[]);
        assert_eq!(d.ids, vec!["default-linux"]);
        let list = effective_list(&d, Os::Linux);
        assert_eq!(
            list,
            vec![OpenInEntry::new("Default app", "xdg-open ${file}")]
        );
    }

    #[test]
    fn windows_detects_known_install_paths_and_always_has_paint() {
        let env = [
            ("ProgramFiles", "C:\\Program Files"),
            ("ProgramFiles(x86)", "C:\\Program Files (x86)"),
        ];
        let d = machine(
            Os::Windows,
            &env,
            &[],
            &[
                "C:\\Program Files\\GIMP 3\\bin\\gimp-3.0.exe",
                "C:\\Program Files\\Adobe\\Adobe Photoshop 2025\\Photoshop.exe",
            ],
        );
        assert_eq!(
            d.ids,
            vec![
                "gimp-3-win",
                "photoshop-2025-win",
                "mspaint",
                "default-windows"
            ]
        );
        // No ProgramFiles variable: those probes just do not match.
        let d = machine(Os::Windows, &[], &[], &["\\paint.net\\paintdotnet.exe"]);
        assert_eq!(d.ids, vec!["mspaint", "default-windows"]);
    }

    #[test]
    fn macos_detects_applications_and_always_has_preview() {
        let d = machine(
            Os::Macos,
            &[("HOME", "/Users/me")],
            &[],
            &[
                "/Applications/Pixelmator Pro.app",
                "/Users/me/Applications/Aseprite.app",
                "/Applications/Adobe Photoshop 2025/Adobe Photoshop 2025.app",
            ],
        );
        assert_eq!(
            d.ids,
            vec![
                "pixelmator-mac",
                "photoshop-2025-mac",
                "aseprite-mac",
                "preview",
                "default-macos"
            ]
        );
        let list = effective_list(&d, Os::Macos);
        assert_eq!(list[0].command, "open -a \"Pixelmator Pro\" ${file}");
    }

    #[test]
    fn presets_for_an_os_never_offer_another_oss_programs() {
        for os in [Os::Linux, Os::Windows, Os::Macos] {
            let list = presets_for(os);
            assert!(list.iter().all(|p| p.available_on(os)));
            assert!(list.iter().any(|p| p.id.starts_with("default-")));
        }
        assert!(presets_for(Os::Linux).iter().all(|p| p.id != "mspaint"));
    }

    #[test]
    fn the_picker_lists_the_detected_presets_first() {
        let d = machine(Os::Linux, &[], &["krita"], &[]);
        let order = picker_order(Os::Linux, &d);
        assert_eq!(order[0].id, "krita");
        assert_eq!(order[1].id, "default-linux");
        assert_eq!(order.len(), presets_for(Os::Linux).len());
    }

    #[test]
    fn the_default_index_falls_back_to_the_first_entry() {
        let list = vec![
            OpenInEntry::new("A", "a ${file}"),
            OpenInEntry::new("B", "b ${file}"),
        ];
        assert_eq!(default_index(&list, "B"), 1);
        assert_eq!(default_index(&list, ""), 0);
        assert_eq!(default_index(&list, "gone"), 0);
        assert_eq!(default_index(&[], "B"), 0);
    }
}
