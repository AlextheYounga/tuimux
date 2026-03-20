use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, _state: &AppState) {
    let help = "c:create session  w:create window  Enter:attach  r:rename  x:close  q:quit";
    let widget = Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(widget, area);
}
