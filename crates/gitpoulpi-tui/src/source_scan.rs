//! The source scanner shared by the UI-text lints: which files are
//! production code, which lines are outside `#[cfg(test)]` items, and the
//! string literals of those lines.
//!
//! Two lints sit on it: `keymap::drift::no_hard_coded_key_labels` (no key
//! label spelled in a literal) and [`no_new_hard_coded_ui_strings`] below
//! (the strict i18n lint: user-visible text moves to `gitpoulpi-i18n`'s
//! catalogs, see `docs/developer/i18n.md`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, skip_keymap: bool, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let p = entry.path();
        if p.is_dir() {
            if skip_keymap && p.file_name().is_some_and(|n| n == "keymap") {
                continue;
            }
            collect(&p, skip_keymap, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// The crate's production `.rs` files (test files, `test_support` and
/// benches left out), sorted. `skip_keymap` leaves out `src/keymap/`.
pub(crate) fn production_source_files(skip_keymap: bool) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect(&root, skip_keymap, &mut files);
    files
        .into_iter()
        .filter(|f| {
            let name = f.strip_prefix(&root).unwrap().to_string_lossy().to_string();
            let unix = name.replace('\\', "/");
            !(unix.ends_with("_tests.rs")
                || unix.ends_with("/tests.rs")
                || unix.contains("/tests/")
                || unix.contains("_tests/")
                || name.contains("test_support")
                || name.contains("bench")
                // In-memory hosting client: `#[cfg(test)] mod fake`.
                || unix == "hosting_task/fake.rs"
                || unix == "source_scan.rs"
                // Test-only lint code (`#[cfg(test)] mod drift;`).
                || unix == "keymap/drift.rs"
                // Test-only doc-table support (`#[cfg(test)] mod docs_tables;`).
                || unix == "docs_tables.rs")
        })
        .collect()
}

/// The lines of `text` outside `#[cfg(test)] mod x { .. }` blocks and
/// `#[cfg(test)]` single items, as `(line number, line)`.
pub(crate) fn production_lines(text: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        // `// i18n-scan: off` .. `// i18n-scan: on` fences English identifiers
        // that are matched, never shown (the palette's category names).
        if lines[i].trim() == "// i18n-scan: off" {
            while i < lines.len() && lines[i].trim() != "// i18n-scan: on" {
                i += 1;
            }
            i += 1;
            continue;
        }
        if lines[i].trim_start() == "#[cfg(test)]" && lines[i].starts_with("#[cfg(test)]") {
            // Skip the item: to the closing `}` at column 0, or to the
            // line ending in `;` for a one-liner.
            let mut j = i + 1;
            while j < lines.len() {
                if lines[j] == "}" || lines[j] == "};" || (lines[j].ends_with(';') && j == i + 1) {
                    break;
                }
                j += 1;
            }
            i = j + 1;
            continue;
        }
        out.push((i + 1, lines[i]));
        i += 1;
    }
    out
}

/// String literals (`"..."`) of the given lines with their first line.
pub(crate) fn literals(lines: &[(usize, &str)]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut cur: Option<(usize, String)> = None;
    for &(n, line) in lines {
        let chars: Vec<char> = line.chars().collect();
        let mut k = 0;
        while k < chars.len() {
            let c = chars[k];
            match &mut cur {
                Some((_, s)) => {
                    if c == '\\' {
                        s.push(c);
                        if let Some(&next) = chars.get(k + 1) {
                            s.push(next);
                        }
                        k += 2;
                        continue;
                    }
                    if c == '"' {
                        out.push(cur.take().unwrap());
                    } else {
                        s.push(c);
                    }
                }
                None => {
                    if c == '/' && chars.get(k + 1) == Some(&'/') {
                        break;
                    }
                    if c == '\'' {
                        // A char literal: skip `'"'` and friends.
                        if chars.get(k + 2) == Some(&'\'') {
                            k += 3;
                            continue;
                        }
                        if chars.get(k + 1) == Some(&'\\') {
                            if let Some(end) = chars[k + 2..].iter().position(|&x| x == '\'') {
                                k += end + 3;
                                continue;
                            }
                        }
                    }
                    if c == '"' {
                        cur = Some((n, String::new()));
                    }
                }
            }
            k += 1;
        }
        if let Some((_, s)) = &mut cur {
            s.push('\n');
        }
    }
    out
}

// ------------------------------------------------------------ i18n lint

/// Brand names and protocol words that are not translated; stripped from a
/// literal before it is judged.
const ALLOWED: &[&str] = &[
    "GitHub",
    "GitLab",
    // The other providers' product names (more-providers).
    "Azure DevOps",
    "Bitbucket",
    "Jira",
    "Trello",
    "Linear",
    "YouTrack",
    "Redmine",
    "Gerrit",
    "GitPoulpi",
    "Gravatar",
    "LFS",
    "HEAD",
    "SSH",
    "HTTPS",
    "HTTP",
    "Nerd Font",
    // syntect's own name for the grammar-less syntax (compared, never shown).
    "Plain Text",
    // The editor's product name: a brand, like GitHub.
    "VS Code",
    // macOS app name passed to `open -a` (repo_menu.rs), not UI text.
    "Terminal",
    // Standard folders of a home directory: names on disk, not prose.
    "Desktop",
    "Documents",
    "Downloads",
];

/// Whether `lit` is a key chord's spelling (`Ctrl+Shift+N`, `PageDown`, `F5`,
/// `Ctrl+,`): key names are never translated, so such a literal is not
/// UI prose (the keymap registry's default chords are all of this shape).
fn is_chord_spelling(lit: &str) -> bool {
    // A leader sequence (`Ctrl+G w`, `Ctrl+N <badge>`): every word is a chord.
    if lit.contains(' ') {
        let words: Vec<&str> = lit.split(' ').collect();
        return words
            .iter()
            .all(|w| *w == "<badge>" || is_chord_spelling(w));
    }
    const KEYS: &[&str] = &[
        "Ctrl",
        "Alt",
        "Shift",
        "Enter",
        "Esc",
        "Tab",
        "BackTab",
        "Backspace",
        "Delete",
        "Insert",
        "Home",
        "End",
        "PageUp",
        "PageDown",
        "Up",
        "Down",
        "Left",
        "Right",
        "Space",
        "Menu",
    ];
    // `+` itself is a key (`Ctrl++`): split on the separating `+` only.
    let parts: Vec<&str> = lit.split('+').collect();
    !lit.is_empty()
        && parts.iter().all(|p| {
            p.chars().count() <= 1
                || KEYS.contains(p)
                || (p.starts_with('F')
                    && p.len() <= 3
                    && p[1..].chars().all(|c| c.is_ascii_digit()))
        })
}

/// Whether `lit` reads as prose a user could see: a run of at least three
/// ASCII letters that is a capitalized word, or a lowercase word followed by
/// a space. Config keys (`ui.language`), identifiers, constants, paths and
/// format-only strings are not prose.
pub(crate) fn is_ui_prose(lit: &str) -> bool {
    if is_chord_spelling(lit) {
        return false;
    }
    let mut s = lit.to_string();
    for brand in ALLOWED {
        s = s.replace(brand, " ");
    }
    // Escapes (`\n`, `\u{2026}`, `\"`) and `{placeholders}` are not words.
    let mut cleaned = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if chars.next() == Some('u') && chars.peek() == Some(&'{') {
                    for d in chars.by_ref() {
                        if d == '}' {
                            break;
                        }
                    }
                }
                cleaned.push(' ');
            }
            '{' => {
                for d in chars.by_ref() {
                    if d == '}' {
                        break;
                    }
                }
                cleaned.push(' ');
            }
            c => cleaned.push(c),
        }
    }
    let s = cleaned;
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if !b[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let start = i;
        while i < b.len() && b[i].is_ascii_alphabetic() {
            i += 1;
        }
        let word: String = b[start..i].iter().collect();
        if word.len() < 3 {
            continue;
        }
        let before_ok = start == 0 || !matches!(b[start - 1], '_' | '.' | '-' | '/' | ':' | '$');
        let after = b.get(i).copied();
        let joined = matches!(after, Some('_' | '.' | '-' | '/' | ':' | '=' | '('))
            || after.is_some_and(|c| c.is_ascii_digit());
        let capitalized = word.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && word[1..].chars().all(|c| c.is_ascii_lowercase());
        let spaced = after == Some(' ');
        if before_ok && !joined && (capitalized || spaced) {
            return true;
        }
    }
    false
}

/// Lines whose literals are never UI text.
fn exempt_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("#[")
        || t.starts_with("#![")
        || [
            "tracing::",
            "panic!",
            ".expect(",
            "debug_assert",
            "unreachable!",
            "assert!(",
            "assert_eq!(",
            "assert_ne!(",
            "include_str!",
            "env!(",
            "std::env::var",
            "write!(f",
            "env::var",
            // `chrome/help_text.rs`: a key column of key names, not prose.
            "Item(keys!(",
            // Developer diagnostics for the exit log (`exit_log`), never shown.
            "quit_how = Some(",
            "exit_log::io_context(",
            // `merge_view/labels.rs`: the English frames the core writes, matched
            // (not shown) to render them from the catalog.
            "// i18n-core-frame",
            // Developer diagnostics: the timing log and the window-open trace.
            "open_trace(",
            "all eight queries in",
            "\"commit-files {}",
            "log(\"cache hit\")",
            // The exit log's description of the window and of the quit gesture.
            "in the center, {:?}",
            "View all files on",
            "no quit gesture recorded",
            // Text for an agent or a model (IPC/MCP replies, prompts), English by
            // design.
            "// i18n-english",
        ]
        .iter()
        .any(|m| t.contains(m))
}

/// Flagged literals per file (path relative to `src/`, `/`-separated),
/// as `(line, literal)`.
pub(crate) fn flagged_literals() -> BTreeMap<String, Vec<(usize, String)>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = BTreeMap::new();
    for file in production_source_files(false) {
        let name = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&file).unwrap();
        let lines = production_lines(&text);
        let by_line: BTreeMap<usize, &str> = lines.iter().copied().collect();
        let hits: Vec<(usize, String)> = literals(&lines)
            .into_iter()
            .filter(|(n, lit)| by_line.get(n).is_none_or(|l| !exempt_line(l)) && is_ui_prose(lit))
            .collect();
        if !hits.is_empty() {
            out.insert(name, hits);
        }
    }
    out
}

/// The strict i18n lint: no production literal may read as UI prose. User
/// text goes into `crates/gitpoulpi-i18n/locales/*.toml` (every shipped
/// language) and is read through `gitpoulpi_i18n::msg::..`. Text that is
/// not for the user stays and is marked: `// i18n-english` on the literal's
/// line (text for an agent or a model, developer diagnostics, protocol
/// strings), `// i18n-core-frame` (English frames the core writes, matched
/// not shown), or an `// i18n-scan: off` .. `// i18n-scan: on` fence around
/// a block that only matches English identifiers. See
/// `docs/developer/i18n.md`.
#[test]
fn no_new_hard_coded_ui_strings() {
    let flagged = flagged_literals();
    let problems: Vec<String> = flagged
        .iter()
        .map(|(file, hits)| {
            let shown: Vec<String> = hits
                .iter()
                .map(|(n, l)| format!("  {file}:{n}: {l:?}"))
                .collect();
            format!(
                "{file}: {} hard-coded UI strings. Put new text in \
                 crates/gitpoulpi-i18n/locales/*.toml (every shipped language) and use \
                 gitpoulpi_i18n::msg::.., or mark text that is not for the user \
                 (// i18n-english):\n{}",
                hits.len(),
                shown.join("\n")
            )
        })
        .collect();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_scanner_finds_the_production_sources() {
    let files = production_source_files(false);
    assert!(files.len() > 50, "the scan found {} files", files.len());
    assert!(production_source_files(true).len() < files.len());
}

#[test]
fn the_prose_heuristic_flags_what_it_should() {
    for prose in [
        "Pull",
        "Stash the working changes",
        "no stash yet",
        "Create a branch",
        "Settings",
    ] {
        assert!(is_ui_prose(prose), "{prose:?}");
    }
    for plain in [
        "ui.language",
        "general.auto_prune",
        "--prune",
        "GITPOULPI_LANG",
        "{}",
        "{n} {x}",
        "\\u{2026}",
        "\n",
        "refs/heads/",
        "origin",
        "ab",
        "GitHub",
        "HEAD",
        "git2",
        "fetch",
    ] {
        assert!(!is_ui_prose(plain), "{plain:?}");
    }
}

// ------------------------------------------------------------ color lint

/// Files whose colors are data or the theme itself: the theme module, the
/// kitty image-id placeholder colors, colors derived from a hash (avatars)
/// and the syntect-to-ratatui converter.
fn color_lint_allowed(name: &str) -> bool {
    name.starts_with("theme/")
        || matches!(
            name,
            "terminal_image/kitty.rs"
                | "color_depth.rs"
                | "terminal_bg.rs"
                | "avatar/initials.rs"
                | "avatar/identicon.rs"
                | "avatar/log_images.rs"
                | "editor_highlight.rs"
        )
}

const NAMED_COLORS: &[&str] = &[
    "Black",
    "Red",
    "Green",
    "Yellow",
    "Blue",
    "Magenta",
    "Cyan",
    "Gray",
    "DarkGray",
    "LightRed",
    "LightGreen",
    "LightYellow",
    "LightBlue",
    "LightMagenta",
    "LightCyan",
    "White",
];

/// The hard-coded colors of one line: `Color::Rgb(<number>, ..)`,
/// `Color::Indexed(..)` and the named `Color::Red`.. variants (not
/// `Color::Reset`, and not a destructuring pattern like `Color::Rgb(r, g, b)`).
/// A line marked `// color-data` is exempt (colors that are data, not theme).
pub(crate) fn hard_coded_colors(line: &str) -> Vec<String> {
    let code = match line.find("//") {
        Some(i) if line[i..].starts_with("// color-data") => return Vec::new(),
        Some(i) => &line[..i],
        None => line,
    };
    let mut out = Vec::new();
    let mut rest = code;
    while let Some(i) = rest.find("Color::") {
        let after = &rest[i + "Color::".len()..];
        let ident: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        let tail = after[ident.len()..].trim_start();
        let flagged = match ident.as_str() {
            "Rgb" => tail
                .strip_prefix('(')
                .is_some_and(|a| a.trim_start().starts_with(|c: char| c.is_ascii_digit())),
            "Indexed" => tail.starts_with('('),
            n => NAMED_COLORS.contains(&n),
        };
        if flagged {
            out.push(format!("Color::{ident}"));
        }
        rest = &after[ident.len()..];
    }
    out
}

/// Production colors come from the theme (`th()`): no literal
/// `Color::Rgb(..)`, `Color::Indexed(..)` or named `Color::Red`.. outside
/// `theme/` and the data allowlist (`color_lint_allowed`). Mark a line whose
/// color is data, not theme, with `// color-data`.
#[test]
fn no_hard_coded_colors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut problems = Vec::new();
    for file in production_source_files(false) {
        let name = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if color_lint_allowed(&name) {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        for (n, line) in production_lines(&text) {
            if line.trim_start().starts_with("//") {
                continue;
            }
            let hits = hard_coded_colors(line);
            if !hits.is_empty() {
                problems.push(format!("  {name}:{n}: {}", hits.join(", ")));
            }
        }
    }
    assert!(
        problems.is_empty(),
        "hard-coded colors: read them from th() (crates/gitpoulpi-tui/src/theme), or mark \
         data colors `// color-data`:\n{}",
        problems.join("\n")
    );
}

#[test]
fn the_color_lint_flags_what_it_should() {
    for bad in [
        "let c = Color::Rgb(1, 2, 3);",
        "Style::default().fg(Color::Rgb(0x10, 0x20, 0x30))",
        "x: Color::Indexed(7),",
        "Style::default().fg(Color::White)",
        "bg(Color::DarkGray)",
    ] {
        assert!(!hard_coded_colors(bad).is_empty(), "{bad}");
    }
    for fine in [
        "let Color::Rgb(r, g, b) = lane else {",
        "Color::Reset",
        "Color::Rgb(1, 2, 3) // color-data",
        "// Color::Red in a comment",
        "let c = th().ui.accent;",
        "Color::Whitelist",
    ] {
        assert!(hard_coded_colors(fine).is_empty(), "{fine}");
    }
}

/// The files the daemon client (and later a second frontend) depends on
/// without wanting a terminal: plain data and logic. They name neither
/// `ratatui` nor `crossterm` and glob nothing from their parent module, so
/// they can move to a neutral crate unchanged. A comment may still mention
/// either name.
const PURE_FILES: &[&str] = &[
    "activity.rs",
    "binary_hint.rs",
    "commit_files_cache.rs",
    "commit_settings.rs",
    "diff_recode.rs",
    "event_loop/fetch_policy.rs",
    "history_view/rename.rs",
    "lfs_ui_pure.rs",
    "live_settings_pure.rs",
    "log_table/rows_now.rs",
    "no_repository.rs",
    "oid_text.rs",
    "redraw.rs",
    "ref_name.rs",
    "repo_state_path.rs",
    "safety_panel/label.rs",
    "ui_viewer_pure.rs",
    "undo_ui_pure.rs",
    "wip_view.rs",
];

#[test]
fn the_pure_files_name_no_terminal_crate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut problems = Vec::new();
    for name in PURE_FILES {
        let text = std::fs::read_to_string(root.join(name))
            .unwrap_or_else(|e| panic!("pure file {name} is missing: {e}"));
        for (n, line) in production_lines(&text) {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for banned in ["ratatui", "crossterm", "use super::*"] {
                if line.contains(banned) {
                    problems.push(format!("{name}:{n}: {banned}: {}", line.trim()));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
