#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    StartFilter,
    Refresh,
    Export,
    Restore,
    MoveUp,
    MoveDown,
    Select,
    Expand,
    Collapse,
    ToggleExpand,
    Attach,
    CreateSession,
    CreateWindow,
    Rename,
    Close,
    Quit,
}
