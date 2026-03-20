use std::fmt::Write;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::state::State;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let mut help = String::new();
    help.push_str("Tab focus  Up/Down move  Enter/Right select  Left back  Space collapse  r refresh  q quit\n");

    if let Some(status) = &state.status {
        let level = if status.is_error { "error" } else { "ok" };
        let _ = write!(help, "Status ({level}): {}", status.message);
    } else {
        let _ = write!(help, "Status: ready");
    }

    let _ = write!(help, " | Focus: {}", state.focus_label());

    let widget = Paragraph::new(help)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(widget, area);
}
