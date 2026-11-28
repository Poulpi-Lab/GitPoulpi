//! The fonts the window ships with. They are compiled into the binary and registered at startup, so
//! the look never depends on what the machine has installed. Licences: `desktop/THIRD-PARTY-NOTICES.md`.

use std::borrow::Cow;

use gpui_kit::App;

/// Family name of the UI font (Adwaita Sans, SIL OFL 1.1).
pub const UI_FAMILY: &str = "Adwaita Sans";
/// Family name of the monospace font (JetBrains Mono, SIL OFL 1.1).
pub const MONO_FAMILY: &str = "JetBrains Mono";

const FILES: &[&[u8]] = &[
    include_bytes!("../assets/fonts/AdwaitaSans-Regular.ttf"),
    include_bytes!("../assets/fonts/AdwaitaSans-Italic.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-BoldItalic.ttf"),
];

/// Registers every bundled font with the text system. Call once, before the first window.
pub fn register(cx: &App) -> anyhow::Result<()> {
    let fonts: Vec<Cow<'static, [u8]>> = FILES.iter().map(|bytes| Cow::Borrowed(*bytes)).collect();
    cx.text_system().add_fonts(fonts)?;
    Ok(())
}
