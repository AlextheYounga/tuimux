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
    Session { name: String },
    Window { session_name: String, window_index: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLine {
    pub message: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    Input { title: String, value: String, action: InputAction },
    Confirm { title: String, prompt: String, action: ConfirmAction },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    CreateSession,
    CreateWindow { session_name: String },
    RenameSession { session_name: String },
    RenameWindow { session_name: String, window_index: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    CloseSession { session_name: String },
    CloseWindow { session_name: String, window_index: String },
    OverwriteSessionExport,
    RunSessionRestore,
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
    pub filter_mode: bool,
    pub filter_query: String,
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
            self.expanded_sessions =
                self.sessions
                    .iter()
                    .filter_map(|session| {
                        if previous_expanded.contains(&session.name) { Some(session.name.clone()) } else { None }
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
        let rows = self.tree_rows();
        let Some(current) = self.selected_row_index(&rows) else {
            return;
        };
        if rows.is_empty() {
            return;
        }

        let next = if current == 0 { rows.len() - 1 } else { current - 1 };
        self.apply_row_selection(&rows[next]);
    }

    pub fn move_down(&mut self) {
        let rows = self.tree_rows();
        let Some(current) = self.selected_row_index(&rows) else {
            return;
        };
        if rows.is_empty() {
            return;
        }

        let next = if current + 1 >= rows.len() { 0 } else { current + 1 };
        self.apply_row_selection(&rows[next]);
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

        self.selected_window = Some(self.selected_window.map_or(0, |index| (index + 1) % session.windows.len()));
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
        self.selected_session_ref().map(|session| session.name.as_str())
    }

    #[must_use]
    pub fn selected_window_index(&self) -> Option<&str> {
        self.selected_window_ref().map(|window| window.index.as_str())
    }

    pub fn select_session_by_name(&mut self, session_name: &str) {
        let Some(session_index) = self.sessions.iter().position(|session| session.name == session_name) else {
            return;
        };

        self.selected_session = Some(session_index);
        self.selected_window = None;
        self.sync_selection();
    }

    pub fn select_window_by_identity(&mut self, session_name: &str, window_index: &str) {
        let Some((session_index, session)) =
            self.sessions.iter().enumerate().find(|(_, session)| session.name == session_name)
        else {
            return;
        };

        let Some(window_pos) = session.windows.iter().position(|window| window.index == window_index) else {
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

    pub fn start_filter(&mut self) {
        self.filter_mode = true;
    }

    pub fn stop_filter(&mut self) {
        self.filter_mode = false;
    }

    pub fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.normalize_selection_after_filter_change();
    }

    pub fn append_filter_char(&mut self, character: char) {
        self.filter_query.push(character);
        self.normalize_selection_after_filter_change();
    }

    pub fn pop_filter_char(&mut self) {
        self.filter_query.pop();
        self.normalize_selection_after_filter_change();
    }

    #[must_use]
    pub fn is_filter_active(&self) -> bool {
        !self.filter_query.trim().is_empty()
    }

    fn restore_selection(&mut self, previous_selection: Option<TreeSelection>) -> bool {
        let Some(previous_selection) = previous_selection else {
            return false;
        };

        match previous_selection {
            TreeSelection::Window { session_name, window_index } => {
                let Some((session_index, session)) =
                    self.sessions.iter().enumerate().find(|(_, session)| session.name == session_name)
                else {
                    return false;
                };

                let Some(window_index_pos) = session.windows.iter().position(|window| window.index == window_index)
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
                let Some(session_index) = self.sessions.iter().position(|session| session.name == name) else {
                    return false;
                };

                self.selected_session = Some(session_index);
                self.selected_window = None;
                self.sync_selection();
                true
            }
        }
    }

    fn tree_rows(&self) -> Vec<TreeRow> {
        let mut rows = Vec::new();
        let show_windows = self.is_filter_active();

        for (session_index, session) in self.sessions.iter().enumerate() {
            if !self.session_matches_filter(session) {
                continue;
            }

            rows.push(TreeRow { session_index, window_index: None });

            if !show_windows && !self.expanded_sessions.contains(&session.name) {
                continue;
            }

            for (window_index, window) in session.windows.iter().enumerate() {
                if self.is_filter_active() && !self.window_matches_filter(window) {
                    continue;
                }
                rows.push(TreeRow { session_index, window_index: Some(window_index) });
            }
        }

        rows
    }

    fn selected_row_index(&self, rows: &[TreeRow]) -> Option<usize> {
        rows.iter().position(|row| {
            self.selected_session == Some(row.session_index) && self.selected_window == row.window_index
        })
    }

    fn normalize_selection_after_filter_change(&mut self) {
        let rows = self.tree_rows();
        if rows.is_empty() {
            self.selected_session = None;
            self.selected_window = None;
            self.selection = None;
            return;
        }

        if self.selected_row_index(&rows).is_none() {
            self.apply_row_selection(&rows[0]);
            return;
        }

        self.sync_selection();
    }

    fn session_matches_filter(&self, session: &Session) -> bool {
        let query = self.filter_query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }

        session.name.to_lowercase().contains(&query)
            || session.windows.iter().any(|window| self.window_matches_filter(window))
    }

    fn window_matches_filter(&self, window: &Window) -> bool {
        let query = self.filter_query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }

        window.name.to_lowercase().contains(&query) || window.index.to_lowercase().contains(&query)
    }

    fn apply_row_selection(&mut self, row: &TreeRow) {
        self.selected_session = Some(row.session_index);
        self.selected_window = row.window_index;
        self.sync_selection();
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
            self.selection = Some(TreeSelection::Session { name: session.name.clone() });
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

        self.selection = Some(TreeSelection::Session { name: session.name.clone() });
    }

    #[must_use]
    pub fn selected_session_ref(&self) -> Option<&Session> {
        self.selected_session.and_then(|index| self.sessions.get(index))
    }

    #[must_use]
    pub fn selected_window_ref(&self) -> Option<&Window> {
        let session = self.selected_session_ref()?;
        let index = self.selected_window?;
        session.windows.get(index)
    }
}

#[derive(Debug, Clone, Copy)]
struct TreeRow {
    session_index: usize,
    window_index: Option<usize>,
}
