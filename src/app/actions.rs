#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    CycleFocus,
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
