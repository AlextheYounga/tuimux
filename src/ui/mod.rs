pub mod details;
pub mod help;
pub mod layout;
pub mod modal;
pub mod tree;

use ratatui::Frame;

use crate::app::state::AppState;

pub fn render(frame: &mut Frame, state: &AppState) {
    let regions = layout::split(frame.area());

    tree::render(frame, regions.left, state);
    details::render(frame, regions.right, state);
    help::render(frame, regions.bottom, state);

    if state.modal.is_some() {
        modal::render(frame, regions.overlay, state);
    }
}
