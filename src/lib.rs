pub mod app;
pub mod tmux;
pub mod ui;

use anyhow::Result;

/// Starts the tuimux application.
///
/// # Errors
/// Returns an error when the app runtime fails.
pub fn run() -> Result<()> {
    let mut app = app::App::new();
    app.run()
}
