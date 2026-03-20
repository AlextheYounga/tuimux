use std::collections::BTreeSet;

use crate::tmux::session::{Session, Window};

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
    Input {
        title: String,
        value: String,
        action: InputAction,
    },
    Confirm {
        title: String,
        prompt: String,
        action: ConfirmAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    CreateSession,
    CreateWindow {
        session_name: String,
    },
    RenameSession {
        session_name: String,
    },
    RenameWindow {
        session_name: String,
        window_index: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    CloseSession {
        session_name: String,
    },
    CloseWindow {
        session_name: String,
        window_index: String,
    },
}

#[derive(Debug, Default)]
pub struct State {
    pub focus: FocusRegion,
    pub sessions: Vec<Session>,
    pub selected_session: Option<usize>,
    pub selected_window: Option<usize>,
    pub selection: Option<TreeSelection>,
    pub expanded_sessions: BTreeSet<String>,
    pub status: Option<StatusLine>,
    pub modal: Option<Modal>,
    pub preview: String,
    pub preview_is_error: bool,
}

impl State {
    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        let previous_selection = self.selection.clone();
        let previous_expanded = self.expanded_sessions.clone();
        self.sessions = sessions;

        if self.sessions.is_empty() {
            self.selected_session = None;
            self.selected_window = None;
            self.selection = None;
            self.expanded_sessions.clear();
            return;
        }

        if previous_expanded.is_empty() {
            self.expanded_sessions.clear();
        } else {
            self.expanded_sessions = self
                .sessions
                .iter()
                .filter_map(|session| {
                    if previous_expanded.contains(&session.name) {
                        Some(session.name.clone())
                    } else {
                        None
                    }
                })
                .collect();
        }

        if !self.restore_selection(previous_selection) {
            self.selected_session = Some(0);
            self.selected_window = None;
            self.sync_selection();
        }
    }

    pub fn move_up(&mut self) {
        let Some(current) = self.selected_session else {
            return;
        };
        if current == 0 {
            return;
        }

        self.selected_session = Some(current - 1);
        self.selected_window = None;
        self.sync_selection();
    }

    pub fn move_down(&mut self) {
        let Some(current) = self.selected_session else {
            return;
        };
        if current + 1 >= self.sessions.len() {
            return;
        }

        self.selected_session = Some(current + 1);
        self.selected_window = None;
        self.sync_selection();
    }

    pub fn select(&mut self) {
        let Some(session_index) = self.selected_session else {
            return;
        };
        let Some(session) = self.sessions.get(session_index) else {
            return;
        };

        if session.windows.is_empty() {
            return;
        }

        self.selected_window = match self.selected_window {
            Some(index) => Some((index + 1) % session.windows.len()),
            None => Some(0),
        };
        self.sync_selection();
    }

    pub fn back(&mut self) {
        self.collapse_selected_session();
    }

    pub fn toggle_expand(&mut self) {
        let Some(session_index) = self.selected_session else {
            return;
        };

        let Some(session) = self.sessions.get(session_index) else {
            return;
        };

        if self.expanded_sessions.contains(&session.name) {
            self.expanded_sessions.remove(&session.name);
            self.selected_window = None;
        } else {
            self.expanded_sessions.insert(session.name.clone());
        }

        self.sync_selection();
    }

    pub fn expand_selected_session(&mut self) {
        let Some(session_index) = self.selected_session else {
            return;
        };

        let Some(session) = self.sessions.get(session_index) else {
            return;
        };

        self.expanded_sessions.insert(session.name.clone());
        self.sync_selection();
    }

    pub fn collapse_selected_session(&mut self) {
        let Some(session_index) = self.selected_session else {
            return;
        };

        let Some(session) = self.sessions.get(session_index) else {
            return;
        };

        self.expanded_sessions.remove(&session.name);
        self.selected_window = None;
        self.sync_selection();
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusRegion::Tree => FocusRegion::Details,
            FocusRegion::Details => FocusRegion::Help,
            FocusRegion::Help | FocusRegion::Modal => FocusRegion::Tree,
        };
    }

    #[must_use]
    pub fn selected_session_name(&self) -> Option<&str> {
        self.selected_session_ref()
            .map(|session| session.name.as_str())
    }

    #[must_use]
    pub fn selected_window_index(&self) -> Option<&str> {
        self.selected_window_ref()
            .map(|window| window.index.as_str())
    }

    pub fn select_session_by_name(&mut self, session_name: &str) {
        let Some(session_index) = self
            .sessions
            .iter()
            .position(|session| session.name == session_name)
        else {
            return;
        };

        self.selected_session = Some(session_index);
        self.selected_window = None;
        self.sync_selection();
    }

    pub fn select_window_by_identity(&mut self, session_name: &str, window_index: &str) {
        let Some((session_index, session)) = self
            .sessions
            .iter()
            .enumerate()
            .find(|(_, session)| session.name == session_name)
        else {
            return;
        };

        let Some(window_pos) = session
            .windows
            .iter()
            .position(|window| window.index == window_index)
        else {
            return;
        };

        self.selected_session = Some(session_index);
        self.selected_window = Some(window_pos);
        self.sync_selection();
    }

    #[must_use]
    pub fn focus_label(&self) -> &'static str {
        match self.focus {
            FocusRegion::Tree => "tree",
            FocusRegion::Details => "details",
            FocusRegion::Help => "help",
            FocusRegion::Modal => "modal",
        }
    }

    fn restore_selection(&mut self, previous_selection: Option<TreeSelection>) -> bool {
        let Some(previous_selection) = previous_selection else {
            return false;
        };

        match previous_selection {
            TreeSelection::Window {
                session_name,
                window_index,
            } => {
                let Some((session_index, session)) = self
                    .sessions
                    .iter()
                    .enumerate()
                    .find(|(_, session)| session.name == session_name)
                else {
                    return false;
                };

                let Some(window_index_pos) = session
                    .windows
                    .iter()
                    .position(|window| window.index == window_index)
                else {
                    self.selected_session = Some(session_index);
                    self.selected_window = None;
                    self.sync_selection();
                    return true;
                };

                self.selected_session = Some(session_index);
                self.selected_window = Some(window_index_pos);
                self.sync_selection();
                true
            }
            TreeSelection::Session { name } => {
                let Some(session_index) = self
                    .sessions
                    .iter()
                    .position(|session| session.name == name)
                else {
                    return false;
                };

                self.selected_session = Some(session_index);
                self.selected_window = None;
                self.sync_selection();
                true
            }
        }
    }

    fn sync_selection(&mut self) {
        let Some(session_index) = self.selected_session else {
            self.selection = None;
            return;
        };

        let Some(session) = self.sessions.get(session_index) else {
            self.selection = None;
            return;
        };

        if !self.expanded_sessions.contains(&session.name) {
            self.selected_window = None;
        }

        if session.windows.is_empty() {
            self.selected_window = None;
            self.selection = Some(TreeSelection::Session {
                name: session.name.clone(),
            });
            return;
        }

        if let Some(window_index) = self.selected_window {
            if let Some(window) = session.windows.get(window_index) {
                self.selection = Some(TreeSelection::Window {
                    session_name: session.name.clone(),
                    window_index: window.index.clone(),
                });
                return;
            }

            self.selected_window = None;
        }

        self.selection = Some(TreeSelection::Session {
            name: session.name.clone(),
        });
    }

    #[must_use]
    pub fn selected_session_ref(&self) -> Option<&Session> {
        self.selected_session
            .and_then(|index| self.sessions.get(index))
    }

    #[must_use]
    pub fn selected_window_ref(&self) -> Option<&Window> {
        let session = self.selected_session_ref()?;
        let index = self.selected_window?;
        session.windows.get(index)
    }
}
