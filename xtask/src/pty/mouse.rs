//! SGR 1006 mouse-report encoding.
//!
//! The TUI turns mouse capture on (`EnableMouseCapture` emits, among
//! others, the SGR-encoding mode `1006h`), so writing these reports down
//! the pty reaches the same `Event::Mouse` dispatch and hit-testing a real
//! terminal would produce.

/// Which mouse button an event carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Middle,
    Right,
}

impl Button {
    /// The protocol's low two bits.
    fn code(self) -> u16 {
        match self {
            Button::Left => 0,
            Button::Middle => 1,
            Button::Right => 2,
        }
    }
}

/// `CSI < <button> ; <col> ; <row> <final>`, with the coordinates moved
/// from this crate's zero-based convention to the protocol's one-based
/// one. Every public function here goes through this, so the conversion
/// exists once.
fn report(button: u16, col: u16, row: u16, final_byte: char) -> String {
    format!("\x1b[<{button};{};{}{final_byte}", col + 1, row + 1)
}

/// Modifier bits of the protocol's button code.
pub const MOD_SHIFT: u16 = 4;
pub const MOD_CTRL: u16 = 16;
/// Alt (xterm calls it Meta in the mouse encoding).
pub const MOD_ALT: u16 = 8;

/// A press of the left button with modifier bits (`MOD_SHIFT`, `MOD_CTRL`).
pub fn press_mod(mods: u16, col: u16, row: u16) -> String {
    report(Button::Left.code() + mods, col, row, 'M')
}

/// The matching release.
pub fn release_mod(mods: u16, col: u16, row: u16) -> String {
    report(Button::Left.code() + mods, col, row, 'm')
}

/// A press report for `button` at zero-based `(col, row)`.
pub fn press(button: Button, col: u16, row: u16) -> String {
    report(button.code(), col, row, 'M')
}

/// The matching release report. SGR 1006's whole advantage over the
/// older encodings is this lowercase `m`: the release names its button,
/// so a press and a release can never be confused.
pub fn release(button: Button, col: u16, row: u16) -> String {
    report(button.code(), col, row, 'm')
}

/// Bit 5 (32) is "this is motion". With no button held the low two bits
/// are 3 ("no button"), giving 35.
pub fn motion(col: u16, row: u16) -> String {
    report(35, col, row, 'M')
}

/// Pointer motion with `button` held down.
pub fn drag_motion(button: Button, col: u16, row: u16) -> String {
    report(32 + button.code(), col, row, 'M')
}

/// A wheel notch; `down` false means scrolling up. Wheel events are
/// reported as presses of buttons 64/65 and have no release.
pub fn scroll(down: bool, col: u16, row: u16) -> String {
    report(if down { 65 } else { 64 }, col, row, 'M')
}

/// A wheel notch with modifier bits (`MOD_SHIFT`, `MOD_CTRL`): Shift+wheel
/// is how most terminals report a sideways scroll.
pub fn scroll_mod(mods: u16, down: bool, col: u16, row: u16) -> String {
    report(if down { 65 } else { 64 } + mods, col, row, 'M')
}

/// A horizontal wheel notch (buttons 66 and 67): `left` false scrolls right.
pub fn scroll_h(left: bool, col: u16, row: u16) -> String {
    report(if left { 66 } else { 67 }, col, row, 'M')
}

/// The whole byte string for a left-button drag from `from` to `to`:
/// press, one motion report per cell crossed, release.
///
/// A pure function rather than a loop inside `Terminal::drag`, so what goes
/// on the wire can be asserted without a child process (and so on Windows
/// too). One report per cell, because a terminal reports position in
/// cells; the intermediate reports must be there because a handler that
/// tracks the pointer sees nothing from a bare press-then-release.
pub fn drag_sequence(from: (u16, u16), to: (u16, u16)) -> String {
    let mut sequence = press(Button::Left, from.0, from.1);
    let steps = from.0.abs_diff(to.0).max(from.1.abs_diff(to.1)).max(1);
    for step in 1..=steps {
        let col = interpolate(from.0, to.0, step, steps);
        let row = interpolate(from.1, to.1, step, steps);
        sequence.push_str(&drag_motion(Button::Left, col, row));
    }
    sequence.push_str(&release(Button::Left, to.0, to.1));
    sequence
}

fn interpolate(from: u16, to: u16, step: u16, steps: u16) -> u16 {
    let from = i32::from(from);
    let to = i32::from(to);
    let value = from + (to - from) * i32::from(step) / i32::from(steps);
    value.clamp(0, i32::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The API is zero-based, like `ratatui::Rect` and every hit target
    /// in the TUI; the wire protocol is one-based. Getting that wrong
    /// puts every click one row and one column off - the exact class of
    /// bug `row_to_index`'s `top_offset` produced by hand.
    #[test]
    fn the_wire_is_one_based_while_the_api_is_zero_based() {
        assert_eq!(press(Button::Left, 0, 0), "\x1b[<0;1;1M");
        assert_eq!(release(Button::Left, 11, 4), "\x1b[<0;12;5m");
    }

    #[test]
    fn buttons_use_their_protocol_numbers() {
        assert_eq!(press(Button::Middle, 0, 0), "\x1b[<1;1;1M");
        assert_eq!(press(Button::Right, 0, 0), "\x1b[<2;1;1M");
    }

    /// Bit 5 (32) means "motion"; with no button held the low two bits
    /// are 3, giving 35. That is the bit `ydotool mousemove --absolute`
    /// never produced, which made every hover affordance in the TUI look
    /// broken when it was fine.
    #[test]
    fn motion_sets_the_motion_bit() {
        assert_eq!(motion(9, 2), "\x1b[<35;10;3M");
        assert_eq!(drag_motion(Button::Left, 9, 2), "\x1b[<32;10;3M");
    }

    #[test]
    fn a_modified_wheel_adds_the_modifier_bits_and_a_horizontal_one_is_66_and_67() {
        assert_eq!(scroll_mod(MOD_SHIFT, true, 0, 0), "\x1b[<69;1;1M");
        assert_eq!(scroll_mod(MOD_SHIFT, false, 0, 0), "\x1b[<68;1;1M");
        assert_eq!(scroll_h(true, 0, 0), "\x1b[<66;1;1M");
        assert_eq!(scroll_h(false, 0, 0), "\x1b[<67;1;1M");
    }

    #[test]
    fn the_wheel_is_buttons_64_and_65() {
        assert_eq!(scroll(false, 0, 0), "\x1b[<64;1;1M");
        assert_eq!(scroll(true, 0, 0), "\x1b[<65;1;1M");
    }

    /// A drag is press, one motion per cell crossed, release. This used
    /// to be checked by echoing the bytes back through a `/bin/sh` child
    /// and reading them off the screen, which was both roundabout and
    /// Unix-only; the sequence is a pure function, so assert on it.
    #[test]
    fn a_drag_emits_one_motion_report_per_cell_crossed() {
        assert_eq!(
            drag_sequence((10, 3), (13, 3)),
            concat!(
                "\x1b[<0;11;4M",  // press at 10,3
                "\x1b[<32;12;4M", // 11,3
                "\x1b[<32;13;4M", // 12,3
                "\x1b[<32;14;4M", // 13,3
                "\x1b[<0;14;4m",  // release at 13,3
            )
        );
    }

    /// A drag that goes nowhere still has to be a press, a motion and a
    /// release rather than an empty string or a division by zero - the
    /// `.max(1)` in `drag_sequence` is what stops the step count being 0.
    #[test]
    fn a_zero_length_drag_is_still_a_drag() {
        assert_eq!(
            drag_sequence((4, 4), (4, 4)),
            "\x1b[<0;5;5M\x1b[<32;5;5M\x1b[<0;5;5m"
        );
    }

    /// Diagonal drags walk both axes together, one report per step of the
    /// longer axis.
    #[test]
    fn a_diagonal_drag_interpolates_both_axes() {
        assert_eq!(
            drag_sequence((0, 0), (2, 4)),
            concat!(
                "\x1b[<0;1;1M",
                "\x1b[<32;1;2M",
                "\x1b[<32;2;3M",
                "\x1b[<32;2;4M",
                "\x1b[<32;3;5M",
                "\x1b[<0;3;5m",
            )
        );
    }
}
