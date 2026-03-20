pub mod actions;
pub mod state;

use anyhow::Result;

use crate::app::state::State;

#[derive(Debug, Default)]
pub struct App {
    pub state: State,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs the application coordinator.
    ///
    /// # Errors
    /// Returns an error when the terminal runtime or event loop fails.
    pub fn run(&mut self) -> Result<()> {
        let _ = &self.state;
        Ok(())
    }
}
