//! The menu bar's content, generated from the registry. Pure data: no window involved, so the structure
//! is unit-tested.
//!
//! Every palette command of the catalogue sits in exactly one heading, under the category its label
//! names (`Git: Pull` is under `Git`, in whichever heading holds that category). Nothing is invented:
//! the only item that is not a registry command is `File > Open Recent`, the desktop's list of recent
//! repositories.

use gitpoulpi_app::features::offered;
use gitpoulpi_app::keymap::chord::{Key, KeyChord, Mods};
use gitpoulpi_app::keymap::{Ctx, Keymap};
use gitpoulpi_app::palette::{
    command_requires, entry_shortcut, unavailable_reason, PaletteCommand, PaletteContext,
    BADGE_SUFFIX, PALETTE_COMMANDS,
};

use crate::commands::{op_for, Command};
use crate::strings;

/// The top-level menus, in bar order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Heading {
    File,
    Edit,
    View,
    Repository,
    Branch,
    Git,
    Hosting,
    Tools,
    Help,
}

impl Heading {
    pub const ALL: [Heading; 9] = [
        Heading::File,
        Heading::Edit,
        Heading::View,
        Heading::Repository,
        Heading::Branch,
        Heading::Git,
        Heading::Hosting,
        Heading::Tools,
        Heading::Help,
    ];

    /// The title in the UI language.
    pub fn title(self) -> String {
        use gitpoulpi_i18n::msg;
        match self {
            Heading::File => strings::MENU_FILE.to_string(),
            Heading::Edit => strings::MENU_EDIT.to_string(),
            Heading::View => strings::MENU_VIEW.to_string(),
            Heading::Repository => strings::MENU_REPOSITORY.to_string(),
            Heading::Branch => msg::toolbar::BRANCH.get().to_string(),
            Heading::Git => strings::MENU_GIT.to_string(),
            Heading::Hosting => msg::help_tabs::TAB_HOSTING.get().to_string(),
            Heading::Tools => strings::MENU_TOOLS.to_string(),
            Heading::Help => msg::keymap_action::GLOBAL_HELP.get().to_string(),
        }
    }
}

/// The categories (the part of a palette label before `: `, in English) each heading holds, in the
/// order its submenus are listed.
const CATEGORIES: &[(Heading, &[&str])] = &[
    (
        Heading::File,
        &[
            "Tab",
            "Workspace",
            "Repositories",
            "Radar",
            "File",
            "Launcher",
            "Updates",
            "Quit",
        ],
    ),
    (
        Heading::Edit,
        &[
            "Edit",
            "Copy",
            "Search",
            "Composer",
            "Editor",
            "Markdown Editor",
        ],
    ),
    (
        Heading::View,
        &[
            "View",
            "Navigation",
            "Left pane",
            "Sidebar",
            "Log",
            "Files",
            "Diff",
            "Image",
            "Blame",
            "File History",
            "Commit Details",
            "Activity Log",
            "Jobs",
            "Dialog",
        ],
    ),
    (
        Heading::Repository,
        &[
            "Repository",
            "Remote",
            "Working Changes",
            "Commit",
            "Conflicts",
            "Banner",
            "Flow",
            "LFS",
            "Sensitive Files",
        ],
    ),
    (Heading::Branch, &["Refs"]),
    (
        Heading::Git,
        &[
            "Git",
            "Rebase",
            "git am",
            "Rewrite",
            "Merge",
            "Undo History",
            "Reflog",
            "Reflog / Lost Work",
            "Lost Work",
        ],
    ),
    (
        Heading::Hosting,
        &[
            "Hosting",
            "Integrations",
            "Pull Requests",
            "Issues",
            "Local Issues",
            "Review",
            "Conversation",
            "Board",
        ],
    ),
    (
        Heading::Tools,
        &["AI", "Agent", "Sequence", "App", "Settings", "Keybindings"],
    ),
    (Heading::Help, &["Help", "Release Notes"]),
];

/// Splits a palette label at its category colon (`Git: Pull` becomes `Git`, `Pull`); the colon may be
/// full-width, or preceded by a no-break space, in a translation.
pub fn split_category(label: &str) -> (&str, &str) {
    match label.find([':', '\u{ff1a}']) {
        Some(at) => {
            let colon_len = label[at..].chars().next().map_or(1, char::len_utf8);
            (
                label[..at].trim_end_matches(['\u{a0}', ' ']),
                label[at + colon_len..].trim_start(),
            )
        }
        None => ("", label),
    }
}

/// The category group a command belongs to, in English (`Quit` is the one command placed by itself,
/// under `File`).
fn english_category(command: PaletteCommand, label_en: &str) -> String {
    if command == PaletteCommand::Quit {
        return "Quit".to_string();
    }
    split_category(label_en).0.to_string()
}

/// The heading holding an English category.
pub fn heading_of(category: &str) -> Option<Heading> {
    CATEGORIES
        .iter()
        .find(|(_, cats)| cats.contains(&category))
        .map(|(h, _)| *h)
}

/// One entry of a menu.
#[derive(Debug, Clone)]
pub enum Action {
    /// A command of the registry.
    Command(Command),
    /// A recent repository (path).
    Recent(String),
}

#[derive(Debug, Clone)]
pub struct Item {
    pub action: Action,
    pub label: String,
    /// The key that runs it, as the terminal UI shows it.
    pub shortcut: String,
    /// Why it cannot be used now; `Some` means the item is disabled.
    pub disabled: Option<String>,
}

/// A submenu: the commands of one category.
#[derive(Debug, Clone)]
pub struct Group {
    pub title: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub heading: Heading,
    pub title: String,
    pub mnemonic: Option<char>,
    pub groups: Vec<Group>,
}

impl Menu {
    /// A menu with one group is shown flat.
    pub fn is_flat(&self) -> bool {
        self.groups.len() == 1
    }

    /// Every item, in order.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.groups.iter().flat_map(|g| g.items.iter())
    }
}

/// Builds the menus for a window state, the keymap and the recent repositories.
pub fn build(ctx: PaletteContext, km: &Keymap, recent: &[String]) -> Vec<Menu> {
    let mut menus: Vec<Menu> = Heading::ALL
        .iter()
        .map(|h| Menu {
            heading: *h,
            title: h.title(),
            mnemonic: None,
            groups: Vec::new(),
        })
        .collect();

    for entry in PALETTE_COMMANDS {
        if !offered(entry.requires) || !offered(command_requires(&entry.command)) {
            continue;
        }
        if !ctx.ai_enabled && entry.command.needs_ai() {
            continue;
        }
        let en_category = english_category(entry.command, entry.english());
        let Some(heading) = heading_of(&en_category) else {
            continue;
        };
        let command = Command::Palette(entry.command);
        let (category, body) = display_parts(entry);
        let (category, body) = if entry.command == PaletteCommand::Quit {
            (strings::MENU_FILE.to_string(), body)
        } else {
            (category, body)
        };
        let disabled = if op_for(command).is_none() {
            Some(strings::NOT_AVAILABLE_YET.to_string())
        } else {
            unavailable_reason(entry.command, ctx).map(str::to_string)
        };
        let item = Item {
            action: Action::Command(command),
            label: body,
            shortcut: entry_shortcut(km, entry).replace(BADGE_SUFFIX, ""),
            disabled,
        };
        let menu = &mut menus[Heading::ALL.iter().position(|h| *h == heading).unwrap_or(0)];
        match menu.groups.iter_mut().find(|g| g.title == category) {
            Some(group) => group.items.push(item),
            None => menu.groups.push(Group {
                title: category,
                items: vec![item],
            }),
        }
    }

    // Groups in the order the table above lists their categories (display titles are in the UI
    // language, so order by the English position kept in `CATEGORIES`).
    for menu in &mut menus {
        let order = CATEGORIES
            .iter()
            .find(|(h, _)| *h == menu.heading)
            .map(|(_, c)| *c)
            .unwrap_or(&[]);
        let rank = |group: &Group| -> usize {
            let first = group.items.first().and_then(|i| match &i.action {
                Action::Command(Command::Palette(c)) => crate::commands::catalog::entry_of(*c),
                _ => None,
            });
            first
                .map(|e| english_category(e.command, e.english()))
                .and_then(|c| order.iter().position(|x| *x == c))
                .unwrap_or(usize::MAX)
        };
        menu.groups.sort_by_key(rank);
    }

    // File > Open Recent: the desktop's own list.
    if !recent.is_empty() {
        let file = &mut menus[0];
        file.groups.insert(
            0,
            Group {
                title: gitpoulpi_i18n::msg::home::SECTION_RECENT.get().to_string(),
                items: recent
                    .iter()
                    .map(|path| Item {
                        action: Action::Recent(path.clone()),
                        label: path.clone(),
                        shortcut: String::new(),
                        disabled: None,
                    })
                    .collect(),
            },
        );
    }

    menus.retain(|m| !m.groups.is_empty());
    let titles: Vec<String> = menus.iter().map(|m| m.title.clone()).collect();
    for (menu, mnemonic) in menus.iter_mut().zip(mnemonics(&titles, km)) {
        menu.mnemonic = mnemonic;
    }
    menus
}

/// The category and the rest of an entry's label as the menu shows them. The category is the shared
/// registry's (`PaletteEntry::category`); only a label that spells its colon the full-width way (the
/// Japanese and Chinese catalogs) is split here, because the registry looks for `": "`.
fn display_parts(entry: &gitpoulpi_app::palette::PaletteEntry) -> (String, String) {
    let label = entry.display_label();
    match entry.category() {
        "" => {
            let (category, body) = split_category(&label);
            (category.to_string(), body.to_string())
        }
        category => {
            // An AI command's label carries its sparkle in front of the name.
            let lead = label.strip_suffix(entry.name()).unwrap_or("");
            let body = entry.name()[category.len()..]
                .trim_start_matches(':')
                .trim_start();
            (format!("{lead}{category}"), body.to_string())
        }
    }
}

/// The Alt + letter chord of a mnemonic.
pub fn mnemonic_chord(c: char) -> KeyChord {
    KeyChord::new(
        Mods {
            ctrl: false,
            alt: true,
            shift: false,
        },
        Key::Char(c.to_ascii_lowercase()),
    )
}

/// One mnemonic per title: the first ASCII letter of the title that no action holds as Alt + letter
/// and no earlier menu took (one key, one meaning). A title with no such letter has none.
pub fn mnemonics(titles: &[String], km: &Keymap) -> Vec<Option<char>> {
    let mut taken: Vec<char> = Vec::new();
    titles
        .iter()
        .map(|title| {
            let pick = title
                .chars()
                .filter(char::is_ascii_alphabetic)
                .map(|c| c.to_ascii_lowercase())
                .find(|c| {
                    !taken.contains(c) && km.action_exact(Ctx::Global, mnemonic_chord(*c)).is_none()
                });
            if let Some(c) = pick {
                taken.push(c);
            }
            pick
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn menus() -> Vec<Menu> {
        build(PaletteContext::default(), &Keymap::defaults(), &[])
    }

    /// A category added to the catalogue must be placed in a menu on purpose.
    #[test]
    fn every_palette_category_has_a_menu() {
        let mut unplaced = BTreeSet::new();
        for entry in PALETTE_COMMANDS {
            let category = english_category(entry.command, entry.english());
            if heading_of(&category).is_none() {
                unplaced.insert(category);
            }
        }
        assert!(unplaced.is_empty(), "unplaced categories: {unplaced:?}");
    }

    #[test]
    fn a_category_is_in_one_heading_only() {
        let mut seen = BTreeSet::new();
        for (_, cats) in CATEGORIES {
            for c in *cats {
                assert!(seen.insert(*c), "{c} is listed twice");
            }
        }
    }

    #[test]
    fn the_bar_has_the_classic_headings() {
        let titles: Vec<_> = menus().iter().map(|m| m.heading).collect();
        for h in [Heading::File, Heading::Edit, Heading::View, Heading::Help] {
            assert!(titles.contains(&h), "{h:?}");
        }
    }

    #[test]
    fn every_offered_command_is_in_a_menu() {
        let in_menus: usize = menus().iter().flat_map(|m| m.items()).count();
        let offered_commands = PALETTE_COMMANDS
            .iter()
            .filter(|e| offered(e.requires) && offered(command_requires(&e.command)))
            .filter(|e| !e.command.needs_ai())
            .count();
        assert_eq!(in_menus, offered_commands);
    }

    #[test]
    fn items_carry_their_keys_and_a_reason_when_disabled() {
        let menus = menus();
        let file = menus.iter().find(|m| m.heading == Heading::File).unwrap();
        let quit = file
            .items()
            .find(|i| {
                matches!(
                    i.action,
                    Action::Command(Command::Palette(PaletteCommand::Quit))
                )
            })
            .expect("Quit is in File");
        assert_eq!(quit.shortcut, "q");
        assert!(quit.disabled.is_none());
        let some_todo = menus
            .iter()
            .flat_map(|m| m.items())
            .find(|i| i.disabled.as_deref() == Some(strings::NOT_AVAILABLE_YET))
            .expect("commands not implemented yet are present but disabled");
        assert!(!some_todo.label.is_empty());
    }

    #[test]
    fn labels_lose_their_category() {
        assert_eq!(split_category("Git: Pull"), ("Git", "Pull"));
        assert_eq!(split_category("Git\u{a0}: Pull"), ("Git", "Pull"));
        assert_eq!(split_category("Git\u{ff1a}Pull"), ("Git", "Pull"));
        assert_eq!(split_category("Pull"), ("", "Pull"));
    }

    #[test]
    fn mnemonics_avoid_taken_keys_and_each_other() {
        let km = Keymap::defaults();
        let titles: Vec<String> = [
            "File",
            "Edit",
            "View",
            "Repository",
            "Branch",
            "Git",
            "Hosting",
            "Tools",
            "Help",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let m = mnemonics(&titles, &km);
        let letters: Vec<char> = m.iter().flatten().copied().collect();
        let unique: BTreeSet<char> = letters.iter().copied().collect();
        assert_eq!(letters.len(), unique.len(), "{m:?}");
        for c in letters {
            assert!(km.action_exact(Ctx::Global, mnemonic_chord(c)).is_none());
        }
        assert_eq!(m[0], Some('f'));
        // Alt+T is "New tab" and Alt+R the Radar: Tools and Repository skip them.
        assert_ne!(m[7], Some('t'));
        assert_ne!(m[3], Some('r'));
    }

    #[test]
    fn recent_repositories_lead_the_file_menu() {
        let menus = build(
            PaletteContext::default(),
            &Keymap::defaults(),
            &["/a/b".to_string()],
        );
        let file = &menus[0];
        assert!(matches!(&file.groups[0].items[0].action, Action::Recent(p) if p == "/a/b"));
    }

    #[test]
    fn items_of_a_menu_group_in_catalogue_order_of_categories() {
        let menus = menus();
        let git = menus.iter().find(|m| m.heading == Heading::Git).unwrap();
        assert!(git.groups.len() > 3);
    }
}
