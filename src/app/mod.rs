pub mod actions;
pub mod state;

use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crate::app::actions::Action;
use crate::app::state::{ConfirmAction, InputAction, Modal, State};
use crate::tmux::interface::{
    attach_to_session, attach_to_window, close_session, close_window, create_session,
    create_window, get_session, list_active_sessions, rename_session, rename_window,
};
use crate::tmux::session::Session;
use crate::ui;

#[derive(Debug, Default)]
pub struct App {
    pub state: State,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs the application coordinator.
    ///
    /// # Errors
    /// Returns an error when the terminal runtime or event loop fails.
    pub fn run(&mut self) -> Result<()> {
        self.refresh_sessions();

        let mut terminal = Self::init_terminal()?;
        let run_result = self.run_loop(&mut terminal);
        let restore_result = Self::restore_terminal(&mut terminal);

        restore_result?;
        run_result
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        let input_poll = Duration::from_millis(200);
        let refresh_interval = Duration::from_secs(2);
        let mut last_refresh = Instant::now();

        loop {
            terminal.draw(|frame| ui::render(frame, &self.state))?;

            if event::poll(input_poll)?
                && let Event::Key(key_event) = event::read()?
                && key_event.kind == KeyEventKind::Press
            {
                if self.handle_modal_key(key_event.code) {
                    continue;
                }

                let action = Self::action_from_key(key_event.code, key_event.modifiers);
                if self.handle_action(action) {
                    break;
                }
            }

            if last_refresh.elapsed() >= refresh_interval {
                self.refresh_sessions();
                last_refresh = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_action(&mut self, action: Option<Action>) -> bool {
        let Some(action) = action else {
            return false;
        };

        match action {
            Action::Quit => return true,
            Action::Refresh => self.refresh_sessions(),
            Action::MoveUp => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.move_up();
                }
            }
            Action::MoveDown => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.move_down();
                }
            }
            Action::Select => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.select();
                }
            }
            Action::Back => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.back();
                }
            }
            Action::ToggleExpand => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.toggle_expand();
                }
            }
            Action::CycleFocus => {
                self.state.cycle_focus();
                self.state.status = Some(state::StatusLine {
                    message: format!("Focus: {}", self.state.focus_label()),
                    is_error: false,
                });
            }
            Action::Attach
            | Action::CreateSession
            | Action::CreateWindow
            | Action::Rename
            | Action::Close => self.start_action(action),
        }

        false
    }

    fn handle_modal_key(&mut self, code: KeyCode) -> bool {
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

    fn handle_input_modal_key(&mut self, modal: Modal, code: KeyCode) {
        let Modal::Input {
            title,
            mut value,
            action,
        } = modal
        else {
            return;
        };

        match code {
            KeyCode::Esc => {
                self.state.modal = None;
            }
            KeyCode::Backspace => {
                value.pop();
                self.state.modal = Some(Modal::Input {
                    title,
                    value,
                    action,
                });
            }
            KeyCode::Enter => {
                self.state.modal = None;
                self.submit_input_action(action, value.trim());
            }
            KeyCode::Char(character) => {
                value.push(character);
                self.state.modal = Some(Modal::Input {
                    title,
                    value,
                    action,
                });
            }
            _ => {
                self.state.modal = Some(Modal::Input {
                    title,
                    value,
                    action,
                });
            }
        }
    }

    fn handle_confirm_modal_key(&mut self, modal: Modal, code: KeyCode) {
        let Modal::Confirm {
            title,
            prompt,
            action,
        } = modal
        else {
            return;
        };

        match code {
            KeyCode::Esc | KeyCode::Char('n') => {
                self.state.modal = None;
            }
            KeyCode::Enter | KeyCode::Char('y') => {
                self.state.modal = None;
                self.submit_confirm_action(action);
            }
            _ => {
                self.state.modal = Some(Modal::Confirm {
                    title,
                    prompt,
                    action,
                });
            }
        }
    }

    fn start_action(&mut self, action: Action) {
        match action {
            Action::Attach => self.perform_attach(),
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
                    action: InputAction::CreateWindow {
                        session_name: session_name.to_string(),
                    },
                });
            }
            Action::Rename => self.open_rename_modal(),
            Action::Close => self.open_close_modal(),
            _ => {}
        }
    }

    fn perform_attach(&mut self) {
        let Some(session_name) = self.state.selected_session_name() else {
            self.set_error_status("No session selected");
            return;
        };

        let result = if let Some(window_index) = self.state.selected_window_index() {
            attach_to_window(session_name, window_index)
        } else {
            attach_to_session(session_name)
        };

        match result {
            Ok(()) => self.set_status("Attach successful"),
            Err(error) => self.set_error_status(&format!("Attach failed: {error}")),
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
            action: InputAction::RenameSession {
                session_name: session_name.to_string(),
            },
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
            action: ConfirmAction::CloseSession {
                session_name: session_name.to_string(),
            },
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
            InputAction::RenameWindow {
                session_name,
                window_index,
            } => {
                let result = rename_window(&session_name, &window_index, value);
                if result.is_ok() {
                    self.refresh_sessions();
                    self.state
                        .select_window_by_identity(&session_name, &window_index);
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
            ConfirmAction::CloseWindow {
                session_name,
                window_index,
            } => {
                let result = close_window(&session_name, &window_index);
                if result.is_ok() {
                    self.refresh_sessions();
                    self.state.select_session_by_name(&session_name);
                    self.set_status("Window closed");
                }
                result
            }
        };

        if let Err(error) = result {
            self.set_error_status(&format!("Action failed: {error}"));
        }
    }

    fn set_status(&mut self, message: &str) {
        self.state.status = Some(state::StatusLine {
            message: message.to_string(),
            is_error: false,
        });
    }

    fn set_error_status(&mut self, message: &str) {
        self.state.status = Some(state::StatusLine {
            message: message.to_string(),
            is_error: true,
        });
    }

    fn action_from_key(code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        if matches!(code, KeyCode::Esc | KeyCode::Char('q'))
            || (matches!(code, KeyCode::Char('c')) && modifiers.contains(KeyModifiers::CONTROL))
        {
            return Some(Action::Quit);
        }

        match code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::Back),
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => Some(Action::Select),
            KeyCode::Tab => Some(Action::CycleFocus),
            KeyCode::Char(' ') => Some(Action::ToggleExpand),
            KeyCode::Char('R' | 'r') => Some(Action::Refresh),
            KeyCode::Char('a') => Some(Action::Attach),
            KeyCode::Char('c') => Some(Action::CreateSession),
            KeyCode::Char('w') => Some(Action::CreateWindow),
            KeyCode::Char('n') => Some(Action::Rename),
            KeyCode::Char('x') => Some(Action::Close),
            _ => None,
        }
    }

    fn refresh_sessions(&mut self) {
        match Self::fetch_sessions() {
            Ok(sessions) => {
                let count = sessions.len();
                self.state.set_sessions(sessions);
                self.state.status = Some(state::StatusLine {
                    message: format!("Loaded {count} sessions"),
                    is_error: false,
                });
            }
            Err(error) => {
                self.state.status = Some(state::StatusLine {
                    message: format!("Refresh failed: {error}"),
                    is_error: true,
                });
            }
        }
    }

    fn fetch_sessions() -> Result<Vec<Session>> {
        let names = list_active_sessions()?;
        let mut sessions = Vec::with_capacity(names.len());

        for name in names {
            let session = get_session(Some(&name))?;
            sessions.push(session);
        }

        Ok(sessions)
    }

    fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }
}
