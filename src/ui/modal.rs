use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};
use ratatui::Frame;

use crate::app::state::{Modal, State};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let body = match &state.modal {
        Some(Modal::Input { title, value, .. }) => {
            format!("{title}\n\n  > {value}_\n\n[Enter] submit  [Esc] cancel")
        }
        Some(Modal::Confirm { title, prompt, .. }) => {
            format!("{title}\n\n  {prompt}")
        }
        None => return,
    };

    let widget = Paragraph::new(body).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::panel_border(true))
            .title(" Action Required ")
            .title_style(theme::panel_title(true))
            .padding(Padding::new(2, 2, 1, 1)),
    );
    frame.render_widget(Clear, area);
    frame.render_widget(widget, area);
}
