pub mod actions;
pub mod state;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crate::app::state::State;
use crate::tmux::interface::{get_session, list_active_sessions};
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
                let should_quit = matches!(key_event.code, KeyCode::Esc | KeyCode::Char('q'))
                    || (matches!(key_event.code, KeyCode::Char('c'))
                        && key_event.modifiers.contains(KeyModifiers::CONTROL));
                if should_quit {
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
