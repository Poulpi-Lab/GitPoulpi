//! The Preferences > Integrations page.
//!
//! Left: the list of products (Hosting, then Issue trackers), each with
//! a connected dot and a profile-count badge; a product without a client
//! is not listed. Right: a *profile strip* (one chip per
//! configured profile of the selected product plus `+ Add profile`) and
//! under it the selected profile's view:
//!
//! - **Empty**: nothing configured for the product yet.
//! - **Form**: a preconfigured connect form per kind -- a personal access
//!   token, plus a *Host Domain* for the self-managed products -- with the
//!   scopes to tick and a `Generate a token` link that opens the host's
//!   token page with them pre-filled.
//! - **Connecting**: `check_token` running on a background thread. The
//!   input thread never waits on the network or on the credential helper.
//! - **Connected**: the account card with Edit / Disconnect / Remove.
//!   An existing profile that is not connected also offers Remove in its form.
//!
//! The page owns only its own view state. The configured profiles live in
//! `Config`, the presence cache in `PreferencesState`; both arrive through
//! [`Env`] / arguments, which is what keeps `render` free of token-store
//! calls (see the guard test in `preferences`).
//!
//! Hit-testing follows the Preferences convention: rects are captured
//! while drawing and consumed by the next mouse event, and every
//! interactive rect gets the strong `theme::plain_button_style(true)`
//! hover.

use crate::redraw::NotifySend as _;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};

use crate::keymap::Action as Bind;
use crate::wide_edge::Clear;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use gitpoulpi_config::{
    set_list_in_file, AccountField, Config, ConnectedUser, Integration, IntegrationCategory,
    IntegrationKind, SshMode, TokenLink,
};
use gitpoulpi_hosting::{client_for, HttpTransport, TokenCheck};
use gitpoulpi_i18n::{
    msg::{integrations as msg, integrations_page as msg2, trust_dialog as tmsg},
    M,
};
use gitpoulpi_secrets::TokenStore;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::Frame;

mod scroll;
use scroll::{Canvas, Scroll};

use crate::folder_listing::{Lister, ListerEvent};
use crate::folder_picker::{picker_rect, FolderPicker, PickerOutcome};
use crate::integrations_ssh::{self as ssh_io, GenerateRequest, SshOutcome};
use crate::text_edit::FieldEditor;
use crate::theme::plain_button_style;
use crate::{display_width, fit, tail_fit, wrap_words};

/// Where a Trello user finds the API key the token link needs.
const TRELLO_KEY_URL: &str = "https://trello.com/power-ups/admin";

/// The text a field item edits, as a place (so the editor, a disjoint
/// field, can be borrowed beside it).
macro_rules! field_mut {
    ($self:ident, $item:expr) => {
        match $item {
            Item::Token => &mut $self.token,
            Item::Org => &mut $self.org,
            Item::Account => &mut $self.account,
            Item::SshTitle => &mut $self.ssh.title,
            Item::SshPath => &mut $self.ssh.path,
            Item::RadarNamespaces => &mut $self.radar_ns,
            _ => &mut $self.host,
        }
    };
}

type Tokens = Arc<dyn TokenStore + Send + Sync>;
type Opener = Box<dyn Fn(&str) -> Result<(), String>>;

/// Which column of the page has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Zone {
    Types,
    Profiles,
    Body,
}

/// One focusable/clickable control of the right-hand view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Item {
    Host,
    Org,
    Account,
    Token,
    /// The form's "Strict certificate pinning" checkbox.
    StrictPin,
    /// The form's "Allow plain http (not encrypted)" checkbox.
    AllowPlain,
    GetKey,
    Generate,
    /// The local Markdown folder's Browse button.
    FolderBrowse,
    Connect,
    CancelForm,
    CancelConnect,
    StartConnect,
    Edit,
    Disconnect,
    Remove,
    ConfirmRemove,
    CancelRemove,
    // -- per-profile SSH key (ADR-0008) --
    SshDefault,
    SshGenerate,
    SshCustom,
    SshTitle,
    SshUpload,
    SshGo,
    SshPath,
    SshBrowse,
    SshUse,
    SshRemove,
    SshDelete,
    SshConfirmDelete,
    SshCancelDelete,
    // -- per-profile Radar options (radar.rs) --
    RadarReview,
    RadarOpen,
    RadarDrafts,
    RadarIssues,
    RadarNamespaces,
    RadarMode,
}

impl Item {
    #[cfg(all(test, feature = "github", feature = "gitlab"))]
    fn is_ssh(self) -> bool {
        matches!(
            self,
            Item::SshDefault
                | Item::SshGenerate
                | Item::SshCustom
                | Item::SshTitle
                | Item::SshUpload
                | Item::SshGo
                | Item::SshPath
                | Item::SshBrowse
                | Item::SshUse
                | Item::SshRemove
                | Item::SshDelete
        )
    }

    fn is_text(self) -> bool {
        matches!(
            self,
            Item::Host
                | Item::Org
                | Item::Account
                | Item::Token
                | Item::SshTitle
                | Item::SshPath
                | Item::RadarNamespaces
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    Type(usize),
    Profile(usize),
    AddProfile,
    Item(Item),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Browse,
    /// The connect form; `Some(id)` edits an existing profile.
    Form(Option<String>),
    Connecting(Option<String>),
    ConfirmRemove,
    /// Deleting the profile's key files; the confirmation names them.
    ConfirmKeyDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Coming,
    Empty,
    Form,
    Connecting,
    Connected(usize),
    Confirm(usize),
    ConfirmKey(usize),
}

/// What a key did to the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nav {
    Stay,
    /// The keyboard goes back to the Preferences sidebar.
    Sections,
}

/// What the page needs from `PreferencesState` to change things.
pub(crate) struct Env<'a> {
    pub(crate) config: &'a mut Config,
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) tokens: &'a Tokens,
    pub(crate) presence: &'a mut HashMap<String, Option<bool>>,
    pub(crate) message: &'a mut Option<String>,
}

/// A background key job (generate / remove / delete) and its answer.
struct SshJob {
    rx: mpsc::Receiver<Result<SshOutcome, String>>,
    /// The profile it is for, by id.
    id: String,
    what: M,
}

/// The SSH section's own view state.
struct SshUi {
    /// Which radio is selected while the profile has no key attached.
    mode: SshMode,
    title: String,
    path: String,
    upload: bool,
    error: Option<String>,
    job: Option<SshJob>,
    picker: Option<FolderPicker>,
    /// The picker is choosing the local notes folder, not a key file.
    picking_folder: bool,
    lister: Lister,
    /// `public key path -> fingerprint`, read once per key.
    fingerprints: HashMap<String, Option<String>>,
    /// Where generated keys go (`~/.ssh`); tests point it at a tempdir.
    ssh_dir: Option<PathBuf>,
}

impl Default for SshUi {
    fn default() -> Self {
        Self {
            mode: SshMode::Default,
            title: String::new(),
            path: String::new(),
            upload: true,
            error: None,
            job: None,
            picker: None,
            picking_folder: false,
            lister: Lister::default(),
            fingerprints: HashMap::new(),
            ssh_dir: None,
        }
    }
}

/// What a token check told about the token's rights and lifetime, kept per
/// profile for the connected card (the host is asked once, at Connect).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Rights {
    /// Granted scopes beyond what GitPoulpi needs, dangerous ones first.
    pub(crate) excess: Vec<String>,
    pub(crate) expires: Option<String>,
    /// Advise a fine-grained token with an expiry date.
    pub(crate) prefer_fine_grained: bool,
}

impl Rights {
    fn of(check: &TokenCheck) -> Self {
        Self {
            excess: check.excess_scopes.clone(),
            expires: check.expires_at.clone(),
            // A fine-grained token is what the advice asks for already.
            prefer_fine_grained: check.token_flavor != Some(gitpoulpi_hosting::TokenFlavor::FineGrained)
                && (check.recommend_fine_grained || !check.excess_scopes.is_empty()),
        }
    }

    fn is_empty(&self) -> bool {
        self.excess.is_empty() && self.expires.is_none() && !self.prefer_fine_grained
    }
}

/// What the form asks of the security check when Connect runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TrustOptions {
    /// The address was changed and a token typed for it: accept it.
    pub(super) trust_new_origin: bool,
    pub(super) allow_plain_http: bool,
    pub(super) strict: bool,
}

struct Pending {
    rx: mpsc::Receiver<Result<TokenCheck, String>>,
    integration: Integration,
    editing: Option<String>,
    /// Where the profile is saved if Preferences closes mid-check.
    config_path: Option<PathBuf>,
    /// Set by `detach_pending` only: a plain drop (Cancel, switching
    /// view) must save nothing.
    detach: bool,
}

impl Drop for Pending {
    /// Preferences closed while the token check ran: the check thread
    /// stores the token itself, so without this the token would exist
    /// with no profile pointing at it. Finish the job on a detached
    /// thread - closing must not wait for the network.
    fn drop(&mut self) {
        if !self.detach {
            return;
        }
        let Some(path) = self.config_path.take() else {
            return;
        };
        let (_, dead) = mpsc::channel();
        let rx = std::mem::replace(&mut self.rx, dead);
        let integration = self.integration.clone();
        std::thread::spawn(move || {
            if let Ok(Ok(check)) = rx.recv_timeout(std::time::Duration::from_secs(90)) {
                let _ = save_checked_profile(&path, integration, &check);
            }
        });
    }
}

/// Upserts `integration`, connected as `check.user`, into the config
/// file at `path`.
fn save_checked_profile(
    path: &std::path::Path,
    mut integration: Integration,
    check: &TokenCheck,
) -> Result<(), String> {
    integration.user = Some(ConnectedUser {
        login: check.user.login.clone(),
        name: check.user.name.clone(),
        id: check.user.id.clone(),
    });
    integration.enabled = true;
    let mut list = Config::load(path)
        .map(|c| c.integrations)
        .map_err(|e| e.to_string())?;
    match list.iter().position(|i| i.id == integration.id) {
        Some(at) => list[at] = integration,
        None => list.push(integration),
    }
    set_list_in_file(path, "integrations", &list).map_err(|e| e.to_string())
}

pub(crate) struct IntegrationsPage {
    /// The products this build lists, Hosting first (`IntegrationKind::ALL`
    /// order filtered by `provider_kinds()`); products without a client are
    /// not listed.
    kinds: Vec<IntegrationKind>,
    /// Index into `kinds`.
    pub(crate) kind: usize,
    /// Index into the selected kind's profiles; `len` is `+ Add profile`.
    pub(crate) sel: usize,
    mode: Mode,
    pub(crate) zone: Zone,
    item: usize,
    host: String,
    /// Azure DevOps organization (a name, or a pasted organization URL).
    org: String,
    /// The non-secret identity: Atlassian email or Trello API key.
    account: String,
    token: String,
    error: Option<String>,
    /// Scopes the host said the token lacks, by profile id.
    warnings: HashMap<String, Vec<String>>,
    /// What the last check said about the token's rights and expiry, by id.
    rights: HashMap<String, Rights>,
    /// The pinned certificate (grouped fingerprint, strict) by profile id,
    /// read once from the trust store.
    pinned: HashMap<String, Option<(String, bool)>>,
    /// The form's two security toggles.
    strict: bool,
    plain_http: bool,
    /// "Re-entering the token is recommended" under the token field: set by
    /// the security check's answer to an address change.
    retype_hint: bool,
    pending: Option<Pending>,
    editor: FieldEditor<Item>,
    ssh: SshUi,
    /// The Namespaces field's text, and the profile (by id) it belongs to.
    radar_ns: String,
    radar_ns_for: String,
    /// The "uncloned request" combobox.
    radar_combo: crate::combo::Combo,
    scroll: Scroll,
    /// `(rect, target, hoverable)`, captured by the last render.
    hits: Vec<(Rect, Hit, bool)>,
    started: std::time::Instant,
    /// Where the security check keeps its records: `None` is the real
    /// store, a test points it at a temporary private directory.
    pub(crate) trust_paths: Option<gitpoulpi_config::ConfigPaths>,
    /// Tests inject a recorded transport; production builds the reqwest one.
    pub(crate) transport: Option<Arc<dyn HttpTransport>>,
    /// Opens a URL in the browser (replaced in tests).
    pub(crate) opener: Opener,
}

impl Default for IntegrationsPage {
    fn default() -> Self {
        Self {
            kinds: page_kinds(),
            kind: 0,
            sel: 0,
            mode: Mode::Browse,
            zone: Zone::Types,
            item: 0,
            host: String::new(),
            org: String::new(),
            account: String::new(),
            token: String::new(),
            error: None,
            warnings: HashMap::new(),
            rights: HashMap::new(),
            pinned: HashMap::new(),
            strict: false,
            plain_http: false,
            retype_hint: false,
            pending: None,
            editor: FieldEditor::default(),
            ssh: SshUi::default(),
            radar_ns: String::new(),
            radar_ns_for: String::new(),
            radar_combo: radar::new_combo(),
            scroll: Scroll::default(),
            hits: Vec::new(),
            started: std::time::Instant::now(),
            trust_paths: default_trust_paths(),
            transport: None,
            opener: Box::new(crate::repo_menu::open_url),
        }
    }
}

/// Where a fresh page keeps its trust records: the real store in the app; in
/// a test a private directory of its own, so a check never touches the user's.
fn default_trust_paths() -> Option<gitpoulpi_config::ConfigPaths> {
    #[cfg(test)]
    {
        let dir = tempfile::tempdir().ok()?.keep();
        Some(gitpoulpi_config::ConfigPaths::for_dir(dir))
    }
    #[cfg(not(test))]
    {
        None
    }
}

/// The kinds the page lists: `provider_kinds()` in the display order, so
/// the two groups (Hosting, then Issue trackers) stay contiguous.
fn page_kinds() -> Vec<IntegrationKind> {
    use IntegrationKind as K;
    let built = crate::build_features::provider_kinds();
    [
        K::Github,
        K::GithubEnterprise,
        K::Gitlab,
        K::GitlabSelfManaged,
        K::AzureDevops,
        K::AzureDevopsServer,
        K::Bitbucket,
        K::BitbucketDataCenter,
        K::Gitea,
        K::Forgejo,
        K::Gerrit,
        K::JiraCloud,
        K::JiraDataCenter,
        K::Trello,
        K::Linear,
        K::Youtrack,
        K::Redmine,
        K::LocalMarkdown,
    ]
    .into_iter()
    .filter(|k| built.contains(k))
    .collect()
}

/// The Azure organization name from what was typed: a bare name, a
/// `dev.azure.com/{org}` URL or a `{org}.visualstudio.com` one.
pub(crate) fn azure_org_input(input: &str) -> Option<String> {
    let t = input.trim().trim_matches('/');
    if t.is_empty() || t.chars().any(char::is_whitespace) {
        return None;
    }
    let url = if t.contains("://") {
        t.to_string()
    } else if t.contains('/') || t.ends_with(".visualstudio.com") {
        format!("https://{t}")
    } else if t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Some(t.to_string());
    } else {
        return None;
    };
    Integration::new("form", IntegrationKind::AzureDevops, url).azure_org()
}

/// `gitlab.example.com`, `https://gitlab.example.com/` or
/// `http://host:8080` to a base URL, or `None` when it isn't a host.
///
/// A dotless name (a Docker service, a LAN machine) needs a port or an
/// explicit `http://`: `gitea:3000`, `http://nas`, `192.168.1.10:3000`,
/// `[::1]:3000`. A bare `gitlab` is still a typo, not a host.
pub(crate) fn normalize_host(input: &str) -> Option<String> {
    let t = input.trim().trim_end_matches('/');
    if t.is_empty() || t.chars().any(char::is_whitespace) || t.contains('@') {
        return None;
    }
    let (scheme, rest) = match t.split_once("://") {
        Some((s, r)) => (s.to_ascii_lowercase(), r),
        None => ("https".to_string(), t),
    };
    if scheme != "https" && scheme != "http" {
        return None;
    }
    let authority = rest.split('/').next().unwrap_or("");
    // `host:` with nothing after the colon is a typo.
    if authority.ends_with(':') {
        return None;
    }
    let (name, port) = if let Some(v6) = authority.strip_prefix('[') {
        let (inner, after) = v6.split_once(']')?;
        let hex = inner.contains(':') && inner.chars().all(|c| c.is_ascii_hexdigit() || c == ':');
        if !hex || !(after.is_empty() || after.starts_with(':')) {
            return None;
        }
        (None, after.strip_prefix(':'))
    } else {
        match authority.split_once(':') {
            Some((n, p)) => (Some(n), Some(p)),
            None => (Some(authority), None),
        }
    };
    if port.is_some_and(|p| p.len() > 5 || !p.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    let reachable = port.is_some() || scheme == "http";
    let plausible = match name {
        // A bracketed IPv6 literal is a host on its own.
        None => true,
        Some(n) => {
            let ok_chars = !n.is_empty()
                && !n.starts_with('.')
                && !n.ends_with('.')
                && n.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
            ok_chars && (n.contains('.') || n == "localhost" || reachable)
        }
    };
    plausible.then(|| format!("{scheme}://{rest}"))
}

impl IntegrationsPage {
    /// A spinner is on screen (a connection check or an SSH job runs).
    /// The first control (the first provider kind) has the keyboard and nothing is
    /// being typed: Up leaves for the generic rows above the page.
    pub(crate) fn at_top(&self) -> bool {
        self.zone == Zone::Types && self.kind == 0 && matches!(self.mode, Mode::Browse)
    }

    pub(crate) fn animating(&self) -> bool {
        self.pending.is_some() || self.ssh.job.is_some()
    }

    fn kind(&self) -> IntegrationKind {
        self.kinds[self.kind.min(self.kinds.len() - 1)]
    }

    /// Where `kind` sits in the listed products.
    pub(crate) fn kind_index(&self, kind: IntegrationKind) -> Option<usize> {
        self.kinds.iter().position(|k| *k == kind)
    }

    fn profile_ids(&self, cfg: &Config) -> Vec<usize> {
        let kind = self.kind();
        cfg.integrations
            .iter()
            .enumerate()
            .filter(|(_, i)| i.kind == kind)
            .map(|(n, _)| n)
            .collect()
    }

    fn selected_profile(&self, cfg: &Config) -> Option<usize> {
        self.profile_ids(cfg).get(self.sel).copied()
    }

    fn view(&self, cfg: &Config) -> View {
        if !self.kind().capabilities().implemented {
            return View::Coming;
        }
        match &self.mode {
            Mode::Form(_) => View::Form,
            Mode::Connecting(_) => View::Connecting,
            Mode::ConfirmRemove => self
                .selected_profile(cfg)
                .map_or(View::Empty, View::Confirm),
            Mode::ConfirmKeyDelete => self
                .selected_profile(cfg)
                .map_or(View::Empty, View::ConfirmKey),
            Mode::Browse => match self.selected_profile(cfg) {
                Some(i) if cfg.integrations[i].user.is_some() => View::Connected(i),
                _ => View::Empty,
            },
        }
    }

    fn items(&self, cfg: &Config) -> Vec<Item> {
        match self.view(cfg) {
            View::Coming => vec![],
            View::Empty => vec![Item::StartConnect],
            View::Form => {
                let caps = self.kind().capabilities();
                let mut items = Vec::new();
                if caps.local_folder {
                    items.extend([
                        Item::Host,
                        Item::FolderBrowse,
                        Item::Connect,
                        Item::CancelForm,
                    ]);
                    if matches!(self.mode, Mode::Form(Some(_))) {
                        items.push(Item::Remove);
                    }
                    return items;
                }
                if caps.needs_host {
                    items.push(Item::Host);
                }
                if caps.org {
                    items.push(Item::Org);
                }
                if caps.account.is_some() {
                    items.push(Item::Account);
                }
                items.push(Item::Token);
                if self.strict_offered() {
                    items.push(Item::StrictPin);
                }
                if self.plain_http_offered() {
                    items.push(Item::AllowPlain);
                }
                if caps.account == Some(AccountField::ApiKey) {
                    items.push(Item::GetKey);
                }
                items.extend([Item::Generate, Item::Connect, Item::CancelForm]);
                // An existing profile that is not connected (never verified,
                // disconnected, or failing) opens in this form, so Remove
                // must be reachable from here too.
                if matches!(self.mode, Mode::Form(Some(_))) {
                    items.push(Item::Remove);
                }
                items
            }
            View::Connecting => vec![Item::CancelConnect],
            View::Connected(i) => {
                let mut items = vec![Item::Edit, Item::Disconnect, Item::Remove];
                // A folder has no token to disconnect.
                if cfg.integrations[i].kind.capabilities().local_folder {
                    items.retain(|it| *it != Item::Disconnect);
                }
                items.extend(self.ssh_items(&cfg.integrations[i]));
                items.extend(self.radar_items(&cfg.integrations[i]));
                items
            }
            View::Confirm(_) => vec![Item::ConfirmRemove, Item::CancelRemove],
            View::ConfirmKey(_) => vec![Item::SshConfirmDelete, Item::SshCancelDelete],
        }
    }

    /// The SSH section's controls for a profile: the key line when one is
    /// attached, else the radio and the selected mode's fields.
    fn ssh_items(&self, e: &Integration) -> Vec<Item> {
        if !e.kind.capabilities().ssh_keys {
            return vec![];
        }
        if has_key(e) {
            return vec![Item::SshRemove, Item::SshDelete];
        }
        let mut items = vec![Item::SshDefault, Item::SshGenerate, Item::SshCustom];
        match self.ssh.mode {
            SshMode::Default => {}
            SshMode::Generated => items.extend([Item::SshTitle, Item::SshUpload, Item::SshGo]),
            SshMode::Custom => items.extend([Item::SshPath, Item::SshBrowse, Item::SshUse]),
        }
        items
    }

    fn current_item(&self, cfg: &Config) -> Option<Item> {
        let items = self.items(cfg);
        items
            .get(self.item.min(items.len().saturating_sub(1)))
            .copied()
    }

    /// Whether the keyboard is in a text field (so `?`, letters, ... are
    /// text, not commands).
    pub(crate) fn text_focused(&self, cfg: &Config) -> bool {
        self.ssh.picker.is_some()
            || self.zone == Zone::Body && self.current_item(cfg).is_some_and(Item::is_text)
    }

    /// Back to the read-only view of the selection. A profile that was
    /// saved but never verified opens straight into its form.
    fn reset_view(&mut self, cfg: &Config) {
        self.pending = None;
        self.host.clear();
        self.org.clear();
        self.account.clear();
        self.token.clear();
        self.error = None;
        self.ssh.mode = SshMode::Default;
        self.ssh.error = None;
        self.mode = Mode::Browse;
        self.item = 0;
        if let Some(i) = self.selected_profile(cfg) {
            if cfg.integrations[i].user.is_none() && self.kind().capabilities().implemented {
                self.begin_form(cfg, Some(i));
            }
        }
    }

    fn begin_form(&mut self, cfg: &Config, editing: Option<usize>) {
        if !self.kind().capabilities().implemented {
            return;
        }
        self.host = match editing {
            Some(i)
                if self.kind().capabilities().needs_host
                    || self.kind().capabilities().local_folder =>
            {
                cfg.integrations[i].url.clone()
            }
            // Forgejo starts on Codeberg, still editable.
            None if self.kind().capabilities().needs_host => self
                .kind()
                .default_url()
                .map(str::to_string)
                .unwrap_or_default(),
            _ => String::new(),
        };
        self.org = match editing {
            Some(i) => cfg.integrations[i].azure_org().unwrap_or_default(),
            None => String::new(),
        };
        self.account = match editing {
            Some(i) => cfg.integrations[i].account.clone(),
            None => String::new(),
        };
        self.token.clear();
        self.error = None;
        self.retype_hint = false;
        // The toggles start from what is trusted for the profile.
        let record = editing.and_then(|i| self.trust_gate().record_of(&cfg.integrations[i].id));
        self.strict = record
            .as_ref()
            .and_then(|r| r.tls.as_ref())
            .is_some_and(|t| t.mode == gitpoulpi_hosting::trust::TlsMode::Strict);
        self.plain_http = record.as_ref().is_some_and(|r| r.allow_plain_http);
        self.mode = Mode::Form(editing.map(|i| cfg.integrations[i].id.clone()));
        self.item = 0;
    }

    /// The security check's answer to an address change: the profile's form,
    /// with the token field focused and a hint to type the token again (the
    /// old one was never sent to the new address).
    pub(crate) fn begin_retoken(&mut self, cfg: &Config, id: &str) {
        let Some(i) = cfg.integrations.iter().position(|e| e.id == id) else {
            return;
        };
        if let Some(kind) = self.kind_index(cfg.integrations[i].kind) {
            self.select_kind(kind, cfg);
            self.sel = self
                .profile_ids(cfg)
                .iter()
                .position(|n| *n == i)
                .unwrap_or(0);
        }
        self.begin_form(cfg, Some(i));
        self.retype_hint = true;
        self.pinned.remove(id);
        self.item = self
            .items(cfg)
            .iter()
            .position(|it| *it == Item::Token)
            .unwrap_or(0);
        self.zone = Zone::Body;
    }

    /// The gate over the store this page reads and writes.
    fn trust_gate(&self) -> gitpoulpi_hosting::TrustGate {
        match &self.trust_paths {
            Some(paths) => gitpoulpi_hosting::TrustGate::load(paths),
            None => gitpoulpi_hosting::TrustGate::ambient(),
        }
    }

    /// Forgets the pinned-certificate readings (the trust store changed).
    pub(crate) fn forget_trust_cache(&mut self) {
        self.pinned.clear();
    }

    /// Whether the form offers the pinning toggle: an https address.
    fn strict_offered(&self) -> bool {
        !self.kind().capabilities().local_folder
            && self
                .form_url()
                .is_some_and(|u| u.to_ascii_lowercase().starts_with("https://"))
    }

    /// Whether the form offers the plain http toggle: only for an `http://`
    /// address that is not this machine.
    fn plain_http_offered(&self) -> bool {
        self.form_url()
            .is_some_and(|u| gitpoulpi_hosting::trust::origin::is_plain_http_remote(&u))
    }

    pub(crate) fn select_kind(&mut self, index: usize, cfg: &Config) {
        self.kind = index.min(self.kinds.len() - 1);
        self.sel = 0;
        self.reset_view(cfg);
    }

    fn select_profile(&mut self, index: usize, cfg: &Config) {
        self.sel = index.min(self.profile_ids(cfg).len());
        self.reset_view(cfg);
    }

    /// The base URL the form currently points at.
    fn form_url(&self) -> Option<String> {
        let caps = self.kind().capabilities();
        if caps.org {
            let org = azure_org_input(&self.org)?;
            return Some(format!("https://dev.azure.com/{org}"));
        }
        if caps.local_folder {
            let folder = self.host.trim();
            return (!folder.is_empty()).then(|| folder.to_string());
        }
        if caps.needs_host {
            normalize_host(&self.host)
        } else {
            self.kind().public_url().map(str::to_string)
        }
    }

    /// The typed account, trimmed.
    fn account_text(&self) -> String {
        self.account.trim().to_string()
    }

    /// Why Connect cannot start yet (no token is checked here: that is
    /// `can_connect`'s second half).
    fn missing_input(&self) -> Option<M> {
        let caps = self.kind().capabilities();
        if self.form_url().is_none() {
            return Some(if caps.local_folder {
                gitpoulpi_i18n::msg::local_issues::ERR_FOLDER
            } else if caps.org {
                msg2::ERR_ORG
            } else {
                msg::ERR_HOST
            });
        }
        match caps.account {
            Some(AccountField::Email) if !self.account_text().contains('@') => {
                Some(msg2::ERR_ACCOUNT_EMAIL)
            }
            Some(AccountField::ApiKey) if self.account_text().is_empty() => {
                Some(msg2::ERR_ACCOUNT_KEY)
            }
            Some(AccountField::Username) if self.account_text().is_empty() => {
                Some(msg2::ERR_ACCOUNT_USERNAME)
            }
            _ => None,
        }
    }

    /// The page that mints the token: it needs the host (or organization)
    /// for most kinds, the typed API key for Trello.
    fn token_url(&self) -> Option<String> {
        let kind = self.kind();
        let help = kind.token_help()?;
        let url = self.form_url().unwrap_or_default();
        if matches!(help.link, TokenLink::Path(_)) && url.is_empty() {
            return None;
        }
        let mut probe = Integration::new("form", kind, url.clone());
        probe.account = self.account_text();
        probe.token_link_url(&url)
    }

    fn can_connect(&self, cfg: &Config, presence: &HashMap<String, Option<bool>>) -> bool {
        if self.kind().capabilities().local_folder {
            return self.missing_input().is_none();
        }
        self.missing_input().is_none()
            && (!self.token.trim().is_empty() || self.stored_token(cfg, presence))
    }

    fn stored_token(&self, cfg: &Config, presence: &HashMap<String, Option<bool>>) -> bool {
        match &self.mode {
            Mode::Form(Some(id)) => cfg
                .integrations
                .iter()
                .find(|i| &i.id == id)
                .and_then(|i| presence.get(&i.token_ref).copied().flatten())
                .unwrap_or(false),
            _ => false,
        }
    }
}

/// Whether the profile has a key attached (generated or custom).
fn has_key(e: &Integration) -> bool {
    e.ssh
        .as_ref()
        .is_some_and(|s| s.mode != SshMode::Default && s.private_key_path.is_some())
}

use render::{kv, label, muted, put};
use render_forms::put_wrapped;

mod input;
mod jobs;
mod radar;
mod render;
mod render_forms;

#[cfg(test)]
#[cfg(all(feature = "github", feature = "gitlab"))]
mod tests;
