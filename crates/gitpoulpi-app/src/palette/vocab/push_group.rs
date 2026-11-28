/// What the palette needs to know about the saved or last grouped run of
/// `root`: whether there is a run to show, one paused on conflicts, whether
/// its Continue is on, and whether some target can be retried.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GroupState {
    pub run: bool,
    pub paused: bool,
    pub ready: bool,
    pub retry: bool,
}
