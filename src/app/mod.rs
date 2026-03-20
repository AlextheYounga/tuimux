pub mod actions;
pub mod state;

use anyhow::Result;

use crate::app::state::AppState;

#[derive(Debug, Default)]
pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) -> Result<()> {
        let _ = &self.state;
        Ok(())
    }
}
