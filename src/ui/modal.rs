use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};

use crate::app::state::{Modal, State};
use crate::ui::theme;

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

    let widget = Paragraph::new(body).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::panel_border(true))
            .title("Modal")
            .title_style(theme::panel_title(true))
            .padding(Padding::new(1, 1, 0, 0)),
    );
    frame.render_widget(Clear, area);
    frame.render_widget(widget, area);
}
