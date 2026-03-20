use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::State;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let mut lines = Vec::new();

    if state.sessions.is_empty() {
        lines.push(Line::from("No active tmux sessions"));
    } else {
        for (session_index, session) in state.sessions.iter().enumerate() {
            let marker = if state.selected_session == Some(session_index) {
                ">"
            } else {
                " "
            };
            lines.push(Line::from(format!("{marker} {}", session.name)));

            for (window_index, window) in session.windows.iter().enumerate() {
                let marker = if state.selected_session == Some(session_index)
                    && state.selected_window == Some(window_index)
                {
                    "*"
                } else {
                    "-"
                };
                lines.push(Line::from(format!(
                    "  {marker} [{}] {}",
                    window.index, window.name
                )));
            }
        }
    }

    let tree =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Sessions"));
    frame.render_widget(tree, area);
}
