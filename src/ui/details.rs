use std::fmt::Write;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::app::state::State;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let focused = state.focus_label() == "details";
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(5)])
        .split(area);

    let body = if let Some(session) = state.selected_session_ref() {
        let mut details = String::new();
        let _ = writeln!(details, "Session: {}", session.name);
        let _ = writeln!(details, "Path: {}", session.work_dir);
        let _ = writeln!(details, "Windows: {}", session.windows.len());

        if let Some(window) = state.selected_window_ref() {
            let _ = writeln!(details, "Window: [{}] {}", window.index, window.name);
        }

        details
    } else {
        String::from("No selection")
    };

    let details_widget = Paragraph::new(body).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::panel_border(focused))
            .title("Details")
            .title_style(theme::panel_title(focused))
            .padding(Padding::new(1, 1, 0, 0)),
    );

    let preview_title = if focused {
        "Preview (live, 1s)"
    } else {
        "Preview"
    };
    let preview_style = if state.preview_is_error {
        theme::error_status()
    } else {
        theme::info_text()
    };

    let preview_widget = Paragraph::new(state.preview.as_str())
        .style(preview_style)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::panel_border(focused))
                .title(preview_title)
                .title_style(theme::panel_title(focused))
                .padding(Padding::new(1, 1, 0, 0)),
        );

    frame.render_widget(details_widget, sections[0]);
    frame.render_widget(preview_widget, sections[1]);
}
