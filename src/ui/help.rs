use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

use crate::app::state::State;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let focused = state.focus_label() == "help";
    let mut help = String::new();
    help.push_str(
        "Nav: Tab focus  Up/Down sessions  Left collapse  Right expand  Enter cycle window\n",
    );
    help.push_str("CRUD: c create session  w create window  n rename  x close  a attach\n");
    help.push_str("Other: Space toggle  r refresh  q quit");

    let status_line = if let Some(status) = &state.status {
        let level = if status.is_error { "error" } else { "ok" };
        format!(
            "Status ({level}): {} | Focus: {}",
            status.message,
            state.focus_label()
        )
    } else {
        format!("Status: ready | Focus: {}", state.focus_label())
    };

    let status_style = if state.status.as_ref().is_some_and(|status| status.is_error) {
        theme::error_status()
    } else {
        theme::ok_status()
    };

    let lines = vec![Line::from(help), Line::styled(status_line, status_style)];

    let widget = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::panel_border(focused))
            .title("Help")
            .title_style(theme::panel_title(focused))
            .padding(Padding::new(1, 1, 0, 0)),
    );
    frame.render_widget(widget, area);
}
