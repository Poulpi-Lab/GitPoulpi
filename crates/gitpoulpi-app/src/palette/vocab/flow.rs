/// A Flow command. `Start(i)` is the i-th topic type of the workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowCmd {
    /// Opens the `Flow ▾` dropdown.
    Menu,
    /// The Start form with the type chosen in a dropdown.
    StartAny,
    /// The Start form of one topic type.
    Start(u8),
    /// New issue (host-driven flows): the form with the type chosen in a
    /// dropdown, or of one topic type.
    NewIssueAny,
    NewIssue(u8),
    /// Take over a branch the host or a bot made.
    PickUp,
    Finish,
    Sync,
    Promote,
    SetParent,
    /// Grouped sidebar branches / the plain trees.
    ToggleGrouping,
    /// Settings > Repository > Workflow.
    Settings,
    /// Continue or abort an action a conflict stopped.
    Resume,
}
