use gitpoulpi_i18n::msg;

/// The two controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabDirButton {
    QuitCd,
    Terminal,
}

impl TabDirButton {
    /// The text on the tab bar.
    pub fn label(self) -> String {
        match self {
            TabDirButton::QuitCd => format!(" {} ", msg::tab_dir::QUIT_CD.get()),
            TabDirButton::Terminal => format!(" {} ", msg::tab_dir::TERMINAL.get()),
        }
    }
}
