/// What the toolbar and the palette need per direction, as plain data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Avail {
    Ready,
    /// Nothing recorded to undo/redo (or one is running).
    #[default]
    Nothing,
    /// Something is recorded but the repository moved under it.
    Blocked,
    /// Blocked for good: the entry can be forgotten (dropped from the
    /// journal) to reach the ones under it.
    Stuck,
    /// `general.undo_enabled` is off: the buttons and palette rows are
    /// not shown at all.
    Off,
}
