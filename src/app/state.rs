use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusRegion {
    #[default]
    Tree,
    Details,
    Help,
    Modal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeSelection {
    Session {
        name: String,
    },
    Window {
        session_name: String,
        window_index: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLine {
    pub message: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    Input { title: String, value: String },
    Confirm { title: String, prompt: String },
}

#[derive(Debug, Default)]
pub struct State {
    pub focus: FocusRegion,
    pub selection: Option<TreeSelection>,
    pub expanded_sessions: BTreeSet<String>,
    pub status: Option<StatusLine>,
    pub modal: Option<Modal>,
}
