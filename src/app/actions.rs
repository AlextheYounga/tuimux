#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    CycleFocus,
    Refresh,
    MoveUp,
    MoveDown,
    Select,
    Back,
    ToggleExpand,
    Attach,
    CreateSession,
    CreateWindow,
    Rename,
    Close,
    Quit,
}
