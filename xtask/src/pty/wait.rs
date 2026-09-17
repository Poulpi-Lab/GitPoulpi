use super::*;

/// Multiplies every wait deadline by this factor (a float, default 1), so a
/// slow or oversubscribed machine can be given more room without editing a
/// test: `GITPOULPI_PTY_TIME_SCALE=3 cargo test --workspace`. Deadlines only
/// bound a *failing* wait; a passing one returns the moment its predicate
/// holds, so scaling costs nothing on the happy path.
pub const TIME_SCALE_ENV: &str = "GITPOULPI_PTY_TIME_SCALE";

/// How long the grid must hold still before a matched wait is believed on
/// Windows (see `Terminal::after_frame_settled`).
const FRAME_QUIET: Duration = Duration::from_millis(120);

/// `Terminal::screen` on Windows: how long the grid must hold still, and the
/// most it waits for that.
const SETTLE_QUIET: Duration = Duration::from_millis(80);
const SETTLE_CAP: Duration = Duration::from_millis(400);

/// `timeout` after the [`TIME_SCALE_ENV`] factor (ignored when unparsable,
/// not positive, or absurd).
pub fn scaled(timeout: Duration) -> Duration {
    static SCALE: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    let scale = *SCALE.get_or_init(|| {
        std::env::var(TIME_SCALE_ENV)
            .ok()
            .and_then(|v| v.trim().parse::<f64>().ok())
            .filter(|v| v.is_finite() && *v > 0.0 && *v <= 100.0)
            .unwrap_or(1.0)
    });
    timeout.mul_f64(scale)
}

impl Terminal {
    /// Performs `action` (a click, a key) and waits for `predicate`; when the
    /// screen has not answered within `attempt`, performs the action again,
    /// until `timeout` in all.
    ///
    /// For an *idempotent* gesture whose target can move under the pointer
    /// or whose input can be swallowed while the UI is not ready: a
    /// right-click on a list row that then reflows because the data behind
    /// it arrived late, a click that lands on a modal still on its way.
    /// `action` is handed the terminal and runs again from scratch each
    /// time, so it should locate its target afresh (`wait_for_text`) rather
    /// than reuse a coordinate.
    ///
    /// **Not for toggles** (a second click on a checkbox undoes the first)
    /// and not for anything that mutates: if the first attempt did land and
    /// is merely slow to show, a repeat would do it twice. Wait for a quiet
    /// screen instead. A test that needs this is also worth a thought about
    /// whether the product should have been ready sooner.
    pub fn act_until(
        &mut self,
        what: &str,
        attempt: Duration,
        timeout: Duration,
        mut action: impl FnMut(&mut Terminal) -> Result<()>,
        mut predicate: impl FnMut(&Screen) -> bool,
    ) -> Result<Screen> {
        let total = scaled(timeout);
        let deadline = Instant::now() + total;
        let mut attempts = 0u32;
        loop {
            action(self)?;
            attempts += 1;
            match self.wait_until_described(attempt, what, &mut predicate) {
                Ok(screen) => return Ok(screen),
                Err(Error::Timeout { .. } | Error::NoOutput { .. })
                    if Instant::now() < deadline => {}
                // Say what the whole wait was, not just the last attempt.
                Err(Error::Timeout {
                    bytes,
                    screen,
                    what,
                    ..
                }) => {
                    return Err(Error::Timeout {
                        what: format!("{what} (after {attempts} attempts)"),
                        waited: total,
                        bytes,
                        screen,
                    });
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Polls until `predicate` accepts the screen, or `timeout` elapses.
    ///
    /// There is no duration to guess at: the assertion *is* the wait, and a
    /// slow machine costs latency rather than a wrong answer.
    ///
    /// # The predicate must describe the *whole* state you are waiting for
    ///
    /// This is the one contract of this API. **A terminal has no frame
    /// boundaries.** The program paints a frame with many writes, the pty
    /// delivers them in whatever chunks the kernel likes, and a snapshot
    /// between two of them shows *half* the old frame and half the new
    /// one. The driver cannot fix that: no marker in the byte stream says
    /// "frame ends here", and inventing one from timing would bring back
    /// the guesswork `wait_until` removes.
    ///
    /// A predicate that checks *part* of what you expect can be satisfied
    /// by a half-painted frame, and the `Screen` you get back is then that
    /// frame.
    ///
    /// ```ignore
    /// // WRONG - the predicate holds on a frame where the divider has
    /// // been repainted but the highlight has not yet been written.
    /// let s = term.wait_until(t, |s| s.find("Log").unwrap().0 > divider)?;
    /// assert_eq!(s.cell(moved, row).unwrap().bg, ACCENT);
    ///
    /// // RIGHT - a torn frame simply fails the predicate, and the poll
    /// // comes round again once the rest of it has arrived.
    /// term.wait_for("the divider to move with its highlight", t, |s| {
    ///     matches!(s.find("Log"), Some((moved, _))
    ///         if moved > divider && lit(s, moved, row))
    /// })?;
    /// ```
    ///
    /// So: put everything in the predicate, and keep post-wait assertions
    /// to things the predicate already guaranteed.
    pub fn wait_until(
        &self,
        timeout: Duration,
        mut predicate: impl FnMut(&Screen) -> bool,
    ) -> Result<Screen> {
        self.wait_until_described(timeout, "the expected screen", &mut predicate)
    }

    /// `wait_until` with a name for what is being waited for, which is what
    /// the timeout message says: "timed out waiting for the divider to move
    /// with its highlight" plus the screen dump is a diagnosis, not a puzzle.
    pub fn wait_for(
        &self,
        what: &str,
        timeout: Duration,
        mut predicate: impl FnMut(&Screen) -> bool,
    ) -> Result<Screen> {
        self.wait_until_described(timeout, what, &mut predicate)
    }

    fn wait_until_described(
        &self,
        timeout: Duration,
        what: &str,
        predicate: &mut dyn FnMut(&Screen) -> bool,
    ) -> Result<Screen> {
        let timeout = scaled(timeout);
        let mut deadline = Instant::now() + timeout;
        let mut repainted = false;
        loop {
            let screen = self.screen_now();
            if predicate(&screen) {
                return Ok(self.after_frame_settled(screen, predicate));
            }
            // Checked *after* the predicate: a program that draws and then
            // exits on purpose should still satisfy a wait for what it
            // drew. Checked *before* the deadline so a program that died
            // fails now rather than in a minute.
            if let Some(status) = self.child_status() {
                return Err(Error::ChildExited {
                    what: what.to_string(),
                    status,
                    bytes: self.bytes_drawn(),
                    screen: screen.text(),
                });
            }
            // (Only for a wait of seconds: a short deadline is a driver
            // self-test counting bytes, which a resize would add to.)
            if Instant::now() >= deadline
                && cfg!(windows)
                && !repainted
                && timeout >= Duration::from_secs(5)
                && self.bytes_drawn() > 0
            {
                // The grid may be what is wrong, not the program: output a
                // child writes straight to the console (a credential helper's
                // stderr) lands on the ConPTY's screen without passing the
                // TUI, which then never repaints those cells. A resize makes
                // the TUI redraw everything; give the predicate one more
                // look at the repainted screen before calling it a timeout.
                repainted = true;
                self.repaint();
                deadline = Instant::now() + Duration::from_secs(2);
                continue;
            }
            if Instant::now() >= deadline {
                let bytes = self.bytes_drawn();
                // "Nothing at all" and "the wrong thing" are different
                // diagnoses and must not share a message: the first is
                // about the program or how it was launched, the second is
                // about the assertion.
                return Err(if bytes == 0 {
                    Error::NoOutput {
                        what: what.to_string(),
                        waited: timeout,
                    }
                } else {
                    Error::Timeout {
                        what: what.to_string(),
                        waited: timeout,
                        bytes,
                        screen: screen.text(),
                    }
                });
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }

    /// On Windows, the screen a predicate just matched may be half a frame:
    /// ConPTY re-renders the child's frame into its own buffer and forwards
    /// it in pieces, so a menu can be on screen with only its first rows
    /// drawn, and the assertions that follow the wait (`find` of the later
    /// rows) fail on a state that is gone a few milliseconds later. So the
    /// match is confirmed after the grid has been quiet for
    /// [`FRAME_QUIET`]. If the settled screen no longer matches (the match
    /// was a transient one), the matching screen is returned as before.
    /// A no-op elsewhere: a unix pty hands over what the child wrote.
    fn after_frame_settled(
        &self,
        matched: Screen,
        predicate: &mut dyn FnMut(&Screen) -> bool,
    ) -> Screen {
        if !cfg!(windows) {
            return matched;
        }
        let cap = Instant::now() + Duration::from_millis(1500);
        let mut last = matched.text();
        let mut quiet_since = Instant::now();
        let mut now = matched.clone();
        while quiet_since.elapsed() < FRAME_QUIET && Instant::now() < cap {
            std::thread::sleep(POLL_INTERVAL);
            now = self.screen_now();
            let text = now.text();
            if text != last {
                last = text;
                quiet_since = Instant::now();
            }
        }
        if predicate(&now) {
            now
        } else {
            matched
        }
    }

    /// The screen after the grid has been quiet for a moment (at most
    /// [`SETTLE_CAP`] of waiting: a spinner never goes quiet).
    pub(super) fn settled_screen(&self) -> Screen {
        let cap = Instant::now() + SETTLE_CAP;
        let mut last = self.screen_now();
        let mut quiet_since = Instant::now();
        while quiet_since.elapsed() < SETTLE_QUIET && Instant::now() < cap {
            std::thread::sleep(POLL_INTERVAL);
            let now = self.screen_now();
            if now.text() != last.text() {
                last = now;
                quiet_since = Instant::now();
            }
        }
        self.screen_now()
    }

    /// Makes the child redraw the whole screen: one column wider, then back.
    /// The cell grid follows each size, as `resize` does, but the terminal's
    /// recorded size is left alone: it ends where it started.
    fn repaint(&self) {
        let Some(master) = self.master.as_ref() else {
            return;
        };
        for (cols, rows) in [(self.cols + 1, self.rows), (self.cols, self.rows)] {
            let size = portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            };
            if master.resize(size).is_ok() {
                self.lock().parser.set_size(rows, cols);
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    /// Waits until the child has written *anything*.
    ///
    /// The first thing a test should wait for: it separates "the program is
    /// slow" from "the program never started".
    pub fn wait_for_first_output(&self, timeout: Duration) -> Result<()> {
        self.wait_until_described(timeout, "the child's first output", &mut |_| {
            self.bytes_drawn() > 0
        })?;
        Ok(())
    }

    /// Waits for `needle` to appear and returns where it is, ready to be
    /// handed straight to `click`.
    pub fn wait_for_text(&self, needle: &str, timeout: Duration) -> Result<(u16, u16)> {
        let screen =
            self.wait_until_described(timeout, &format!("{needle:?} on screen"), &mut |screen| {
                screen.contains(needle)
            })?;
        Ok(screen.find(needle).expect("predicate just matched"))
    }

    /// Like [`wait_for_text`](Self::wait_for_text), but returns only once
    /// `needle` has been drawn at the *same position* for `settle`, and
    /// the first match is the one that counts.
    ///
    /// Use it to find a click target in a pane whose contents arrive in
    /// stages (a commit's details, a list filled by the daemon): the text
    /// can show up at a first position and be pushed down a row when the
    /// rest of the pane lands, and a click at the first position then hits
    /// whatever moved there. `settle` is real time, not scaled: it is a
    /// quiet period, and it cannot prove nothing more is coming, so pair it
    /// with [`act_until`](Self::act_until) when the gesture can be repeated.
    pub fn wait_for_settled_text(
        &self,
        needle: &str,
        settle: Duration,
        timeout: Duration,
    ) -> Result<(u16, u16)> {
        let mut seen: Option<((u16, u16), Instant)> = None;
        self.wait_until_described(
            timeout,
            &format!("{needle:?} to hold still"),
            &mut |screen| match (screen.find(needle), seen) {
                (Some(pos), Some((was, since))) if pos == was => since.elapsed() >= settle,
                (Some(pos), _) => {
                    seen = Some((pos, Instant::now()));
                    false
                }
                (None, _) => {
                    seen = None;
                    false
                }
            },
        )?;
        Ok(seen.expect("predicate just matched").0)
    }

    /// Waits for `needle` to *stop* being on screen.
    pub fn wait_for_text_to_go(&self, needle: &str, timeout: Duration) -> Result<()> {
        self.wait_until_described(
            timeout,
            &format!("{needle:?} to disappear"),
            &mut |screen| !screen.contains(needle),
        )?;
        Ok(())
    }

    /// Waits until the child is up: alternate screen entered and mouse
    /// capture on.
    /// Waits for [`mouse_capture_enabled`](Self::mouse_capture_enabled),
    /// and inherits its portability caveat - read that first. Tests
    /// should wait on drawn content instead.
    pub fn wait_for_mouse_capture(&self, timeout: Duration) -> Result<()> {
        self.wait_until_described(timeout, "the child to enable mouse capture", &mut |_| {
            self.mouse_capture_enabled()
        })?;
        Ok(())
    }

    /// Waits until the *screen* has not changed for `stable_for`.
    ///
    /// The blunt instrument: prefer `wait_until` with a real predicate.
    /// Use this when the interesting change has no name, typically while
    /// exploring a screen you cannot yet assert about.
    ///
    /// It watches the grid, not the byte stream: "no bytes for N ms" never
    /// fires against this TUI, which redraws on a tick and never goes quiet.
    pub fn wait_for_stable(&self, stable_for: Duration, timeout: Duration) -> Result<Screen> {
        let timeout = scaled(timeout);
        let deadline = Instant::now() + timeout;
        let mut last = self.screen_now().text();
        let mut unchanged_since = Instant::now();
        loop {
            std::thread::sleep(POLL_INTERVAL);
            let now = self.screen_now().text();
            if now == last {
                if unchanged_since.elapsed() >= stable_for {
                    return Ok(self.screen_now());
                }
            } else {
                last = now;
                unchanged_since = Instant::now();
            }
            if Instant::now() >= deadline {
                return Err(Error::Timeout {
                    what: format!("{stable_for:?} with an unchanging screen"),
                    waited: timeout,
                    bytes: self.bytes_drawn(),
                    screen: self.screen_now().text(),
                });
            }
        }
    }
}
