use ratatui::style::{Color, Modifier, Style};

#[must_use]
pub fn panel_border(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

#[must_use]
pub fn panel_title(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    }
}

#[must_use]
pub fn selected_row() -> Style {
    Style::default().fg(Color::Black).bg(Color::Green)
}

#[must_use]
pub fn info_text() -> Style {
    Style::default().fg(Color::Gray)
}

#[must_use]
pub fn ok_status() -> Style {
    Style::default().fg(Color::LightGreen)
}

#[must_use]
pub fn error_status() -> Style {
    Style::default().fg(Color::LightRed)
}
