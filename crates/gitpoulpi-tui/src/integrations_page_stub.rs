//! `integrations_page` for a build with no hosting provider: the
//! Settings Integrations page is hidden (`PrefSection::requires`), so this
//! is only the inert state `PreferencesState` carries. `[[integrations]]`
//! in the config file still parses and is kept as is.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crossterm::event::{KeyEvent, MouseEventKind};
use gitpoulpi_config::Config;
use gitpoulpi_secrets::TokenStore;
use ratatui::layout::Rect;
use ratatui::Frame;

type Tokens = Arc<dyn TokenStore + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Zone {
    #[default]
    Types,
    Profiles,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nav {
    Stay,
    Sections,
}

pub(crate) struct Env<'a> {
    pub(crate) config: &'a mut Config,
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) tokens: &'a Tokens,
    pub(crate) presence: &'a mut HashMap<String, Option<bool>>,
    pub(crate) message: &'a mut Option<String>,
}

#[derive(Default)]
pub(crate) struct IntegrationsPage {
    pub(crate) zone: Zone,
    pub(crate) sel: usize,
}

impl IntegrationsPage {
    pub(crate) fn detach_pending(&mut self) {}
    pub(crate) fn combo_open(&self) -> bool {
        false
    }
    pub(crate) fn owns_pointer(&self) -> bool {
        false
    }
    pub(crate) fn focus_radar(&mut self, _cfg: &Config) {}
    pub(crate) fn at_top(&self) -> bool {
        false
    }
    pub(crate) fn animating(&self) -> bool {
        false
    }
    pub(crate) fn poll(&mut self, _env: &mut Env) {}
    pub(crate) fn text_focused(&self, _cfg: &Config) -> bool {
        false
    }
    pub(crate) fn kind_index(&self, _kind: gitpoulpi_config::IntegrationKind) -> Option<usize> {
        None
    }
    pub(crate) fn select_kind(&mut self, _index: usize, _cfg: &Config) {}
    pub(crate) fn on_key(&mut self, _key: &KeyEvent, _env: &mut Env) -> Nav {
        Nav::Sections
    }
    pub(crate) fn paste(&mut self, _text: &str, _cfg: &Config) -> bool {
        false
    }
    pub(crate) fn on_mouse(
        &mut self,
        _kind: MouseEventKind,
        _col: u16,
        _row: u16,
        _env: &mut Env,
    ) -> bool {
        false
    }
    pub(crate) fn render(
        &mut self,
        _frame: &mut Frame,
        _area: Rect,
        _cfg: &Config,
        _presence: &HashMap<String, Option<bool>>,
        _focused: bool,
        _mouse: Option<(u16, u16)>,
    ) {
    }
}
