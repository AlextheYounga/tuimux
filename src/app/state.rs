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
    Input { title: String, value: String },
    Confirm { title: String, prompt: String },
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
}

impl State {
    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;

        if self.sessions.is_empty() {
            self.selected_session = None;
            self.selected_window = None;
            self.selection = None;
            self.expanded_sessions.clear();
            return;
        }

        let selected = self.selected_session.unwrap_or(0);
        let selected = selected.min(self.sessions.len() - 1);
        self.selected_session = Some(selected);

        self.expanded_sessions = self
            .sessions
            .iter()
            .map(|session| session.name.clone())
            .collect();

        let session = &self.sessions[selected];
        if session.windows.is_empty() {
            self.selected_window = None;
            self.selection = Some(TreeSelection::Session {
                name: session.name.clone(),
            });
            return;
        }

        let selected_window = self
            .selected_window
            .unwrap_or(0)
            .min(session.windows.len() - 1);
        self.selected_window = Some(selected_window);
        self.selection = Some(TreeSelection::Window {
            session_name: session.name.clone(),
            window_index: session.windows[selected_window].index.clone(),
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
