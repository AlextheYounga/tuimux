use serde::{Deserialize, Serialize};

/// Represents a tmux window that has one or more panes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Window {
    /// Index of the window.
    pub index: String,
    /// Name of the window.
    pub name: String,
    /// Tmux layout string describing the window structure.
    pub layout: String,
    /// Active pane working directory.
    pub active_pane_path: String,
}

/// Represents a tmux session that has one or more windows.
#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    /// Name of the session.
    pub name: String,
    /// Default working directory for new panes.
    pub work_dir: String,
    /// List of windows inside the session.
    pub windows: Vec<Window>,
}
