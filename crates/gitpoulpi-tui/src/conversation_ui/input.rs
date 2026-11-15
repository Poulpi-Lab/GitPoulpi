//! Keys and the mouse over the timeline.
//!
//! Movement and scrolling happen here; anything that leaves the widget (a
//! reply, a link, a refresh) is answered as an [`Act`] for the surface around
//! it. Every mouse action has its key: the arrows and `Ctrl+J` / `Ctrl+K`
//! move between entries, `PageUp` / `PageDown` / `Home` / `End` scroll, and
//! each button of the selected entry has a letter.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::{Btn, Conversation, Hit};
use crate::md_render::LinkTarget;

/// What the surface around the timeline has to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Act {
    /// The widget took the event (a move, a scroll, a hover).
    Handled,
    Reply,
    Quote,
    Edit,
    Delete,
    /// Open the menu of the selected entry's links.
    Links,
    /// Open the Mermaid diagram of the selected entry.
    OpenDiagram,
    CopyLink,
    OpenBrowser,
    NewComment,
    Refresh,
    LoadOlder,
    ToggleEvents,
    Back,
    Close,
    /// A link was activated.
    Link(LinkTarget),
}

/// The wheel moves this many rows per notch.
const WHEEL: isize = 3;

/// The key twin of a button.
pub(crate) type KeyAct = Act;

fn plain(key: &KeyEvent) -> bool {
    !key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
}

impl Conversation {
    /// A key while the timeline has the keyboard. `None`: not the timeline's.
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Option<Act> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL)
            && !key.modifiers.intersects(KeyModifiers::ALT);
        match key.code {
            KeyCode::Up if plain(&key) => self.step(-1),
            KeyCode::Down if plain(&key) => self.step(1),
            KeyCode::Char('k') | KeyCode::Char('K') if ctrl => self.step(-1),
            KeyCode::Char('j') | KeyCode::Char('J') if ctrl => self.step(1),
            KeyCode::PageUp if plain(&key) => self.page(false),
            KeyCode::PageDown if plain(&key) => self.page(true),
            KeyCode::Home if plain(&key) => self.select(0),
            KeyCode::End if plain(&key) => self.select(usize::MAX),
            KeyCode::Enter if plain(&key) => return Some(self.enter()),
            KeyCode::F(5) => return Some(Act::Refresh),
            KeyCode::Left if key.modifiers == KeyModifiers::ALT => return Some(Act::Back),
            KeyCode::Backspace if plain(&key) => return Some(Act::Back),
            KeyCode::Esc => return Some(Act::Close),
            KeyCode::Char(c) if plain(&key) => {
                return match c {
                    'q' => Some(Act::Close),
                    'r' => Some(Act::Reply),
                    'Q' => Some(Act::Quote),
                    'e' => Some(Act::Edit),
                    'd' => Some(Act::Delete),
                    'l' => Some(Act::Links),
                    'g' => Some(Act::OpenDiagram),
                    'y' => Some(Act::CopyLink),
                    'o' => Some(Act::OpenBrowser),
                    'c' => Some(Act::NewComment),
                    'v' => Some(Act::ToggleEvents),
                    'm' => Some(Act::LoadOlder),
                    'n' => {
                        self.next_unread();
                        Some(Act::Handled)
                    }
                    'k' => {
                        self.step(-1);
                        Some(Act::Handled)
                    }
                    'j' => {
                        self.step(1);
                        Some(Act::Handled)
                    }
                    _ => None,
                };
            }
            _ => return None,
        }
        Some(Act::Handled)
    }

    /// Enter on the selection: the only link it holds, else the menu of them.
    fn enter(&mut self) -> Act {
        let mut links = self.selected_links();
        match links.len() {
            0 => Act::Handled,
            1 => Act::Link(links.remove(0)),
            _ => Act::Links,
        }
    }

    /// A mouse event over the timeline.
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> Option<Act> {
        let rows = self.layout.rows.len();
        if let Some(top) = self.drag.on_mouse(self.bar, mouse, rows, self.top) {
            self.scroll_to(top);
            return Some(Act::Handled);
        }
        if self.drag.held() {
            return Some(Act::Handled);
        }
        let hit = self.hit_at(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = hit;
                None
            }
            MouseEventKind::ScrollUp if self.area.contains((mouse.column, mouse.row).into()) => {
                self.scroll(-WHEEL);
                Some(Act::Handled)
            }
            MouseEventKind::ScrollDown if self.area.contains((mouse.column, mouse.row).into()) => {
                self.scroll(WHEEL);
                Some(Act::Handled)
            }
            MouseEventKind::Down(MouseButton::Left) => match hit? {
                Hit::Item(i) => {
                    self.select(i);
                    Some(Act::Handled)
                }
                Hit::Btn(btn, i) => {
                    self.select(i);
                    Some(match btn {
                        Btn::Reply => Act::Reply,
                        Btn::Quote => Act::Quote,
                        Btn::Edit => Act::Edit,
                        Btn::Delete => Act::Delete,
                        Btn::Links => Act::Links,
                    })
                }
                Hit::Link(id) => self.link(id).cloned().map(Act::Link),
                Hit::LoadOlder => Some(Act::LoadOlder),
            },
            _ => None,
        }
    }
}
