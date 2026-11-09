use super::*;
use gitpoulpi_secrets::InMemoryTokens;

mod instructions;
mod limits;
mod privacy;
mod profiles;
mod repo;
mod review;
mod run_mode;
mod vars;

pub(super) fn state(dir: &Path) -> PreferencesState {
    let mut state = PreferencesState::with_parts(
        Config::default(),
        Some(dir.join("config.toml")),
        dir,
        RepoState::default(),
        RepoState::path_for(dir, ".gitpoulpi"),
        std::sync::Arc::new(InMemoryTokens::default()),
    );
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Ai)
        .unwrap();
    state.focus = PrefFocus::Fields;
    state
}

pub(super) fn select(state: &mut PreferencesState, label: &str) {
    // The row may be on the other sub-tab: bring that one forward.
    if !state.fields().iter().any(|f| f.label == label) {
        let other = match state.sub_tab() {
            SubTab::General => SubTab::List,
            SubTab::List => SubTab::General,
        };
        state.set_sub_tab(other);
    }
    let index = state
        .fields()
        .iter()
        .position(|f| f.label == label)
        .unwrap_or_else(|| panic!("no row {label:?}"));
    state.field = index;
}

pub(super) fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

pub(super) fn type_text(state: &mut PreferencesState, text: &str) {
    for c in text.chars() {
        state.on_key(key(KeyCode::Char(c)));
    }
}

pub(super) fn file(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

/// Selects the first row labeled `label` after the heading `heading`.
pub(super) fn select_in(state: &mut PreferencesState, heading: &str, label: &str) {
    if !state
        .fields()
        .iter()
        .any(|f| matches!(f.control, Control::Heading) && f.label == heading)
    {
        let other = match state.sub_tab() {
            SubTab::General => SubTab::List,
            SubTab::List => SubTab::General,
        };
        state.set_sub_tab(other);
    }
    let fields = state.fields();
    let from = fields
        .iter()
        .position(|f| matches!(f.control, Control::Heading) && f.label == heading)
        .unwrap_or_else(|| panic!("no heading {heading:?}"));
    let at = fields[from..]
        .iter()
        .position(|f| f.label == label)
        .unwrap_or_else(|| panic!("no row {label:?} under {heading:?}"))
        + from;
    state.field = at;
}

pub(super) fn draw(state: &mut PreferencesState, w: u16, h: u16) -> String {
    use ratatui::backend::TestBackend;
    let mut terminal = ratatui::Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..h)
        .map(|y| (0..w).map(|x| buffer[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn repo_state(dir: &Path) -> RepoState {
    RepoState::load(&RepoState::path_for(dir, ".gitpoulpi")).unwrap_or_default()
}

pub(super) fn repo_state_on_page(dir: &Path) -> PreferencesState {
    let mut state = state(dir);
    state.config.ai.enabled = true;
    state.config.ai.profiles.push(AiProfile::new(
        "second",
        "Second",
        AiProviderKind::Anthropic,
    ));
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::RepoAi)
        .unwrap();
    state
}

mod french;
