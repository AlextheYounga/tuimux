pub mod actions;
pub mod export;
mod input;
mod restore;
mod runtime;
pub mod state;

use crossterm::event::{KeyCode, KeyModifiers};
use std::sync::mpsc::{Receiver, Sender};

use crate::app::actions::Action;
use crate::app::state::State;
use crate::tmux::session::Session;

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
    Fetch { seq: u64, session_name: String, window_index: Option<String> },
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

    fn action_from_key(code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        if matches!(code, KeyCode::Esc | KeyCode::Char('q'))
            || (matches!(code, KeyCode::Char('c')) && modifiers.contains(KeyModifiers::CONTROL))
        {
            return Some(Action::Quit);
        }

        match code {
            KeyCode::Char('E') => Some(Action::Export),
            KeyCode::Char('R') => Some(Action::Restore),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::Collapse),
            KeyCode::Right | KeyCode::Char('l') => Some(Action::Expand),
            KeyCode::Enter | KeyCode::Char('a') => Some(Action::Attach),
            KeyCode::Char(' ') => Some(Action::ToggleExpand),
            KeyCode::Char('r') => Some(Action::Refresh),
            KeyCode::Char('s') => Some(Action::CreateSession),
            KeyCode::Char('w') => Some(Action::CreateWindow),
            KeyCode::Char('n') => Some(Action::Rename),
            KeyCode::Char('x') => Some(Action::Close),
            _ => None,
        }
    }
}
