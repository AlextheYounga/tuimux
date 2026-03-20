#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Refresh,
    MoveUp,
    MoveDown,
    Select,
    Back,
    Attach,
    CreateSession,
    CreateWindow,
    Rename,
    Close,
    Quit,
}
