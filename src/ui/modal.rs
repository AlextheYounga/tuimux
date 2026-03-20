use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::state::{Modal, State};

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let body = match &state.modal {
        Some(Modal::Input { title, value, .. }) => {
            format!("{title}\n\n{value}\n\nEnter submit  Esc cancel")
        }
        Some(Modal::Confirm { title, prompt, .. }) => {
            format!("{title}\n\n{prompt}")
        }
        None => return,
    };

    let widget = Paragraph::new(body).block(Block::default().borders(Borders::ALL).title("Modal"));
    frame.render_widget(Clear, area);
    frame.render_widget(widget, area);
}
