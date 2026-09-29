use super::*;
use gitpoulpi_config::IntegrationSsh;
use gitpoulpi_hosting::RecordedTransport;
use gitpoulpi_secrets::InMemoryTokens;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;
use std::sync::Mutex;

mod connect;
mod i18n;
mod input;
mod profiles;
mod providers;
mod radar;
mod render;
mod scroll;
mod ssh;
mod wrap;

const GITLAB_SM: usize = 3;

struct H {
    _dir: tempfile::TempDir,
    path: PathBuf,
    config: Config,
    tokens: Tokens,
    presence: HashMap<String, Option<bool>>,
    message: Option<String>,
}

impl H {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        Self {
            _dir: dir,
            path,
            config: Config::default(),
            tokens: Arc::new(InMemoryTokens::default()),
            presence: HashMap::new(),
            message: None,
        }
    }

    fn env(&mut self) -> Env<'_> {
        Env {
            config: &mut self.config,
            config_path: Some(self.path.clone()),
            tokens: &self.tokens,
            presence: &mut self.presence,
            message: &mut self.message,
        }
    }

    fn connected(&mut self, kind: IntegrationKind, login: &str) -> String {
        let id = Integration::new_id(kind, self.config.integrations.iter());
        let mut i = Integration::new(id.clone(), kind, "https://gitlab.example.com");
        i.user = Some(ConnectedUser {
            login: login.into(),
            name: "Kai G".into(),
            id: "7".into(),
        });
        self.tokens.store(&i.token_ref, "glpat-secret").unwrap();
        self.presence.insert(i.token_ref.clone(), Some(true));
        self.config.integrations.push(i);
        id
    }
}

fn page_with(transport: RecordedTransport) -> (IntegrationsPage, Arc<RecordedTransport>) {
    let transport = Arc::new(transport);
    let page = IntegrationsPage {
        transport: Some(transport.clone()),
        ..Default::default()
    };
    (page, transport)
}

fn draw(
    page: &mut IntegrationsPage,
    h: &H,
    w: u16,
    rows: u16,
    mouse: Option<(u16, u16)>,
) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(w, rows)).unwrap();
    terminal
        .draw(|f| page.render(f, f.area(), &h.config, &h.presence, true, mouse))
        .unwrap();
    terminal.backend().buffer().clone()
}

fn text(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn key(page: &mut IntegrationsPage, h: &mut H, code: KeyCode) -> Nav {
    page.on_key(&KeyEvent::new(code, KeyModifiers::NONE), &mut h.env())
}

fn type_str(page: &mut IntegrationsPage, h: &mut H, s: &str) {
    for c in s.chars() {
        key(page, h, KeyCode::Char(c));
    }
}

fn wait_connected(page: &mut IntegrationsPage, h: &mut H) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while page.pending.is_some() {
        page.poll(&mut h.env());
        assert!(std::time::Instant::now() < deadline, "check never finished");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn gitlab_ok() -> RecordedTransport {
    RecordedTransport::new()
        .on(
            "/personal_access_tokens/self",
            200,
            r#"{"scopes":["api","read_user"],"expires_at":null}"#,
        )
        .on(
            "/api/v4/user",
            200,
            r#"{"id":7,"username":"kai","name":"Kai G","web_url":"https://gitlab.example.com/kai"}"#,
        )
}

/// Selects GitLab Self-Managed, opens the form and fills it.
fn fill_sm_form(page: &mut IntegrationsPage, h: &mut H, host: &str, token: &str) {
    page.select_kind(GITLAB_SM, &h.config);
    page.add_profile(&h.config);
    type_str(page, h, host);
    key(page, h, KeyCode::Tab);
    type_str(page, h, token);
}

// -- per-profile SSH key ----------------------------------------
