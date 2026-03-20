pub mod app;
pub mod tmux;
pub mod ui;

use anyhow::Result;

pub fn run() -> Result<()> {
    let mut app = app::App::new();
    app.run()
}
