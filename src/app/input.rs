use crate::app::App;
use crate::app::actions::Action;
use crate::app::state::{self, ConfirmAction, InputAction, Modal};
use crate::tmux::interface::{
    attach_to_session, attach_to_window, close_session, close_window, create_session, create_window, rename_session,
    rename_window,
};
use crossterm::event::KeyCode;

impl App {
    pub(super) fn handle_action(&mut self, action: Option<Action>) -> bool {
        let Some(action) = action else {
            return false;
        };

        match action {
            Action::Quit => return true,
            Action::Refresh => self.refresh_sessions(),
            Action::Export => self.export_sessions(),
            Action::Restore => self.restore_sessions(),
            Action::MoveUp => self.with_tree_focus(state::State::move_up),
            Action::MoveDown => self.with_tree_focus(state::State::move_down),
            Action::Select => self.with_tree_focus(state::State::select),
            Action::Expand => self.with_tree_focus(state::State::expand_selected_session),
            Action::Collapse => self.with_tree_focus(state::State::collapse_selected_session),
            Action::ToggleExpand => self.with_tree_focus(state::State::toggle_expand),
            Action::Attach => {
                if self.perform_attach() {
                    return true;
                }
            }
            Action::CreateSession | Action::CreateWindow | Action::Rename | Action::Close => self.start_action(action),
        }

        false
    }

    pub(super) fn handle_modal_key(&mut self, code: KeyCode) -> bool {
        let Some(modal) = self.state.modal.clone() else {
            return false;
        };

        match modal {
            Modal::Input { .. } => {
                self.handle_input_modal_key(modal, code);
                true
            }
            Modal::Confirm { .. } => {
                self.handle_confirm_modal_key(modal, code);
                true
            }
        }
    }

    fn with_tree_focus(&mut self, callback: impl FnOnce(&mut state::State)) {
        if matches!(self.state.focus, state::FocusRegion::Tree) {
            callback(&mut self.state);
            self.request_preview_refresh();
        }
    }

    fn handle_input_modal_key(&mut self, modal: Modal, code: KeyCode) {
        let Modal::Input { title, mut value, action } = modal else {
            return;
        };

        match code {
            KeyCode::Esc => self.state.modal = None,
            KeyCode::Backspace => {
                value.pop();
                self.state.modal = Some(Modal::Input { title, value, action });
            }
            KeyCode::Enter => {
                self.state.modal = None;
                self.submit_input_action(action, value.trim());
            }
            KeyCode::Char(character) => {
                value.push(character);
                self.state.modal = Some(Modal::Input { title, value, action });
            }
            _ => self.state.modal = Some(Modal::Input { title, value, action }),
        }
    }

    fn handle_confirm_modal_key(&mut self, modal: Modal, code: KeyCode) {
        let Modal::Confirm { title, prompt, action } = modal else {
            return;
        };

        match code {
            KeyCode::Esc | KeyCode::Char('n') => self.state.modal = None,
            KeyCode::Enter | KeyCode::Char('y') => {
                self.state.modal = None;
                self.submit_confirm_action(action);
            }
            _ => self.state.modal = Some(Modal::Confirm { title, prompt, action }),
        }
    }

    fn start_action(&mut self, action: Action) {
        match action {
            Action::CreateSession => {
                self.state.modal = Some(Modal::Input {
                    title: String::from("Create session"),
                    value: String::new(),
                    action: InputAction::CreateSession,
                });
            }
            Action::CreateWindow => {
                let Some(session_name) = self.state.selected_session_name() else {
                    self.set_error_status("No session selected");
                    return;
                };

                self.state.modal = Some(Modal::Input {
                    title: format!("Create window in {session_name}"),
                    value: String::new(),
                    action: InputAction::CreateWindow { session_name: session_name.to_string() },
                });
            }
            Action::Rename => self.open_rename_modal(),
            Action::Close => self.open_close_modal(),
            _ => {}
        }
    }

    fn perform_attach(&mut self) -> bool {
        let Some(session_name) = self.state.selected_session_name() else {
            self.set_error_status("No session selected");
            return false;
        };

        let result = if let Some(window_index) = self.state.selected_window_index() {
            attach_to_window(session_name, window_index)
        } else {
            attach_to_session(session_name)
        };

        match result {
            Ok(()) => true,
            Err(error) => {
                self.set_error_status(&format!("Attach failed: {error}"));
                false
            }
        }
    }

    fn open_rename_modal(&mut self) {
        if let Some(window_index) = self.state.selected_window_index() {
            let Some(session_name) = self.state.selected_session_name() else {
                self.set_error_status("No session selected");
                return;
            };

            self.state.modal = Some(Modal::Input {
                title: format!("Rename window {session_name}:{window_index}"),
                value: String::new(),
                action: InputAction::RenameWindow {
                    session_name: session_name.to_string(),
                    window_index: window_index.to_string(),
                },
            });
            return;
        }

        let Some(session_name) = self.state.selected_session_name() else {
            self.set_error_status("No session selected");
            return;
        };

        self.state.modal = Some(Modal::Input {
            title: format!("Rename session {session_name}"),
            value: String::new(),
            action: InputAction::RenameSession { session_name: session_name.to_string() },
        });
    }

    fn open_close_modal(&mut self) {
        if let Some(window_index) = self.state.selected_window_index() {
            let Some(session_name) = self.state.selected_session_name() else {
                self.set_error_status("No session selected");
                return;
            };

            self.state.modal = Some(Modal::Confirm {
                title: format!("Close window {session_name}:{window_index}"),
                prompt: String::from("Press y/Enter to confirm, n/Esc to cancel"),
                action: ConfirmAction::CloseWindow {
                    session_name: session_name.to_string(),
                    window_index: window_index.to_string(),
                },
            });
            return;
        }

        let Some(session_name) = self.state.selected_session_name() else {
            self.set_error_status("No session selected");
            return;
        };

        self.state.modal = Some(Modal::Confirm {
            title: format!("Close session {session_name}"),
            prompt: String::from("Press y/Enter to confirm, n/Esc to cancel"),
            action: ConfirmAction::CloseSession { session_name: session_name.to_string() },
        });
    }

    fn submit_input_action(&mut self, action: InputAction, value: &str) {
        if value.is_empty() {
            self.set_error_status("Input cannot be empty");
            return;
        }

        let result = match action {
            InputAction::CreateSession => {
                let result = create_session(value);
                if result.is_ok() {
                    self.refresh_sessions();
                    self.state.select_session_by_name(value);
                    self.set_status("Session created");
                }
                result
            }
            InputAction::CreateWindow { session_name } => {
                let result = create_window(&session_name, value);
                if result.is_ok() {
                    self.refresh_sessions();
                    self.state.select_session_by_name(&session_name);
                    self.set_status("Window created");
                }
                result
            }
            InputAction::RenameSession { session_name } => {
                let result = rename_session(&session_name, value);
                if result.is_ok() {
                    self.refresh_sessions();
                    self.state.select_session_by_name(value);
                    self.set_status("Session renamed");
                }
                result
            }
            InputAction::RenameWindow { session_name, window_index } => {
                let result = rename_window(&session_name, &window_index, value);
                if result.is_ok() {
                    self.refresh_sessions();
                    self.state.select_window_by_identity(&session_name, &window_index);
                    self.set_status("Window renamed");
                }
                result
            }
        };

        if let Err(error) = result {
            self.set_error_status(&format!("Action failed: {error}"));
        }
    }

    fn submit_confirm_action(&mut self, action: ConfirmAction) {
        let result = match action {
            ConfirmAction::CloseSession { session_name } => {
                let result = close_session(&session_name);
                if result.is_ok() {
                    self.refresh_sessions();
                    self.set_status("Session closed");
                }
                result
            }
            ConfirmAction::CloseWindow { session_name, window_index } => {
                let result = close_window(&session_name, &window_index);
                if result.is_ok() {
                    self.refresh_sessions();
                    self.state.select_session_by_name(&session_name);
                    self.set_status("Window closed");
                }
                result
            }
            ConfirmAction::OverwriteSessionExport => {
                self.perform_export();
                Ok(())
            }
            ConfirmAction::RunSessionRestore => {
                self.perform_restore_sessions();
                Ok(())
            }
        };

        if let Err(error) = result {
            self.set_error_status(&format!("Action failed: {error}"));
        }
    }

    pub(super) fn set_status(&mut self, message: &str) {
        self.state.status = Some(state::StatusLine { message: message.to_string(), is_error: false });
    }

    pub(super) fn set_error_status(&mut self, message: &str) {
        self.state.status = Some(state::StatusLine { message: message.to_string(), is_error: true });
    }
}
