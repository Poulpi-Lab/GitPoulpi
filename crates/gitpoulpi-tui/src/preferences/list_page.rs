//! Master-detail pages (`PrefSection::has_list`): how Settings drives a
//! [`ListEditor`](super::list_editor::ListEditor) and lays the page out
//! around it.
//!
//! A page plugs in by (1) answering `PrefSection::has_list`, (2) building its
//! `ListSpec` in `list_spec`, (3) applying each `ListOp` to its own vector in
//! `apply_list_op`, and (4) making `fields()` return the *detail* rows of the
//! selected item (page-level rows such as "Active profile" stay first, at the
//! top of the detail column). Layout, focus (`PrefFocus::List`), keys and
//! mouse are generic and live here.
//!
//! Wide body: the list on the left (`clamp(body / 3, 26, 36)`), the detail
//! rows right of it. Narrow body (< 72 columns): the list on top, the detail
//! below; only the detail scrolls.

use super::list_editor::{ListEvent, ListItem, ListOp, ListSpec, Mark};
use super::persist::unique_id;
use super::*;
use gitpoulpi_i18n::msg::settings_profiles as profiles;

/// Below this body width the list stacks above the detail.
const NARROW: u16 = 72;

impl PreferencesState {
    /// The current page's list, from the config (empty on a page without one).
    fn list_spec(&self) -> ListSpec {
        match self.current_section() {
            PrefSection::Profiles => {
                let items = self
                    .config
                    .profiles
                    .iter()
                    .map(|p| {
                        let mark = if p.author_email.trim().is_empty() {
                            Mark::Problem(profiles::NO_EMAIL.get().to_string())
                        } else if p.id == self.config.general.active_profile {
                            Mark::Active
                        } else {
                            Mark::None
                        };
                        ListItem {
                            label: if p.name.trim().is_empty() {
                                p.id.clone()
                            } else {
                                p.name.clone()
                            },
                            sub: Some(p.author_email.clone()).filter(|e| !e.is_empty()),
                            mark,
                            pristine: self.is_pristine_profile(p),
                            ..ListItem::default()
                        }
                    })
                    .collect();
                ListSpec {
                    title: profiles::LIST_TITLE.get().to_string(),
                    items,
                    duplicate: true,
                    reorder: true,
                    ..ListSpec::default()
                }
            }
            PrefSection::Workflow => self.wf_list_spec(),
            PrefSection::Ai => self.ai_list_spec(),
            PrefSection::OpenIn => self.open_in_list_spec(),
            PrefSection::Images => self.images_list_spec(),
            PrefSection::Sequences => self.seq_list_spec(),
            _ => ListSpec::default(),
        }
    }

    /// A profile exactly as "Add" made it: removing it asks nothing.
    fn is_pristine_profile(&self, p: &Profile) -> bool {
        p.name == profiles::NEW_PROFILE_NAME.get()
            && p.author_name.is_empty()
            && p.author_email.is_empty()
    }

    /// The selected item of the current page's list.
    fn list_selected(&self) -> usize {
        match self.current_section() {
            PrefSection::Profiles => self.profile_index,
            PrefSection::Workflow => self.wf_list_selected(),
            PrefSection::Ai => self.ai.selected_index(),
            PrefSection::OpenIn => self.open_in_index,
            PrefSection::Images => self.image_index,
            PrefSection::Sequences => self.seq_selected(),
            _ => 0,
        }
    }

    fn set_list_selected(&mut self, index: usize) {
        match self.current_section() {
            PrefSection::Profiles => self.profile_index = index,
            PrefSection::Workflow => {
                self.wf_select_index(index);
                self.scroll = 0;
            }
            PrefSection::Ai => self.ai.select_index(index),
            PrefSection::OpenIn => self.open_in_index = index,
            PrefSection::Images => self.image_index = index,
            PrefSection::Sequences => self.seq_select(index),
            _ => {}
        }
    }

    /// Feeds the editor the page's current items and selection.
    pub(super) fn sync_list(&mut self) {
        if self.current_section() == PrefSection::Workflow {
            self.wf_normalise();
        }
        if self.current_section() == PrefSection::Sequences {
            self.seq_normalise();
        }
        let spec = self.list_spec();
        self.list.set_spec(spec);
        self.list.select(self.list_selected());
    }

    /// The first row of the detail (where Enter or Right from the list lands).
    fn first_detail_field(&self) -> usize {
        self.fields()
            .iter()
            .position(|f| {
                f.selectable()
                    && matches!(
                        f.target,
                        Target::ProfileField(..)
                            | Target::Ai(ai_page::AiTarget::ProfileName)
                            | Target::OpenIn(_)
                            | Target::ImageEditor(_)
                            | Target::Workflow(_)
                            | Target::Sequence(_)
                    )
            })
            .unwrap_or_else(|| self.first_selectable())
    }

    /// A key while the list has the keyboard. `None` means "not the list's":
    /// the generic keys (Tab, Esc, ...) handle it.
    pub(super) fn list_key(&mut self, key: KeyEvent) -> Option<bool> {
        self.sync_list();
        match self.list.on_key(key) {
            ListEvent::Ignored => None,
            ListEvent::Handled => Some(true),
            ListEvent::Op(op) => {
                self.apply_list_op(op);
                Some(true)
            }
        }
    }

    /// A mouse event for the list (its popups and drags first). Whether the
    /// list took it.
    pub(super) fn list_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        self.sync_list();
        if kind == MouseEventKind::Down(MouseButton::Left)
            && !self.list.owns_pointer()
            && self.list_area.contains((col, row).into())
        {
            self.focus = PrefFocus::List;
            self.strip = false;
        }
        let event = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        match self.list.on_mouse(event) {
            ListEvent::Ignored => false,
            ListEvent::Handled => true,
            ListEvent::Op(op) => {
                self.apply_list_op(op);
                true
            }
        }
    }

    pub(super) fn apply_list_op(&mut self, op: ListOp) {
        match op {
            ListOp::Select(i) => self.set_list_selected(i),
            ListOp::FocusDetail => {
                self.focus = PrefFocus::Fields;
                self.strip = false;
                self.field = self.first_detail_field();
            }
            ListOp::Back => self.focus = PrefFocus::Sections,
            _ if self.current_section() == PrefSection::Profiles => self.profile_op(op),
            _ if self.current_section() == PrefSection::Workflow => self.wf_list_op(op),
            _ if self.current_section() == PrefSection::Ai => self.ai_profile_op(op),
            _ if self.current_section() == PrefSection::OpenIn => self.open_in_op(op),
            _ if self.current_section() == PrefSection::Images => self.images_op(op),
            _ if self.current_section() == PrefSection::Sequences => self.seq_list_op(op),
            _ => {}
        }
        self.sync_list();
    }

    /// Focuses the first detail row whose target satisfies `want`.
    pub(super) fn focus_detail_target(&mut self, want: impl Fn(&Target) -> bool) {
        if let Some(at) = self.fields().iter().position(|f| want(&f.target)) {
            self.focus = PrefFocus::Fields;
            self.strip = false;
            self.field = at;
        }
    }

    fn profile_op(&mut self, op: ListOp) {
        let len = self.config.profiles.len();
        match op {
            ListOp::Add(_) => {
                let id = unique_id("profile", |candidate| {
                    self.config.profiles.iter().any(|p| p.id == candidate)
                });
                self.config
                    .profiles
                    .push(Profile::new(id, profiles::NEW_PROFILE_NAME.get()));
                self.profile_index = len;
            }
            ListOp::Duplicate(i) if i < len => {
                let mut copy = self.config.profiles[i].clone();
                let stem = copy.id.clone();
                copy.id = unique_id(&stem, |candidate| {
                    self.config.profiles.iter().any(|p| p.id == candidate)
                });
                copy.name = profiles::duplicate_name(&copy.name);
                self.config.profiles.insert(i + 1, copy);
                self.profile_index = i + 1;
            }
            ListOp::Remove(i) if i < len => {
                let removed = self.config.profiles.remove(i);
                self.profile_index = i.min(len.saturating_sub(2));
                self.write_profiles();
                if self.config.general.active_profile == removed.id {
                    self.write_global("general.active_profile", "");
                }
                self.field = self.first_selectable();
                return;
            }
            ListOp::Move { from, to } if from < len && to < len => {
                let item = self.config.profiles.remove(from);
                self.config.profiles.insert(to, item);
                self.profile_index = to;
            }
            _ => return,
        }
        self.write_profiles();
    }

    /// Paints a master-detail page: the list, and the detail rows beside or
    /// below it.
    pub(super) fn render_list_body(&mut self, frame: &mut Frame, area: Rect) {
        self.sync_list();
        // The title and the sub-tab strip above, then the General tab's rows
        // or the list with the selected item's rows.
        let area = self.render_tab_head(frame, area);
        self.body_head = 1;
        if self.sub_tab() == SubTab::General {
            self.render_field_list(frame, area);
            return;
        }
        let (list, detail) = if area.width < NARROW {
            let rows = self.list.spec.items.len() as u16 + 3;
            let h = rows.min(8).min(area.height.saturating_sub(6));
            (
                Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), h),
                Rect::new(area.x, area.y + h + 1, area.width, area.height - h - 1),
            )
        } else {
            let w = (area.width / 3).clamp(26, 36);
            (
                Rect::new(area.x + 1, area.y + 1, w, area.height.saturating_sub(1)),
                Rect::new(area.x + w + 2, area.y, area.width - w - 2, area.height),
            )
        };
        self.list_area = list;
        self.render_field_list(frame, detail);
        let focused = self.focus == PrefFocus::List;
        self.list
            .render(frame.buffer_mut(), list, focused, self.area);
    }
}
