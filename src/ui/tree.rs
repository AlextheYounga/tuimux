use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::app::state::State;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let mut lines = Vec::new();

    if state.sessions.is_empty() {
        lines.push(Line::from("No active tmux sessions"));
    } else {
        for (session_index, session) in state.sessions.iter().enumerate() {
            let expanded = state.expanded_sessions.contains(&session.name);
            let fold = if expanded { "[-]" } else { "[+]" };
            let is_selected_session =
                state.selected_session == Some(session_index) && state.selected_window.is_none();
            let marker = if is_selected_session { ">" } else { " " };
            let mut line = Line::from(format!("{marker} {fold} {}", session.name));
            if is_selected_session {
                line = line.style(theme::selected_row());
            }
            lines.push(line);

            if !expanded {
                continue;
            }

            for (window_index, window) in session.windows.iter().enumerate() {
                let is_selected_window = state.selected_session == Some(session_index)
                    && state.selected_window == Some(window_index);
                let marker = if is_selected_window { "*" } else { "-" };
                let mut line = Line::from(format!("  {marker} [{}] {}", window.index, window.name));
                if is_selected_window {
                    line = line.style(theme::selected_row());
                }
                lines.push(line);
            }
        }
    }

    let focused = state.focus_label() == "tree";
    let title = if focused {
        "Sessions (focus)"
    } else {
        "Sessions"
    };

    let tree = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::panel_border(focused))
            .title(title)
            .title_style(theme::panel_title(focused))
            .padding(Padding::new(1, 1, 0, 0)),
    );
    frame.render_widget(tree, area);
}
