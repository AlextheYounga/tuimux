use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::State;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let body = if let Some(status) = &state.status {
        format!("Status: {}", status.message)
    } else {
        String::from("Selected item details and preview will render here")
    };

    let widget =
        Paragraph::new(body).block(Block::default().borders(Borders::ALL).title("Details"));
    frame.render_widget(widget, area);
}
