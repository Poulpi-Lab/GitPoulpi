//! The "This installation" rows of the Launcher page: what they show, what
//! they write (this installation's entry only), and the buttons around them.
//! A fake installation and temp directories; never the real config.

use super::*;
use crate::preferences::launcher_install::{fake_current, InstallButton};
use gitpoulpi_config::desktop::Place;
use gitpoulpi_config::{InstallationEntry, Kind};

const ME: &str = "aaaa000000000001";

fn install_state(dir: &Path) -> PreferencesState {
    let mut st = state(dir);
    st.set_this(fake_current(false));
    st
}

fn rows(st: &PreferencesState) -> Vec<Field> {
    st.fields_of(PrefSection::Launcher)
}

fn row(st: &PreferencesState, label: &str) -> Field {
    rows(st)
        .into_iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("no row {label}"))
}

/// Draws the page `width` columns wide.
fn draw_at(st: &mut PreferencesState, width: u16) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(width, 60)).unwrap();
    terminal
        .draw(|frame| st.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..60)
        .map(|y| {
            (0..width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn other_entry(name: &str, path: &str, submenu: &str, env: &str) -> InstallationEntry {
    InstallationEntry {
        path: path.to_string(),
        kind: Kind::Dev,
        name: name.to_string(),
        folder_menu_submenu: submenu.to_string(),
        env: env.to_string(),
        ..Default::default()
    }
}

fn written(dir: &Path) -> toml::Value {
    toml::from_str(&std::fs::read_to_string(dir.join("config.toml")).unwrap()).unwrap()
}

#[test]
fn installation_rows_render_name_colour_env_and_path() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    go(&mut st, PrefSection::Launcher);
    let screen = draw_at(&mut st, 120);
    for want in [
        "This installation",
        "Installation",
        "/opt/gp/a",
        "Launcher name",
        "GitPoulpi (portable)",
        "Include version in the launcher",
        "Octopus colour",
        "Become the primary",
        "Environment",
        "own (default)",
        "Folder-menu submenu",
    ] {
        assert!(screen.contains(want), "{want} is missing:\n{screen}");
    }
    // The Installation row names kind, path and environment.
    let Control::Info(text) = row(&st, "Installation").control else {
        panic!("an info row");
    };
    assert_eq!(text, format!("portable, /opt/gp/a, environment {ME}"));
}

#[test]
fn a_narrow_page_ellipsises_the_path_never_drops_it() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let mut current = fake_current(false);
    current.identity.root = "/very/long/folder/of/an/install/gitpoulpi-tools".to_string();
    st.set_this(current);
    go(&mut st, PrefSection::Launcher);
    let screen = draw_at(&mut st, 70);
    let line = screen
        .lines()
        .find(|l| l.contains('\u{2026}') && l.contains("portable, /"))
        .unwrap_or_else(|| panic!("the Installation row is drawn:\n{screen}"));
    assert!(line.contains('\u{2026}'), "cut in the middle: {line}");
    assert!(
        line.contains("/ver"),
        "the start of the path stays: {line}"
    );
    assert!(
        line.contains("ols"),
        "the end of the path stays: {line}"
    );
    // The full text stays one key away: the help line under the row.
    let full = row(&st, "Installation");
    assert!(full
        .help
        .as_deref()
        .unwrap()
        .contains("/very/long/folder/of/an/install/gitpoulpi-tools"));
}

#[test]
fn legacy_launchers_row_offers_both_buttons() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    // Nothing found: no row.
    assert!(rows(&st).iter().all(|f| f.label != "Legacy launchers"));
    st.set_legacy(vec![Place::Launcher]);
    let legacy = row(&st, "Legacy launchers");
    assert!(matches!(&legacy.control, Control::Info(t) if t.starts_with("found: ")));
    let buttons: Vec<(String, PrefAction)> = rows(&st)
        .into_iter()
        .filter_map(|f| match f.control {
            Control::Buttons(b) => Some(b),
            _ => None,
        })
        .flatten()
        .filter(|b| matches!(b.action, PrefAction::Installation(_)))
        .map(|b| (b.label, b.action))
        .collect();
    assert_eq!(
        buttons,
        [
            (
                "Become the primary".to_string(),
                PrefAction::Installation(InstallButton::BecomePrimary)
            ),
            (
                "Remove legacy launchers".to_string(),
                PrefAction::Installation(InstallButton::RemoveLegacy)
            ),
        ]
    );
    // A primary owns the legacy names: nothing left to take over.
    st.config.installation.primary = Some(ME.to_string());
    assert!(rows(&st).iter().all(|f| f.label != "Legacy launchers"));
}

#[test]
fn name_colour_and_version_write_this_installations_entry_only() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    assert!(st.write_global("installation.name", "GitPoulpi (mine)"));
    assert!(st.write_global("installation.logo_color", "teal"));
    assert!(st.write_global("installation.name_with_version", "true"));
    let file = written(tmp.path());
    let entry = &file["installation"]["settings"][ME];
    assert_eq!(entry["name"].as_str(), Some("GitPoulpi (mine)"));
    assert_eq!(entry["logo_color"].as_str(), Some("teal"));
    assert_eq!(entry["name_with_version"].as_bool(), Some(true));
    assert!(file.get("ui").is_none(), "ui.* is for the primary: {file}");
    assert_eq!(row(&st, "Octopus colour").value_text(), "Teal");
}

#[test]
fn the_primary_also_writes_the_keys_older_builds_read() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = state(tmp.path());
    std::fs::write(
        tmp.path().join("config.toml"),
        format!("[installation]\nprimary = \"{ME}\"\n"),
    )
    .unwrap();
    st.config = Config::load(&tmp.path().join("config.toml")).unwrap();
    st.set_this(fake_current(true));
    assert!(st.write_global("installation.logo_color", "teal"));
    assert!(st.write_global("installation.command_alias", "gp"));
    let file = written(tmp.path());
    assert_eq!(file["ui"]["logo_color"].as_str(), Some("teal"));
    assert_eq!(file["ui"]["command_alias"].as_str(), Some("gp"));
    // The primary is always called GitPoulpi.
    assert!(!st.write_global("installation.name", "Other"));
    assert!(row(&st, "Launcher name").coming.is_some());
}

#[test]
fn a_name_another_live_installation_uses_is_refused_with_a_note() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let live = tmp.path().display().to_string();
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", &live, "", ""),
    );
    assert!(!st.write_global("installation.name", "gitpoulpi (DEV)"));
    assert_eq!(
        st.take_message().as_deref(),
        Some("gitpoulpi (DEV) is already the name of another installation")
    );
    assert!(!tmp.path().join("config.toml").exists(), "nothing written");
    // A gone installation does not hold its name.
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", "/no/such/folder", "", ""),
    );
    assert!(st.write_global("installation.name", "GitPoulpi (dev)"));
    // A name no launcher file can have is refused.
    assert!(!st.write_global("installation.name", "a/b"));
}

#[test]
fn environment_and_submenu_offer_what_others_use_and_warn_before_restart() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let live = tmp.path().display().to_string();
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", &live, "Tools", "shared"),
    );
    let values = |f: Field| match f.control {
        Control::Combo {
            options, custom, ..
        } => {
            assert!(custom, "a name can be typed");
            options.into_iter().map(|(_, v)| v).collect::<Vec<_>>()
        }
        other => panic!("a combobox: {other:?}"),
    };
    assert_eq!(values(row(&st, "Environment")), ["", "shared"]);
    if cfg!(not(target_os = "macos")) {
        assert_eq!(values(row(&st, "Folder-menu submenu")), ["", "Tools"]);
    }
    assert!(st.write_global("installation.env", "shared"));
    assert_eq!(
        st.take_message().as_deref(),
        Some("Restart this installation's open windows for the new environment to apply.")
    );
    let file = written(tmp.path());
    assert_eq!(
        file["installation"]["settings"][ME]["env"].as_str(),
        Some("shared")
    );
    assert!(st.write_global("installation.folder_menu_submenu", "Tools"));
    assert_eq!(
        st.own_entry().folder_menu_submenu,
        "Tools",
        "the submenu is this installation's"
    );
}

#[test]
fn a_gone_installation_gets_a_forget_button_and_a_live_one_does_not() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    let live = tmp.path().display().to_string();
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", &live, "", ""),
    );
    st.config.installation.settings.insert(
        "cccc000000000003".to_string(),
        other_entry("GitPoulpi (old)", "/no/such/folder", "", ""),
    );
    let forgets = |st: &PreferencesState| {
        rows(st)
            .into_iter()
            .filter(|f| matches!(f.target, Target::Action(PrefAction::Installation(_))))
            .count()
    };
    assert_eq!(forgets(&st), 1, "only the gone one");
    assert!(row(&st, "Other: GitPoulpi (old)").value_text().starts_with("gone, "));
    assert!(row(&st, "Other: GitPoulpi (dev)")
        .value_text()
        .starts_with("dev, "));
    // Forget asks first (a destructive button).
    let forget = rows(&st)
        .into_iter()
        .find_map(|f| match f.control {
            Control::Buttons(b) => b.into_iter().find(|b| b.label == "Forget"),
            _ => None,
        })
        .unwrap();
    st.press_button(&forget);
    assert!(st.confirm_action.is_some(), "a question, not a deletion");
}

#[test]
fn an_alias_of_another_installation_reads_used_by_and_create_is_disabled() {
    use gitpoulpi_config::desktop::alias::{State, Status};
    let tmp = tempfile::tempdir().unwrap();
    let mut st = install_state(tmp.path());
    st.config.installation.settings.insert(
        "bbbb000000000002".to_string(),
        other_entry("GitPoulpi (dev)", &tmp.path().display().to_string(), "", ""),
    );
    let text = st.alias_owner_text(&Status {
        state: State::Other,
        owner: Some("bbbb000000000002".to_string()),
        shadowed_by: None,
        powershell_builtin: false,
    });
    assert_eq!(text, "Used by GitPoulpi (dev)");
    // The reason is shown as the row's value, not behind "coming".
    let row = Field::plain("x", Control::Info(String::new()), Target::None).coming(&text);
    assert_eq!(row.value_text(), text);
}
