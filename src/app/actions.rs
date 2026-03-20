#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Refresh,
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
