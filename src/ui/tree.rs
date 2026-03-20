use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let title = match &state.selection {
        Some(_) => "Sessions",
        None => "Sessions (no selection)",
    };

    let placeholder = Paragraph::new("Session tree will render here")
        .block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(placeholder, area);
}
