use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::state::State;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let mut lines = Vec::new();

    if state.sessions.is_empty() {
        lines.push(Line::from("No active tmux sessions"));
    } else {
        for (session_index, session) in state.sessions.iter().enumerate() {
            let expanded = state.expanded_sessions.contains(&session.name);
            let fold = if expanded { "[-]" } else { "[+]" };
            let marker = if state.selected_session == Some(session_index) {
                ">"
            } else {
                " "
            };
            lines.push(Line::from(format!("{marker} {fold} {}", session.name)));

            if !expanded {
                continue;
            }

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

    let title = if state.focus_label() == "tree" {
        "Sessions (focus)"
    } else {
        "Sessions"
    };

    let tree = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(tree, area);
}
