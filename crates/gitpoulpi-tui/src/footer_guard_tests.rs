//! Nothing is typed in the status line. A prompt is drawn inline (a Log row,
//! a Refs row, the Status list), in the small name box, or in a
//! `prompt_dialog`; the footer only names keys. This structural guard fails
//! if `footer.rs` starts hosting a text field again: it has no field editor
//! in its draw context, and none of the helpers that draw a caret line may
//! appear in it. `tests/pty_status_line_never_types.rs` is the behavioural
//! twin.

const FOOTER: &str = include_str!("event_loop/draw/footer.rs");

/// The footer's code, without its test modules.
fn footer_code() -> &'static str {
    FOOTER
        .split("#[cfg(test)]")
        .next()
        .expect("footer.rs has code")
}

#[test]
fn the_footer_draws_no_text_field() {
    let code = footer_code();
    for forbidden in [
        "MainField",
        "field_editor",
        "FieldEditor",
        "text_field::",
        "edit_line",
        "text_input",
        "caret",
    ] {
        assert!(
            !code.contains(forbidden),
            "footer.rs mentions `{forbidden}`: nothing is typed in the status line; \
             draw the prompt inline or as a prompt_dialog instead"
        );
    }
}

#[test]
fn the_guard_reads_the_real_footer() {
    // A guard over an empty or moved file would pass for nothing.
    assert!(
        footer_code().contains("fn draw_footer"),
        "footer.rs no longer holds draw_footer: update the guard's path"
    );
}
