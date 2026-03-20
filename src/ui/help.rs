use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::app::state::State;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &State) {
    let nav_style = Style::default().fg(Color::Cyan);
    let crud_style = Style::default().fg(Color::Yellow);
    let other_style = Style::default().fg(Color::Magenta);

    let lines = vec![
        Line::from(vec![
            Span::styled("NAV ", nav_style),
            Span::raw("Up/Down rows  Left collapse  Right expand  Enter cycle window"),
        ]),
        Line::from(vec![
            Span::styled("CRUD ", crud_style),
            Span::raw("c session  w window  n rename  x close  a attach"),
        ]),
        Line::from(vec![
            Span::styled("OTHER ", other_style),
            Span::raw("Space toggle  r refresh  q quit"),
        ]),
        build_status_line(state),
    ];

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::panel_border(false))
            .title("Help")
            .title_style(theme::panel_title(false))
            .padding(Padding::new(1, 1, 0, 0)),
    );
    frame.render_widget(widget, area);
}

fn build_status_line(state: &State) -> Line<'static> {
    if let Some(status) = &state.status {
        let label = if status.is_error { "ERROR" } else { "OK" };
        let style = if status.is_error {
            theme::error_status()
        } else {
            theme::ok_status()
        };

        return Line::from(vec![
            Span::styled(format!("STATUS {label}: "), style),
            Span::raw(status.message.clone()),
        ]);
    }

    Line::from(vec![
        Span::styled("STATUS OK: ", theme::ok_status()),
        Span::raw("ready"),
    ])
}
