use super::*;
use crate::keymap::term::TermKeymap;

impl FolderPicker {
    // -- keys -------------------------------------------------------------------

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> PickerOutcome {
        // The "New folder" field, while it is open, claims every key.
        if self.new_folder_draft.is_some() {
            return self.new_folder_key(key);
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let km = crate::keymap::current();
        self.message = None;

        // `/` types into the path and filter fields; elsewhere it opens the
        // filter, like `Ctrl+F` does everywhere.
        let slash_typed = matches!(key.code, KeyCode::Char('/'))
            && !ctrl
            && !alt
            && matches!(self.zone, Zone::Path | Zone::Filter);

        // Everywhere.
        match key.code {
            // The filter field: Esc clears what it holds first, then leaves it.
            KeyCode::Esc if self.zone == Zone::Filter => {
                if self.filter.is_empty() {
                    self.focus(Zone::Content);
                } else {
                    self.filter.clear();
                    self.after_filter_edit();
                }
                return PickerOutcome::Stay;
            }
            KeyCode::Esc if !self.filter.is_empty() => {
                self.filter.clear();
                self.after_filter_edit();
                return PickerOutcome::Stay;
            }
            KeyCode::Esc if self.typing && !self.completions().is_empty() => {
                self.typing = false;
                self.completion = None;
                return PickerOutcome::Stay;
            }
            KeyCode::Esc => return PickerOutcome::Cancel,
            _ if km.is(Bind::FolderPickerChoose, &key) => return self.choose_selected(),
            _ if km.is(Bind::FolderPickerFilter, &key) && !slash_typed => {
                self.focus(Zone::Filter);
                return PickerOutcome::Stay;
            }
            _ if km.is(Bind::FolderPickerNewFolder, &key) => {
                self.start_new_folder();
                return PickerOutcome::Stay;
            }
            _ if km.is(Bind::FolderPickerSiblings, &key) => {
                self.toggle_show_siblings();
                return PickerOutcome::ShowSiblingsChanged(self.show_siblings);
            }
            // The splitter's keyboard equivalent.
            _ if km.is(Bind::FolderPickerHidden, &key) => {
                self.toggle_show_hidden();
                return PickerOutcome::ShowHiddenChanged(self.show_hidden);
            }
            KeyCode::Left if alt => {
                self.nudge_splitter(-SPLIT_STEP);
                return PickerOutcome::Stay;
            }
            KeyCode::Right if alt => {
                self.nudge_splitter(SPLIT_STEP);
                return PickerOutcome::Stay;
            }
            KeyCode::Up if alt => {
                self.go_up();
                return PickerOutcome::Stay;
            }
            KeyCode::Tab if self.zone == Zone::Path && !shift => {
                let found = self.completions();
                if self.typing && (self.completion.is_some() || found.len() == 1) {
                    self.accept_completion(self.completion.unwrap_or(0));
                    return PickerOutcome::Stay;
                }
                self.cycle_zone(true);
                return PickerOutcome::Stay;
            }
            KeyCode::Tab => {
                self.cycle_zone(!shift);
                return PickerOutcome::Stay;
            }
            KeyCode::BackTab => {
                self.cycle_zone(false);
                return PickerOutcome::Stay;
            }
            _ => {}
        }

        match self.zone {
            Zone::Path => self.path_key(key),
            Zone::Filter => self.filter_key(key),
            Zone::Left => self.left_key(key),
            Zone::Content => self.content_key(key, ctrl),
        }
    }

    fn cycle_zone(&mut self, forward: bool) {
        const ORDER: [Zone; 4] = [Zone::Path, Zone::Filter, Zone::Left, Zone::Content];
        let at = ORDER.iter().position(|z| *z == self.zone).unwrap_or(0);
        let next = (if forward {
            at + 1
        } else {
            at + ORDER.len() - 1
        }) % ORDER.len();
        self.focus(ORDER[next]);
    }

    pub(crate) fn focus(&mut self, zone: Zone) {
        self.zone = zone;
        match zone {
            Zone::Path => {
                self.draft_pristine = !self.typing;
            }
            Zone::Filter => self.sync_edit(),
            Zone::Left if self.left_sel.is_none() => {
                // Start on the current folder's node.
                self.left_sel = self
                    .left_rows()
                    .iter()
                    .position(|r| matches!(r, LeftRow::Tree { current: true, .. }));
                self.follow = true;
            }
            _ => {}
        }
    }

    fn path_key(&mut self, key: KeyEvent) -> PickerOutcome {
        // The editing keys, through the shared core:
        // caret, selection, Ctrl+A/U/K (Ctrl+U deletes to the start, so
        // with the caret at the end it clears the path), Ctrl+C/X.
        self.sync_edit();
        if let Some((edit, copied)) =
            self.edit
                .dialog_key(PickerField::Path, &mut self.path_draft, &key)
        {
            self.draft_pristine = false;
            if edit == crate::text_edit::KeyEdit::Edited {
                self.after_edit();
            }
            if copied.is_some() {
                self.message = copied;
            }
            return PickerOutcome::Stay;
        }
        let found = self.completions().len();
        match key.code {
            KeyCode::Enter => {
                if let Some(index) = self.completion {
                    self.accept_completion(index);
                    return PickerOutcome::Stay;
                }
                return self.submit_path();
            }
            KeyCode::Down if found > 0 => {
                self.completion = match self.completion {
                    None => Some(0),
                    Some(i) if i + 1 < found => Some(i + 1),
                    // Off the end, back to the field.
                    Some(_) => None,
                };
            }
            KeyCode::Up if found > 0 => {
                self.completion = match self.completion {
                    None => Some(found - 1),
                    Some(0) => None,
                    Some(i) => Some(i - 1),
                };
            }
            KeyCode::Down => self.focus(Zone::Content),
            _ => {}
        }
        PickerOutcome::Stay
    }

    /// A key in the filter field: the shared editing core, then `Enter` /
    /// `Down` (into the list, on its first match) and `Up` (to the path).
    fn filter_key(&mut self, key: KeyEvent) -> PickerOutcome {
        self.sync_edit();
        if let Some((edit, copied)) =
            self.edit
                .dialog_key(PickerField::Filter, &mut self.filter, &key)
        {
            if edit == crate::text_edit::KeyEdit::Edited {
                self.after_filter_edit();
            }
            if copied.is_some() {
                self.message = copied;
            }
            return PickerOutcome::Stay;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Down => {
                self.focus(Zone::Content);
                // The first real match, not the `..` row.
                let first = self
                    .content_rows()
                    .iter()
                    .position(|r| !matches!(r, ContentRow::Parent(_)));
                if first.is_some() && self.content_selectable() > 0 {
                    self.content_sel = first.filter(|i| *i < self.content_selectable());
                    self.follow = true;
                }
            }
            KeyCode::Up => self.focus(Zone::Path),
            _ => {}
        }
        PickerOutcome::Stay
    }

    fn left_key(&mut self, key: KeyEvent) -> PickerOutcome {
        let rows = self.left_rows();
        let selectable: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].selectable()).collect();
        let at = self
            .left_sel
            .and_then(|s| selectable.iter().position(|&i| i == s));
        let step = |delta: isize| -> Option<usize> {
            if selectable.is_empty() {
                return None;
            }
            let from = at.map(|a| a as isize).unwrap_or(-1);
            let to = (from + delta).clamp(0, (selectable.len() as isize - 1).max(0)) as usize;
            Some(selectable[to])
        };
        match key.code {
            KeyCode::Up => self.left_sel = step(-1),
            KeyCode::Down => self.left_sel = step(1),
            KeyCode::PageUp => self.left_sel = step(-10),
            KeyCode::PageDown => self.left_sel = step(10),
            KeyCode::Home => self.left_sel = selectable.first().copied(),
            KeyCode::End => self.left_sel = selectable.last().copied(),
            KeyCode::Enter | KeyCode::Right => {
                if let Some(target) = self
                    .left_sel
                    .and_then(|i| rows.get(i))
                    .and_then(|r| r.target(&self.places))
                {
                    self.navigate(target);
                    self.focus(Zone::Left);
                }
            }
            KeyCode::Backspace | KeyCode::Left => self.go_up(),
            _ => {}
        }
        self.follow = true;
        PickerOutcome::Stay
    }

    pub(super) fn content_key(&mut self, key: KeyEvent, ctrl: bool) -> PickerOutcome {
        let n = self.content_selectable();
        let move_to = |sel: Option<usize>, delta: isize| -> Option<usize> {
            if n == 0 {
                return None;
            }
            let from = sel.map(|s| s as isize).unwrap_or(-1);
            Some((from + delta).clamp(0, (n as isize - 1).max(0)) as usize)
        };
        match key.code {
            KeyCode::Up => self.content_sel = move_to(self.content_sel, -1),
            KeyCode::Down => self.content_sel = move_to(self.content_sel, 1),
            KeyCode::PageUp => self.content_sel = move_to(self.content_sel, -10),
            KeyCode::PageDown => self.content_sel = move_to(self.content_sel, 10),
            KeyCode::Home => self.content_sel = move_to(None, 1),
            KeyCode::End => self.content_sel = n.checked_sub(1),
            KeyCode::Enter | KeyCode::Right => {
                return match self.content_sel {
                    Some(i)
                        if key.code == KeyCode::Enter
                            && self.mode == PickerMode::File
                            && matches!(self.content_rows().get(i), Some(ContentRow::File(_))) =>
                    {
                        PickerOutcome::Choose(self.chosen())
                    }
                    Some(i) => {
                        self.activate_content(i);
                        PickerOutcome::Stay
                    }
                    // Nothing selected: Enter chooses the folder shown --
                    // a file picker has nothing to choose in that case.
                    None if key.code == KeyCode::Enter && self.mode == PickerMode::Folder => {
                        PickerOutcome::Choose(self.dir.clone())
                    }
                    None => PickerOutcome::Stay,
                };
            }
            KeyCode::Backspace | KeyCode::Left => self.go_up(),
            // Type-ahead: the next folder whose name starts with it.
            KeyCode::Char(c) if !ctrl => self.type_ahead(c),
            _ => {}
        }
        self.follow = true;
        PickerOutcome::Stay
    }

    fn type_ahead(&mut self, c: char) {
        let Some(listing) = &self.listing else { return };
        let wanted = c.to_lowercase().to_string();
        let rows = self.content_rows();
        let start = self.content_sel.map(|s| s + 1).unwrap_or(0);
        let n = self.content_selectable();
        for offset in 0..n {
            let i = (start + offset) % n;
            if let Some(ContentRow::Dir(d)) = rows.get(i) {
                if listing.dirs[*d].name.to_lowercase().starts_with(&wanted) {
                    self.content_sel = Some(i);
                    return;
                }
            }
        }
    }

    /// Carries out a content row: the parent goes up, a folder goes in.
    fn activate_content(&mut self, index: usize) {
        match self.content_rows().get(index).cloned() {
            Some(ContentRow::Parent(_)) => self.go_up(),
            Some(row @ ContentRow::Dir(_)) => {
                if let Some(path) = self.row_path(&row) {
                    self.navigate(path);
                }
            }
            _ => {}
        }
    }

    // -- mouse ------------------------------------------------------------------

    /// A mouse event inside the dialog. `double` is the event loop's
    /// double-click verdict for a left press.
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent, double: bool) -> PickerOutcome {
        let (col, row) = (mouse.column, mouse.row);
        let hit = dialog_kit::hit_at(&self.hits, col, row);
        // A drag selecting in the path field or the "New folder" field,
        // and its release -- `self.edit` tracks which one owns it.
        let drag_text = match (&self.new_folder_draft, self.zone) {
            (Some(draft), _) => draft.as_str(),
            (None, Zone::Filter) => self.filter.as_str(),
            (None, _) => self.path_draft.as_str(),
        };
        if !matches!(mouse.kind, MouseEventKind::Down(_))
            && self
                .edit
                .dialog_mouse(None, drag_text, mouse.kind, col, row)
        {
            return PickerOutcome::Stay;
        }
        // The lists' scrollbars take their presses and thumb drags first.
        if self.bar_mouse(mouse) {
            return PickerOutcome::Stay;
        }
        // A splitter drag in progress claims every move and its release,
        // wherever the pointer strays.
        if self.dragging_splitter {
            match mouse.kind {
                MouseEventKind::Drag(_) | MouseEventKind::Moved => {
                    self.resize_splitter_to(col);
                    self.hover = Some(PickerHit::Splitter);
                    return PickerOutcome::Stay;
                }
                MouseEventKind::Up(_) => {
                    self.dragging_splitter = false;
                    self.remember_split();
                    // A release brings no `Moved`: re-read the hover here, or
                    // the line stays lit after the button comes up.
                    self.hover = dialog_kit::hit_at(&self.hits, col, row);
                    return PickerOutcome::Stay;
                }
                _ => {}
            }
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                self.hover = hit;
                PickerOutcome::Stay
            }
            MouseEventKind::Up(_) => {
                self.dragging_splitter = false;
                self.hover = hit;
                PickerOutcome::Stay
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let delta: isize = if mouse.kind == MouseEventKind::ScrollDown {
                    3
                } else {
                    -3
                };
                if self.drop_area.contains((col, row).into()) {
                    self.drop_top = (self.drop_top as isize + delta).max(0) as usize;
                } else if self.content_area.contains((col, row).into()) {
                    self.content_scroll = (self.content_scroll as isize + delta).max(0) as usize;
                } else if self.left_area.contains((col, row).into()) {
                    self.left_scroll = (self.left_scroll as isize + delta).max(0) as usize;
                    self.reveal_current = false;
                }
                PickerOutcome::Stay
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.message = None;
                self.last_press = Some((col, row));
                self.on_click(hit, double)
            }
            _ => PickerOutcome::Stay,
        }
    }

    /// Moves the splitter so the divider sits under `col` (a drag), kept
    /// within [`SPLIT_MIN_LEFT`]..width-[`SPLIT_MIN_CONTENT`].
    fn resize_splitter_to(&mut self, col: u16) {
        if self.panes_w == 0 {
            return;
        }
        let want = col.saturating_sub(self.left_area.x);
        self.splitter_w = Some(self.clamp_left_w(want));
    }

    /// `Alt+Left`/`Alt+Right`: the splitter's keyboard equivalent.
    fn nudge_splitter(&mut self, delta: i32) {
        if self.panes_w == 0 {
            return;
        }
        let current = self.want_left_w().unwrap_or(self.default_left_w()) as i32;
        let want = (current + delta).max(0) as u16;
        self.splitter_w = Some(self.clamp_left_w(want));
        self.remember_split();
    }

    /// The left column width the user chose (this session of the dialog or
    /// remembered from an earlier one), before clamping.
    pub(super) fn want_left_w(&self) -> Option<u16> {
        self.splitter_w.or_else(|| {
            self.restored_permille
                .map(|p| (u32::from(self.panes_w) * u32::from(p) / 1000) as u16)
        })
    }

    /// Hands the share the user just set to the window for saving (a
    /// ratio of the pane row, so another terminal width scales it).
    fn remember_split(&self) {
        if let (Some(w), true) = (self.splitter_w, self.panes_w > 0) {
            let permille = (u32::from(w) * 1000 / u32::from(self.panes_w)) as u16;
            SPLIT.with(|c| c.set((Some(permille.max(1)), true)));
        }
    }

    pub(super) fn default_left_w(&self) -> u16 {
        (self.panes_w / 3).clamp(16, 34).min(self.panes_w / 2)
    }

    pub(super) fn clamp_left_w(&self, want: u16) -> u16 {
        let max_left = self
            .panes_w
            .saturating_sub(SPLIT_MIN_CONTENT + 2)
            .max(SPLIT_MIN_LEFT.min(self.panes_w));
        want.clamp(SPLIT_MIN_LEFT.min(max_left), max_left)
    }

    fn on_click(&mut self, hit: Option<PickerHit>, double: bool) -> PickerOutcome {
        // While the "New folder" field is open, a click on it moves the
        // caret; a click anywhere else cancels it, like a rename box.
        if self.new_folder_draft.is_some() && hit != Some(PickerHit::NewFolderField) {
            self.new_folder_draft = None;
            self.message = None;
            if hit.is_none() {
                return PickerOutcome::Stay;
            }
        }
        match hit {
            None => {}
            Some(PickerHit::NewFolderField) => {
                if let Some(draft) = self.new_folder_draft.clone() {
                    if let Some((col, row)) = self.last_press {
                        let now = std::time::Instant::now();
                        self.edit
                            .mouse_down(PickerField::NewFolder, &draft, col, row, now);
                    }
                }
            }
            Some(PickerHit::NewFolder) => self.start_new_folder(),
            Some(PickerHit::ShowHidden) => {
                self.toggle_show_hidden();
                return PickerOutcome::ShowHiddenChanged(self.show_hidden);
            }
            Some(PickerHit::ShowSiblings) => {
                self.toggle_show_siblings();
                return PickerOutcome::ShowSiblingsChanged(self.show_siblings);
            }
            Some(PickerHit::FilterField) => {
                self.focus(Zone::Filter);
                if let Some((col, row)) = self.last_press {
                    let now = std::time::Instant::now();
                    self.edit
                        .mouse_down(PickerField::Filter, &self.filter, col, row, now);
                }
            }
            Some(PickerHit::Splitter) => self.dragging_splitter = true,
            Some(PickerHit::UpButton) => self.go_up(),
            Some(PickerHit::PathField) => {
                self.focus(Zone::Path);
                self.sync_edit();
                // A click places the caret (a double-click selects a
                // word): the draft is no longer "all selected".
                if let Some((col, row)) = self.last_press {
                    let now = std::time::Instant::now();
                    self.edit
                        .mouse_down(PickerField::Path, &self.path_draft, col, row, now);
                    self.draft_pristine = false;
                }
            }
            Some(PickerHit::Completion(i)) => {
                self.accept_completion(i);
                self.zone = Zone::Path;
            }
            // The left column navigates on a single click, like the
            // Windows navigation pane.
            Some(PickerHit::Left(i)) => {
                if let Some(target) = self.left_rows().get(i).and_then(|r| r.target(&self.places)) {
                    self.navigate(target);
                    self.zone = Zone::Left;
                    self.left_sel = None;
                    self.focus(Zone::Left);
                }
            }
            // The content: single click selects, double click opens --
            // the Log's and the third pane's gesture pair.
            Some(PickerHit::Content(i)) => {
                self.zone = Zone::Content;
                if double {
                    if self.mode == PickerMode::File
                        && matches!(self.content_rows().get(i), Some(ContentRow::File(_)))
                    {
                        self.content_sel = Some(i);
                        return PickerOutcome::Choose(self.chosen());
                    }
                    self.activate_content(i);
                } else {
                    self.content_sel = Some(i);
                }
            }
            Some(PickerHit::LeftList) => self.focus(Zone::Left),
            Some(PickerHit::ContentList) => self.focus(Zone::Content),
            Some(PickerHit::Cancel) => return PickerOutcome::Cancel,
            Some(PickerHit::Choose) => return self.choose_selected(),
        }
        PickerOutcome::Stay
    }
}
