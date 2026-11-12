use super::*;

impl Terminal {
    /// Lets the child finish the frame it is drawing before a mouse press.
    ///
    /// A press sent the instant a button appears can be dropped on a loaded
    /// machine: the cell is painted a moment before its hit target is live,
    /// and the stray press lands on whatever was there (usually closing a
    /// menu). So wait, best effort, for a short quiet grid; a screen that
    /// never goes quiet (a spinner) costs at most the cap.
    ///
    /// Skipped when input was sent less than a second ago: that is a test
    /// pacing its own clicks (a hand-paced double click), and delaying the
    /// second would change what it proves.
    fn settle(&self) {
        if self.last_input.elapsed() < Duration::from_secs(1) {
            return;
        }
        let _ = self.wait_for_stable(Duration::from_millis(120), Duration::from_millis(1500));
    }

    /// Writes `bytes` to the child in **one** `write` call.
    ///
    /// A correctness requirement: split an escape sequence across two
    /// writes and crossterm's input parser, which tells a lone `ESC` from an
    /// escape sequence on *timing*, reads the `ESC` as the Escape key and the
    /// remainder as literal characters ("the click did nothing").
    fn write_once(&mut self, bytes: &str) -> Result<()> {
        let mut guard = self
            .writer
            .lock()
            .map_err(|_| Error::Pty("the pty writer lock is poisoned".into()))?;
        let writer = guard
            .as_mut()
            .ok_or_else(|| Error::Pty("the pty has been shut down".into()))?;
        writer.write_all(bytes.as_bytes())?;
        writer.flush()?;
        self.last_input = Instant::now();
        Ok(())
    }

    /// Sends a key spec - see [`keys::encode_keys`] for the grammar.
    pub fn send_keys(&mut self, spec: &str) -> Result<()> {
        // `cfg!` and not `#[cfg]`: both encodings compile (and are unit
        // tested) everywhere; only a ConPTY needs the second.
        let bytes = if cfg!(windows) {
            keys::encode_keys_win32(spec)
        } else {
            keys::encode_keys(spec)
        }
        .map_err(Error::Keys)?;
        self.write_once(&bytes)
    }

    /// Sends literal text, exactly as typed. Use this rather than
    /// `send_keys` for anything that contains spaces or could collide
    /// with a key name.
    pub fn send_text(&mut self, text: &str) -> Result<()> {
        self.write_once(text)
    }

    /// Moves the pointer to `(col, row)` with no button held.
    ///
    /// Unlike `ydotool mousemove --absolute`, which delivers no motion
    /// event, this *is* the motion event, so hover affordances can be tested.
    pub fn move_to(&mut self, col: u16, row: u16) -> Result<()> {
        // The Windows console merges consecutive mouse-move records that the
        // reader has not consumed yet, so a move away and straight back is
        // seen as no move at all (a tooltip that never restarts). Let the
        // previous input be read first.
        if cfg!(windows) {
            std::thread::sleep(Duration::from_millis(150));
        }
        self.write_once(&mouse::motion(col, row))
    }

    /// Press and release the left button at `(col, row)`.
    pub fn click(&mut self, col: u16, row: u16) -> Result<()> {
        self.click_with(Button::Left, col, row)
    }

    /// A left click with Ctrl held.
    pub fn ctrl_click(&mut self, col: u16, row: u16) -> Result<()> {
        self.click_mod(mouse::MOD_CTRL, col, row)
    }

    /// A left click with Shift held.
    pub fn shift_click(&mut self, col: u16, row: u16) -> Result<()> {
        self.click_mod(mouse::MOD_SHIFT, col, row)
    }

    /// A left click with Alt held.
    pub fn alt_click(&mut self, col: u16, row: u16) -> Result<()> {
        self.click_mod(mouse::MOD_ALT, col, row)
    }

    fn click_mod(&mut self, mods: u16, col: u16, row: u16) -> Result<()> {
        self.settle();
        let sequence = format!(
            "{}{}",
            mouse::press_mod(mods, col, row),
            mouse::release_mod(mods, col, row)
        );
        self.write_once(&sequence)
    }

    pub fn right_click(&mut self, col: u16, row: u16) -> Result<()> {
        self.click_with(Button::Right, col, row)
    }

    pub fn click_with(&mut self, button: Button, col: u16, row: u16) -> Result<()> {
        self.settle();
        let sequence = format!(
            "{}{}",
            mouse::press(button, col, row),
            mouse::release(button, col, row)
        );
        self.write_once(&sequence)
    }

    /// Two clicks at the same cell, close enough together to fall inside
    /// the TUI's own `DOUBLE_CLICK_WINDOW`. They go out in one write, so
    /// the gap between them is however long the child takes to run one
    /// event-loop iteration - never the several hundred milliseconds a
    /// synthetic-input tool would introduce.
    pub fn double_click(&mut self, col: u16, row: u16) -> Result<()> {
        self.settle();
        let one = format!(
            "{}{}",
            mouse::press(Button::Left, col, row),
            mouse::release(Button::Left, col, row)
        );
        self.write_once(&format!("{one}{one}"))
    }

    /// Press the left button and hold it.
    ///
    /// `press`/`drag_to`/`release` exist besides the one-shot `drag` because
    /// the interesting assertion about a drag is often *mid*-drag (the
    /// pane-divider highlight must follow the divider while the button is
    /// down), which the state after the release cannot show.
    pub fn press(&mut self, col: u16, row: u16) -> Result<()> {
        self.settle();
        self.write_once(&mouse::press(Button::Left, col, row))
    }

    /// Motion with the left button held - only meaningful between
    /// `press` and `release`.
    pub fn drag_to(&mut self, col: u16, row: u16) -> Result<()> {
        self.write_once(&mouse::drag_motion(Button::Left, col, row))
    }

    /// Release the left button.
    pub fn release(&mut self, col: u16, row: u16) -> Result<()> {
        self.write_once(&mouse::release(Button::Left, col, row))
    }

    /// Press at `from`, walk to `to`, release there.
    ///
    /// The intermediate positions are real: a drag handler that tracks the
    /// pointer needs motion reports between press and release, or it sees a
    /// click. One report per cell crossed, because a terminal reports
    /// position in cells.
    pub fn drag(&mut self, from: (u16, u16), to: (u16, u16)) -> Result<()> {
        if cfg!(windows) {
            // One report per write, paced: see `mouse::drag_reports`.
            for report in mouse::drag_reports(from, to) {
                self.write_once(&report)?;
                std::thread::sleep(Duration::from_millis(25));
            }
            return Ok(());
        }
        self.write_once(&mouse::drag_sequence(from, to))
    }

    /// One wheel notch at `(col, row)`.
    pub fn scroll(&mut self, down: bool, col: u16, row: u16) -> Result<()> {
        self.write_once(&mouse::scroll(down, col, row))
    }

    /// One wheel notch with Shift held: the sideways scroll.
    pub fn shift_scroll(&mut self, down: bool, col: u16, row: u16) -> Result<()> {
        self.write_once(&mouse::scroll_mod(mouse::MOD_SHIFT, down, col, row))
    }

    /// One horizontal wheel notch (`left` false scrolls right).
    pub fn scroll_sideways(&mut self, left: bool, col: u16, row: u16) -> Result<()> {
        self.write_once(&mouse::scroll_h(left, col, row))
    }

    /// Tells the child the terminal was resized, and resizes the
    /// emulator to match.
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.master
            .as_ref()
            .ok_or_else(|| Error::Pty("the pty has been shut down".into()))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| Error::Pty(e.to_string()))?;
        self.lock().parser.set_size(rows, cols);
        self.cols = cols;
        self.rows = rows;
        Ok(())
    }
}
