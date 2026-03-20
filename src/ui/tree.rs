use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::State;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let title = match &state.selection {
        Some(_) => "Sessions",
        None => "Sessions (no selection)",
    };

    let placeholder = Paragraph::new("Session tree will render here")
        .block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(placeholder, area);
}
