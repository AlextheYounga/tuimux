use anyhow::Result;
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use crate::app::state;
use crate::app::{App, PreviewRequest, PreviewResult, PreviewRuntime, RefreshOutcome};
use crate::tmux::interface::{capture_preview, fetch_all_sessions};
use crate::ui;

impl App {
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

        'main: loop {
            self.apply_preview_results();
            terminal.draw(|frame| ui::render(frame, &self.state))?;

            let mut handled_event = false;
            while event::poll(if handled_event { Duration::from_millis(0) } else { input_poll })? {
                handled_event = true;
                if let Event::Key(key_event) = event::read()?
                    && key_event.kind == KeyEventKind::Press
                {
                    if self.handle_modal_key(key_event.code) {
                        continue;
                    }

                    let action = Self::action_from_key(key_event.code, key_event.modifiers);
                    if self.handle_action(action) {
                        break 'main;
                    }
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

    pub(super) fn refresh_sessions(&mut self) {
        match Self::fetch_sessions() {
            Ok(outcome) => {
                let count = outcome.sessions.len();
                self.state.set_sessions(outcome.sessions);
                self.request_preview_refresh();

                if !outcome.skipped_sessions.is_empty() {
                    self.state.status = Some(state::StatusLine {
                        message: format!("Loaded {count} sessions, skipped {}", outcome.skipped_sessions.len()),
                        is_error: true,
                    });
                } else if count == 0 {
                    self.state.status =
                        Some(state::StatusLine { message: String::from("No tmux sessions"), is_error: false });
                } else {
                    self.state.status =
                        Some(state::StatusLine { message: format!("Loaded {count} sessions"), is_error: false });
                }
            }
            Err(error) => {
                self.state.status =
                    Some(state::StatusLine { message: format!("Refresh failed: {error}"), is_error: true });
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

                let PreviewRequest::Fetch { seq, session_name, window_index } = latest;
                let result = match capture_preview(&session_name, window_index.as_deref()) {
                    Ok(output) => PreviewResult {
                        seq,
                        output: if output.trim().is_empty() { String::from("(empty output)") } else { output },
                        is_error: false,
                    },
                    Err(error) => {
                        PreviewResult { seq, output: format!("Preview unavailable: {error}"), is_error: true }
                    }
                };

                if result_tx.send(result).is_err() {
                    return;
                }
            }
        });

        self.preview_runtime = Some(PreviewRuntime { request_tx, result_rx, request_seq: 0, latest_applied_seq: 0 });
    }

    pub(super) fn request_preview_refresh(&mut self) {
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
        match fetch_all_sessions() {
            Ok(sessions) => Ok(RefreshOutcome { sessions, skipped_sessions: Vec::new() }),
            Err(error) => Err(error),
        }
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
