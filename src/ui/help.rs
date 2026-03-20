use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::State;

pub fn render(frame: &mut Frame, area: Rect, _state: &State) {
    let help = "Phase 3 read-only: q/esc quit  Ctrl-C quit  auto refresh every 2s";
    let widget = Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(widget, area);
}
