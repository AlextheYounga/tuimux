use std::fmt::Write;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::state::State;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let body = if let Some(session) = state.selected_session_ref() {
        let mut details = String::new();
        let _ = writeln!(details, "Session: {}", session.name);
        let _ = writeln!(details, "Working dir: {}", session.work_dir);
        let _ = writeln!(details, "Windows: {}", session.windows.len());

        if let Some(window) = state.selected_window_ref() {
            details.push_str("\nFocused window\n");
            let _ = writeln!(details, "- Index: {}", window.index);
            let _ = writeln!(details, "- Name: {}", window.name);
            let _ = writeln!(details, "- Layout: {}", window.layout);
            let _ = writeln!(details, "- Panes: {}", window.panes.len());
            details.push_str("\nPane commands\n");

            for pane in &window.panes {
                let command = pane.current_command.as_deref().unwrap_or("_");
                let _ = writeln!(details, "- [{}] {}", pane.index, command);
            }
        }

        details
    } else {
        String::from("No selection")
    };

    let widget = Paragraph::new(body)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("Details"));
    frame.render_widget(widget, area);
}
