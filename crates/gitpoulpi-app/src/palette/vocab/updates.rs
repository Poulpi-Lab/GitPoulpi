/// The palette's (and the tab's) actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatesCommand {
    /// `Help: Release notes`: opens the tab.
    ReleaseNotes,
    /// `Updates: Check for updates now`.
    CheckNow,
    /// `Updates: Download`: the release page in the browser.
    Download,
    /// `Updates: Skip this version`.
    Skip,
    /// `Updates: Update now`: download, verify and stage the release (a
    /// package-manager channel runs its command).
    UpdateNow,
    /// `Updates: Restart to update`: hand the staged release to the updater.
    Restart,
    /// `Updates: Revert to the previous version`.
    Revert,
    /// `Release notes: Close tab`.
    CloseTab,
    /// `Updates: Show update`: opens the update panel.
    ShowPanel,
    /// `Updates: Open security notice`: the link of the notice that names
    /// this version.
    SecurityNotice,
    /// `Updates: Copy update command`.
    CopyCommand,
    /// `Updates: Run update command in a terminal`.
    RunInTerminal,
}

/// Why each self-update command is unavailable now (`None`: available).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Whys {
    pub update_now: Option<&'static str>,
    pub restart: Option<&'static str>,
    pub revert: Option<&'static str>,
    /// `Updates: Copy update command`.
    pub copy: Option<&'static str>,
    /// `Updates: Run update command in a terminal`.
    pub terminal: Option<&'static str>,
    /// `Updates: Open Security Notice`.
    pub notice: Option<&'static str>,
}
