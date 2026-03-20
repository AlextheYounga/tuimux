use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::State;

pub fn render(frame: &mut Frame, area: Rect, _state: &State) {
    let help = "c:create session  w:create window  Enter:attach  r:rename  x:close  q:quit";
    let widget = Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(widget, area);
}
