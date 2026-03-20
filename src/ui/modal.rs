use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::state::{AppState, ModalState};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let body = match &state.modal {
        Some(ModalState::Input { title, value }) => {
            format!("{}\n\n{}", title, value)
        }
        Some(ModalState::Confirm { title, prompt }) => {
            format!("{}\n\n{}", title, prompt)
        }
        None => return,
    };

    let widget = Paragraph::new(body).block(Block::default().borders(Borders::ALL).title("Modal"));
    frame.render_widget(Clear, area);
    frame.render_widget(widget, area);
}
