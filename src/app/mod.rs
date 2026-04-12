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
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use crate::app::actions::Action;
use crate::app::state::{ConfirmAction, InputAction, Modal, State};
use crate::tmux::interface::{
    attach_to_session, attach_to_window, capture_preview, close_session, close_window,
    create_session, create_window, get_session, list_active_sessions, rename_session,
    rename_window,
};
use crate::tmux::session::Session;
use crate::ui;

#[derive(Debug, Default)]
struct RefreshOutcome {
    sessions: Vec<Session>,
    skipped_sessions: Vec<String>,
}

#[derive(Debug, Default)]
pub struct App {
    pub state: State,
    preview_runtime: Option<PreviewRuntime>,
}

#[derive(Debug)]
struct PreviewRuntime {
    request_tx: Sender<PreviewRequest>,
    result_rx: Receiver<PreviewResult>,
    request_seq: u64,
    latest_applied_seq: u64,
}

#[derive(Debug)]
enum PreviewRequest {
    Fetch {
        seq: u64,
        session_name: String,
        window_index: Option<String>,
    },
}

#[derive(Debug)]
struct PreviewResult {
    seq: u64,
    output: String,
    is_error: bool,
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
        self.init_preview_runtime();
        self.refresh_sessions();

        let mut terminal = Self::init_terminal()?;
        let run_result = self.run_loop(&mut terminal);
        let restore_result = Self::restore_terminal(&mut terminal);
        self.preview_runtime = None;

        restore_result?;
        run_result
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        let input_poll = Duration::from_millis(200);
        let refresh_interval = Duration::from_secs(2);
        let preview_interval = Duration::from_secs(1);
        let mut last_refresh = Instant::now();
        let mut last_preview = Instant::now();

        loop {
            self.apply_preview_results();
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

            if last_preview.elapsed() >= preview_interval {
                self.request_preview_refresh();
                last_preview = Instant::now();
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
                    self.request_preview_refresh();
                }
            }
            Action::MoveDown => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.move_down();
                    self.request_preview_refresh();
                }
            }
            Action::Select => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.select();
                    self.request_preview_refresh();
                }
            }
            Action::Expand => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.expand_selected_session();
                    self.request_preview_refresh();
                }
            }
            Action::Collapse => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.collapse_selected_session();
                    self.request_preview_refresh();
                }
            }
            Action::ToggleExpand => {
                if matches!(self.state.focus, state::FocusRegion::Tree) {
                    self.state.toggle_expand();
                    self.request_preview_refresh();
                }
            }
            Action::Attach => {
                if self.perform_attach() {
                    return true;
                }
            }
            Action::CreateSession | Action::CreateWindow | Action::Rename | Action::Close => {
                self.start_action(action);
            }
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
            KeyCode::Left | KeyCode::Char('h') => Some(Action::Collapse),
            KeyCode::Right | KeyCode::Char('l') => Some(Action::Expand),
            KeyCode::Enter | KeyCode::Char('a') => Some(Action::Attach),
            KeyCode::Char(' ') => Some(Action::ToggleExpand),
            KeyCode::Char('R' | 'r') => Some(Action::Refresh),
            KeyCode::Char('s') => Some(Action::CreateSession),
            KeyCode::Char('w') => Some(Action::CreateWindow),
            KeyCode::Char('n') => Some(Action::Rename),
            KeyCode::Char('x') => Some(Action::Close),
            _ => None,
        }
    }

    fn refresh_sessions(&mut self) {
        match Self::fetch_sessions() {
            Ok(outcome) => {
                let count = outcome.sessions.len();
                self.state.set_sessions(outcome.sessions);
                self.request_preview_refresh();

                if !outcome.skipped_sessions.is_empty() {
                    self.state.status = Some(state::StatusLine {
                        message: format!(
                            "Loaded {count} sessions, skipped {}",
                            outcome.skipped_sessions.len()
                        ),
                        is_error: true,
                    });
                } else if count == 0 {
                    self.state.status = Some(state::StatusLine {
                        message: String::from("No active tmux sessions"),
                        is_error: false,
                    });
                } else {
                    self.state.status = Some(state::StatusLine {
                        message: format!("Loaded {count} sessions"),
                        is_error: false,
                    });
                }
            }
            Err(error) => {
                self.state.status = Some(state::StatusLine {
                    message: format!("Refresh failed: {error}"),
                    is_error: true,
                });
            }
        }
    }

    fn init_preview_runtime(&mut self) {
        let (request_tx, request_rx) = mpsc::channel::<PreviewRequest>();
        let (result_tx, result_rx) = mpsc::channel::<PreviewResult>();

        thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                let mut latest = request;
                loop {
                    match request_rx.try_recv() {
                        Ok(next) => latest = next,
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => return,
                    }
                }

                let PreviewRequest::Fetch {
                    seq,
                    session_name,
                    window_index,
                } = latest;

                let result = match capture_preview(&session_name, window_index.as_deref()) {
                    Ok(output) => PreviewResult {
                        seq,
                        output: if output.trim().is_empty() {
                            String::from("(empty output)")
                        } else {
                            output
                        },
                        is_error: false,
                    },
                    Err(error) => PreviewResult {
                        seq,
                        output: format!("Preview unavailable: {error}"),
                        is_error: true,
                    },
                };

                if result_tx.send(result).is_err() {
                    return;
                }
            }
        });

        self.preview_runtime = Some(PreviewRuntime {
            request_tx,
            result_rx,
            request_seq: 0,
            latest_applied_seq: 0,
        });
    }

    fn request_preview_refresh(&mut self) {
        let Some(runtime) = self.preview_runtime.as_mut() else {
            return;
        };

        let Some(session_name) = self.state.selected_session_name() else {
            self.state.preview = String::from("No selection");
            self.state.preview_is_error = false;
            return;
        };

        runtime.request_seq = runtime.request_seq.saturating_add(1);
        let request = PreviewRequest::Fetch {
            seq: runtime.request_seq,
            session_name: session_name.to_string(),
            window_index: self.state.selected_window_index().map(str::to_string),
        };

        if runtime.request_tx.send(request).is_err() {
            self.state.preview = String::from("Preview unavailable: worker disconnected");
            self.state.preview_is_error = true;
        }
    }

    fn apply_preview_results(&mut self) {
        let Some(runtime) = self.preview_runtime.as_mut() else {
            return;
        };

        while let Ok(result) = runtime.result_rx.try_recv() {
            if result.seq >= runtime.latest_applied_seq {
                runtime.latest_applied_seq = result.seq;
                self.state.preview = result.output;
                self.state.preview_is_error = result.is_error;
            }
        }
    }

    fn fetch_sessions() -> Result<RefreshOutcome> {
        let names = list_active_sessions()?;
        let mut outcome = RefreshOutcome {
            sessions: Vec::with_capacity(names.len()),
            skipped_sessions: Vec::new(),
        };

        for name in names {
            match get_session(Some(&name)) {
                Ok(session) => outcome.sessions.push(session),
                Err(error) => {
                    outcome.skipped_sessions.push(format!("{name}: {error}"));
                }
            }
        }

        Ok(outcome)
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
