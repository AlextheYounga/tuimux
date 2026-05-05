use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::State;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let mut lines = Vec::new();
    let query = state.filter_query.trim().to_lowercase();
    let filter_active = !query.is_empty();

    if state.filter_mode {
        lines.push(Line::from(format!("Search: {}", state.filter_query)));
        lines.push(Line::from(""));
    }

    if state.sessions.is_empty() {
        lines.push(Line::from("No active tmux sessions"));
    } else {
        for (session_index, session) in state.sessions.iter().enumerate() {
            let session_match = if filter_active {
                let session_name_match = session.name.to_lowercase().contains(&query);
                let window_match = session.windows.iter().any(|window| {
                    window.name.to_lowercase().contains(&query) || window.index.to_lowercase().contains(&query)
                });
                session_name_match || window_match
            } else {
                true
            };

            if !session_match {
                continue;
            }

            let expanded = filter_active || state.expanded_sessions.contains(&session.name);
            let fold = if expanded { "[-]" } else { "[+]" };
            let is_selected_session = state.selected_session == Some(session_index) && state.selected_window.is_none();
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
                if filter_active
                    && !window.name.to_lowercase().contains(&query)
                    && !window.index.to_lowercase().contains(&query)
                {
                    continue;
                }

                let is_selected_window =
                    state.selected_session == Some(session_index) && state.selected_window == Some(window_index);
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
    let title = if focused { "Sessions (focus)" } else { "Sessions" };

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
